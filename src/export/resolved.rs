//! Flat resolved JSON (O2).

use serde_json::{Map, Value};

use super::metadata;
use crate::dtcg::graph::ResolvedToken;
use crate::dtcg::tree::Layer;
use crate::project::Composition;

/// `{"metadata": {…}, "tokens": {path: {type, value, source, original}}}`
/// with paths in byte order. `value` is the authored value with every
/// reference substituted; `original` is the authored `$value`.
#[must_use]
pub fn resolved(c: &Composition) -> Value {
    let mut tokens: Vec<_> = c.tokens().collect();
    tokens.sort_unstable_by(|a, b| a.0.cmp(b.0));
    let layers = c.layers();
    let mut out = Map::new();
    for (path, token) in tokens {
        out.insert(path.to_string(), entry(token, layers));
    }
    let mut document = Map::new();
    document.insert("metadata".to_owned(), Value::Object(metadata(c.mode())));
    document.insert("tokens".to_owned(), Value::Object(out));
    Value::Object(document)
}

/// One token record. `layers` is indexed by `LayerId`.
fn entry(token: &ResolvedToken, layers: &[Layer]) -> Value {
    let source = token
        .origin
        .and_then(|id| layers.get(usize::from(id)))
        .map_or(Value::Null, |layer| Value::String(layer.source_label()));
    let mut record = Map::new();
    record.insert("type".to_owned(), token.ty.as_str().into());
    record.insert("value".to_owned(), token.json.clone());
    record.insert("source".to_owned(), source);
    record.insert(
        "original".to_owned(),
        token
            .definition
            .get("$value")
            .cloned()
            .unwrap_or(Value::Null),
    );
    Value::Object(record)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use std::path::PathBuf;

    use serde_json::json;

    use super::*;
    use crate::dtcg::tree::LayerKind;
    use crate::dtcg::value::{Color, TokenType, TypedValue};

    fn layer(id: u16, kind: LayerKind, file: &str) -> Layer {
        Layer {
            id,
            kind,
            file: PathBuf::from(file),
            document: Map::new(),
        }
    }

    #[test]
    fn o2_record_has_type_value_source_and_original() {
        let layers = [
            layer(0, LayerKind::Base, "/p/tokens/base.tokens.json"),
            layer(
                1,
                LayerKind::Mode("night"),
                "/p/tokens/modes/night.tokens.json",
            ),
            layer(2, LayerKind::Theme, "/p/tokens/themes/fern.tokens.json"),
            layer(3, LayerKind::Override(1), "/x/compact.tokens.json"),
        ];
        let original = json!({"$ref": "#/semantic/color/ink/$value"});
        let value = json!({"colorSpace": "srgb", "components": [0, 0, 0]});
        let Value::Object(definition) = json!({"$description": "d", "$value": original}) else {
            panic!()
        };
        let mut token = ResolvedToken {
            ty: TokenType::Color,
            definition,
            value: TypedValue::Color(Color {
                components: [0.0; 3],
                alpha: 1.0,
            }),
            json: value.clone(),
            origin: Some(2),
            deprecated: None,
            references: Vec::new(),
        };
        assert_eq!(
            entry(&token, &layers),
            json!({"type": "color", "value": value, "source": "theme:fern.tokens.json", "original": original})
        );
        let keys: Vec<_> = entry(&token, &layers)
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect();
        assert_eq!(keys, ["type", "value", "source", "original"]);

        let labels: Vec<Value> = (0..4)
            .map(|id| {
                token.origin = Some(id);
                entry(&token, &layers)["source"].clone()
            })
            .collect();
        assert_eq!(
            labels,
            [
                json!("base"),
                json!("mode:night"),
                json!("theme:fern.tokens.json"),
                json!("override:1:compact.tokens.json")
            ]
        );
    }
}
