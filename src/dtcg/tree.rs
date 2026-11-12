//! Composed token tree (C1), provenance (C6) and group inheritance (C7).
//!
//! A [`Tree`] is built by applying [`Layer`]s in merge order (C2). The raw
//! composed document ([`Tree::document`]) keeps every `$extends` and group
//! `$ref` declaration as authored. [`Tree::expand`] materializes that
//! inheritance into an [`Expanded`] document: inherited tokens, nested groups
//! and group properties are written out, while the declarations stay. The
//! expanded document is the input of [`crate::dtcg::graph`], the target space
//! of JSON pointers, and the body of the DTCG export (O1). Every token of an
//! expanded document knows the layer whose definition it carries (C6).

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use indexmap::IndexSet;
use serde_json::{Map, Value};

use crate::diagnostic::{Diagnostic, Failure, Outcome, Rule};
use crate::dtcg::pointer::{self, Reference};

/// Identifies a layer of one composition; assigned by the composer.
pub type LayerId = u16;

/// Dotted token path such as `semantic.color.ink` or `primitive.space.$root`.
/// The document root group has the empty path.
pub type TokenPath = Arc<str>;

/// Role of a layer in the merge order (C2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayerKind {
    Base,
    /// The mode layer, carrying the mode name (`night`, `day`, `aurora`).
    Mode(&'static str),
    Theme,
    /// An override, numbered from 1 in command-line order.
    Override(u16),
}

/// One source document of a composition.
#[derive(Clone, Debug)]
pub struct Layer {
    pub id: LayerId,
    pub kind: LayerKind,
    pub file: PathBuf,
    pub document: Map<String, Value>,
}

impl Layer {
    /// Provenance label used by the resolved export: `base`, `mode:<mode>`,
    /// `theme:<file name>` or `override:<n>:<file name>`.
    #[must_use]
    pub fn source_label(&self) -> String {
        let name = self.file.file_name().unwrap_or_default().to_string_lossy();
        match self.kind {
            LayerKind::Base => "base".to_owned(),
            LayerKind::Mode(mode) => format!("mode:{mode}"),
            LayerKind::Theme => format!("theme:{name}"),
            LayerKind::Override(n) => format!("override:{n}:{name}"),
        }
    }
}

/// Whether `key` names a child (token or group) rather than a property.
pub(crate) fn is_child_key(key: &str) -> bool {
    !key.starts_with('$') || key == "$root"
}

/// D1: an object containing `$value` is a token; any other object is a group.
pub(crate) fn is_token(node: &Map<String, Value>) -> bool {
    node.contains_key("$value")
}

fn declares_inheritance(group: &Map<String, Value>) -> bool {
    group.contains_key("$extends") || group.contains_key("$ref")
}

fn join(prefix: &str, key: &str) -> String {
    if prefix.is_empty() {
        key.to_owned()
    } else {
        format!("{prefix}.{key}")
    }
}

fn dotted(segs: &[String]) -> String {
    if segs.is_empty() {
        "<root>".to_owned()
    } else {
        segs.join(".")
    }
}

/// Call `f` with the dotted path and definition of every token below
/// `group`, in document order.
pub(crate) fn visit_tokens<'m>(
    group: &'m Map<String, Value>,
    f: &mut impl FnMut(&str, &'m Map<String, Value>),
) {
    fn walk<'m>(
        group: &'m Map<String, Value>,
        path: &mut String,
        f: &mut impl FnMut(&str, &'m Map<String, Value>),
    ) {
        for (key, value) in group {
            let Value::Object(node) = value else { continue };
            if !is_child_key(key) {
                continue;
            }
            let len = path.len();
            if len > 0 {
                path.push('.');
            }
            path.push_str(key);
            if is_token(node) {
                f(path, node);
            } else {
                walk(node, path, f);
            }
            path.truncate(len);
        }
    }
    walk(group, &mut String::new(), f);
}

/// The raw composed document plus the origin of every token definition and
/// inheritance declaration in it.
#[derive(Clone, Debug, Default)]
pub struct Tree {
    root: Map<String, Value>,
    token_origin: HashMap<String, LayerId>,
    extends_origin: HashMap<String, LayerId>,
}

impl Tree {
    /// An empty tree, ready for the base layer.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A tree holding one standalone document without layer provenance
    /// (the plain DTCG profile).
    #[must_use]
    pub fn from_document(document: Map<String, Value>) -> Self {
        Self {
            root: document,
            ..Self::default()
        }
    }

    /// Merge `layer` into the tree (C1) and record it as the origin of every
    /// token definition and `$extends`/group `$ref` declaration it supplies.
    ///
    /// Groups merge recursively, `$extensions` objects deep-merge, other group
    /// properties are replaced, and token definitions are replaced whole.
    /// Existing keys keep their position; new keys are appended.
    pub fn apply(&mut self, layer: &Layer) {
        let mut recorder = Recorder {
            layer: layer.id,
            tokens: &mut self.token_origin,
            extends: &mut self.extends_origin,
        };
        if declares_inheritance(&layer.document) {
            recorder.extends.insert(String::new(), layer.id);
        }
        merge(&mut self.root, &layer.document, "", Some(&mut recorder));
    }

    /// The raw composed document, with inheritance declared but not
    /// materialized.
    #[must_use]
    pub fn document(&self) -> &Map<String, Value> {
        &self.root
    }

    /// Materialize `$extends` and group `$ref` inheritance (C7).
    ///
    /// A target is a brace path (`{a.b}`) or a local pointer (`#/a/b`, or a
    /// `{"$ref": "#/a/b"}` object) naming a group of the composed document.
    /// The expanded target merges under the local group with C1 rules, the
    /// local definitions winning; the target's own declaration is dropped and
    /// the local one is kept. Expanded targets are memoized.
    ///
    /// # Errors
    /// [`Rule::Extends`] for a group declaring both `$extends` and `$ref`, a
    /// missing or non-group target (token, value or metadata) and an
    /// inheritance cycle (the message shows the chain);
    /// [`Rule::ReferenceSyntax`]/[`Rule::ReferenceExternal`] for malformed or
    /// external pointers. Every failing group is reported once.
    pub fn expand(&self, location: &str) -> Outcome<Expanded> {
        let mut expander = Expander {
            raw: &self.root,
            location,
            targets: HashMap::new(),
            visiting: Vec::new(),
            introduced: HashMap::new(),
            reported: HashSet::new(),
            diags: Vec::new(),
        };
        let mut document = self.root.clone();
        expander.group(&mut Vec::new(), &mut document, true);
        if !expander.diags.is_empty() {
            return Err(Failure(expander.diags));
        }
        let mut origins = HashMap::new();
        visit_tokens(&document, &mut |path, _| {
            let origin = match expander.introduced.get(path) {
                Some(group) => self.extends_origin.get(group),
                None => self.token_origin.get(path),
            };
            if let Some(&origin) = origin {
                origins.insert(TokenPath::from(path), origin);
            }
        });
        Ok(Expanded { document, origins })
    }
}

struct Recorder<'a> {
    layer: LayerId,
    tokens: &'a mut HashMap<String, LayerId>,
    extends: &'a mut HashMap<String, LayerId>,
}

impl Recorder<'_> {
    /// Record every definition in a subtree inserted whole at `path`.
    fn subtree(&mut self, node: &Value, path: &str) {
        let Value::Object(node) = node else { return };
        if is_token(node) {
            self.tokens.insert(path.to_owned(), self.layer);
            return;
        }
        if declares_inheritance(node) {
            self.extends.insert(path.to_owned(), self.layer);
        }
        for (key, child) in node {
            if is_child_key(key) {
                self.subtree(child, &join(path, key));
            }
        }
    }
}

/// C1: merge the `overlay` group into the `target` group. `path` is only
/// used when recording origins.
fn merge(
    target: &mut Map<String, Value>,
    overlay: &Map<String, Value>,
    path: &str,
    mut recorder: Option<&mut Recorder<'_>>,
) {
    for (key, value) in overlay {
        if key == "$extensions" {
            if let (Some(Value::Object(existing)), Value::Object(added)) =
                (target.get_mut(key), value)
            {
                deep_merge(existing, added);
                continue;
            }
        } else if is_child_key(key) {
            let child_path = if recorder.is_some() {
                join(path, key)
            } else {
                String::new()
            };
            if let (Some(Value::Object(existing)), Value::Object(added)) =
                (target.get_mut(key), value)
                && !is_token(existing)
                && !is_token(added)
            {
                if let Some(recorder) = recorder.as_deref_mut()
                    && declares_inheritance(added)
                {
                    recorder.extends.insert(child_path.clone(), recorder.layer);
                }
                merge(existing, added, &child_path, recorder.as_deref_mut());
                continue;
            }
            if let Some(recorder) = recorder.as_deref_mut() {
                recorder.subtree(value, &child_path);
            }
        }
        target.insert(key.clone(), value.clone());
    }
}

/// Objects merge recursively; anything else is replaced.
fn deep_merge(target: &mut Map<String, Value>, overlay: &Map<String, Value>) {
    for (key, value) in overlay {
        if let (Some(Value::Object(existing)), Value::Object(added)) = (target.get_mut(key), value)
        {
            deep_merge(existing, added);
        } else {
            target.insert(key.clone(), value.clone());
        }
    }
}

struct Expander<'t> {
    raw: &'t Map<String, Value>,
    location: &'t str,
    /// Memoized expansions of inheritance targets; `None` failed.
    targets: HashMap<Vec<String>, Option<Map<String, Value>>>,
    /// Targets being expanded, outermost first.
    visiting: Vec<Vec<String>>,
    /// Inherited token path → path of the nearest group that inherited it.
    introduced: HashMap<String, String>,
    reported: HashSet<String>,
    diags: Vec<Diagnostic>,
}

impl Expander<'_> {
    /// Report a problem with the inheritance declared by group `at`, once.
    fn fail(&mut self, rule: Rule, at: &[String], message: impl Into<String>) {
        let at = dotted(at);
        if self.reported.insert(at.clone()) {
            self.diags.push(
                Diagnostic::error(rule, format!("{}: {at}", self.location), message).with_help(
                    "point $extends (or group $ref) at an existing group, as {a.b} or #/a/b",
                ),
            );
        }
    }

    /// Expand `content`, the group at `path`, in place. When `record` is set
    /// (the main walk), inherited tokens are noted for provenance. Returns
    /// whether the group and its descendants expanded without error.
    fn group(
        &mut self,
        path: &mut Vec<String>,
        content: &mut Map<String, Value>,
        record: bool,
    ) -> bool {
        let mut ok = true;
        match self.declared_target(path, content) {
            Err(()) => ok = false,
            Ok(None) => {}
            Ok(Some(target)) => match self.target(&target, path) {
                Some(mut inherited) => {
                    inherited.shift_remove("$extends");
                    inherited.shift_remove("$ref");
                    if record {
                        let group = path.join(".");
                        self.note_inherited(&inherited, Some(content), &group, &group);
                    }
                    merge(&mut inherited, content, "", None);
                    *content = inherited;
                }
                None => ok = false,
            },
        }
        for (key, child) in content.iter_mut() {
            let Value::Object(child) = child else {
                continue;
            };
            if !is_child_key(key) || is_token(child) {
                continue;
            }
            path.push(key.clone());
            ok &= self.group(path, child, record);
            path.pop();
        }
        ok
    }

    /// The target segments declared by a group, if any.
    fn declared_target(
        &mut self,
        path: &[String],
        content: &Map<String, Value>,
    ) -> Result<Option<Vec<String>>, ()> {
        let spec = match (content.get("$extends"), content.get("$ref")) {
            (None, None) => return Ok(None),
            (Some(_), Some(_)) => {
                self.fail(
                    Rule::Extends,
                    path,
                    "declares both $extends and $ref; keep one",
                );
                return Err(());
            }
            (Some(spec), None) | (None, Some(spec)) => spec,
        };
        let parsed = match spec {
            Value::String(s) if !s.contains(['{', '}']) => {
                let mut object = Map::new();
                object.insert("$ref".to_owned(), spec.clone());
                pointer::parse(&Value::Object(object))
            }
            Value::String(_) | Value::Object(_) => pointer::parse(spec),
            _ => Ok(None),
        };
        match parsed {
            Ok(Some(Reference::Alias(target))) => {
                Ok(Some(target.split('.').map(str::to_owned).collect()))
            }
            Ok(Some(Reference::Pointer(segs))) => Ok(Some(segs)),
            Ok(None) => {
                self.fail(
                    Rule::Extends,
                    path,
                    format!(
                        "inheritance target {spec} is neither a brace path nor a local pointer"
                    ),
                );
                Err(())
            }
            Err((rule, message)) => {
                self.fail(rule, path, message);
                Err(())
            }
        }
    }

    /// The expanded group at `segs`, as inherited by the group at `from`.
    fn target(&mut self, segs: &[String], from: &[String]) -> Option<Map<String, Value>> {
        if let Some(start) = self.visiting.iter().position(|v| v == segs) {
            let chain: Vec<String> = self.visiting[start..]
                .iter()
                .map(|v| dotted(v))
                .chain(std::iter::once(dotted(segs)))
                .collect();
            self.fail(
                Rule::Extends,
                from,
                format!("inheritance cycle {}", chain.join(" -> ")),
            );
            return None;
        }
        if let Some(done) = self.targets.get(segs) {
            return done.clone();
        }
        let mut content = self.lookup(segs, from)?;
        self.visiting.push(segs.to_vec());
        let ok = self.group(&mut segs.to_vec(), &mut content, false);
        self.visiting.pop();
        let result = ok.then_some(content);
        self.targets.insert(segs.to_vec(), result.clone());
        result
    }

    /// Find the raw group at `segs`. When a segment is missing below a group
    /// that inherits, the walk continues in that group's expansion, so a
    /// target can name an inherited group.
    fn lookup(&mut self, segs: &[String], from: &[String]) -> Option<Map<String, Value>> {
        if segs.iter().any(|s| s.starts_with('$')) {
            self.fail(
                Rule::Extends,
                from,
                format!(
                    "inheritance target {} is a token or metadata, not a group",
                    dotted(segs)
                ),
            );
            return None;
        }
        let mut node = self.raw;
        for (i, seg) in segs.iter().enumerate() {
            match node.get(seg) {
                Some(Value::Object(child)) if !is_token(child) => node = child,
                Some(_) => return self.not_group(segs, from),
                None if declares_inheritance(node) => {
                    let parent = self.target(&segs[..i], from)?;
                    return self.walk_expanded(&parent, &segs[i..], segs, from);
                }
                None => return self.missing(segs, from),
            }
        }
        Some(node.clone())
    }

    fn walk_expanded(
        &mut self,
        parent: &Map<String, Value>,
        rest: &[String],
        segs: &[String],
        from: &[String],
    ) -> Option<Map<String, Value>> {
        let mut node = parent;
        for seg in rest {
            match node.get(seg) {
                Some(Value::Object(child)) if !is_token(child) => node = child,
                Some(_) => return self.not_group(segs, from),
                None => return self.missing(segs, from),
            }
        }
        Some(node.clone())
    }

    fn not_group(&mut self, segs: &[String], from: &[String]) -> Option<Map<String, Value>> {
        self.fail(
            Rule::Extends,
            from,
            format!(
                "inheritance target {} is a token or value, not a group",
                dotted(segs)
            ),
        );
        None
    }

    fn missing(&mut self, segs: &[String], from: &[String]) -> Option<Map<String, Value>> {
        self.fail(
            Rule::Extends,
            from,
            format!("inheritance target {} does not exist", dotted(segs)),
        );
        None
    }

    /// Note every token of `inherited` that `content` does not define as
    /// introduced by `group`. Deeper groups are walked later and overwrite,
    /// so the nearest inheriting group wins.
    fn note_inherited(
        &mut self,
        inherited: &Map<String, Value>,
        content: Option<&Map<String, Value>>,
        prefix: &str,
        group: &str,
    ) {
        for (key, child) in inherited {
            let Value::Object(child) = child else {
                continue;
            };
            if !is_child_key(key) {
                continue;
            }
            let local = content.and_then(|c| c.get(key)).and_then(Value::as_object);
            let path = join(prefix, key);
            if is_token(child) {
                if !local.is_some_and(is_token) {
                    self.introduced.insert(path, group.to_owned());
                }
            } else {
                self.note_inherited(child, local.filter(|l| !is_token(l)), &path, group);
            }
        }
    }
}

/// A composed document with inheritance materialized, plus per-token origin.
#[derive(Clone, Debug)]
pub struct Expanded {
    document: Map<String, Value>,
    origins: HashMap<TokenPath, LayerId>,
}

impl Expanded {
    /// The expanded document: every inherited token and group property is
    /// written out, and `$extends`/group `$ref` declarations are kept.
    #[must_use]
    pub fn document(&self) -> &Map<String, Value> {
        &self.document
    }

    /// The layer whose definition the token at `path` carries (C6). For a
    /// token inherited through `$extends`, this is the layer that declared
    /// the nearest inheriting group's `$extends`. `None` for a standalone
    /// document or an unknown path.
    #[must_use]
    pub fn origin(&self, path: &str) -> Option<LayerId> {
        self.origins.get(path).copied()
    }

    /// The token paths `layer` assigns in this state: every token it defines
    /// directly, then every token its own `$extends` declarations
    /// materialize (expanded tokens whose origin is `layer`). Call it on the
    /// expansion made right after applying `layer`.
    #[must_use]
    pub fn assigned(&self, layer: &Layer) -> Assigned {
        let mut paths = IndexSet::new();
        visit_tokens(&layer.document, &mut |path, _| {
            paths.insert(TokenPath::from(path));
        });
        visit_tokens(&self.document, &mut |path, _| {
            if let Some((path, &origin)) = self.origins.get_key_value(path)
                && origin == layer.id
            {
                paths.insert(path.clone());
            }
        });
        Assigned { paths }
    }
}

/// Token paths a layer assigns (C3, C4, P9), in first-seen order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Assigned {
    paths: IndexSet<TokenPath>,
}

impl Assigned {
    #[must_use]
    pub fn contains(&self, path: &str) -> bool {
        self.paths.contains(path)
    }

    #[must_use]
    pub fn iter(&self) -> indexmap::set::Iter<'_, TokenPath> {
        self.paths.iter()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.paths.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }
}

impl<'a> IntoIterator for &'a Assigned {
    type Item = &'a TokenPath;
    type IntoIter = indexmap::set::Iter<'a, TokenPath>;
    fn into_iter(self) -> Self::IntoIter {
        self.paths.iter()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use serde_json::json;

    use super::*;

    fn map(v: Value) -> Map<String, Value> {
        match v {
            Value::Object(m) => m,
            _ => panic!("not an object"),
        }
    }

    fn layer(id: LayerId, kind: LayerKind, file: &str, document: Value) -> Layer {
        Layer {
            id,
            kind,
            file: PathBuf::from(file),
            document: map(document),
        }
    }

    fn token(v: i64) -> Value {
        json!({"$value": {"value": v, "unit": "px"}, "$description": "d"})
    }

    fn keys(m: &Map<String, Value>) -> Vec<&str> {
        m.keys().map(String::as_str).collect()
    }

    fn rules(f: &Failure) -> Vec<Rule> {
        f.0.iter().map(|d| d.rule).collect()
    }

    #[test]
    fn source_labels() {
        let doc = json!({});
        let cases = [
            (LayerKind::Base, "tokens/base.tokens.json", "base"),
            (
                LayerKind::Mode("night"),
                "tokens/modes/night.tokens.json",
                "mode:night",
            ),
            (
                LayerKind::Theme,
                "tokens/themes/fern.tokens.json",
                "theme:fern.tokens.json",
            ),
            (
                LayerKind::Override(2),
                "/tmp/x/compact.tokens.json",
                "override:2:compact.tokens.json",
            ),
        ];
        for (kind, file, label) in cases {
            assert_eq!(layer(0, kind, file, doc.clone()).source_label(), label);
        }
    }

    #[test]
    fn c1_merge_rules_and_key_order() {
        let mut tree = Tree::new();
        tree.apply(&layer(
            0,
            LayerKind::Base,
            "base.json",
            json!({
                "$extensions": {"org.x": {"a": 1, "keep": {"x": 1}}},
                "primitive": {
                    "$type": "dimension",
                    "$description": "old",
                    "one": token(1),
                    "shadow": {"$value": {"color": "c", "blur": 1}, "$description": "s"},
                    "two": token(2),
                }
            }),
        ));
        tree.apply(&layer(
            1,
            LayerKind::Theme,
            "theme.json",
            json!({
                "$extensions": {"org.x": {"a": 2, "keep": {"y": 2}}, "org.y": true},
                "primitive": {
                    "$description": "new",
                    "three": token(3),
                    "shadow": {"$value": {"blur": 2}, "$description": "t"},
                    "one": token(10),
                }
            }),
        ));
        let doc = tree.document();
        assert_eq!(
            doc["$extensions"],
            json!({"org.x": {"a": 2, "keep": {"x": 1, "y": 2}}, "org.y": true})
        );
        let primitive = doc["primitive"].as_object().unwrap();
        assert_eq!(
            keys(primitive),
            ["$type", "$description", "one", "shadow", "two", "three"]
        );
        assert_eq!(primitive["$description"], json!("new"));
        assert_eq!(primitive["$type"], json!("dimension"));
        // Token definitions are replaced whole: no partial composite merge.
        assert_eq!(
            primitive["shadow"],
            json!({"$value": {"blur": 2}, "$description": "t"})
        );
        assert_eq!(primitive["one"]["$value"]["value"], json!(10));
    }

    #[test]
    fn c1_token_replaces_group_and_group_replaces_token() {
        let mut tree = Tree::new();
        tree.apply(&layer(
            0,
            LayerKind::Base,
            "b.json",
            json!({"primitive": {"a": {"b": token(1)}, "c": token(2)}}),
        ));
        tree.apply(&layer(
            1,
            LayerKind::Theme,
            "t.json",
            json!({"primitive": {"a": token(3), "c": {"d": token(4)}}}),
        ));
        let expanded = tree.expand("x").unwrap();
        assert_eq!(expanded.document()["primitive"]["a"], token(3));
        assert_eq!(
            expanded.document()["primitive"]["c"],
            json!({"d": token(4)})
        );
        assert_eq!(expanded.origin("primitive.a"), Some(1));
        assert_eq!(expanded.origin("primitive.c.d"), Some(1));
        assert_eq!(expanded.origin("primitive.a.b"), None);
    }

    #[test]
    fn c6_origin_is_the_layer_whose_definition_a_token_carries() {
        let mut tree = Tree::new();
        let base = layer(
            0,
            LayerKind::Base,
            "b.json",
            json!({"semantic": {"a": token(1), "b": token(2)}}),
        );
        let mode = layer(
            1,
            LayerKind::Mode("day"),
            "m.json",
            json!({"semantic": {"b": token(3)}}),
        );
        tree.apply(&base);
        tree.apply(&mode);
        let expanded = tree.expand("x").unwrap();
        assert_eq!(expanded.origin("semantic.a"), Some(0));
        assert_eq!(expanded.origin("semantic.b"), Some(1));
        let assigned = expanded.assigned(&mode);
        assert_eq!(
            assigned.iter().map(|p| &**p).collect::<Vec<_>>(),
            ["semantic.b"]
        );
    }

    fn regular() -> Value {
        json!({
            "$type": "dimension",
            "$root": token(8),
            "wide": token(16),
            "nested": {"deep": token(32)}
        })
    }

    #[test]
    fn c6_c7_inherited_tokens_take_the_extends_declaring_layer() {
        let mut tree = Tree::new();
        let base = layer(
            0,
            LayerKind::Base,
            "b.json",
            json!({"primitive": {"regular": regular()}}),
        );
        let theme = layer(
            1,
            LayerKind::Theme,
            "t.json",
            json!({"primitive": {"compact": {"$extends": "{primitive.regular}", "$root": token(4)}}}),
        );
        tree.apply(&base);
        tree.apply(&theme);
        let expanded = tree.expand("x").unwrap();
        let compact = expanded.document()["primitive"]["compact"]
            .as_object()
            .unwrap();
        assert_eq!(compact["$type"], json!("dimension"));
        assert_eq!(compact["$extends"], json!("{primitive.regular}"));
        assert_eq!(compact["$root"], token(4));
        assert_eq!(compact["nested"]["deep"], token(32));
        assert_eq!(expanded.origin("primitive.compact.$root"), Some(1));
        assert_eq!(expanded.origin("primitive.compact.wide"), Some(1));
        assert_eq!(expanded.origin("primitive.compact.nested.deep"), Some(1));
        assert_eq!(expanded.origin("primitive.regular.wide"), Some(0));
        let assigned = expanded.assigned(&theme);
        let assigned: Vec<&str> = assigned.iter().map(|p| &**p).collect();
        assert_eq!(
            assigned,
            [
                "primitive.compact.$root",
                "primitive.compact.wide",
                "primitive.compact.nested.deep"
            ]
        );
        assert_eq!(expanded.assigned(&base).len(), 3);

        // A later change of the target updates the value, not the origin.
        let over = layer(
            2,
            LayerKind::Override(1),
            "o.json",
            json!({"primitive": {"regular": {"wide": token(20)}}}),
        );
        tree.apply(&over);
        let expanded = tree.expand("x").unwrap();
        assert_eq!(
            expanded.document()["primitive"]["compact"]["wide"],
            token(20)
        );
        assert_eq!(expanded.origin("primitive.compact.wide"), Some(1));
        assert!(!expanded.assigned(&over).contains("primitive.compact.wide"));
        assert!(expanded.assigned(&over).contains("primitive.regular.wide"));
    }

    #[test]
    fn c6_nearest_inheriting_group_wins() {
        let mut tree = Tree::new();
        tree.apply(&layer(0, LayerKind::Base, "b.json", json!({
            "primitive": {"a": {"k": {"x": token(1)}}, "b": {"y": token(2)}, "p": {"$extends": "{primitive.a}"}}
        })));
        tree.apply(&layer(
            1,
            LayerKind::Theme,
            "t.json",
            json!({
                "primitive": {"p": {"k": {"$extends": "#/primitive/b"}}}
            }),
        ));
        let expanded = tree.expand("x").unwrap();
        assert_eq!(expanded.origin("primitive.p.k.x"), Some(0));
        assert_eq!(expanded.origin("primitive.p.k.y"), Some(1));
    }

    #[test]
    fn c7_target_reached_through_an_inheriting_group() {
        let tree = Tree::from_document(map(json!({
            "primitive": {
                "a": {"k": {"x": token(1)}},
                "b": {"$extends": "{primitive.a}"},
                "c": {"$ref": "#/primitive/b/k"}
            }
        })));
        let expanded = tree.expand("x").unwrap();
        assert_eq!(expanded.document()["primitive"]["c"]["x"], token(1));
        assert_eq!(expanded.origin("primitive.c.x"), None);
    }

    #[test]
    fn c7_invalid_declarations_fail_once_each() {
        let tree = Tree::from_document(map(json!({
            "primitive": {
                "a": {"x": token(1)},
                "both": {"$extends": "{primitive.a}", "$ref": "#/primitive/a"},
                "missing": {"$extends": "{primitive.none}"},
                "token": {"$extends": "{primitive.a.x}"},
                "value": {"$extends": "#/primitive/a/x/$value"},
                "meta": {"$extends": "#/primitive/a/$type"},
                "number": {"$extends": 3},
                "external": {"$ref": "other.json#/a"},
                "loop1": {"$extends": "{primitive.loop2}"},
                "loop2": {"$extends": "{primitive.loop1}"},
                "self": {"inner": {"$extends": "{primitive.self}"}}
            }
        })));
        let failure = tree.expand("x").unwrap_err();
        let mut found = rules(&failure);
        found.sort_by_key(|r| r.id());
        let mut expected = vec![Rule::Extends; 8];
        expected.push(Rule::ReferenceExternal);
        expected.sort_by_key(|r| r.id());
        assert_eq!(found, expected, "{failure}");
        let locations: HashSet<&str> = failure.0.iter().map(|d| d.location.as_str()).collect();
        assert_eq!(locations.len(), failure.0.len(), "{failure}");
    }
}
