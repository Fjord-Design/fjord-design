#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(
    clippy::needless_pass_by_value,
    reason = "rstest case tables pass owned JSON"
)]
//! Format profile (D1–D4, RF1–RF8, V1–V6, C7) through
//! `dtcg::graph::resolve` and `Tree::expand` on in-memory documents.
//!
//! Cases already covered by the unit tests in `src/dtcg/*.rs` are not
//! repeated here; these tests add the positions, combinations and
//! boundaries the unit tests leave out.

mod common;

use fjord::diagnostic::Rule;
use fjord::dtcg::graph::{Deprecation, Resolved, resolve};
use fjord::dtcg::tree::Tree;
use fjord::dtcg::value::{Shadow, TokenType, TypedValue};
use rstest::rstest;
use serde_json::{Map, Value, json};

fn doc(v: Value) -> Map<String, Value> {
    let Value::Object(m) = v else {
        panic!("not an object: {v}")
    };
    m
}

fn ok(v: Value) -> Resolved {
    resolve(&doc(v), "test.json").unwrap_or_else(|f| panic!("{f}"))
}

/// Rules of a failing document, sorted by ID.
fn fails(v: Value) -> Vec<Rule> {
    let failure = resolve(&doc(v.clone()), "test.json")
        .err()
        .unwrap_or_else(|| panic!("document should fail: {v}"));
    let mut found = common::rules(&failure);
    found.sort_by_key(|r| r.id());
    found
}

fn tok(ty: &str, value: Value) -> Value {
    json!({"$type": ty, "$value": value, "$description": "d"})
}

fn untyped(value: Value) -> Value {
    json!({"$value": value, "$description": "d"})
}

fn ptr(target: &str) -> Value {
    json!({ "$ref": target })
}

fn dim(v: f64) -> Value {
    json!({"value": v, "unit": "px"})
}

fn rgb(r: f64, g: f64, b: f64) -> Value {
    json!({"colorSpace": "srgb", "components": [r, g, b]})
}

fn shadow_layer() -> Value {
    json!({
        "color": "{primitive.ink}",
        "offsetX": dim(0.0), "offsetY": dim(1.0), "blur": dim(2.0), "spread": dim(0.0)
    })
}

fn get<'a>(r: &'a Resolved, path: &str) -> &'a fjord::dtcg::graph::ResolvedToken {
    r.get(path).unwrap_or_else(|| panic!("{path} missing"))
}

fn refs(r: &Resolved, path: &str) -> Vec<String> {
    get(r, path)
        .references
        .iter()
        .map(ToString::to_string)
        .collect()
}

fn shadow_layers(r: &Resolved, path: &str) -> usize {
    match &get(r, path).value {
        TypedValue::Shadow(Shadow::Single(_)) => 1,
        TypedValue::Shadow(Shadow::Layers(l)) => l.len(),
        other => panic!("{path} is not a shadow: {other:?}"),
    }
}

// ----- RF2 / RF7: pointers -------------------------------------------------

#[test]
fn rf2_rf7_escaped_pointers_select_token_and_group_metadata() {
    let meta = "#/primitive/meta/$extensions/org.example/x%20y/%C3%A5";
    let r = ok(json!({"primitive": {
        "$extensions": {"org.example": {"a/b": {"c~d": 0.5}}},
        "meta": {
            "$type": "number",
            "$value": 1,
            "$description": "d",
            "$extensions": {"org.example": {"x y": {"å": {"01": 0.25, "list": [0.75]}}}}
        },
        "colour": tok("color", json!({"colorSpace": "srgb", "components": [
            ptr("#/primitive/$extensions/org.example/a~1b/c~0d"),
            ptr(&format!("{meta}/01")),
            ptr(&format!("{meta}/list/0")),
        ]})),
        "size": tok("dimension", json!({"value": ptr(&format!("{meta}/01")), "unit": "rem"})),
        "ease": tok("cubicBezier", json!([
            ptr("#/primitive/$extensions/org.example/a%7E1b/c%7E0d"),
            0,
            ptr(&format!("{meta}/list/0")),
            1
        ])),
        "encoded": untyped(ptr("#/primitive%2Fmeta/%24value")),
    }}));
    assert_eq!(
        get(&r, "primitive.colour").json,
        json!({"colorSpace": "srgb", "components": [0.5, 0.25, 0.75]})
    );
    // Token metadata counts as a reference to the token; group metadata does not.
    assert_eq!(refs(&r, "primitive.colour"), ["primitive.meta"]);
    assert_eq!(
        get(&r, "primitive.size").json,
        json!({"value": 0.25, "unit": "rem"})
    );
    assert_eq!(
        get(&r, "primitive.ease").value,
        TypedValue::CubicBezier([0.5, 0.0, 0.75, 1.0])
    );
    // `%2F` is a separator and `%24value` is `$value`: a whole-token pointer.
    assert_eq!(get(&r, "primitive.encoded").ty, TokenType::Number);
    assert_eq!(get(&r, "primitive.encoded").json, json!(1));
    assert_eq!(refs(&r, "primitive.encoded"), ["primitive.meta"]);
}

#[rstest]
#[case::leading_zero("01")]
#[case::out_of_range("4")]
#[case::dash("-")]
#[case::plus("+1")]
fn rf2_array_indexes_must_be_canonical_and_in_range(#[case] index: &str) {
    let resolved = ok(json!({"primitive": {
        "ease": tok("cubicBezier", json!([0.1, 0.2, 0.3, 0.4])),
        "meta": {"$type": "number", "$value": 1, "$description": "d", "$extensions": {"org.x": {"01": 2}}},
        "key": tok("number", ptr("#/primitive/meta/$extensions/org.x/01")),
        "last": tok("number", ptr("#/primitive/ease/$value/3")),
    }}));
    // `01` is an ordinary object key.
    assert_eq!(get(&resolved, "primitive.key").json, json!(2));
    assert_eq!(get(&resolved, "primitive.last").json, json!(0.4));
    let found = fails(json!({"primitive": {
        "ease": tok("cubicBezier", json!([0.1, 0.2, 0.3, 0.4])),
        "bad": tok("number", ptr(&format!("#/primitive/ease/$value/{index}"))),
    }}));
    assert_eq!(found, [Rule::ReferenceMissing]);
}

/// A document placing `reference` at one position of a value or group.
fn at_position(position: &str, reference: Value) -> Value {
    let n = tok("number", json!(1));
    match position {
        "whole" => json!({"primitive": {"n": n, "x": tok("number", reference)}}),
        "field" => json!({"primitive": {
            "n": n,
            "x": tok("dimension", json!({"value": reference, "unit": "px"})),
        }}),
        "element" => json!({"primitive": {
            "n": n,
            "x": tok("cubicBezier", json!([reference, 0, 1, 1])),
        }}),
        "extends" => json!({"primitive": {
            "n": n,
            "g": {"$extends": reference, "y": tok("number", json!(2))},
        }}),
        other => panic!("unknown position {other}"),
    }
}

#[rstest]
fn rf2_malformed_and_external_references_fail_in_every_position(
    #[values("whole", "field", "element", "extends")] position: &str,
    #[values(
        (ptr("#/primitive/n/$value/~2"), Rule::ReferenceSyntax),
        (ptr("#/primitive/n~"), Rule::ReferenceSyntax),
        (ptr("#/primitive/%"), Rule::ReferenceSyntax),
        (ptr("#/primitive/%2"), Rule::ReferenceSyntax),
        (ptr("#/primitive/%zz"), Rule::ReferenceSyntax),
        (ptr("#/primitive/%FF"), Rule::ReferenceSyntax),
        (ptr("#primitive/n/$value"), Rule::ReferenceSyntax),
        (json!({"$ref": 3}), Rule::ReferenceSyntax),
        (json!({"$ref": "#/primitive/n/$value", "$type": "number"}), Rule::ReferenceSyntax),
        (ptr("tokens.json"), Rule::ReferenceExternal),
        (ptr("tokens.json#/primitive/n/$value"), Rule::ReferenceExternal),
        (ptr("https://example.com/t.json#/primitive/n/$value"), Rule::ReferenceExternal),
        (ptr("/primitive/n/$value"), Rule::ReferenceExternal)
    )]
    case: (Value, Rule),
) {
    let (reference, rule) = case;
    assert_eq!(fails(at_position(position, reference)), [rule]);
}

#[test]
fn rf7_selected_aliases_and_pointers_resolve_in_place() {
    let r = ok(json!({"primitive": {
        "ink": tok("color", rgb(0.1, 0.2, 0.3)),
        "n": tok("number", json!(3)),
        "shadow": tok("shadow", shadow_layer()),
        "tint": tok("color", ptr("#/primitive/shadow/$value/color")),
        "size": tok("dimension", json!({"value": ptr("#/primitive/n/$value"), "unit": "px"})),
        "double": tok("number", ptr("#/primitive/size/$value/value")),
    }}));
    assert_eq!(get(&r, "primitive.tint").json, rgb(0.1, 0.2, 0.3));
    assert_eq!(
        refs(&r, "primitive.tint"),
        ["primitive.shadow", "primitive.ink"]
    );
    assert_eq!(get(&r, "primitive.double").json, json!(3));
    assert_eq!(
        refs(&r, "primitive.double"),
        ["primitive.size", "primitive.n"]
    );
}

#[test]
fn rf7_pointer_cannot_reach_through_an_alias_field() {
    let found = fails(json!({"primitive": {
        "ink": tok("color", rgb(0.1, 0.2, 0.3)),
        "shadow": tok("shadow", shadow_layer()),
        "bad": tok("number", ptr("#/primitive/shadow/$value/color/components/0")),
    }}));
    assert_eq!(found, [Rule::ReferenceMissing]);
}

// ----- RF3: braces ---------------------------------------------------------

#[rstest]
#[case::family_element(tok("fontFamily", json!(["Noto Sans", "Noto {primitive.f}"])))]
#[case::unit_field(tok("dimension", json!({"value": 1, "unit": "{px"})))]
#[case::colour_space(tok("color", json!({"colorSpace": "srgb}", "components": [0, 0, 0]})))]
#[case::two_aliases_in_a_field(tok("typography", json!({
    "fontFamily": "{primitive.f}{primitive.f}", "fontSize": dim(16.0), "fontWeight": 400,
    "letterSpacing": dim(0.0), "lineHeight": 1.5
})))]
#[case::padded_element(tok("fontFamily", json!(["{primitive.f} "])))]
fn rf3_brace_strings_inside_composites_fail(#[case] token: Value) {
    let found = fails(json!({"primitive": {
        "f": tok("fontFamily", json!("Noto Sans")),
        "x": token,
    }}));
    assert_eq!(found, [Rule::ReferenceSyntax]);
}

#[test]
fn rf3_v6_alias_array_elements_resolve_and_must_stay_flat() {
    let r = ok(json!({"primitive": {
        "font": tok("fontFamily", json!("Noto Sans")),
        "stack": tok("fontFamily", json!(["{primitive.font}", "sans-serif"])),
    }}));
    assert_eq!(
        get(&r, "primitive.stack").value,
        TypedValue::FontFamily(vec!["Noto Sans".to_owned(), "sans-serif".to_owned()])
    );
    let found = fails(json!({"primitive": {
        "stack": tok("fontFamily", json!(["Noto Sans", "sans-serif"])),
        "nested": tok("fontFamily", json!(["{primitive.stack}"])),
    }}));
    assert_eq!(found, [Rule::Value]);
}

// ----- RF4 / L13: missing targets, cycles, chains -------------------------

#[rstest]
#[case::root_alias(json!({"primitive": {
    "g": {"$type": "number", "$root": untyped(json!("{primitive.x}"))},
    "x": tok("number", json!("{primitive.g.$root}")),
}}))]
#[case::inherited_self_alias(json!({"primitive": {
    "$type": "number",
    "a": {"x": untyped(json!("{primitive.b.x}"))},
    "b": {"$extends": "{primitive.a}"},
}}))]
#[case::shadow_element(json!({"primitive": {
    "ink": tok("color", rgb(0.0, 0.0, 0.0)),
    "s": tok("shadow", json!([shadow_layer(), "{primitive.s}"])),
}}))]
#[case::own_field(json!({"primitive": {
    "t": tok("typography", json!({
        "fontFamily": "Noto Sans", "fontSize": dim(16.0), "fontWeight": 400,
        "letterSpacing": dim(0.0), "lineHeight": ptr("#/primitive/t/$value/lineHeight")
    })),
}}))]
#[case::field_pointer_pair(json!({"primitive": {
    "a": tok("color", json!({"colorSpace": "srgb", "components": [ptr("#/primitive/b/$value/components/0"), 0, 0]})),
    "b": tok("color", json!({"colorSpace": "srgb", "components": [ptr("#/primitive/a/$value/components/0"), 0, 0]})),
}}))]
fn rf4_cycles_fail_once(#[case] document: Value) {
    assert_eq!(fails(document), [Rule::ReferenceCycle]);
}

#[rstest]
#[case::pointer_to_missing_token(ptr("#/primitive/none/$value"))]
#[case::pointer_to_missing_field(ptr("#/primitive/c/$value/components/3"))]
#[case::pointer_to_missing_metadata(ptr("#/$extensions/org.none/x"))]
#[case::pointer_to_missing_token_metadata(ptr("#/primitive/c/$extensions/org.none"))]
#[case::brace_to_a_value(json!("{primitive.c.$value}"))]
#[case::empty_pointer(ptr("#"))]
fn rf4_missing_targets_fail(#[case] reference: Value) {
    let found = fails(json!({"primitive": {
        "c": tok("color", rgb(0.1, 0.2, 0.3)),
        "x": tok("number", reference),
    }}));
    assert_eq!(found, [Rule::ReferenceMissing]);
}

/// `c0 → c1 → … → c<links>`, each link copying the next link's `value`
/// field in place.
fn property_chain(links: usize) -> Resolved {
    let mut chain = Map::new();
    for i in 0..links {
        let next = format!("#/primitive/chain/c{}/$value/value", i + 1);
        chain.insert(
            format!("c{i}"),
            tok("dimension", json!({"value": ptr(&next), "unit": "px"})),
        );
    }
    chain.insert(format!("c{links}"), tok("dimension", dim(7.0)));
    let r = ok(json!({"primitive": {"chain": chain}}));
    assert_eq!(r.len(), links + 1);
    assert_eq!(get(&r, "primitive.chain.c0").json, dim(7.0));
    assert_eq!(refs(&r, "primitive.chain.c0").len(), links);
    r
}

#[test]
fn rf4_property_pointer_chain_resolves_in_place() {
    property_chain(200);
}

#[test]
fn l13_ten_thousand_link_property_pointer_chain_resolves() {
    property_chain(9_999);
}

// ----- RF8 / L12: types ----------------------------------------------------

#[rstest]
#[case::nearest_group_type(
    json!({"primitive": {"$type": "number", "inner": {"$type": "fontWeight", "w": untyped(json!(400))}}}),
    "primitive.inner.w",
    TokenType::FontWeight
)]
#[case::outer_group_type_through_untyped_groups(
    json!({"primitive": {"$type": "number", "g": {"h": {"x": untyped(json!(2))}}}}),
    "primitive.g.h.x",
    TokenType::Number
)]
#[case::alias_chain_to_a_group_typed_token(
    json!({"primitive": {
        "d": {"$type": "duration", "x": untyped(json!({"value": 1, "unit": "s"}))},
        "a": untyped(json!("{primitive.d.x}")),
        "b": untyped(ptr("#/primitive/a/$value")),
    }}),
    "primitive.b",
    TokenType::Duration
)]
#[case::local_group_type_beats_the_inherited_one(
    json!({"primitive": {
        "base": {"$type": "number", "x": untyped(json!(1))},
        "derived": {"$extends": "{primitive.base}", "$type": "fontWeight"},
    }}),
    "primitive.derived.x",
    TokenType::FontWeight
)]
#[case::inherited_group_type_beats_an_outer_one(
    json!({"primitive": {
        "$type": "fontWeight",
        "base": {"$type": "number", "x": untyped(json!(1))},
        "derived": {"$extends": "{primitive.base}"},
    }}),
    "primitive.derived.x",
    TokenType::Number
)]
#[case::whole_alias_beats_the_group_type(
    json!({"primitive": {
        "w": tok("fontWeight", json!(400)),
        "g": {"$type": "number", "a": untyped(ptr("#/primitive/w/$value"))},
    }}),
    "primitive.g.a",
    TokenType::FontWeight
)]
fn rf8_type_precedence(#[case] document: Value, #[case] path: &str, #[case] ty: TokenType) {
    assert_eq!(get(&ok(document), path).ty, ty);
}

#[test]
fn rf8_explicit_type_is_checked_against_a_whole_alias_target() {
    let found = fails(json!({"primitive": {
        "n": tok("number", json!(1)),
        "w": tok("fontWeight", json!("{primitive.n}")),
    }}));
    assert_eq!(found, [Rule::ReferenceType]);
}

#[rstest]
#[case::colour_component(ptr("#/primitive/c/$value/components/0"))]
#[case::colour_alpha(ptr("#/primitive/c/$value/alpha"))]
#[case::bezier_item(ptr("#/primitive/ease/$value/1"))]
#[case::root_metadata(ptr("#/$extensions/org.example/n"))]
#[case::token_metadata(ptr("#/primitive/c/$description"))]
#[case::literal_array(json!([0, 0, 1, 1]))]
#[case::literal_string(json!("Noto Sans"))]
#[case::literal_object(json!({"value": 1, "unit": "px"}))]
fn l12_untyped_literals_and_partial_pointers_are_undetermined(#[case] value: Value) {
    let found = fails(json!({
        "$extensions": {"org.example": {"n": 1}},
        "primitive": {
            "c": tok("color", json!({"colorSpace": "srgb", "components": [0, 0, 0], "alpha": 0.5})),
            "ease": tok("cubicBezier", json!([0, 0, 1, 1])),
            "x": untyped(value),
        }
    }));
    assert_eq!(found, [Rule::TypeUndetermined]);
}

// ----- RF1: $root ----------------------------------------------------------

#[test]
fn rf1_inherited_root_and_root_fields_resolve() {
    let r = ok(json!({"primitive": {
        "space": {"$type": "dimension", "$root": untyped(dim(8.0)), "wide": untyped(dim(16.0))},
        "copy": {"$extends": "{primitive.space}"},
        "a": untyped(json!("{primitive.copy.$root}")),
        "field": tok("number", ptr("#/primitive/space/$root/$value/value")),
    }}));
    assert_eq!(get(&r, "primitive.a").json, dim(8.0));
    assert_eq!(get(&r, "primitive.a").ty, TokenType::Dimension);
    assert_eq!(get(&r, "primitive.field").json, json!(8.0));
}

#[rstest]
#[case::root_definition(ptr("#/primitive/space/$root"))]
#[case::alias_below_root(json!("{primitive.space.$root.$value}"))]
#[case::implied_root_through_inheritance(json!("{primitive.copy}"))]
fn rf1_root_is_never_implied(#[case] reference: Value) {
    let found = fails(json!({"primitive": {
        "space": {"$type": "dimension", "$root": untyped(dim(8.0))},
        "copy": {"$extends": "{primitive.space}"},
        "x": tok("dimension", reference),
    }}));
    assert_eq!(found, [Rule::ReferenceMissing]);
}

// ----- C7: $extends and group $ref ----------------------------------------

#[test]
fn c7_group_properties_merge_and_local_tokens_replace_whole() {
    let input = doc(json!({"primitive": {
        "base": {
            "$type": "dimension",
            "$description": "Base group.",
            "$extensions": {"org.x": {"a": 1, "b": 1}},
            "x": {"$value": dim(1.0), "$description": "d", "$deprecated": true, "$extensions": {"org.x": {"k": 1}}},
            "y": untyped(dim(2.0)),
            "nested": {"z": untyped(dim(3.0))},
        },
        "derived": {
            "$extends": "{primitive.base}",
            "$extensions": {"org.x": {"b": 2}},
            "x": untyped(dim(10.0)),
            "nested": {"w": untyped(dim(4.0))},
        },
        "object": {"$extends": {"$ref": "#/primitive/derived"}},
        "chain": {"$ref": "#/primitive/object"},
        "early": {"$extends": "{primitive.late}"},
        "late": {"$type": "number", "n": untyped(json!(5))},
    }}));
    let expanded = Tree::from_document(input.clone())
        .expand("test.json")
        .unwrap();
    let derived = expanded.document()["primitive"]["derived"]
        .as_object()
        .unwrap();
    assert_eq!(derived["$extends"], json!("{primitive.base}"));
    assert_eq!(derived["$description"], json!("Base group."));
    assert_eq!(derived["$extensions"], json!({"org.x": {"a": 1, "b": 2}}));
    assert_eq!(derived["x"], untyped(dim(10.0)));
    assert_eq!(
        expanded.document()["primitive"]["chain"]["$ref"],
        json!("#/primitive/object")
    );

    let r = resolve(&input, "test.json").unwrap();
    let x = get(&r, "primitive.derived.x");
    assert_eq!(x.json, dim(10.0));
    assert_eq!(x.deprecated, None);
    assert!(!x.definition.contains_key("$extensions"));
    assert_eq!(get(&r, "primitive.derived.y").json, dim(2.0));
    assert_eq!(get(&r, "primitive.derived.nested.z").json, dim(3.0));
    assert_eq!(get(&r, "primitive.derived.nested.w").json, dim(4.0));
    assert_eq!(get(&r, "primitive.object.x").json, dim(10.0));
    assert_eq!(get(&r, "primitive.chain.nested.w").json, dim(4.0));
    assert_eq!(get(&r, "primitive.chain.y").ty, TokenType::Dimension);
    assert_eq!(get(&r, "primitive.early.n").value, TypedValue::Number(5.0));
    assert_eq!(
        get(&r, "primitive.base.x").deprecated,
        Some(Deprecation { reason: None })
    );
}

#[test]
fn c7_d4_deprecation_follows_the_inheriting_tree() {
    let r = ok(json!({"primitive": {
        "$type": "number",
        "old": {"$deprecated": true, "set": {"x": untyped(json!(1))}},
        "flagged": {"$deprecated": "Use primitive.new.", "y": untyped(json!(2))},
        "new": {"$extends": "{primitive.old.set}"},
        "revived": {"$extends": "{primitive.flagged}", "$deprecated": false},
        "still": {"$extends": "{primitive.flagged}"},
    }}));
    assert_eq!(
        get(&r, "primitive.old.set.x").deprecated,
        Some(Deprecation { reason: None })
    );
    // The target's ancestors are not part of the inherited group.
    assert_eq!(get(&r, "primitive.new.x").deprecated, None);
    assert_eq!(get(&r, "primitive.revived.y").deprecated, None);
    assert_eq!(
        get(&r, "primitive.still.y").deprecated,
        Some(Deprecation {
            reason: Some("Use primitive.new.".to_owned())
        })
    );
}

#[rstest]
#[case::group_ref_to_a_token(
    json!({"primitive": {"t": tok("number", json!(1)), "g": {"$ref": "#/primitive/t"}}}),
    Rule::Extends
)]
#[case::root_token(
    json!({"primitive": {"s": {"$type": "number", "$root": untyped(json!(1))}, "g": {"$extends": "{primitive.s.$root}"}}}),
    Rule::Extends
)]
#[case::ancestor(json!({"primitive": {"g": {"$extends": "#/primitive"}}}), Rule::Extends)]
#[case::document_root(json!({"primitive": {"g": {"$extends": "#"}}}), Rule::Extends)]
#[case::root_metadata(
    json!({"$extensions": {"org.x": {}}, "primitive": {"g": {"$extends": "#/$extensions/org.x"}}}),
    Rule::Extends
)]
#[case::group_metadata(
    json!({"primitive": {"h": {"$extensions": {"org.x": {}}}, "g": {"$ref": "#/primitive/h/$extensions/org.x"}}}),
    Rule::Extends
)]
#[case::boolean(json!({"primitive": {"g": {"$extends": true}}}), Rule::Extends)]
#[case::three_group_cycle(
    json!({"primitive": {
        "a": {"$extends": "{primitive.b}"},
        "b": {"$ref": "#/primitive/c"},
        "c": {"$extends": {"$ref": "#/primitive/a"}},
    }}),
    Rule::Extends
)]
#[case::malformed_brace(
    json!({"primitive": {"h": {}, "g": {"$extends": "{primitive..h}"}}}),
    Rule::ReferenceSyntax
)]
fn c7_invalid_inheritance_targets_fail(#[case] document: Value, #[case] rule: Rule) {
    let failure = Tree::from_document(doc(document.clone()))
        .expand("test.json")
        .err()
        .unwrap_or_else(|| panic!("should fail: {document}"));
    assert_eq!(common::rules(&failure), [rule]);
    assert_eq!(fails(document), [rule]);
}

// ----- V4: shadows ---------------------------------------------------------

#[test]
fn v4_shadow_elements_by_pointer_and_aliased_fields() {
    let r = ok(json!({"primitive": {
        "ink": tok("color", rgb(0.0, 0.0, 0.0)),
        "gap": tok("dimension", dim(3.0)),
        "n": tok("number", json!(5)),
        "one": tok("shadow", shadow_layer()),
        "many": tok("shadow", json!([shadow_layer(), shadow_layer()])),
        "pointed": tok("shadow", json!([ptr("#/primitive/one/$value"), shadow_layer()])),
        "whole-pointer": untyped(ptr("#/primitive/many/$value")),
        "alias-of-alias": untyped(json!("{primitive.whole-pointer}")),
        "fields": tok("shadow", json!({
            "color": ptr("#/primitive/ink/$value"),
            "offsetX": ptr("#/primitive/one/$value/offsetX"),
            "offsetY": "{primitive.gap}",
            "blur": {"value": ptr("#/primitive/n/$value"), "unit": "px"},
            "spread": dim(0.0),
            "inset": true
        })),
    }}));
    assert_eq!(shadow_layers(&r, "primitive.pointed"), 2);
    assert_eq!(get(&r, "primitive.whole-pointer").ty, TokenType::Shadow);
    assert_eq!(shadow_layers(&r, "primitive.whole-pointer"), 2);
    assert_eq!(shadow_layers(&r, "primitive.alias-of-alias"), 2);
    let TypedValue::Shadow(Shadow::Single(layer)) = &get(&r, "primitive.fields").value else {
        panic!("single shadow expected")
    };
    assert!(layer.inset);
    assert_eq!(
        get(&r, "primitive.fields").json["blur"],
        json!({"value": 5, "unit": "px"})
    );
    assert_eq!(get(&r, "primitive.fields").json["offsetY"], dim(3.0));
}

#[rstest]
#[case::element_pointer_to_an_array(json!([shadow_layer(), ptr("#/primitive/many/$value")]), Rule::Value)]
#[case::element_alias_to_an_array_alias(json!(["{primitive.alias}"]), Rule::Value)]
#[case::element_of_another_type(json!(["{primitive.ink}"]), Rule::ReferenceType)]
#[case::negative_blur_by_alias(json!({
    "color": "{primitive.ink}", "offsetX": dim(0.0), "offsetY": dim(0.0),
    "blur": "{primitive.negative}", "spread": dim(0.0)
}), Rule::Value)]
#[case::inset_from_a_number(json!({
    "color": "{primitive.ink}", "offsetX": dim(0.0), "offsetY": dim(0.0),
    "blur": dim(0.0), "spread": dim(0.0), "inset": ptr("#/$extensions/org.x/n")
}), Rule::Value)]
fn v4_shadow_element_rules(#[case] value: Value, #[case] rule: Rule) {
    let found = fails(json!({
        "$extensions": {"org.x": {"n": 1}},
        "primitive": {
            "ink": tok("color", rgb(0.0, 0.0, 0.0)),
            "negative": tok("dimension", dim(-1.0)),
            "many": tok("shadow", json!([shadow_layer(), shadow_layer()])),
            "alias": untyped(json!("{primitive.many}")),
            "x": tok("shadow", value),
        }
    }));
    assert_eq!(found, [rule]);
}

// ----- V1–V6: boundaries not covered by the value.rs tables --------------

fn typography(line_height: Value, font_size: Value, letter_spacing: Value) -> Value {
    json!({
        "fontFamily": "Noto Sans", "fontSize": font_size, "fontWeight": 1,
        "letterSpacing": letter_spacing, "lineHeight": line_height
    })
}

#[rstest]
#[case::alpha_zero("color", json!({"colorSpace": "srgb", "components": [0, 0, 0], "alpha": 0}))]
#[case::hex_ignores_alpha("color", json!({"colorSpace": "srgb", "components": [1, 1, 1], "alpha": 0.5, "hex": "#ffffff"}))]
#[case::negative_zero_dimension("dimension", json!({"value": -0.0, "unit": "px"}))]
#[case::tiny_duration("duration", json!({"value": 0.0001, "unit": "s"}))]
#[case::fractional_weight("fontWeight", json!(1.5))]
#[case::bezier_x_at_one("cubicBezier", json!([1, -5, 1, 5]))]
#[case::padded_family("fontFamily", json!(" Noto Sans "))]
#[case::small_typography("typography", typography(json!(0.01), json!({"value": 0.001, "unit": "rem"}), json!({"value": -0.5, "unit": "px"})))]
fn v1_v6_boundary_values_pass(#[case] ty: &str, #[case] value: Value) {
    ok(json!({"primitive": {"x": tok(ty, value)}}));
}

#[rstest]
#[case::negative_alpha("color", json!({"colorSpace": "srgb", "components": [0, 0, 0], "alpha": -0.01}))]
#[case::short_hex("color", json!({"colorSpace": "srgb", "components": [1, 1, 1], "hex": "#FFF"}))]
#[case::hex_with_alpha("color", json!({"colorSpace": "srgb", "components": [1, 0, 0], "alpha": 0.5, "hex": "#ff000080"}))]
#[case::colour_space_case("color", json!({"colorSpace": "SRGB", "components": [0, 0, 0]}))]
#[case::four_components("color", json!({"colorSpace": "srgb", "components": [0, 0, 0, 0]}))]
#[case::unit_case("dimension", json!({"value": 1, "unit": "PX"}))]
#[case::em_unit("dimension", json!({"value": 1, "unit": "em"}))]
#[case::duration_unit_case("duration", json!({"value": 1, "unit": "MS"}))]
#[case::negative_seconds("duration", json!({"value": -0.0001, "unit": "s"}))]
#[case::weight_below_one("fontWeight", json!(0.5))]
#[case::weight_above_1000("fontWeight", json!(1000.5))]
#[case::five_bezier_items("cubicBezier", json!([0, 0, 1, 1, 0]))]
#[case::bezier_x2_above_one("cubicBezier", json!([0, 0, 1.01, 1]))]
#[case::number_object("number", json!({"value": 1}))]
#[case::negative_line_height("typography", typography(json!(-1), dim(16.0), dim(0.0)))]
#[case::dimension_line_height("typography", typography(dim(1.5), dim(16.0), dim(0.0)))]
#[case::zero_rem_font_size("typography", typography(json!(1.5), json!({"value": 0, "unit": "rem"}), dim(0.0)))]
#[case::boolean_weight("typography", json!({
    "fontFamily": "Noto Sans", "fontSize": dim(16.0), "fontWeight": true,
    "letterSpacing": dim(0.0), "lineHeight": 1.5
}))]
#[case::extra_key_in_nested_dimension("shadow", json!({
    "color": {"colorSpace": "srgb", "components": [0, 0, 0]},
    "offsetX": dim(0.0), "offsetY": dim(0.0),
    "blur": {"value": 1, "unit": "px", "x": 1}, "spread": dim(0.0)
}))]
fn v1_v6_boundary_values_fail(#[case] ty: &str, #[case] value: Value) {
    assert_eq!(
        fails(json!({"primitive": {"x": tok(ty, value)}})),
        [Rule::Value]
    );
}

#[rstest]
#[case::boolean_component(tok("color", json!({"colorSpace": "srgb", "components": [ptr("#/$extensions/org.x/flag"), 0, 0]})))]
#[case::boolean_inset_as_number(tok("number", ptr("#/primitive/s/$value/inset")))]
#[case::string_metadata_as_dimension_value(tok("dimension", json!({"value": ptr("#/$extensions/org.x/text"), "unit": "px"})))]
fn v6_substituted_values_keep_their_field_types(#[case] token: Value) {
    let found = fails(json!({
        "$extensions": {"org.x": {"flag": true, "text": "1"}},
        "primitive": {
            "ink": tok("color", rgb(0.0, 0.0, 0.0)),
            "s": tok("shadow", json!({
                "color": "{primitive.ink}", "offsetX": dim(0.0), "offsetY": dim(0.0),
                "blur": dim(0.0), "spread": dim(0.0), "inset": true
            })),
            "x": token,
        }
    }));
    assert_eq!(found, [Rule::Value]);
}

// ----- D1–D4: structure, names, descriptions, types -----------------------

#[rstest]
#[case::root_token(json!({"$value": 1, "$type": "number", "$description": "d"}), Rule::Structure)]
#[case::array_child(json!({"primitive": {"x": [1]}}), Rule::Structure)]
#[case::null_child(json!({"primitive": {"x": null}}), Rule::Structure)]
#[case::group_type_unknown(json!({"primitive": {"$type": "colour", "x": untyped(json!(1))}}), Rule::Type)]
#[case::group_type_not_a_string(json!({"primitive": {"$type": 3, "x": untyped(json!(1))}}), Rule::Type)]
#[case::token_type_case(json!({"primitive": {"x": tok("Color", json!(1))}}), Rule::Type)]
#[case::group_extensions_not_an_object(json!({"primitive": {"$extensions": "x", "y": tok("number", json!(1))}}), Rule::Structure)]
#[case::token_extensions_not_an_object(json!({"primitive": {"x": {"$type": "number", "$value": 1, "$description": "d", "$extensions": []}}}), Rule::Structure)]
#[case::token_deprecated_number(json!({"primitive": {"x": {"$type": "number", "$value": 1, "$description": "d", "$deprecated": 1}}}), Rule::Structure)]
#[case::token_deprecated_blank(json!({"primitive": {"x": {"$type": "number", "$value": 1, "$description": "d", "$deprecated": "  "}}}), Rule::Structure)]
#[case::group_description_number(json!({"primitive": {"$description": 3, "x": tok("number", json!(1))}}), Rule::Description)]
#[case::token_description_blank(json!({"primitive": {"x": {"$type": "number", "$value": 1, "$description": " \t"}}}), Rule::Description)]
#[case::token_description_number(json!({"primitive": {"x": {"$type": "number", "$value": 1, "$description": 3}}}), Rule::Description)]
#[case::group_unknown_property(json!({"primitive": {"$id": "x", "y": tok("number", json!(1))}}), Rule::Structure)]
#[case::token_ref_property(json!({"primitive": {"x": {"$type": "number", "$value": 1, "$description": "d", "$ref": "#/primitive"}}}), Rule::Structure)]
fn d1_d4_structure_rules(#[case] document: Value, #[case] rule: Rule) {
    assert_eq!(fails(document), [rule]);
}

#[rstest]
#[case("a--b")]
#[case("-a")]
#[case("a-")]
#[case("1a")]
#[case("aB")]
#[case("a_b")]
#[case("a.b")]
#[case("a b")]
#[case("é")]
#[case("")]
fn d2_invalid_names_fail(#[case] name: &str) {
    let found = fails(json!({"primitive": {name: tok("number", json!(1))}}));
    assert_eq!(found, [Rule::Name]);
}

#[rstest]
#[case("a")]
#[case("a1")]
#[case("a-1")]
#[case("text-2xs")]
#[case("z9-y8")]
fn d2_valid_names_pass(#[case] name: &str) {
    let r = ok(json!({"primitive": {"g": {name: tok("number", json!(1))}}}));
    assert!(r.get(&format!("primitive.g.{name}")).is_some());
}

#[test]
fn d4_nearest_deprecation_wins() {
    let r = ok(json!({"primitive": {
        "$type": "number",
        "old": {
            "$deprecated": "Use primitive.new.",
            "x": untyped(json!(1)),
            "keep": {"$deprecated": false, "y": untyped(json!(2))},
            "z": {"$value": 3, "$description": "d", "$deprecated": true},
        },
    }}));
    assert_eq!(
        get(&r, "primitive.old.x").deprecated,
        Some(Deprecation {
            reason: Some("Use primitive.new.".to_owned())
        })
    );
    assert_eq!(get(&r, "primitive.old.keep.y").deprecated, None);
    assert_eq!(
        get(&r, "primitive.old.z").deprecated,
        Some(Deprecation { reason: None })
    );
}

// ----- Inputs ---------------------------------------------------------------

#[test]
fn inputs_are_not_mutated_by_failing_resolution_or_expansion() {
    let input = doc(json!({"primitive": {
        "$type": "dimension",
        "base": {"a": untyped(dim(1.0))},
        "derived": {"$extends": "{primitive.base}"},
        "loop": untyped(json!("{primitive.loop}")),
        "missing": {"$ref": "#/primitive/none"},
    }}));
    let before = input.clone();
    assert!(resolve(&input, "test.json").is_err());
    assert_eq!(input, before);

    let ok_input = doc(json!({"primitive": {
        "$type": "dimension",
        "base": {"a": untyped(dim(1.0))},
        "derived": {"$extends": "{primitive.base}", "b": untyped(ptr("#/primitive/derived/a/$value"))},
    }}));
    let tree = Tree::from_document(ok_input.clone());
    let expanded = tree.expand("test.json").unwrap();
    assert_eq!(tree.document(), &ok_input);
    assert!(
        expanded.document()["primitive"]["derived"]
            .get("a")
            .is_some()
    );
    assert!(ok_input["primitive"]["derived"].get("a").is_none());
}
