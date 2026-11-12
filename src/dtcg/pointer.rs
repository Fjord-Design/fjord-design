//! Reference syntax (RF2, RF3): brace aliases and `{"$ref": "#…"}` pointers,
//! and RFC 6901 selection over the raw composed document.

use std::borrow::Cow;

use percent_encoding::percent_decode_str;
use serde_json::{Map, Value};

use crate::diagnostic::Rule;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reference {
    /// Dotted token path from `{…}`; may end in `.$root`.
    Alias(String),
    /// Decoded RFC 6901 segments; empty selects the whole document.
    Pointer(Vec<String>),
}

/// Parse a value as a reference.
///
/// Returns `Ok(None)` when the value is neither a reference string nor a
/// `$ref` object.
///
/// # Errors
/// Returns a rule and message for malformed or external references.
pub fn parse(v: &Value) -> Result<Option<Reference>, (Rule, String)> {
    match v {
        Value::String(s) if s.contains(['{', '}']) => alias(s).map(Some),
        Value::Object(m) if m.contains_key("$ref") => pointer_object(m).map(Some),
        _ => Ok(None),
    }
}

fn alias(s: &str) -> Result<Reference, (Rule, String)> {
    let inner = s
        .strip_prefix('{')
        .and_then(|rest| rest.strip_suffix('}'))
        .filter(|p| !p.contains(['{', '}']));
    let Some(path) = inner else {
        return Err((
            Rule::ReferenceSyntax,
            format!("\"{s}\" must be exactly one alias such as {{semantic.color.ink}}"),
        ));
    };
    let bad_segment = path
        .split('.')
        .any(|seg| seg.is_empty() || seg.chars().any(char::is_whitespace));
    if path.is_empty() || bad_segment {
        return Err((
            Rule::ReferenceSyntax,
            format!("alias \"{{{path}}}\" is not a dotted token path"),
        ));
    }
    Ok(Reference::Alias(path.to_owned()))
}

fn pointer_object(m: &Map<String, Value>) -> Result<Reference, (Rule, String)> {
    if m.len() != 1 {
        return Err((
            Rule::ReferenceSyntax,
            "a $ref object must have \"$ref\" as its only key".to_owned(),
        ));
    }
    let Some(Value::String(raw)) = m.get("$ref") else {
        return Err((
            Rule::ReferenceSyntax,
            "\"$ref\" must be a string".to_owned(),
        ));
    };
    let Some(fragment) = raw.strip_prefix('#') else {
        return Err((
            Rule::ReferenceExternal,
            format!("\"{raw}\" is external; only local \"#…\" pointers are allowed"),
        ));
    };
    let decoded = decode_fragment(fragment).map_err(|e| (Rule::ReferenceSyntax, e))?;
    pointer_segments(&decoded)
        .map(Reference::Pointer)
        .map_err(|e| (Rule::ReferenceSyntax, e))
}

/// URI-fragment percent-decoding: every `%` must start two hex digits, and
/// the decoded bytes must be UTF-8.
fn decode_fragment(fragment: &str) -> Result<String, String> {
    let bytes = fragment.as_bytes();
    let malformed = fragment.match_indices('%').any(|(at, _)| {
        !bytes
            .get(at + 1..at + 3)
            .is_some_and(|h| h.iter().all(u8::is_ascii_hexdigit))
    });
    if malformed {
        return Err(format!("\"#{fragment}\" has a malformed percent-encoding"));
    }
    percent_decode_str(fragment)
        .decode_utf8()
        .map(Cow::into_owned)
        .map_err(|_| format!("\"#{fragment}\" does not percent-decode to UTF-8"))
}

/// RFC 6901 segments. `~1` → `/`, `~0` → `~`, decoded once; any other `~` fails.
fn pointer_segments(pointer: &str) -> Result<Vec<String>, String> {
    if pointer.is_empty() {
        return Ok(Vec::new());
    }
    let Some(body) = pointer.strip_prefix('/') else {
        return Err(format!(
            "JSON pointer \"{pointer}\" must be empty or start with \"/\""
        ));
    };
    body.split('/').map(unescape_segment).collect()
}

fn unescape_segment(seg: &str) -> Result<String, String> {
    let mut out = String::with_capacity(seg.len());
    let mut chars = seg.chars();
    while let Some(c) = chars.next() {
        if c != '~' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('0') => out.push('~'),
            Some('1') => out.push('/'),
            _ => return Err(format!("segment \"{seg}\" has an invalid \"~\" escape")),
        }
    }
    Ok(out)
}

/// RFC 6901 canonical array index: digits only, no leading zero, in range.
fn canonical_index(seg: &str) -> Option<usize> {
    let digits = !seg.is_empty() && seg.bytes().all(|b| b.is_ascii_digit());
    if digits && (seg == "0" || !seg.starts_with('0')) {
        seg.parse().ok()
    } else {
        None
    }
}

/// Select a node of the raw composed document. Returns `None` for an empty
/// path, a missing key, a non-canonical or out-of-range index, or a walk
/// into a scalar.
#[must_use]
pub fn select<'a>(doc: &'a Map<String, Value>, segs: &[String]) -> Option<&'a Value> {
    let (first, rest) = segs.split_first()?;
    let mut node = doc.get(first)?;
    for seg in rest {
        node = match node {
            Value::Object(m) => m.get(seg)?,
            Value::Array(items) => items.get(canonical_index(seg)?)?,
            _ => return None,
        };
    }
    Some(node)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use rstest::rstest;
    use serde_json::json;

    use super::*;

    fn doc() -> Map<String, Value> {
        json!({
            "a": {"b~c": 1, "d/e": 2, "01": 3, "f g": 4, "%": 5, "å": 6},
            "list": [{"x": 7}, 8],
            "s": "{a.b}",
            "p": "#/a",
        })
        .as_object()
        .unwrap()
        .clone()
    }

    fn ptr(r: &str) -> Result<Option<Reference>, (Rule, String)> {
        parse(&json!({ "$ref": r }))
    }

    fn segs(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn rf2_tilde_escapes_decode_once() {
        assert_eq!(
            ptr("#/a/b~0c").unwrap(),
            Some(Reference::Pointer(segs(&["a", "b~c"])))
        );
        assert_eq!(
            ptr("#/a/d~1e").unwrap(),
            Some(Reference::Pointer(segs(&["a", "d/e"])))
        );
        // "~01" must decode to "~1", not "/".
        assert_eq!(
            ptr("#/a/~01").unwrap(),
            Some(Reference::Pointer(segs(&["a", "~1"])))
        );
        assert_eq!(select(&doc(), &segs(&["a", "b~c"])), Some(&json!(1)));
        assert_eq!(select(&doc(), &segs(&["a", "d/e"])), Some(&json!(2)));
    }

    #[rstest]
    #[case("#/a/b~2c")]
    #[case("#/a/b~")]
    #[case("#/~")]
    #[case("#/a/%7E2")]
    fn rf2_invalid_tilde_escapes_fail(#[case] r: &str) {
        let (rule, _) = ptr(r).unwrap_err();
        assert_eq!(rule, Rule::ReferenceSyntax);
    }

    #[test]
    fn rf2_percent_decoding_then_pointer_segments() {
        assert_eq!(
            ptr("#/a/f%20g").unwrap(),
            Some(Reference::Pointer(segs(&["a", "f g"])))
        );
        assert_eq!(
            ptr("#/a/%C3%A5").unwrap(),
            Some(Reference::Pointer(segs(&["a", "å"])))
        );
        // %2F decodes to "/" before the pointer is split, so it is a separator.
        assert_eq!(
            ptr("#/a/d%2Fe").unwrap(),
            Some(Reference::Pointer(segs(&["a", "d", "e"])))
        );
        assert_eq!(select(&doc(), &segs(&["a", "å"])), Some(&json!(6)));
    }

    #[rstest]
    #[case("#/a/%")]
    #[case("#/a/%2")]
    #[case("#/a/%G1")]
    #[case("#/a/%zz")]
    #[case("#/a/%2G")]
    fn rf2_malformed_percent_fails(#[case] r: &str) {
        let (rule, msg) = ptr(r).unwrap_err();
        assert_eq!(rule, Rule::ReferenceSyntax);
        assert!(msg.contains("malformed percent"), "{msg}");
    }

    #[test]
    fn rf2_percent_decoding_must_be_utf8() {
        let (rule, msg) = ptr("#/a/%FF").unwrap_err();
        assert_eq!(rule, Rule::ReferenceSyntax);
        assert!(msg.contains("UTF-8"), "{msg}");
    }

    #[rstest]
    #[case("file.json#/a")]
    #[case("https://example.com/t.json#/a")]
    #[case("/a/b")]
    fn rf2_external_forms_without_leading_hash(#[case] r: &str) {
        let (rule, _) = ptr(r).unwrap_err();
        assert_eq!(rule, Rule::ReferenceExternal);
    }

    #[test]
    fn rf2_file_reference_without_hash_is_external() {
        let (rule, msg) = ptr("tokens/base.json").unwrap_err();
        assert_eq!(rule, Rule::ReferenceExternal);
        assert!(msg.contains("external"), "{msg}");
    }

    #[test]
    fn rf2_pointer_must_start_with_slash() {
        let (rule, _) = ptr("#a/b").unwrap_err();
        assert_eq!(rule, Rule::ReferenceSyntax);
    }

    #[test]
    fn rf2_ref_must_be_only_key_and_a_string() {
        let extra = json!({"$ref": "#/a", "description": "x"});
        assert_eq!(parse(&extra).unwrap_err().0, Rule::ReferenceSyntax);
        let non_string = json!({"$ref": 3});
        assert_eq!(parse(&non_string).unwrap_err().0, Rule::ReferenceSyntax);
    }

    #[test]
    fn rf2_objects_without_ref_are_not_references() {
        assert_eq!(parse(&json!({"value": 1, "unit": "px"})).unwrap(), None);
        assert_eq!(parse(&json!(3)).unwrap(), None);
        assert_eq!(parse(&json!(["{a}"])).unwrap(), None);
        assert_eq!(parse(&json!("plain")).unwrap(), None);
    }

    #[rstest]
    #[case("{a.b}", "a.b")]
    #[case("{semantic.color.ink.$root}", "semantic.color.ink.$root")]
    #[case("{a-b.c_d}", "a-b.c_d")]
    fn rf3_exact_alias_parses(#[case] s: &str, #[case] path: &str) {
        assert_eq!(
            parse(&json!(s)).unwrap(),
            Some(Reference::Alias(path.to_owned()))
        );
    }

    #[rstest]
    #[case("{}")]
    #[case("{a}{b}")]
    #[case("x{a}")]
    #[case("{a}x")]
    #[case("{a")]
    #[case("a}")]
    #[case("{{a}}")]
    #[case("{a..b}")]
    #[case("{a.}")]
    #[case("{ a.b }")]
    #[case("{a. b}")]
    #[case("{.a}")]
    fn rf3_brace_strings_that_are_not_exactly_one_alias_fail(#[case] s: &str) {
        let (rule, _) = parse(&json!(s)).unwrap_err();
        assert_eq!(rule, Rule::ReferenceSyntax);
    }

    #[test]
    fn strings_without_braces_are_literals() {
        assert_eq!(parse(&json!("#ff0000")).unwrap(), None);
        assert_eq!(parse(&json!("Noto Sans")).unwrap(), None);
    }

    #[test]
    fn select_walks_objects_and_canonical_indexes() {
        assert_eq!(select(&doc(), &segs(&["list", "0", "x"])), Some(&json!(7)));
        assert_eq!(select(&doc(), &segs(&["list", "1"])), Some(&json!(8)));
        assert_eq!(select(&doc(), &segs(&["a", "01"])), Some(&json!(3)));
    }

    #[rstest]
    #[case(&["list", "01"])]
    #[case(&["list", "+0"])]
    #[case(&["list", "-"])]
    #[case(&["list", "2"])]
    #[case(&["list", ""])]
    #[case(&["list", "0x1"])]
    fn select_rejects_non_canonical_or_out_of_range_indexes(#[case] p: &[&str]) {
        assert_eq!(select(&doc(), &segs(p)), None);
    }

    #[test]
    fn select_does_not_reach_into_alias_strings() {
        assert_eq!(select(&doc(), &segs(&["s", "x"])), None);
        assert_eq!(select(&doc(), &segs(&["p", "0"])), None);
    }

    #[test]
    fn select_empty_path_and_missing_keys_are_none() {
        assert_eq!(select(&doc(), &[]), None);
        assert_eq!(select(&doc(), &segs(&["missing"])), None);
        assert_eq!(select(&doc(), &segs(&["a", "missing"])), None);
    }

    #[test]
    fn rf2_empty_pointer_selects_whole_document() {
        assert_eq!(ptr("#").unwrap(), Some(Reference::Pointer(Vec::new())));
    }
}
