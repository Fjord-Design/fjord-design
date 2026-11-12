//! Strict JSON reading and numeric-aware comparison.

use std::fmt;
use std::path::Path;

use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Number, Value};

use crate::diagnostic::{Diagnostic, Failure, Outcome, Rule};

/// Display a path relative to `root` when inside it.
#[must_use]
pub fn display_path(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

struct Strict;

impl<'de> DeserializeSeed<'de> for Strict {
    type Value = Value;
    fn deserialize<D: serde::Deserializer<'de>>(self, d: D) -> Result<Value, D::Error> {
        d.deserialize_any(StrictVisitor)
    }
}

struct StrictVisitor;

impl<'de> Visitor<'de> for StrictVisitor {
    type Value = Value;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a JSON value")
    }
    fn visit_bool<E>(self, v: bool) -> Result<Value, E> {
        Ok(Value::Bool(v))
    }
    fn visit_i64<E>(self, v: i64) -> Result<Value, E> {
        Ok(Value::Number(v.into()))
    }
    fn visit_u64<E>(self, v: u64) -> Result<Value, E> {
        Ok(Value::Number(v.into()))
    }
    fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Value, E> {
        Number::from_f64(v)
            .map(Value::Number)
            .ok_or_else(|| E::custom("non-finite number"))
    }
    fn visit_str<E>(self, v: &str) -> Result<Value, E> {
        Ok(Value::String(v.to_owned()))
    }
    fn visit_string<E>(self, v: String) -> Result<Value, E> {
        Ok(Value::String(v))
    }
    fn visit_unit<E>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_none<E>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
        let mut out = Vec::new();
        while let Some(v) = seq.next_element_seed(Strict)? {
            out.push(v);
        }
        Ok(Value::Array(out))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
        let mut out = Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if out.contains_key(&key) {
                return Err(serde::de::Error::custom(format!("duplicate key \"{key}\"")));
            }
            let v = map.next_value_seed(Strict)?;
            out.insert(key, v);
        }
        Ok(Value::Object(out))
    }
}

/// Parse JSON text strictly (no duplicate keys, nothing after the value).
///
/// # Errors
/// Returns the serde error text, which includes line and column.
pub fn parse_strict(text: &str) -> Result<Value, String> {
    let mut de = serde_json::Deserializer::from_str(text);
    let value = Strict.deserialize(&mut de).map_err(|e| e.to_string())?;
    de.end().map_err(|e| e.to_string())?;
    Ok(value)
}

/// Read a strict UTF-8 JSON file.
///
/// # Errors
/// `Rule::Io` when unreadable or not UTF-8, `Rule::Json` when malformed.
pub fn read(path: &Path, root: &Path) -> Outcome<Value> {
    let location = display_path(path, root);
    let bytes = std::fs::read(path).map_err(|e| {
        Failure::one(Diagnostic::error(
            Rule::Io,
            &location,
            format!("cannot read file: {e}"),
        ))
    })?;
    let text = String::from_utf8(bytes).map_err(|e| {
        Failure::one(Diagnostic::error(
            Rule::Json,
            &location,
            format!("not valid UTF-8: {e}"),
        ))
    })?;
    parse_strict(&text).map_err(|e| {
        Failure::one(Diagnostic::error(
            Rule::Json,
            &location,
            format!("invalid JSON: {e}"),
        ))
    })
}

/// Structural equality where numbers compare numerically (`1 == 1.0`).
#[must_use]
pub fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => {
            if let (Some(x), Some(y)) = (x.as_i64(), y.as_i64()) {
                x == y
            } else if let (Some(x), Some(y)) = (x.as_u64(), y.as_u64()) {
                x == y
            } else {
                x.as_f64() == y.as_f64()
            }
        }
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(a, b)| same(a, b))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len() && x.iter().all(|(k, a)| y.get(k).is_some_and(|b| same(a, b)))
        }
        _ => a == b,
    }
}

/// Integral values without a decimal point; others shortest round-trip, no exponent; `-0` → `0`.
#[must_use]
pub fn number_text(v: f64) -> String {
    if v == 0.0 {
        return "0".to_owned();
    }
    format!("{v}")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn in1_duplicate_key_is_rejected() {
        assert!(
            parse_strict(r#"{"a":1,"a":2}"#)
                .unwrap_err()
                .contains("duplicate key")
        );
        assert!(parse_strict(r#"{"a":{"b":1,"b":1}}"#).is_err());
    }

    #[test]
    fn in1_trailing_data_is_rejected() {
        assert!(parse_strict("{} x").is_err());
    }

    #[test]
    fn same_compares_numbers_numerically() {
        let a = parse_strict(r#"{"v":[1,2.5]}"#).unwrap();
        let b = parse_strict(r#"{"v":[1.0,2.5]}"#).unwrap();
        assert!(same(&a, &b));
        assert!(!same(&a, &parse_strict(r#"{"v":[1,2]}"#).unwrap()));
    }

    #[test]
    fn number_text_is_canonical() {
        assert_eq!(number_text(15.0), "15");
        assert_eq!(number_text(-0.0), "0");
        assert_eq!(number_text(0.25), "0.25");
        assert_eq!(number_text(1e-7), "0.0000001");
    }
}
