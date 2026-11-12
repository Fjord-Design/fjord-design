//! Self-contained DTCG export (O1).

use serde_json::{Map, Value};

use super::metadata;
use crate::dtcg::tree::{is_child_key, is_token};
use crate::dtcg::value::TokenType;
use crate::policy::contract::EXTENSION;
use crate::project::Composition;

/// The expanded composed document: inherited tokens are written out and
/// `$extends` declarations stay, aliases and vendor `$extensions` are kept,
/// every token carries its effective `$type`, and the root
/// `$extensions.org.xerrion.fjord` is deep-merged with the export metadata.
#[must_use]
pub fn dtcg(c: &Composition) -> Value {
    let mut document = c.expanded_document().clone();
    stamp_types(&mut document, &mut String::new(), &|path| {
        c.get(path).map(|t| t.ty)
    });
    let mut root = Value::Object(document);
    let mut vendor = Map::new();
    vendor.insert(EXTENSION.to_owned(), Value::Object(metadata(c.mode())));
    let mut patch = Map::new();
    patch.insert("$extensions".to_owned(), Value::Object(vendor));
    deep_merge(&mut root, patch);
    root
}

/// Set `$type` on every token below `group` to its effective type: an
/// existing key keeps its position, otherwise `$type` becomes the first key.
fn stamp_types(
    group: &mut Map<String, Value>,
    path: &mut String,
    effective: &dyn Fn(&str) -> Option<TokenType>,
) {
    for (key, value) in group.iter_mut() {
        if !is_child_key(key) {
            continue;
        }
        let Value::Object(node) = value else { continue };
        let len = path.len();
        if len > 0 {
            path.push('.');
        }
        path.push_str(key);
        if is_token(node) {
            if let Some(ty) = effective(path) {
                let ty = Value::from(ty.as_str());
                match node.get_mut("$type") {
                    Some(slot) => *slot = ty,
                    None => {
                        node.shift_insert(0, "$type".to_owned(), ty);
                    }
                }
            }
        } else {
            stamp_types(node, path, effective);
        }
        path.truncate(len);
    }
}

/// Merge `patch` into `target`: objects merge recursively, other values
/// replace, existing keys keep their position and new keys are appended.
fn deep_merge(target: &mut Value, patch: Map<String, Value>) {
    if !target.is_object() {
        *target = Value::Object(Map::new());
    }
    let Value::Object(map) = target else { return };
    for (key, value) in patch {
        match value {
            Value::Object(sub) => deep_merge(map.entry(key).or_insert(Value::Null), sub),
            other => {
                map.insert(key, other);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::policy::contract::Mode;
    use serde_json::json;

    fn object(v: Value) -> Map<String, Value> {
        let Value::Object(m) = v else {
            panic!("not an object")
        };
        m
    }

    #[test]
    fn o1_effective_type_keeps_position_or_comes_first() {
        let mut doc = object(json!({
            "primitive": {
                "$type": "dimension",
                "space": {
                    "$root": {"$description": "d", "$value": {"value": 4, "unit": "px"}},
                    "wide": {"$description": "d", "$type": "dimension", "$value": {"value": 8, "unit": "px"}}
                },
                "$extensions": {"vendor.x": {"$value": 1}}
            }
        }));
        stamp_types(&mut doc, &mut String::new(), &|path| match path {
            "primitive.space.$root" | "primitive.space.wide" => Some(TokenType::Dimension),
            _ => None,
        });
        let root = &doc["primitive"]["space"]["$root"];
        assert_eq!(
            root.as_object().unwrap().keys().collect::<Vec<_>>(),
            ["$type", "$description", "$value"]
        );
        let wide = &doc["primitive"]["space"]["wide"];
        assert_eq!(
            wide.as_object().unwrap().keys().collect::<Vec<_>>(),
            ["$description", "$type", "$value"]
        );
        // Metadata objects are not children, so `$value` inside them is not a token.
        assert_eq!(
            doc["primitive"]["$extensions"],
            json!({"vendor.x": {"$value": 1}})
        );
    }

    #[test]
    fn o1_metadata_deep_merges_into_vendor_extensions() {
        let mut root = json!({
            "$extensions": {
                "org.xerrion.fjord": {"schemaVersion": 1, "name": "Fjord", "mode": "x"},
                "com.example": {"keep": true}
            },
            "semantic": {}
        });
        let mut vendor = Map::new();
        vendor.insert(EXTENSION.to_owned(), Value::Object(metadata(Mode::Day)));
        let mut patch = Map::new();
        patch.insert("$extensions".to_owned(), Value::Object(vendor));
        deep_merge(&mut root, patch);
        assert_eq!(
            root,
            json!({
                "$extensions": {
                    "org.xerrion.fjord": {
                        "schemaVersion": 2, "name": "Fjord", "mode": "day",
                        "contractVersion": "0.5.0", "dtcgVersion": "2025.10", "colorSpace": "srgb"
                    },
                    "com.example": {"keep": true}
                },
                "semantic": {}
            })
        );
        let keys: Vec<_> = root["$extensions"]["org.xerrion.fjord"]
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect();
        assert_eq!(
            keys,
            [
                "schemaVersion",
                "name",
                "mode",
                "contractVersion",
                "dtcgVersion",
                "colorSpace"
            ]
        );
    }

    #[test]
    fn o1_metadata_is_added_when_absent() {
        let mut root = json!({"semantic": {}});
        let mut patch = Map::new();
        patch.insert(
            "$extensions".to_owned(),
            json!({EXTENSION: {"mode": "night"}}),
        );
        deep_merge(&mut root, patch);
        assert_eq!(
            root,
            json!({"semantic": {}, "$extensions": {"org.xerrion.fjord": {"mode": "night"}}})
        );
    }
}
