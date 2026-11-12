//! Token table (D1–D5), effective types (RF8), references (RF1–RF7) and
//! resolution to typed values (V1–V6).
//!
//! [`Resolved::from_expanded`] builds the table from an [`Expanded`]
//! document, checks its structure, infers every token's type, follows every
//! reference and parses each substituted value. [`resolve`] does the same
//! for one standalone DTCG document. Each phase collects every independent
//! diagnostic; a token whose dependency failed is skipped silently, so one
//! root cause produces one error. Chains are followed with explicit stacks,
//! so their length is unlimited (L13), and a node selected in place is
//! resolved once per use-site expectation and then reused, so a chain of
//! property pointers costs time proportional to the references it yields.

use std::cell::Cell;
use std::collections::HashMap;
use std::fmt::Write as _;

use indexmap::IndexMap;
use serde_json::{Map, Value};

use crate::diagnostic::{Diagnostic, Failure, Outcome, Rule, finish};
use crate::dtcg::pointer::{self, Reference};
use crate::dtcg::tree::{Expanded, Layer, LayerId, TokenPath, Tree, is_child_key, is_token};
use crate::dtcg::value::{TokenType, TypedValue, field_type};

const TOKEN_PROPERTIES: [&str; 5] = [
    "$value",
    "$type",
    "$description",
    "$deprecated",
    "$extensions",
];
const NAMESPACES: [&str; 3] = ["primitive", "semantic", "component"];

/// Effective deprecation of a token (D4: the nearest declaration wins).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Deprecation {
    /// The explanation; `None` for `$deprecated: true`.
    pub reason: Option<String>,
}

/// One token after resolution.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedToken {
    /// Effective type (RF8).
    pub ty: TokenType,
    /// The token object of the expanded document, as authored or inherited.
    pub definition: Map<String, Value>,
    /// The parsed value.
    pub value: TypedValue,
    /// The authored `$value` with every reference substituted.
    pub json: Value,
    /// The layer whose definition the token carries (C6); `None` standalone.
    pub origin: Option<LayerId>,
    /// Own or inherited deprecation.
    pub deprecated: Option<Deprecation>,
    /// Tokens owning the target of a reference used by this value, directly
    /// or through a pointer chain, in first-seen order. A pointer into a
    /// token's `$value` or metadata counts as a reference to that token.
    pub references: Vec<TokenPath>,
}

/// Every token of a document, resolved, in composed (document) order.
#[derive(Clone, Debug, Default)]
pub struct Resolved {
    tokens: IndexMap<TokenPath, ResolvedToken>,
}

/// Resolve one standalone DTCG document (Fjord format profile, no project
/// policy and no layer provenance). The input is not modified.
///
/// # Errors
/// Every independent inheritance, structure, type, reference and value
/// violation, as [`Resolved::from_expanded`].
pub fn resolve(document: &Map<String, Value>, location: &str) -> Outcome<Resolved> {
    let expanded = Tree::from_document(document.clone()).expand(location)?;
    Resolved::from_expanded(&expanded, &[], location)
}

impl Resolved {
    /// Resolve every token of an expanded composition.
    ///
    /// `layers` maps origins to source labels for diagnostics; `location`
    /// prefixes every diagnostic location (`<location>: <token path>`).
    /// JSON pointers select nodes of the expanded document.
    ///
    /// # Errors
    /// - structure (D1–D5): [`Rule::Structure`], [`Rule::Name`],
    ///   [`Rule::Description`], [`Rule::Type`], [`Rule::Namespace`];
    /// - types (RF8): [`Rule::TypeUndetermined`];
    /// - references (RF1–RF5, RF7): [`Rule::ReferenceSyntax`],
    ///   [`Rule::ReferenceExternal`], [`Rule::ReferenceMissing`],
    ///   [`Rule::ReferenceCycle`], [`Rule::ReferenceType`];
    /// - values (V1–V6): [`Rule::Value`].
    pub fn from_expanded(
        expanded: &Expanded,
        layers: &[Layer],
        location: &str,
    ) -> Outcome<Resolved> {
        let mut graph = Graph {
            doc: expanded.document(),
            expanded,
            location,
            labels: layers.iter().map(|l| (l.id, l.source_label())).collect(),
            tokens: Vec::new(),
            index: HashMap::new(),
            diags: Vec::new(),
        };
        if expanded.document().contains_key("$value") {
            return Err(Failure::one(graph.diagnostic(
                Rule::Structure,
                "",
                None,
                "the document root must be a group, not a token",
            )));
        }
        graph.group(expanded.document(), &mut String::new(), Scope::ROOT);
        let types = graph.infer_types();
        let results = graph.resolve_all(&types);
        let mut tokens = IndexMap::with_capacity(graph.tokens.len());
        for ((token, ty), result) in graph.tokens.iter().zip(&types).zip(results) {
            let (Mark::Done(ty), Some(result)) = (ty, result) else {
                continue;
            };
            tokens.insert(
                token.path.clone(),
                ResolvedToken {
                    ty: *ty,
                    definition: token.definition.clone(),
                    value: result.value,
                    json: result.json,
                    origin: token.origin,
                    deprecated: match token.deprecated {
                        Status::Active => None,
                        Status::Deprecated(reason) => Some(Deprecation {
                            reason: reason.map(str::to_owned),
                        }),
                    },
                    references: result.references,
                },
            );
        }
        finish(Resolved { tokens }, graph.diags)
    }

    #[must_use]
    pub fn get(&self, path: &str) -> Option<&ResolvedToken> {
        self.tokens.get(path)
    }

    /// Tokens in composed order.
    #[must_use]
    pub fn iter(&self) -> indexmap::map::Iter<'_, TokenPath, ResolvedToken> {
        self.tokens.iter()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
}

impl<'a> IntoIterator for &'a Resolved {
    type Item = (&'a TokenPath, &'a ResolvedToken);
    type IntoIter = indexmap::map::Iter<'a, TokenPath, ResolvedToken>;
    fn into_iter(self) -> Self::IntoIter {
        self.tokens.iter()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Status<'d> {
    Active,
    Deprecated(Option<&'d str>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GroupType {
    None,
    Type(TokenType),
    /// The nearest group `$type` is invalid (already reported).
    Invalid,
}

/// What a token inherits from its groups.
#[derive(Clone, Copy)]
struct Scope<'d> {
    ty: GroupType,
    deprecated: Status<'d>,
}

impl Scope<'_> {
    const ROOT: Scope<'static> = Scope {
        ty: GroupType::None,
        deprecated: Status::Active,
    };
}

struct Token<'d> {
    path: TokenPath,
    definition: &'d Map<String, Value>,
    value: &'d Value,
    explicit: Option<TokenType>,
    group_type: GroupType,
    deprecated: Status<'d>,
    origin: Option<LayerId>,
    /// A structural error makes the token's type or value unusable.
    broken: bool,
}

/// Worklist state shared by type inference and resolution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mark<T> {
    Unknown,
    Visiting,
    Done(T),
    Failed,
}

/// The top-level shape of a `$value`, for type inference.
enum Head {
    Literal,
    /// A whole-token reference (brace alias, or pointer to `…/$value`).
    Whole(usize),
    /// A pointer to anything other than a whole token value.
    Partial,
    Invalid(Rule, String),
}

/// Where a pointer lands in the expanded document.
enum Located {
    /// Inside or at token `id`, whose path spans the first `end` segments.
    Token {
        id: usize,
        end: usize,
    },
    /// The document root or a group object.
    Group,
    /// Group metadata (`$type`, `$extensions`, …).
    Metadata,
    Missing,
}

struct Resolution {
    json: Value,
    value: TypedValue,
    /// First-seen order, no duplicates.
    references: Vec<TokenPath>,
}

struct Graph<'d> {
    doc: &'d Map<String, Value>,
    expanded: &'d Expanded,
    location: &'d str,
    labels: HashMap<LayerId, String>,
    tokens: Vec<Token<'d>>,
    index: HashMap<TokenPath, usize>,
    diags: Vec<Diagnostic>,
}

fn non_blank(v: &Value) -> bool {
    v.as_str().is_some_and(|s| !s.trim().is_empty())
}

fn token_type(v: &Value) -> Option<TokenType> {
    v.as_str()?.parse().ok()
}

fn deprecation(v: &Value) -> Option<Status<'_>> {
    match v {
        Value::Bool(false) => Some(Status::Active),
        Value::Bool(true) => Some(Status::Deprecated(None)),
        Value::String(s) if !s.trim().is_empty() => Some(Status::Deprecated(Some(s))),
        _ => None,
    }
}

/// D2: `[a-z][a-z0-9]*(-[a-z0-9]+)*`.
fn valid_name(name: &str) -> bool {
    name.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && name.split('-').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        })
}

fn is_whole_value(rest: &[String]) -> bool {
    matches!(rest, [v] if v == "$value")
}

fn shown(segs: &[String]) -> String {
    let mut out = "#".to_owned();
    for seg in segs {
        out.push('/');
        out.push_str(seg);
    }
    out
}

impl<'d> Graph<'d> {
    fn diagnostic(
        &self,
        rule: Rule,
        path: &str,
        origin: Option<LayerId>,
        message: impl Into<String>,
    ) -> Diagnostic {
        let location = if path.is_empty() {
            self.location.to_owned()
        } else {
            format!("{}: {path}", self.location)
        };
        let mut message = message.into();
        if let Some(label) = origin.and_then(|o| self.labels.get(&o)) {
            let _ = write!(message, " (defined by {label})");
        }
        Diagnostic::error(rule, location, message)
    }

    fn report(
        &mut self,
        rule: Rule,
        path: &str,
        origin: Option<LayerId>,
        message: impl Into<String>,
    ) {
        let d = self.diagnostic(rule, path, origin, message);
        self.diags.push(d);
    }

    fn report_token(&mut self, rule: Rule, id: usize, message: impl Into<String>) {
        let token = &self.tokens[id];
        let d = self.diagnostic(rule, &token.path, token.origin, message);
        self.diags.push(d);
    }

    // ----- Structure (D1–D5) -------------------------------------------

    fn group(&mut self, group: &'d Map<String, Value>, path: &mut String, inherited: Scope<'d>) {
        let mut scope = inherited;
        for (key, value) in group {
            if is_child_key(key) {
                continue;
            }
            match key.as_str() {
                "$type" => {
                    scope.ty = if let Some(ty) = token_type(value) {
                        GroupType::Type(ty)
                    } else {
                        self.report(
                            Rule::Type,
                            path,
                            None,
                            format!("unsupported group $type {value}"),
                        );
                        GroupType::Invalid
                    };
                }
                "$description" => {
                    if !non_blank(value) {
                        self.report(
                            Rule::Description,
                            path,
                            None,
                            "a group $description must be a non-blank string",
                        );
                    }
                }
                "$deprecated" => match deprecation(value) {
                    Some(status) => scope.deprecated = status,
                    None => self.report(
                        Rule::Structure,
                        path,
                        None,
                        "$deprecated must be a boolean or a non-blank explanation",
                    ),
                },
                "$extensions" => {
                    if !value.is_object() {
                        self.report(Rule::Structure, path, None, "$extensions must be an object");
                    }
                }
                "$extends" | "$ref" => {}
                other => self.report(
                    Rule::Structure,
                    path,
                    None,
                    format!("unsupported group property {other}"),
                ),
            }
        }
        for (key, value) in group {
            if !is_child_key(key) {
                continue;
            }
            let len = path.len();
            if len > 0 {
                path.push('.');
            }
            path.push_str(key);
            if key != "$root" && !valid_name(key) {
                self.report(
                    Rule::Name,
                    path,
                    None,
                    format!(
                        "invalid name \"{key}\"; use lower-case hyphenated segments such as text-sm"
                    ),
                );
            }
            match value {
                Value::Object(child) if is_token(child) => self.token(child, path, scope),
                Value::Object(_) if key == "$root" => {
                    self.report(
                        Rule::Structure,
                        path,
                        None,
                        "$root must be a token with a $value",
                    );
                }
                Value::Object(child) => self.group(child, path, scope),
                _ => self.report(
                    Rule::Structure,
                    path,
                    None,
                    "expected a token or group object",
                ),
            }
            path.truncate(len);
        }
    }

    fn token(&mut self, definition: &'d Map<String, Value>, path: &str, scope: Scope<'d>) {
        let origin = self.expanded.origin(path);
        let mut explicit = None;
        let mut broken = false;
        let mut deprecated = scope.deprecated;
        for (key, value) in definition {
            match key.as_str() {
                "$value" | "$description" => {}
                "$type" => {
                    explicit = token_type(value);
                    if explicit.is_none() {
                        self.report(
                            Rule::Type,
                            path,
                            origin,
                            format!("unsupported $type {value}"),
                        );
                        broken = true;
                    }
                }
                "$deprecated" => match deprecation(value) {
                    Some(status) => deprecated = status,
                    None => self.report(
                        Rule::Structure,
                        path,
                        origin,
                        "$deprecated must be a boolean or a non-blank explanation",
                    ),
                },
                "$extensions" => {
                    if !value.is_object() {
                        self.report(
                            Rule::Structure,
                            path,
                            origin,
                            "$extensions must be an object",
                        );
                    }
                }
                child if is_child_key(child) => {
                    self.report(
                        Rule::Structure,
                        path,
                        origin,
                        format!("a token cannot contain the child \"{child}\""),
                    );
                }
                other => self.report(
                    Rule::Structure,
                    path,
                    origin,
                    format!(
                        "unsupported token property {other}; allowed: {}",
                        TOKEN_PROPERTIES.join(" ")
                    ),
                ),
            }
        }
        if !definition.get("$description").is_some_and(non_blank) {
            self.report(
                Rule::Description,
                path,
                origin,
                "a token needs a non-blank $description",
            );
        }
        let namespace = path.split('.').next().unwrap_or_default();
        if !NAMESPACES.contains(&namespace) {
            self.report(
                Rule::Namespace,
                path,
                origin,
                format!(
                    "token paths start with {}, not \"{namespace}\"",
                    NAMESPACES.join(", ")
                ),
            );
        }
        let Some(value) = definition.get("$value") else {
            return;
        };
        let path = TokenPath::from(path);
        let id = self.tokens.len();
        self.index.entry(path.clone()).or_insert(id);
        self.tokens.push(Token {
            path,
            definition,
            value,
            explicit,
            group_type: scope.ty,
            deprecated,
            origin,
            broken,
        });
    }

    // ----- References ---------------------------------------------------

    /// Find what a pointer's segments name, walking child keys only.
    fn locate(&self, segs: &[String]) -> Located {
        let mut node = self.doc;
        for (i, seg) in segs.iter().enumerate() {
            if !is_child_key(seg) {
                return Located::Metadata;
            }
            match node.get(seg) {
                Some(Value::Object(child)) if is_token(child) => {
                    let path = segs[..=i].join(".");
                    return self
                        .index
                        .get(path.as_str())
                        .map_or(Located::Missing, |&id| Located::Token { id, end: i + 1 });
                }
                Some(Value::Object(child)) => node = child,
                _ => return Located::Missing,
            }
        }
        Located::Group
    }

    fn head(&self, id: usize) -> Head {
        match pointer::parse(self.tokens[id].value) {
            Err((rule, message)) => Head::Invalid(rule, message),
            Ok(None) => Head::Literal,
            Ok(Some(Reference::Alias(path))) => match self.index.get(path.as_str()) {
                Some(&target) => Head::Whole(target),
                None => Head::Invalid(Rule::ReferenceMissing, format!("{{{path}}} names no token")),
            },
            Ok(Some(Reference::Pointer(segs))) => match self.locate(&segs) {
                Located::Token { id, end } if is_whole_value(&segs[end..]) => Head::Whole(id),
                _ => Head::Partial,
            },
        }
    }

    // ----- Types (RF8) --------------------------------------------------

    fn infer_types(&mut self) -> Vec<Mark<TokenType>> {
        let heads: Vec<Head> = (0..self.tokens.len()).map(|id| self.head(id)).collect();
        let mut marks = vec![Mark::Unknown; self.tokens.len()];
        let mut stack = Vec::new();
        for start in 0..self.tokens.len() {
            if marks[start] != Mark::Unknown {
                continue;
            }
            stack.push(start);
            while let Some(&t) = stack.last() {
                match marks[t] {
                    Mark::Done(_) | Mark::Failed => {
                        stack.pop();
                    }
                    Mark::Visiting => {
                        // Its whole-token target has settled.
                        marks[t] = match heads[t] {
                            Head::Whole(target) => match marks[target] {
                                Mark::Done(ty) => Mark::Done(ty),
                                _ => Mark::Failed,
                            },
                            _ => Mark::Failed,
                        };
                    }
                    Mark::Unknown => self.type_step(t, &heads, &mut marks, &mut stack),
                }
            }
        }
        marks
    }

    fn type_step(
        &mut self,
        t: usize,
        heads: &[Head],
        marks: &mut [Mark<TokenType>],
        stack: &mut Vec<usize>,
    ) {
        let token = &self.tokens[t];
        if token.broken {
            marks[t] = Mark::Failed;
            return;
        }
        if let Some(ty) = token.explicit {
            marks[t] = Mark::Done(ty);
            return;
        }
        let group_type = token.group_type;
        match &heads[t] {
            Head::Whole(target) => {
                marks[t] = Mark::Visiting;
                match marks[*target] {
                    Mark::Unknown => stack.push(*target),
                    Mark::Visiting => self.cycle(stack, *target, marks),
                    Mark::Done(_) | Mark::Failed => {}
                }
            }
            Head::Invalid(rule, message) => {
                self.report_token(*rule, t, message.clone());
                marks[t] = Mark::Failed;
            }
            Head::Literal | Head::Partial => {
                marks[t] = match group_type {
                    GroupType::Type(ty) => Mark::Done(ty),
                    GroupType::Invalid => Mark::Failed,
                    GroupType::None => {
                        self.report_token(
                            Rule::TypeUndetermined,
                            t,
                            "no explicit $type, whole-token alias or group $type determines the type",
                        );
                        Mark::Failed
                    }
                };
            }
        }
    }

    /// Report the cycle closing at `target` (on `stack`) once and fail its
    /// members; tokens below them fail silently when revisited.
    fn cycle<T: Copy>(&mut self, stack: &mut Vec<usize>, target: usize, marks: &mut [Mark<T>]) {
        let start = stack.iter().position(|&s| s == target).unwrap_or(0);
        let chain: Vec<&str> = stack[start..]
            .iter()
            .chain(std::iter::once(&target))
            .map(|&id| &*self.tokens[id].path)
            .collect();
        let message = format!("reference cycle {}", chain.join(" -> "));
        for &member in &stack[start..] {
            marks[member] = Mark::Failed;
        }
        stack.truncate(start);
        self.report_token(Rule::ReferenceCycle, target, message);
    }

    // ----- Resolution ---------------------------------------------------

    fn resolve_all(&mut self, types: &[Mark<TokenType>]) -> Vec<Option<Resolution>> {
        let mut marks: Vec<Mark<()>> = vec![Mark::Unknown; self.tokens.len()];
        let mut results: Vec<Option<Resolution>> = (0..self.tokens.len()).map(|_| None).collect();
        let mut memo = Memo::new(self.tokens.len());
        let mut stack = Vec::new();
        for start in 0..self.tokens.len() {
            if marks[start] != Mark::Unknown {
                continue;
            }
            marks[start] = Mark::Visiting;
            stack.push(start);
            while let Some(&t) = stack.last() {
                let Mark::Done(ty) = types[t] else {
                    // Untyped: already reported, or a dependant of a failure.
                    marks[t] = Mark::Failed;
                    memo.failures += 1;
                    stack.pop();
                    continue;
                };
                if marks[t] == Mark::Failed {
                    // Failed as a member of a property cycle found elsewhere.
                    stack.pop();
                    continue;
                }
                match Walker::new(self, &mut memo, t, ty, types, &marks, &results).run() {
                    Walk::Need(target) if marks[target] == Mark::Visiting => {
                        self.cycle(&mut stack, target, &mut marks);
                        memo.failures += 1;
                    }
                    Walk::Need(target) => {
                        marks[target] = Mark::Visiting;
                        stack.push(target);
                    }
                    Walk::Failed { errors, members } => {
                        for (at, rule, message) in errors {
                            self.report_token(rule, at, message);
                        }
                        for member in members {
                            marks[member] = Mark::Failed;
                        }
                        marks[t] = Mark::Failed;
                        memo.failures += 1;
                        stack.pop();
                    }
                    Walk::Silent => {
                        marks[t] = Mark::Failed;
                        memo.failures += 1;
                        stack.pop();
                    }
                    Walk::Done(json, references) => {
                        match TypedValue::parse(ty, &json) {
                            Ok(value) => {
                                results[t] = Some(Resolution {
                                    json,
                                    value,
                                    references,
                                });
                                marks[t] = Mark::Done(());
                            }
                            Err(message) => {
                                self.report_token(
                                    Rule::Value,
                                    t,
                                    format!("invalid {ty}: {message}"),
                                );
                                marks[t] = Mark::Failed;
                                memo.failures += 1;
                            }
                        }
                        stack.pop();
                    }
                }
            }
        }
        results
    }
}

enum Walk {
    /// The substituted value and its references in first-seen order.
    Done(Value, Vec<TokenPath>),
    /// The value needs this token's resolved JSON first.
    Need(usize),
    Failed {
        /// Token to report at, rule, message.
        errors: Vec<(usize, Rule, String)>,
        /// Tokens that fail with this one (property-cycle members).
        members: Vec<usize>,
    },
    /// A dependency failed and was reported elsewhere.
    Silent,
}

enum Task<'d> {
    /// Produce the substituted value of `node` at use-site field path `at`.
    Visit(&'d Value, Vec<String>),
    Object(Vec<&'d str>),
    Array(usize),
    /// Leave an in-place pointer target.
    Leave,
}

enum Item {
    /// A referenced token: its id and path.
    Token(usize, TokenPath),
    /// Every reference of [`Memo`] entry `n`, in its order.
    Entry(usize),
}

/// A node selected by a pointer into a token's `$value` and resolved in
/// place without error, failed dependency or pending target.
struct Entry {
    json: Value,
    /// The token owning the node; the entry is silent once it fails.
    owner: usize,
    /// The node's references in first-seen order: these items, then
    /// `tail`'s. A trailing entry is kept apart so that taking a chain of
    /// entries needs no stack.
    items: Vec<Item>,
    tail: Option<usize>,
    /// When every item is a token and there is no tail or the tail's span
    /// ended where this one's tokens start: the same references, possibly
    /// repeated, as `Memo::chains[start..end]` read backwards.
    span: Option<(usize, usize)>,
    /// `Memo::failures` when no in-place owner inside the entry had failed;
    /// [`UNVERIFIED`] when that is unknown.
    verified: Cell<usize>,
    /// The last walker run that took the entry's references.
    run: Cell<usize>,
}

const UNVERIFIED: usize = usize::MAX;

/// In-place pointer targets already resolved, shared by every walker of
/// one resolution, so each node of a property-pointer chain is walked once
/// per use-site expectation instead of once per token upstream of it.
///
/// Reusing an entry is exact: the walk of a node depends only on the node,
/// the use-site expectations (walker type and field path, the key), the
/// settled types, the results of whole-token targets (an entry has all of
/// them) and the failure marks of in-place owners (checked on reuse unless
/// no token failed since the last check). An entry reaches no property
/// cycle, so the walk does not depend on the locations open around it.
struct Memo {
    entries: Vec<Entry>,
    /// (pointer location, walker type, use-site field path) → entry.
    index: HashMap<(Vec<String>, TokenType, Vec<String>), usize>,
    /// Token items of chains of entries, innermost entry first and each
    /// entry's items reversed, so an entry's span extends its tail's.
    chains: Vec<(usize, TokenPath)>,
    /// Grows whenever resolution fails a token.
    failures: usize,
    /// The current walker run; tokens it has already referenced carry it.
    run: usize,
    token_run: Vec<usize>,
}

impl Memo {
    fn new(tokens: usize) -> Self {
        Memo {
            entries: Vec::new(),
            index: HashMap::new(),
            chains: Vec::new(),
            failures: 0,
            run: 0,
            token_run: vec![0; tokens],
        }
    }

    /// The span of a new entry with `items` and `tail`, if it can extend
    /// the tail's span (or start one).
    fn span(&mut self, items: &[Item], tail: Option<usize>) -> Option<(usize, usize)> {
        let start = match tail {
            None => self.chains.len(),
            Some(tail) => match self.entries[tail].span {
                Some((start, end)) if end == self.chains.len() => start,
                _ => return None,
            },
        };
        if !items.iter().all(|item| matches!(item, Item::Token(..))) {
            return None;
        }
        for item in items.iter().rev() {
            if let Item::Token(id, path) = item {
                self.chains.push((*id, path.clone()));
            }
        }
        Some((start, self.chains.len()))
    }
}

/// An in-place pointer target being walked.
struct Frame {
    location: Vec<String>,
    at: Vec<String>,
    owner: usize,
    items: Vec<Item>,
    /// [`Walker::taint`] when the frame opened.
    taint: usize,
}

/// Materializes one token's `$value` with an explicit task stack, so
/// pointer chains of any length never recurse.
struct Walker<'g, 'd> {
    graph: &'g Graph<'d>,
    memo: &'g mut Memo,
    id: usize,
    ty: TokenType,
    types: &'g [Mark<TokenType>],
    marks: &'g [Mark<()>],
    results: &'g [Option<Resolution>],
    tasks: Vec<Task<'d>>,
    out: Vec<Value>,
    /// In-place pointer targets being walked, outermost first.
    frames: Vec<Frame>,
    /// The frame of each open location (property-cycle detection).
    open: HashMap<Vec<String>, usize>,
    /// Owners in first-seen order; `memo.token_run` marks members.
    references: Vec<TokenPath>,
    errors: Vec<(usize, Rule, String)>,
    members: Vec<usize>,
    need: Option<usize>,
    silent: bool,
    /// Failed or pending dependencies met so far.
    blocked: usize,
}

impl<'g, 'd> Walker<'g, 'd> {
    fn new(
        graph: &'g Graph<'d>,
        memo: &'g mut Memo,
        id: usize,
        ty: TokenType,
        types: &'g [Mark<TokenType>],
        marks: &'g [Mark<()>],
        results: &'g [Option<Resolution>],
    ) -> Self {
        memo.run += 1;
        Walker {
            graph,
            memo,
            id,
            ty,
            types,
            marks,
            results,
            tasks: vec![Task::Visit(graph.tokens[id].value, Vec::new())],
            out: Vec::new(),
            frames: Vec::new(),
            open: HashMap::new(),
            references: Vec::new(),
            errors: Vec::new(),
            members: Vec::new(),
            need: None,
            silent: false,
            blocked: 0,
        }
    }

    fn run(mut self) -> Walk {
        while let Some(task) = self.tasks.pop() {
            match task {
                Task::Visit(node, at) => self.visit(node, at),
                Task::Object(keys) => {
                    let values = self.out.split_off(self.out.len() - keys.len());
                    let object = keys.into_iter().map(str::to_owned).zip(values).collect();
                    self.out.push(Value::Object(object));
                }
                Task::Array(len) => {
                    let items = self.out.split_off(self.out.len() - len);
                    self.out.push(Value::Array(items));
                }
                Task::Leave => self.leave(),
            }
        }
        if !self.errors.is_empty() {
            Walk::Failed {
                errors: self.errors,
                members: self.members,
            }
        } else if self.silent {
            Walk::Silent
        } else if let Some(target) = self.need {
            Walk::Need(target)
        } else {
            Walk::Done(self.out.pop().unwrap_or(Value::Null), self.references)
        }
    }

    /// Grows with every error and every failed or pending dependency, so a
    /// frame is clean when it is unchanged on leaving.
    fn taint(&self) -> usize {
        self.errors.len() + self.blocked
    }

    fn expected(&self, at: &[String]) -> Option<TokenType> {
        if at.is_empty() {
            Some(self.ty)
        } else {
            field_type(self.ty, at)
        }
    }

    fn fail(&mut self, rule: Rule, message: String) {
        self.errors.push((self.id, rule, message));
        self.out.push(Value::Null);
    }

    /// A dependency failed and was reported elsewhere.
    fn dependency_failed(&mut self) {
        self.silent = true;
        self.blocked += 1;
        self.out.push(Value::Null);
    }

    fn reference(&mut self, id: usize) {
        let path = &self.graph.tokens[id].path;
        if let Some(frame) = self.frames.last_mut() {
            frame.items.push(Item::Token(id, path.clone()));
        }
        let seen = &mut self.memo.token_run[id];
        if *seen != self.memo.run {
            *seen = self.memo.run;
            self.references.push(path.clone());
        }
    }

    fn visit(&mut self, node: &'d Value, at: Vec<String>) {
        match pointer::parse(node) {
            Err((rule, message)) => self.fail(rule, message),
            Ok(Some(Reference::Alias(path))) => match self.graph.index.get(path.as_str()) {
                Some(&target) => self.whole(target, &at),
                None => self.fail(
                    Rule::ReferenceMissing,
                    format!("{{{path}}} names no token; a group or property is not a token alias"),
                ),
            },
            Ok(Some(Reference::Pointer(segs))) => self.pointer(segs, at),
            Ok(None) => match node {
                Value::Object(map) => {
                    self.tasks
                        .push(Task::Object(map.keys().map(String::as_str).collect()));
                    for (key, child) in map.iter().rev() {
                        let mut child_at = at.clone();
                        child_at.push(key.clone());
                        self.tasks.push(Task::Visit(child, child_at));
                    }
                }
                Value::Array(items) => {
                    self.tasks.push(Task::Array(items.len()));
                    for (i, child) in items.iter().enumerate().rev() {
                        let mut child_at = at.clone();
                        child_at.push(i.to_string());
                        self.tasks.push(Task::Visit(child, child_at));
                    }
                }
                scalar => self.out.push(scalar.clone()),
            },
        }
    }

    /// A reference to token `target`'s whole `$value`.
    fn whole(&mut self, target: usize, at: &[String]) {
        self.reference(target);
        let Mark::Done(found) = self.types[target] else {
            self.dependency_failed();
            return;
        };
        if let Some(expected) = self.expected(at)
            && expected != found
        {
            let message = format!(
                "{{{}}} is a {found} token where a {expected} is expected",
                self.graph.tokens[target].path
            );
            self.fail(Rule::ReferenceType, message);
            return;
        }
        if let Some(result) = &self.results[target] {
            self.out.push(result.json.clone());
            return;
        }
        if self.marks[target] == Mark::Failed {
            self.silent = true;
        } else if self.need.is_none() {
            self.need = Some(target);
        }
        self.blocked += 1;
        self.out.push(Value::Null);
    }

    /// A JSON pointer into the expanded document (RF7).
    fn pointer(&mut self, segs: Vec<String>, at: Vec<String>) {
        let located = self.graph.locate(&segs);
        if let Located::Token { id, end } = located
            && is_whole_value(&segs[end..])
        {
            self.whole(id, &at);
            return;
        }
        let Some(node) = pointer::select(self.graph.doc, &segs) else {
            let message = format!(
                "{} selects nothing; a pointer cannot reach into an alias",
                shown(&segs)
            );
            self.fail(Rule::ReferenceMissing, message);
            return;
        };
        match located {
            Located::Token { id, end } => {
                self.reference(id);
                match segs.get(end).map(String::as_str) {
                    None => {
                        let message = format!(
                            "{} selects a token definition; point at its /$value",
                            shown(&segs)
                        );
                        self.fail(Rule::ReferenceMissing, message);
                    }
                    Some("$value") => self.in_place(id, end + 1, node, segs, at),
                    // Token metadata is literal.
                    Some(_) => self.out.push(node.clone()),
                }
            }
            Located::Metadata => self.out.push(node.clone()),
            Located::Group => {
                let message = format!("{} selects a group, not a value", shown(&segs));
                self.fail(Rule::ReferenceMissing, message);
            }
            Located::Missing => {
                let message = format!("{} selects nothing usable", shown(&segs));
                self.fail(Rule::ReferenceMissing, message);
            }
        }
    }

    /// Resolve `node`, at `segs[field..]` inside token `owner`'s `$value`,
    /// in place at the use site `at`.
    fn in_place(
        &mut self,
        owner: usize,
        field: usize,
        node: &'d Value,
        segs: Vec<String>,
        at: Vec<String>,
    ) {
        let Mark::Done(owner_type) = self.types[owner] else {
            self.dependency_failed();
            return;
        };
        if self.marks[owner] == Mark::Failed {
            self.dependency_failed();
            return;
        }
        if let (Some(expected), Some(found)) =
            (self.expected(&at), field_type(owner_type, &segs[field..]))
            && expected != found
        {
            let message = format!(
                "{} is a {found} where a {expected} is expected",
                shown(&segs)
            );
            self.fail(Rule::ReferenceType, message);
            return;
        }
        if let Some(&start) = self.open.get(&segs) {
            let first = self.frames[start].owner;
            self.members
                .extend(self.frames[start..].iter().map(|frame| frame.owner));
            self.errors.push((
                first,
                Rule::ReferenceCycle,
                format!("property reference cycle through {}", shown(&segs)),
            ));
            self.out.push(Value::Null);
            return;
        }
        let key = (segs, self.ty, at);
        if let Some(&entry) = self.memo.index.get(&key) {
            self.reuse(entry);
            return;
        }
        let (location, _, at) = key;
        self.open.insert(location.clone(), self.frames.len());
        self.frames.push(Frame {
            location,
            at: at.clone(),
            owner,
            items: Vec::new(),
            taint: self.taint(),
        });
        self.tasks.push(Task::Leave);
        self.tasks.push(Task::Visit(node, at));
    }

    /// Close the innermost in-place target; remember it when it is clean.
    fn leave(&mut self) {
        let Some(frame) = self.frames.pop() else {
            return;
        };
        self.open.remove(&frame.location);
        if self.taint() != frame.taint {
            return;
        }
        let Some(json) = self.out.last() else {
            return;
        };
        let mut items = frame.items;
        let tail = if let Some(&Item::Entry(last)) = items.last() {
            items.pop();
            Some(last)
        } else {
            None
        };
        let span = self.memo.span(&items, tail);
        let entry = self.memo.entries.len();
        self.memo.entries.push(Entry {
            json: json.clone(),
            owner: frame.owner,
            items,
            tail,
            span,
            // A silent walk may have skipped a failed owner inside.
            verified: Cell::new(if self.silent {
                UNVERIFIED
            } else {
                self.memo.failures
            }),
            // Its references are already this run's.
            run: Cell::new(self.memo.run),
        });
        self.memo
            .index
            .insert((frame.location, self.ty, frame.at), entry);
        if let Some(parent) = self.frames.last_mut() {
            parent.items.push(Item::Entry(entry));
        }
    }

    /// Take a remembered in-place target: its references in order, unless
    /// an owner inside it has failed since (then fail silently, as walking
    /// it would).
    fn reuse(&mut self, entry: usize) {
        let run = self.memo.run;
        let Memo {
            entries,
            chains,
            failures,
            token_run,
            ..
        } = &mut *self.memo;
        let mut items = [].iter();
        let mut tail = Some(entry);
        // Where to resume the entries being taken, innermost last.
        let mut resume = Vec::new();
        // Entries whose owners were checked here.
        let mut checked = Vec::new();
        let mut failed = false;
        loop {
            let (next, nested) = match items.next() {
                Some(Item::Token(id, path)) => {
                    let seen = &mut token_run[*id];
                    if *seen != run {
                        *seen = run;
                        self.references.push(path.clone());
                    }
                    continue;
                }
                Some(Item::Entry(child)) => (*child, true),
                None => match tail.take() {
                    Some(last) => (last, false),
                    None => match resume.pop() {
                        Some(rest) => {
                            (items, tail) = rest;
                            continue;
                        }
                        None => break,
                    },
                },
            };
            let id = next;
            let next = &entries[id];
            if next.run.get() == run {
                continue;
            }
            if next.verified.get() == *failures
                && let Some((start, end)) = next.span
            {
                next.run.set(run);
                self.references.reserve(end - start);
                for (token, path) in chains[start..end].iter().rev() {
                    let seen = &mut token_run[*token];
                    if *seen != run {
                        *seen = run;
                        self.references.push(path.clone());
                    }
                }
                continue;
            }
            if matches!(self.marks[next.owner], Mark::Failed) {
                failed = true;
                break;
            }
            next.run.set(run);
            checked.push(id);
            if nested {
                resume.push((items, tail));
            }
            (items, tail) = (next.items.iter(), next.tail);
        }
        if failed {
            self.dependency_failed();
            return;
        }
        // Unless an earlier failure in this run left an entry it marked
        // taken half-checked, every owner inside the checked entries is
        // sound.
        if !self.silent {
            for id in checked {
                entries[id].verified.set(*failures);
            }
        }
        if let Some(frame) = self.frames.last_mut() {
            frame.items.push(Item::Entry(entry));
        }
        self.out.push(self.memo.entries[entry].json.clone());
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::needless_pass_by_value,
        reason = "test helpers take owned JSON for readable case tables"
    )]

    use std::path::PathBuf;

    use serde_json::json;

    use super::*;
    use crate::dtcg::tree::LayerKind;
    use crate::dtcg::value::Shadow;

    fn doc(v: Value) -> Map<String, Value> {
        match v {
            Value::Object(m) => m,
            _ => panic!("not an object"),
        }
    }

    fn ok(v: Value) -> Resolved {
        resolve(&doc(v), "test.json").unwrap_or_else(|f| panic!("{f}"))
    }

    fn rules(v: Value) -> Vec<Rule> {
        let failure = resolve(&doc(v), "test.json").expect_err("should fail");
        let mut rules: Vec<Rule> = failure.0.iter().map(|d| d.rule).collect();
        rules.sort_by_key(|r| r.id());
        rules
    }

    fn tok(ty: &str, value: Value) -> Value {
        json!({"$type": ty, "$value": value, "$description": "d"})
    }

    fn untyped(value: Value) -> Value {
        json!({"$value": value, "$description": "d"})
    }

    fn dim(v: f64) -> Value {
        json!({"value": v, "unit": "px"})
    }

    fn rgb(r: f64, g: f64, b: f64) -> Value {
        json!({"colorSpace": "srgb", "components": [r, g, b]})
    }

    fn json_of<'a>(r: &'a Resolved, path: &str) -> &'a Value {
        &r.get(path).unwrap_or_else(|| panic!("{path} missing")).json
    }

    fn refs(r: &Resolved, path: &str) -> Vec<String> {
        r.get(path)
            .unwrap()
            .references
            .iter()
            .map(ToString::to_string)
            .collect()
    }

    #[test]
    fn rf1_rf2_rf7_brace_pointer_and_property_pointers_resolve() {
        let r = ok(json!({
            "$extensions": {"org.example": {"a/b": {"c~d": {"x y": {"å": {"01": 0.25}}}}}},
            "primitive": {
                "base": tok("color", json!({"colorSpace": "srgb", "components": [0.1, 0.2, 0.3], "alpha": 0.5})),
                "space": tok("dimension", dim(8.0)),
                "ease": tok("cubicBezier", json!([0.1, 0.2, 0.3, 1.5])),
                "brace": untyped(json!("{primitive.base}")),
                "whole": untyped(json!({"$ref": "#/primitive/base/$value"})),
                "mixed": tok("color", json!({
                    "colorSpace": "srgb",
                    "components": [
                        {"$ref": "#/primitive/base/$value/components/2"},
                        {"$ref": "#/$extensions/org.example/a~1b/c~0d/x%20y/%C3%A5/01"},
                        1
                    ],
                    "alpha": {"$ref": "#/primitive/base/$value/alpha"}
                })),
                "copy": tok("dimension", json!({"value": {"$ref": "#/primitive/space/$value/value"}, "unit": "rem"})),
                "curve": tok("cubicBezier", json!([{"$ref": "#/primitive/ease/$value/0"}, 0, 1, {"$ref": "#/primitive/ease/$value/3"}])),
            }
        }));
        let base = json_of(&r, "primitive.base").clone();
        assert_eq!(r.get("primitive.brace").unwrap().ty, TokenType::Color);
        assert_eq!(json_of(&r, "primitive.brace"), &base);
        assert_eq!(r.get("primitive.whole").unwrap().ty, TokenType::Color);
        assert_eq!(json_of(&r, "primitive.whole"), &base);
        assert_eq!(
            json_of(&r, "primitive.mixed"),
            &json!({"colorSpace": "srgb", "components": [0.3, 0.25, 1], "alpha": 0.5})
        );
        assert_eq!(refs(&r, "primitive.mixed"), ["primitive.base"]);
        assert_eq!(
            json_of(&r, "primitive.copy"),
            &json!({"value": 8.0, "unit": "rem"})
        );
        assert_eq!(
            r.get("primitive.curve").unwrap().value,
            TypedValue::CubicBezier([0.1, 0.0, 1.0, 1.5])
        );
        assert_eq!(refs(&r, "primitive.curve"), ["primitive.ease"]);
        let order: Vec<&str> = r.iter().map(|(p, _)| &**p).collect();
        assert_eq!(
            order[..3],
            ["primitive.base", "primitive.space", "primitive.ease"]
        );
        assert_eq!(r.len(), 8);
    }

    #[test]
    fn rf7_pointer_cannot_reach_into_an_alias_string() {
        let found = rules(json!({"primitive": {
            "base": tok("color", rgb(0.1, 0.2, 0.3)),
            "alias": tok("color", json!("{primitive.base}")),
            "bad": tok("number", json!({"$ref": "#/primitive/alias/$value/components/0"})),
        }}));
        assert_eq!(found, [Rule::ReferenceMissing]);
    }

    #[test]
    fn rf7_metadata_targets_are_literal() {
        let r = ok(json!({"primitive": {
            "$extensions": {"org.x": {"weight": 450}},
            "w": tok("fontWeight", json!({"$ref": "#/primitive/$extensions/org.x/weight"})),
            "d": tok("fontFamily", json!({"$ref": "#/primitive/w/$description"})),
        }}));
        assert_eq!(json_of(&r, "primitive.w"), &json!(450));
        assert_eq!(json_of(&r, "primitive.d"), &json!("d"));
        assert_eq!(refs(&r, "primitive.d"), ["primitive.w"]);
    }

    #[test]
    fn rf4_self_mixed_and_property_cycles_fail_once() {
        let cases = [
            json!({"primitive": {"a": tok("number", json!("{primitive.a}"))}}),
            json!({"primitive": {"a": untyped(json!("{primitive.a}"))}}),
            json!({"primitive": {
                "a": tok("number", json!("{primitive.b}")),
                "b": tok("number", json!({"$ref": "#/primitive/c/$value"})),
                "c": untyped(json!("{primitive.a}")),
                "dependant": tok("number", json!("{primitive.c}")),
            }}),
            json!({"primitive": {
                "a": tok("dimension", json!({"value": {"$ref": "#/primitive/b/$value/value"}, "unit": "px"})),
                "b": tok("dimension", json!({"value": {"$ref": "#/primitive/a/$value/value"}, "unit": "px"})),
                "user": tok("number", json!({"$ref": "#/primitive/b/$value/value"})),
            }}),
            json!({"primitive": {
                "a": tok("dimension", json!({"value": {"$ref": "#/primitive/a/$value/value"}, "unit": "px"})),
            }}),
            json!({"primitive": {
                "a": tok("dimension", json!({"value": {"$ref": "#/primitive/b/$value/value"}, "unit": "px"})),
                "b": tok("dimension", json!({"value": "{primitive.n}", "unit": "px"})),
                "n": tok("number", json!({"$ref": "#/primitive/a/$value/value"})),
            }}),
        ];
        for case in cases {
            assert_eq!(rules(case.clone()), [Rule::ReferenceCycle], "{case}");
        }
    }

    #[test]
    fn rf4_l13_ten_thousand_link_chain_resolves() {
        // c0 → c1 → … → c9999: resolving c0 first walks the whole chain.
        let mut chain = Map::new();
        for i in 0..9_999 {
            let link = if i % 2 == 0 {
                json!(format!("{{primitive.chain.c{}}}", i + 1))
            } else {
                json!({"$ref": format!("#/primitive/chain/c{}/$value", i + 1)})
            };
            chain.insert(format!("c{i}"), untyped(link));
        }
        chain.insert("c9999".to_owned(), tok("number", json!(7)));
        let r = ok(json!({"primitive": {"chain": chain}}));
        let first = r.get("primitive.chain.c0").unwrap();
        assert_eq!(first.ty, TokenType::Number);
        assert_eq!(first.value, TypedValue::Number(7.0));
        assert_eq!(first.references.len(), 1);
        assert_eq!(r.len(), 10_000);
        let cyclic = json!({"primitive": {"a": tok("number", json!("{primitive.a}"))}});
        assert_eq!(rules(cyclic), [Rule::ReferenceCycle]);
    }

    #[test]
    fn rf8_type_precedence() {
        let r = ok(json!({
            "primitive": {
                "$type": "number",
                "dim": tok("dimension", dim(4.0)),
                "explicit": tok("fontWeight", json!(400)),
                "alias": untyped(json!("{primitive.dim}")),
                "literal": untyped(json!(5)),
                "partial": untyped(json!({"$ref": "#/primitive/dim/$value/value"})),
            },
            "semantic": {
                "base": {"$type": "duration", "x": untyped(json!({"value": 1, "unit": "s"}))},
                "derived": {"$extends": "{semantic.base}", "y": untyped(json!({"value": 3, "unit": "ms"}))},
            }
        }));
        let ty = |p: &str| r.get(p).unwrap().ty;
        assert_eq!(ty("primitive.explicit"), TokenType::FontWeight);
        assert_eq!(ty("primitive.alias"), TokenType::Dimension);
        assert_eq!(ty("primitive.literal"), TokenType::Number);
        assert_eq!(ty("primitive.partial"), TokenType::Number);
        assert_eq!(
            r.get("primitive.partial").unwrap().value,
            TypedValue::Number(4.0)
        );
        assert_eq!(ty("semantic.derived.y"), TokenType::Duration);
        assert_eq!(ty("semantic.derived.x"), TokenType::Duration);
    }

    #[test]
    fn rf8_l12_untyped_literals_and_property_pointers_fail() {
        let found = rules(json!({"primitive": {
            "dim": tok("dimension", dim(4.0)),
            "literal": untyped(json!(5)),
            "partial": untyped(json!({"$ref": "#/primitive/dim/$value/value"})),
        }}));
        assert_eq!(found, [Rule::TypeUndetermined, Rule::TypeUndetermined]);
    }

    #[test]
    fn rf5_known_target_type_must_match_the_use_site() {
        let found = rules(json!({"primitive": {
            "dim": tok("dimension", dim(4.0)),
            "type": tok("typography", json!({
                "fontFamily": "Noto Sans", "fontSize": dim(16.0), "fontWeight": 400,
                "letterSpacing": dim(0.0), "lineHeight": 1.5
            })),
            "whole": tok("color", json!("{primitive.dim}")),
            "field": tok("dimension", json!({"value": "{primitive.dim}", "unit": "px"})),
            "property": tok("number", json!({"$ref": "#/primitive/type/$value/fontSize"})),
        }}));
        assert_eq!(found, [Rule::ReferenceType; 3]);
    }

    #[test]
    fn rf3_rf2_reference_syntax_and_external_fail() {
        let found = rules(json!({"primitive": {
            "two": tok("number", json!("{primitive.a}{primitive.b}")),
            "embedded": tok("fontFamily", json!("Noto {primitive.a}")),
            "extra": tok("number", json!({"$ref": "#/primitive/x/$value", "note": 1})),
            "external": tok("number", json!({"$ref": "other.json#/primitive/x/$value"})),
        }}));
        let mut expected = vec![Rule::ReferenceSyntax; 3];
        expected.push(Rule::ReferenceExternal);
        expected.sort_by_key(|r| r.id());
        assert_eq!(found, expected);
    }

    #[test]
    fn rf1_root_is_explicit_and_group_aliases_fail() {
        let space = json!({
            "$type": "dimension",
            "$root": untyped(dim(8.0)),
            "wide": untyped(dim(16.0)),
        });
        let r = ok(json!({"primitive": {
            "space": space,
            "a": untyped(json!("{primitive.space.$root}")),
            "b": untyped(json!({"$ref": "#/primitive/space/$root/$value"})),
        }}));
        assert_eq!(json_of(&r, "primitive.a"), &dim(8.0));
        assert_eq!(json_of(&r, "primitive.b"), &dim(8.0));
        assert!(r.get("primitive.space.$root").is_some());
        let found = rules(json!({"primitive": {
            "space": space,
            "brace": tok("dimension", json!("{primitive.space}")),
            "pointer": tok("dimension", json!({"$ref": "#/primitive/space"})),
            "definition": tok("dimension", json!({"$ref": "#/primitive/space/wide"})),
        }}));
        assert_eq!(found, [Rule::ReferenceMissing; 3]);
    }

    #[test]
    fn c7_extends_and_group_ref_merge_and_inherit_deprecation() {
        let r = ok(json!({"primitive": {"spacing": {
            "$type": "dimension",
            "regular": {
                "$deprecated": "use primitive.spacing.next",
                "$root": untyped(dim(8.0)),
                "wide": untyped(dim(16.0)),
                "nested": {"deep": untyped(dim(32.0)), "keep": untyped(dim(1.0))},
            },
            "compact": {
                "$extends": "{primitive.spacing.regular}",
                "$root": json!({"$value": dim(4.0), "$description": "c", "$deprecated": false}),
                "nested": {"deep": untyped(dim(2.0))},
            },
            "pointer": {"$ref": "#/primitive/spacing/compact"},
            "flag": {"$deprecated": true, "x": untyped(dim(1.0))},
        }}}));
        let get = |p: &str| r.get(p).unwrap_or_else(|| panic!("{p}"));
        assert_eq!(get("primitive.spacing.compact.$root").json, dim(4.0));
        assert_eq!(get("primitive.spacing.compact.wide").json, dim(16.0));
        assert_eq!(get("primitive.spacing.compact.nested.deep").json, dim(2.0));
        assert_eq!(get("primitive.spacing.compact.nested.keep").json, dim(1.0));
        assert_eq!(get("primitive.spacing.pointer.$root").json, dim(4.0));
        assert_eq!(get("primitive.spacing.pointer.nested.deep").json, dim(2.0));
        let reason = Some(Deprecation {
            reason: Some("use primitive.spacing.next".to_owned()),
        });
        assert_eq!(get("primitive.spacing.compact.wide").deprecated, reason);
        assert_eq!(get("primitive.spacing.compact.$root").deprecated, None);
        assert_eq!(
            get("primitive.spacing.flag.x").deprecated,
            Some(Deprecation { reason: None })
        );
    }

    #[test]
    fn c7_invalid_targets_and_cycles_fail() {
        let cases = [
            json!({"primitive": {"g": {"$extends": "{primitive.none}"}}}),
            json!({"primitive": {"t": tok("number", json!(1)), "g": {"$extends": "{primitive.t}"}}}),
            json!({"primitive": {"t": tok("number", json!(1)), "g": {"$extends": "#/primitive/t/$value"}}}),
            json!({"primitive": {"h": {"$type": "number"}, "g": {"$extends": "#/primitive/h/$type"}}}),
            json!({"primitive": {"h": {}, "g": {"$extends": "{primitive.h}", "$ref": "#/primitive/h"}}}),
            json!({"primitive": {"a": {"$extends": "{primitive.b}"}, "b": {"$ref": "#/primitive/a"}}}),
            json!({"primitive": {"a": {"$extends": "{primitive.a}"}}}),
        ];
        for case in cases {
            assert_eq!(rules(case.clone()), [Rule::Extends], "{case}");
        }
    }

    fn layer_obj() -> Value {
        json!({
            "color": "{primitive.ink}",
            "offsetX": dim(0.0), "offsetY": dim(1.0), "blur": dim(2.0), "spread": dim(0.0)
        })
    }

    #[test]
    fn v4_shadow_elements_and_arrays() {
        let r = ok(json!({"primitive": {
            "ink": tok("color", rgb(0.0, 0.0, 0.0)),
            "one": tok("shadow", layer_obj()),
            "many": tok("shadow", json!([layer_obj(), layer_obj()])),
            "elements": tok("shadow", json!([layer_obj(), "{primitive.one}", {"$ref": "#/primitive/many/$value/1"}])),
            "alias-array": tok("shadow", json!("{primitive.many}")),
            "inferred": untyped(json!("{primitive.many}")),
        }}));
        let layers = |p: &str| match &r.get(p).unwrap().value {
            TypedValue::Shadow(Shadow::Layers(l)) => l.len(),
            TypedValue::Shadow(Shadow::Single(_)) => 1,
            other => panic!("{other:?}"),
        };
        assert_eq!(layers("primitive.elements"), 3);
        assert_eq!(layers("primitive.alias-array"), 2);
        assert_eq!(layers("primitive.inferred"), 2);
        assert!(matches!(
            r.get("primitive.one").unwrap().value,
            TypedValue::Shadow(Shadow::Single(_))
        ));
        let found = rules(json!({"primitive": {
            "ink": tok("color", rgb(0.0, 0.0, 0.0)),
            "many": tok("shadow", json!([layer_obj(), layer_obj()])),
            "nested": tok("shadow", json!([[layer_obj()]])),
            "element-array": tok("shadow", json!([layer_obj(), "{primitive.many}"])),
            "empty": tok("shadow", json!([])),
        }}));
        assert_eq!(found, [Rule::Value; 3]);
    }

    #[test]
    fn d1_d5_structure_violations_are_all_reported() {
        let found = rules(json!({
            "other": {"x": tok("number", json!(1))},
            "primitive": {
                "$bogus": 1,
                "Bad": tok("number", json!(1)),
                "nodesc": {"$type": "number", "$value": 1},
                "badtype": tok("colour", json!(1)),
                "unknown": {"$type": "number", "$value": 1, "$description": "d", "$foo": 1},
                "child": {"$type": "number", "$value": 1, "$description": "d", "kid": {}},
                "scalar": 3,
                "group": {"$root": {"x": tok("number", json!(1))}},
                "blank": {"$description": " ", "y": tok("number", json!(1))},
                "dep": {"$deprecated": "", "z": tok("number", json!(1))},
            }
        }));
        let mut expected = vec![
            Rule::Namespace,
            Rule::Structure,
            Rule::Name,
            Rule::Description,
            Rule::Type,
            Rule::Structure,
            Rule::Structure,
            Rule::Structure,
            Rule::Structure,
            Rule::Description,
            Rule::Structure,
        ];
        expected.sort_by_key(|r| r.id());
        assert_eq!(found, expected);
    }

    #[test]
    fn dependants_of_a_failed_token_are_skipped_silently() {
        let found = rules(json!({"primitive": {
            "a": tok("number", json!("{primitive.none}")),
            "b": tok("number", json!("{primitive.a}")),
            "c": untyped(json!({"$ref": "#/primitive/b/$value"})),
            "bad": tok("number", json!("text")),
            "user": tok("number", json!("{primitive.bad}")),
        }}));
        assert_eq!(found, [Rule::ReferenceMissing, Rule::Value]);
    }

    #[test]
    fn a_reused_in_place_target_whose_owner_failed_since_is_silent() {
        // `a` resolves `y`'s field in place before `x` fails; `c` takes the
        // same target after `x` failed, so it fails silently as a fresh
        // walk would, and `user` (which would reject 5) is skipped.
        let found = rules(json!({"primitive": {
            "a": tok("number", json!({"$ref": "#/primitive/y/$value/value"})),
            "x": tok("dimension", json!({"value": 5, "unit": "{primitive.none}"})),
            "c": tok("number", json!({"$ref": "#/primitive/y/$value/value"})),
            "user": tok("cubicBezier", json!(["{primitive.c}", 0, 1, 1])),
            "y": tok("dimension", json!({"value": {"$ref": "#/primitive/x/$value/value"}, "unit": "px"})),
        }}));
        assert_eq!(found, [Rule::ReferenceMissing]);
    }

    #[test]
    fn references_list_owners_in_first_seen_order() {
        let r = ok(json!({"primitive": {
            "font": tok("fontFamily", json!(["Noto Sans", "sans-serif"])),
            "size": tok("dimension", dim(16.0)),
            "weight": json!({"$type": "fontWeight", "$value": 400, "$description": "d", "$deprecated": true}),
            "ease": tok("cubicBezier", json!([0, 0, 1, 1.25])),
            "text": tok("typography", json!({
                "fontFamily": "{primitive.font}",
                "fontSize": {"$ref": "#/primitive/size/$value"},
                "fontWeight": {"$ref": "#/primitive/weight/$value"},
                "letterSpacing": {"value": {"$ref": "#/primitive/size/$value/value"}, "unit": "px"},
                "lineHeight": {"$ref": "#/primitive/ease/$value/3"},
            })),
        }}));
        assert_eq!(
            refs(&r, "primitive.text"),
            [
                "primitive.font",
                "primitive.size",
                "primitive.weight",
                "primitive.ease"
            ]
        );
        assert_eq!(
            r.get("primitive.weight").unwrap().deprecated,
            Some(Deprecation { reason: None })
        );
        assert_eq!(r.get("primitive.text").unwrap().deprecated, None);
    }

    #[test]
    fn inputs_are_not_mutated() {
        let input = doc(json!({"primitive": {
            "$type": "dimension",
            "base": {"a": untyped(dim(1.0))},
            "derived": {"$extends": "{primitive.base}", "b": untyped(json!({"value": {"$ref": "#/primitive/derived/a/$value/value"}, "unit": "px"}))},
        }}));
        let before = input.clone();
        let r = resolve(&input, "test.json").unwrap();
        assert_eq!(input, before);
        assert_eq!(r.get("primitive.derived.b").unwrap().json, dim(1.0));
        assert_eq!(
            r.get("primitive.derived.a").unwrap().definition,
            doc(untyped(dim(1.0)))
        );
    }

    #[test]
    fn c6_resolved_tokens_keep_their_origin() {
        let base = Layer {
            id: 0,
            kind: LayerKind::Base,
            file: PathBuf::from("tokens/base.tokens.json"),
            document: doc(
                json!({"primitive": {"$type": "number", "a": untyped(json!(1)), "g": {"x": untyped(json!(2))}}}),
            ),
        };
        let theme = Layer {
            id: 1,
            kind: LayerKind::Theme,
            file: PathBuf::from("tokens/themes/fern.tokens.json"),
            document: doc(
                json!({"primitive": {"a": untyped(json!(3)), "h": {"$extends": "{primitive.g}"}}}),
            ),
        };
        let mut tree = Tree::new();
        tree.apply(&base);
        tree.apply(&theme);
        let expanded = tree.expand("composition").unwrap();
        let layers = [base, theme];
        let r = Resolved::from_expanded(&expanded, &layers, "composition").unwrap();
        assert_eq!(r.get("primitive.a").unwrap().origin, Some(1));
        assert_eq!(r.get("primitive.g.x").unwrap().origin, Some(0));
        assert_eq!(r.get("primitive.h.x").unwrap().origin, Some(1));
        assert_eq!(
            r.get("primitive.h.x").unwrap().value,
            TypedValue::Number(2.0)
        );
    }
}
