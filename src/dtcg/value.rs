//! Typed values (V1–V6): the single place that decides whether a fully
//! reference-substituted `$value` is a valid token of its type.

use std::fmt::{self, Write as _};
use std::str::FromStr;

use serde_json::{Map, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TokenType {
    Color,
    Dimension,
    Duration,
    Number,
    FontFamily,
    FontWeight,
    CubicBezier,
    Shadow,
    Typography,
}

impl TokenType {
    pub const ALL: [TokenType; 9] = [
        TokenType::Color,
        TokenType::Dimension,
        TokenType::Duration,
        TokenType::Number,
        TokenType::FontFamily,
        TokenType::FontWeight,
        TokenType::CubicBezier,
        TokenType::Shadow,
        TokenType::Typography,
    ];

    /// The DTCG type name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            TokenType::Color => "color",
            TokenType::Dimension => "dimension",
            TokenType::Duration => "duration",
            TokenType::Number => "number",
            TokenType::FontFamily => "fontFamily",
            TokenType::FontWeight => "fontWeight",
            TokenType::CubicBezier => "cubicBezier",
            TokenType::Shadow => "shadow",
            TokenType::Typography => "typography",
        }
    }
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for TokenType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        TokenType::ALL
            .into_iter()
            .find(|t| t.as_str() == s)
            .ok_or_else(|| format!("unknown type \"{s}\""))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LengthUnit {
    Px,
    Rem,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeUnit {
    Ms,
    S,
}

/// sRGB colour with components and alpha in `0..=1`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub components: [f64; 3],
    pub alpha: f64,
}

impl Color {
    /// Lowercase `#rrggbb`, with an alpha byte appended only when it is below 255.
    #[must_use]
    pub fn to_hex(&self) -> String {
        let mut out = rgb_hex(&self.components);
        let a = channel(self.alpha);
        if a < 255 {
            let _ = write!(out, "{a:02x}");
        }
        out
    }
}

/// Rounds a unit-interval value to an 8-bit channel, half up.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "input is validated to 0..=1, so the rounded value fits in u8"
)]
fn channel(c: f64) -> u8 {
    (c * 255.0).round().clamp(0.0, 255.0) as u8
}

fn rgb_hex(components: &[f64; 3]) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        channel(components[0]),
        channel(components[1]),
        channel(components[2])
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dimension {
    pub value: f64,
    pub unit: LengthUnit,
}

impl Dimension {
    /// Length in CSS pixels (`rem` × 16).
    #[must_use]
    pub fn px(&self) -> f64 {
        match self.unit {
            LengthUnit::Px => self.value,
            LengthUnit::Rem => self.value * 16.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Duration {
    pub value: f64,
    pub unit: TimeUnit,
}

impl Duration {
    /// Time in milliseconds.
    #[must_use]
    pub fn ms(&self) -> f64 {
        match self.unit {
            TimeUnit::Ms => self.value,
            TimeUnit::S => self.value * 1000.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShadowLayer {
    pub color: Color,
    pub offset_x: Dimension,
    pub offset_y: Dimension,
    pub blur: Dimension,
    pub spread: Dimension,
    pub inset: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Shadow {
    Single(ShadowLayer),
    Layers(Vec<ShadowLayer>),
}

impl Shadow {
    /// The shadow layers in authored order.
    #[must_use]
    pub fn layers(&self) -> &[ShadowLayer] {
        match self {
            Shadow::Single(layer) => std::slice::from_ref(layer),
            Shadow::Layers(layers) => layers,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Typography {
    pub font_family: Vec<String>,
    pub font_size: Dimension,
    pub font_weight: f64,
    pub letter_spacing: Dimension,
    pub line_height: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypedValue {
    Color(Color),
    Dimension(Dimension),
    Duration(Duration),
    Number(f64),
    FontFamily(Vec<String>),
    FontWeight(f64),
    CubicBezier([f64; 4]),
    Shadow(Shadow),
    Typography(Typography),
}

impl TypedValue {
    /// Parse a fully reference-substituted `$value` as `ty`.
    ///
    /// # Errors
    /// Returns a message naming the first violated constraint.
    pub fn parse(ty: TokenType, v: &Value) -> Result<TypedValue, String> {
        Ok(match ty {
            TokenType::Color => TypedValue::Color(color(v)?),
            TokenType::Dimension => TypedValue::Dimension(dimension(v)?),
            TokenType::Duration => TypedValue::Duration(duration(v)?),
            TokenType::Number => TypedValue::Number(number(v)?),
            TokenType::FontFamily => TypedValue::FontFamily(font_family(v)?),
            TokenType::FontWeight => TypedValue::FontWeight(font_weight(v)?),
            TokenType::CubicBezier => TypedValue::CubicBezier(cubic_bezier(v)?),
            TokenType::Shadow => TypedValue::Shadow(shadow(v)?),
            TokenType::Typography => TypedValue::Typography(typography(v)?),
        })
    }

    #[must_use]
    pub fn token_type(&self) -> TokenType {
        match self {
            TypedValue::Color(_) => TokenType::Color,
            TypedValue::Dimension(_) => TokenType::Dimension,
            TypedValue::Duration(_) => TokenType::Duration,
            TypedValue::Number(_) => TokenType::Number,
            TypedValue::FontFamily(_) => TokenType::FontFamily,
            TypedValue::FontWeight(_) => TokenType::FontWeight,
            TypedValue::CubicBezier(_) => TokenType::CubicBezier,
            TypedValue::Shadow(_) => TokenType::Shadow,
            TypedValue::Typography(_) => TokenType::Typography,
        }
    }
}

/// Expected type of the node at `path` inside a value of type `ty` (V6).
/// Returns `None` when the position carries no token type.
#[must_use]
pub fn field_type(ty: TokenType, path: &[String]) -> Option<TokenType> {
    match (ty, path) {
        (TokenType::Color, [f, i]) if f == "components" && is_index(i) => Some(TokenType::Number),
        (TokenType::Color, [f]) if f == "alpha" => Some(TokenType::Number),
        (TokenType::Dimension | TokenType::Duration, [f]) if f == "value" => {
            Some(TokenType::Number)
        }
        (TokenType::Typography, [f]) => typography_field(f),
        (TokenType::Typography, [f, i]) if f == "fontFamily" && is_index(i) => {
            Some(TokenType::FontFamily)
        }
        (TokenType::Shadow, [i]) if is_index(i) => Some(TokenType::Shadow),
        (TokenType::Shadow, [i, f]) if is_index(i) => shadow_field(f),
        (TokenType::Shadow, [f]) => shadow_field(f),
        (TokenType::CubicBezier, [i]) if is_index(i) => Some(TokenType::Number),
        (TokenType::FontFamily, [i]) if is_index(i) => Some(TokenType::FontFamily),
        _ => None,
    }
}

fn typography_field(f: &str) -> Option<TokenType> {
    match f {
        "fontFamily" => Some(TokenType::FontFamily),
        "fontSize" | "letterSpacing" => Some(TokenType::Dimension),
        "fontWeight" => Some(TokenType::FontWeight),
        "lineHeight" => Some(TokenType::Number),
        _ => None,
    }
}

fn shadow_field(f: &str) -> Option<TokenType> {
    match f {
        "color" => Some(TokenType::Color),
        "offsetX" | "offsetY" | "blur" | "spread" => Some(TokenType::Dimension),
        _ => None,
    }
}

fn is_index(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

fn kind(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "a boolean",
        Value::Number(_) => "a number",
        Value::String(_) => "a string",
        Value::Array(_) => "an array",
        Value::Object(_) => "an object",
    }
}

/// Require an object with exactly the given keys (plus optional ones).
fn object<'a>(
    v: &'a Value,
    what: &str,
    required: &[&str],
    optional: &[&str],
) -> Result<&'a Map<String, Value>, String> {
    let Value::Object(map) = v else {
        return Err(format!("{what} must be an object, found {}", kind(v)));
    };
    if let Some(k) = map
        .keys()
        .find(|k| !required.contains(&k.as_str()) && !optional.contains(&k.as_str()))
    {
        return Err(format!("{what} has unexpected key \"{k}\""));
    }
    if let Some(k) = required.iter().find(|k| !map.contains_key(**k)) {
        return Err(format!("{what} is missing \"{k}\""));
    }
    Ok(map)
}

fn required<'a>(map: &'a Map<String, Value>, key: &str) -> Result<&'a Value, String> {
    map.get(key).ok_or_else(|| format!("missing \"{key}\""))
}

fn number(v: &Value) -> Result<f64, String> {
    match v {
        Value::Number(n) => n
            .as_f64()
            .filter(|x| x.is_finite())
            .ok_or_else(|| "number is not finite".to_owned()),
        other => Err(format!("expected a number, found {}", kind(other))),
    }
}

fn number_in(v: &Value, what: &str, lo: f64, hi: f64) -> Result<f64, String> {
    let x = number(v).map_err(|e| format!("{what}: {e}"))?;
    if x < lo || x > hi {
        return Err(format!("{what} {x} is outside {lo}..={hi}"));
    }
    Ok(x)
}

fn string<'a>(v: &'a Value, what: &str) -> Result<&'a str, String> {
    match v {
        Value::String(s) => Ok(s),
        other => Err(format!("{what} must be a string, found {}", kind(other))),
    }
}

fn color(v: &Value) -> Result<Color, String> {
    let m = object(
        v,
        "colour",
        &["colorSpace", "components"],
        &["alpha", "hex"],
    )?;
    let space = string(required(m, "colorSpace")?, "colour colorSpace")?;
    if space != "srgb" {
        return Err(format!(
            "colour colorSpace must be \"srgb\", found \"{space}\""
        ));
    }
    let Value::Array(items) = required(m, "components")? else {
        return Err("colour components must be an array of 3 numbers".to_owned());
    };
    if items.len() != 3 {
        return Err(format!(
            "colour components must have 3 entries, found {}",
            items.len()
        ));
    }
    let mut components = [0.0; 3];
    for (slot, item) in components.iter_mut().zip(items) {
        *slot = number_in(item, "colour component", 0.0, 1.0)?;
    }
    let alpha = match m.get("alpha") {
        Some(a) => number_in(a, "colour alpha", 0.0, 1.0)?,
        None => 1.0,
    };
    if let Some(hex) = m.get("hex") {
        let hex = string(hex, "colour hex")?;
        let expected = rgb_hex(&components);
        if !hex.eq_ignore_ascii_case(&expected) {
            return Err(format!(
                "colour hex \"{hex}\" does not match the components (rounded: {expected})"
            ));
        }
    }
    Ok(Color { components, alpha })
}

fn length_unit(v: &Value) -> Result<LengthUnit, String> {
    match string(v, "dimension unit")? {
        "px" => Ok(LengthUnit::Px),
        "rem" => Ok(LengthUnit::Rem),
        other => Err(format!(
            "dimension unit must be \"px\" or \"rem\", found \"{other}\""
        )),
    }
}

fn dimension(v: &Value) -> Result<Dimension, String> {
    let m = object(v, "dimension", &["value", "unit"], &[])?;
    let value = number(required(m, "value")?).map_err(|e| format!("dimension value: {e}"))?;
    let unit = length_unit(required(m, "unit")?)?;
    Ok(Dimension { value, unit })
}

fn duration(v: &Value) -> Result<Duration, String> {
    let m = object(v, "duration", &["value", "unit"], &[])?;
    let value = number_in(required(m, "value")?, "duration value", 0.0, f64::MAX)?;
    let unit = match string(required(m, "unit")?, "duration unit")? {
        "ms" => TimeUnit::Ms,
        "s" => TimeUnit::S,
        other => {
            return Err(format!(
                "duration unit must be \"ms\" or \"s\", found \"{other}\""
            ));
        }
    };
    Ok(Duration { value, unit })
}

fn font_family(v: &Value) -> Result<Vec<String>, String> {
    let non_blank = |s: &str| -> Result<String, String> {
        if s.trim().is_empty() {
            Err("font family names must not be blank".to_owned())
        } else {
            Ok(s.to_owned())
        }
    };
    match v {
        Value::String(s) => Ok(vec![non_blank(s)?]),
        Value::Array(items) if !items.is_empty() => items
            .iter()
            .map(|item| non_blank(string(item, "font family name")?))
            .collect(),
        Value::Array(_) => Err("fontFamily array must not be empty".to_owned()),
        other => Err(format!(
            "fontFamily must be a string or an array of strings, found {}",
            kind(other)
        )),
    }
}

fn font_weight(v: &Value) -> Result<f64, String> {
    number_in(v, "fontWeight", 1.0, 1000.0)
}

fn cubic_bezier(v: &Value) -> Result<[f64; 4], String> {
    let Value::Array(items) = v else {
        return Err(format!(
            "cubicBezier must be an array of 4 numbers, found {}",
            kind(v)
        ));
    };
    if items.len() != 4 {
        return Err(format!(
            "cubicBezier must have 4 entries, found {}",
            items.len()
        ));
    }
    let mut out = [0.0; 4];
    for (i, (slot, item)) in out.iter_mut().zip(items).enumerate() {
        *slot = match i {
            0 | 2 => number_in(item, "cubicBezier x control point", 0.0, 1.0)?,
            _ => number(item).map_err(|e| format!("cubicBezier: {e}"))?,
        };
    }
    Ok(out)
}

fn shadow(v: &Value) -> Result<Shadow, String> {
    match v {
        Value::Object(_) => Ok(Shadow::Single(shadow_layer(v)?)),
        Value::Array(items) if !items.is_empty() => items
            .iter()
            .enumerate()
            .map(|(i, item)| shadow_layer(item).map_err(|e| format!("shadow[{i}]: {e}")))
            .collect::<Result<Vec<_>, _>>()
            .map(Shadow::Layers),
        Value::Array(_) => Err("shadow array must not be empty".to_owned()),
        other => Err(format!(
            "shadow must be an object or an array, found {}",
            kind(other)
        )),
    }
}

fn shadow_layer(v: &Value) -> Result<ShadowLayer, String> {
    let m = object(
        v,
        "shadow layer",
        &["color", "offsetX", "offsetY", "blur", "spread"],
        &["inset"],
    )?;
    let blur = dimension(required(m, "blur")?).map_err(|e| format!("blur: {e}"))?;
    if blur.value < 0.0 {
        return Err(format!("blur {} must be >= 0", blur.value));
    }
    let inset = match m.get("inset") {
        None => false,
        Some(Value::Bool(b)) => *b,
        Some(other) => return Err(format!("inset must be a boolean, found {}", kind(other))),
    };
    Ok(ShadowLayer {
        color: color(required(m, "color")?).map_err(|e| format!("color: {e}"))?,
        offset_x: dimension(required(m, "offsetX")?).map_err(|e| format!("offsetX: {e}"))?,
        offset_y: dimension(required(m, "offsetY")?).map_err(|e| format!("offsetY: {e}"))?,
        blur,
        spread: dimension(required(m, "spread")?).map_err(|e| format!("spread: {e}"))?,
        inset,
    })
}

fn typography(v: &Value) -> Result<Typography, String> {
    let m = object(
        v,
        "typography",
        &[
            "fontFamily",
            "fontSize",
            "fontWeight",
            "letterSpacing",
            "lineHeight",
        ],
        &[],
    )?;
    let font_size = dimension(required(m, "fontSize")?).map_err(|e| format!("fontSize: {e}"))?;
    if font_size.value <= 0.0 {
        return Err(format!("fontSize {} must be > 0", font_size.value));
    }
    let line_height = number(required(m, "lineHeight")?).map_err(|e| format!("lineHeight: {e}"))?;
    if line_height <= 0.0 {
        return Err(format!("lineHeight {line_height} must be > 0"));
    }
    Ok(Typography {
        font_family: font_family(required(m, "fontFamily")?)
            .map_err(|e| format!("fontFamily: {e}"))?,
        font_size,
        font_weight: font_weight(required(m, "fontWeight")?)
            .map_err(|e| format!("fontWeight: {e}"))?,
        letter_spacing: dimension(required(m, "letterSpacing")?)
            .map_err(|e| format!("letterSpacing: {e}"))?,
        line_height,
    })
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::needless_pass_by_value,
    reason = "test helpers take owned JSON for readable case tables"
)]
mod tests {
    use rstest::rstest;
    use serde_json::json;

    use super::*;

    fn parse(ty: TokenType, v: Value) -> Result<TypedValue, String> {
        TypedValue::parse(ty, &v)
    }

    fn assert_ok(ty: TokenType, v: Value) {
        if let Err(e) = parse(ty, v.clone()) {
            panic!("{ty} should accept {v}: {e}");
        }
    }

    fn assert_err(ty: TokenType, v: Value) {
        assert!(parse(ty, v.clone()).is_err(), "{ty} should reject {v}");
    }

    fn srgb(c: Value) -> Value {
        json!({"colorSpace": "srgb", "components": c})
    }

    #[rstest]
    #[case(json!({"colorSpace":"srgb","components":[0,0,0]}))]
    #[case(json!({"colorSpace":"srgb","components":[1,0.5,0],"alpha":0.5}))]
    #[case(json!({"colorSpace":"srgb","components":[1,0,0],"hex":"#FF0000"}))]
    #[case(json!({"colorSpace":"srgb","components":[0.5,0.5,0.5],"alpha":1,"hex":"#808080"}))]
    fn v1_colour_valid(#[case] v: Value) {
        assert_ok(TokenType::Color, v);
    }

    #[rstest]
    #[case(json!({"colorSpace":"display-p3","components":[0,0,0]}))]
    #[case(json!({"components":[0,0,0]}))]
    #[case(json!({"colorSpace":"srgb","components":[0,0]}))]
    #[case(json!({"colorSpace":"srgb","components":[0,0,1.01]}))]
    #[case(json!({"colorSpace":"srgb","components":[0,0,-0.01]}))]
    #[case(json!({"colorSpace":"srgb","components":["0",0,0]}))]
    #[case(json!({"colorSpace":"srgb","components":[true,0,0]}))]
    #[case(json!({"colorSpace":"srgb","components":[0,0,0],"alpha":1.5}))]
    #[case(json!({"colorSpace":"srgb","components":[0,0,0],"alpha":true}))]
    #[case(json!({"colorSpace":"srgb","components":[0,0,0],"hex":"#000001"}))]
    #[case(json!({"colorSpace":"srgb","components":[1,0,0],"hex":"ff0000"}))]
    #[case(json!({"colorSpace":"srgb","components":[1,0,0],"hex":"#ff00"}))]
    #[case(json!({"colorSpace":"srgb","components":[0,0,0],"extra":1}))]
    #[case(json!("#000000"))]
    #[case(json!(null))]
    fn v1_colour_invalid(#[case] v: Value) {
        assert_err(TokenType::Color, v);
    }

    #[test]
    fn colour_hex_rounds_half_up_and_appends_alpha_only_below_255() {
        let c = Color {
            components: [0.5, 0.5 / 255.0, 1.0],
            alpha: 1.0,
        };
        assert_eq!(c.to_hex(), "#8001ff");
        let translucent = Color {
            components: [0.0, 0.0, 0.0],
            alpha: 0.5,
        };
        assert_eq!(translucent.to_hex(), "#00000080");
        let opaque_alpha = Color {
            components: [1.0, 1.0, 1.0],
            alpha: 0.999,
        };
        assert_eq!(opaque_alpha.to_hex(), "#ffffff");
    }

    #[test]
    fn colour_alpha_defaults_to_one_and_hex_is_checked_against_rounded_components() {
        let TypedValue::Color(c) =
            parse(TokenType::Color, srgb(json!([0.465, 0.465, 0.465]))).unwrap()
        else {
            panic!("not a colour")
        };
        assert_eq!(c.alpha, 1.0);
        // 0.465 * 255 = 118.575 -> 119 (0x77); a hex of #767676 must fail.
        let bad = json!({"colorSpace":"srgb","components":[0.465,0.465,0.465],"hex":"#767676"});
        assert!(parse(TokenType::Color, bad).is_err());
    }

    #[rstest]
    #[case(json!({"value":16,"unit":"px"}))]
    #[case(json!({"value":-2.5,"unit":"rem"}))]
    #[case(json!({"value":0,"unit":"px"}))]
    fn v2_dimension_valid(#[case] v: Value) {
        assert_ok(TokenType::Dimension, v);
    }

    #[rstest]
    #[case(json!({"value":"16","unit":"px"}))]
    #[case(json!({"value":true,"unit":"px"}))]
    #[case(json!({"value":16,"unit":"pt"}))]
    #[case(json!({"value":16}))]
    #[case(json!({"unit":"px"}))]
    #[case(json!({"value":16,"unit":"px","extra":1}))]
    #[case(json!(16))]
    #[case(json!("16px"))]
    fn v2_dimension_invalid(#[case] v: Value) {
        assert_err(TokenType::Dimension, v);
    }

    #[test]
    fn dimension_px_converts_rem_at_sixteen() {
        let TypedValue::Dimension(d) =
            parse(TokenType::Dimension, json!({"value":2.75,"unit":"rem"})).unwrap()
        else {
            panic!()
        };
        assert!((d.px() - 44.0).abs() < 1e-12);
    }

    #[rstest]
    #[case(json!({"value":0,"unit":"ms"}))]
    #[case(json!({"value":0.26,"unit":"s"}))]
    fn v2_duration_valid(#[case] v: Value) {
        assert_ok(TokenType::Duration, v);
    }

    #[rstest]
    #[case(json!({"value":-1,"unit":"ms"}))]
    #[case(json!({"value":1,"unit":"px"}))]
    #[case(json!({"value":"1","unit":"ms"}))]
    #[case(json!({"value":1,"unit":"ms","x":0}))]
    #[case(json!(100))]
    fn v2_duration_invalid(#[case] v: Value) {
        assert_err(TokenType::Duration, v);
    }

    #[test]
    fn duration_ms_converts_seconds() {
        let TypedValue::Duration(d) =
            parse(TokenType::Duration, json!({"value":0.26,"unit":"s"})).unwrap()
        else {
            panic!()
        };
        assert!((d.ms() - 260.0).abs() < 1e-9);
    }

    #[rstest]
    #[case(json!(0))]
    #[case(json!(-3.5))]
    #[case(json!(1e300))]
    fn v3_number_valid(#[case] v: Value) {
        assert_ok(TokenType::Number, v);
    }

    #[rstest]
    #[case(json!(true))]
    #[case(json!("1"))]
    #[case(json!([1]))]
    #[case(json!(null))]
    fn v6_number_rejects_booleans_and_others(#[case] v: Value) {
        assert_err(TokenType::Number, v);
    }

    #[rstest]
    #[case(json!(1))]
    #[case(json!(400))]
    #[case(json!(1000))]
    fn v3_font_weight_valid(#[case] v: Value) {
        assert_ok(TokenType::FontWeight, v);
    }

    #[rstest]
    #[case(json!(0))]
    #[case(json!(1001))]
    #[case(json!(true))]
    #[case(json!("bold"))]
    fn v3_font_weight_invalid(#[case] v: Value) {
        assert_err(TokenType::FontWeight, v);
    }

    #[rstest]
    #[case(json!("Noto Sans"))]
    #[case(json!(["Noto Sans", "system-ui"]))]
    fn v3_font_family_valid(#[case] v: Value) {
        assert_ok(TokenType::FontFamily, v);
    }

    #[rstest]
    #[case(json!(""))]
    #[case(json!("   "))]
    #[case(json!([]))]
    #[case(json!(["Noto Sans", " "]))]
    #[case(json!([1]))]
    #[case(json!({"name":"Noto Sans"}))]
    fn v3_font_family_invalid(#[case] v: Value) {
        assert_err(TokenType::FontFamily, v);
    }

    #[test]
    fn single_string_font_family_normalises_to_one_element_vector() {
        assert_eq!(
            parse(TokenType::FontFamily, json!("Noto Sans")).unwrap(),
            TypedValue::FontFamily(vec!["Noto Sans".to_owned()])
        );
    }

    #[rstest]
    #[case(json!([0.25, 0.1, 0.25, 1]))]
    #[case(json!([0, 0, 1, 1]))]
    #[case(json!([0.2, -3, 0.8, 9]))]
    fn v3_cubic_bezier_valid(#[case] v: Value) {
        assert_ok(TokenType::CubicBezier, v);
    }

    #[rstest]
    #[case(json!([0.25, 0.1, 1.2, 1]))]
    #[case(json!([-0.1, 0.1, 0.5, 1]))]
    #[case(json!([0.25, 0.1, 0.25]))]
    #[case(json!([0.25, 0.1, 0.25, true]))]
    #[case(json!([0.25, 0.1, 0.25, "1"]))]
    #[case(json!("ease"))]
    fn v3_cubic_bezier_invalid(#[case] v: Value) {
        assert_err(TokenType::CubicBezier, v);
    }

    fn shadow_obj() -> Value {
        json!({
            "color": {"colorSpace":"srgb","components":[0,0,0],"alpha":0.2},
            "offsetX": {"value":0,"unit":"px"},
            "offsetY": {"value":1,"unit":"px"},
            "blur": {"value":2,"unit":"px"},
            "spread": {"value":0,"unit":"px"}
        })
    }

    #[test]
    fn v4_shadow_single_object_and_inset_default() {
        let TypedValue::Shadow(s) = parse(TokenType::Shadow, shadow_obj()).unwrap() else {
            panic!()
        };
        assert!(matches!(&s, Shadow::Single(l) if !l.inset));
        assert_eq!(s.layers().len(), 1);
    }

    #[test]
    fn v4_shadow_array_keeps_order_and_inset_flag() {
        let mut inset = shadow_obj();
        inset["inset"] = json!(true);
        let TypedValue::Shadow(s) = parse(TokenType::Shadow, json!([shadow_obj(), inset])).unwrap()
        else {
            panic!()
        };
        let layers = s.layers();
        assert_eq!(layers.len(), 2);
        assert!(!layers[0].inset);
        assert!(layers[1].inset);
    }

    #[test]
    fn v4_shadow_rejects_nested_arrays_and_empty_arrays() {
        assert!(parse(TokenType::Shadow, json!([[shadow_obj()]])).is_err());
        assert!(parse(TokenType::Shadow, json!([])).is_err());
        assert!(parse(TokenType::Shadow, json!([shadow_obj(), 3])).is_err());
    }

    #[rstest]
    #[case("blur")]
    #[case("offsetX")]
    #[case("spread")]
    #[case("color")]
    fn v4_shadow_missing_field_rejected(#[case] field: &str) {
        let mut v = shadow_obj();
        v.as_object_mut().unwrap().remove(field);
        assert_err(TokenType::Shadow, v);
    }

    #[test]
    fn v4_shadow_negative_blur_rejected_but_negative_offset_allowed() {
        let mut v = shadow_obj();
        v["blur"]["value"] = json!(-1);
        assert_err(TokenType::Shadow, v.clone());
        let mut offset = shadow_obj();
        offset["offsetX"]["value"] = json!(-4);
        assert_ok(TokenType::Shadow, offset);
    }

    #[test]
    fn v4_shadow_inset_must_be_boolean_and_no_extra_keys() {
        let mut v = shadow_obj();
        v["inset"] = json!("yes");
        assert_err(TokenType::Shadow, v);
        let mut extra = shadow_obj();
        extra["spreadX"] = json!({"value":0,"unit":"px"});
        assert_err(TokenType::Shadow, extra);
    }

    fn typo() -> Value {
        json!({
            "fontFamily": ["Noto Sans", "system-ui"],
            "fontSize": {"value":1,"unit":"rem"},
            "fontWeight": 400,
            "letterSpacing": {"value":0,"unit":"px"},
            "lineHeight": 1.5
        })
    }

    #[test]
    fn v5_typography_valid() {
        let TypedValue::Typography(t) = parse(TokenType::Typography, typo()).unwrap() else {
            panic!()
        };
        assert_eq!(t.font_family, vec!["Noto Sans", "system-ui"]);
        assert!((t.font_size.px() - 16.0).abs() < 1e-12);
        assert!((t.line_height - 1.5).abs() < 1e-12);
    }

    #[rstest]
    #[case("fontSize", json!({"value":0,"unit":"px"}))]
    #[case("fontSize", json!({"value":-1,"unit":"px"}))]
    #[case("lineHeight", json!(0))]
    #[case("fontWeight", json!(1001))]
    #[case("fontFamily", json!(""))]
    #[case("letterSpacing", json!(4))]
    fn v5_typography_invalid_fields(#[case] field: &str, #[case] bad: Value) {
        let mut v = typo();
        v[field] = bad;
        assert_err(TokenType::Typography, v);
    }

    #[test]
    fn v5_typography_extra_or_missing_key_rejected() {
        let mut extra = typo();
        extra["color"] = json!("#000");
        assert_err(TokenType::Typography, extra);
        let mut missing = typo();
        missing.as_object_mut().unwrap().remove("lineHeight");
        assert_err(TokenType::Typography, missing);
    }

    fn path(segs: &[&str]) -> Vec<String> {
        segs.iter().map(|s| (*s).to_owned()).collect()
    }

    #[rstest]
    #[case(TokenType::Color, &["components", "0"], Some(TokenType::Number))]
    #[case(TokenType::Color, &["components", "2"], Some(TokenType::Number))]
    #[case(TokenType::Color, &["alpha"], Some(TokenType::Number))]
    #[case(TokenType::Color, &["components"], None)]
    #[case(TokenType::Color, &["colorSpace"], None)]
    #[case(TokenType::Dimension, &["value"], Some(TokenType::Number))]
    #[case(TokenType::Dimension, &["unit"], None)]
    #[case(TokenType::Duration, &["value"], Some(TokenType::Number))]
    #[case(TokenType::Typography, &["fontFamily"], Some(TokenType::FontFamily))]
    #[case(TokenType::Typography, &["fontFamily", "1"], Some(TokenType::FontFamily))]
    #[case(TokenType::Typography, &["fontSize"], Some(TokenType::Dimension))]
    #[case(TokenType::Typography, &["letterSpacing"], Some(TokenType::Dimension))]
    #[case(TokenType::Typography, &["fontWeight"], Some(TokenType::FontWeight))]
    #[case(TokenType::Typography, &["lineHeight"], Some(TokenType::Number))]
    #[case(TokenType::Typography, &["nope"], None)]
    #[case(TokenType::Shadow, &["0"], Some(TokenType::Shadow))]
    #[case(TokenType::Shadow, &["1", "color"], Some(TokenType::Color))]
    #[case(TokenType::Shadow, &["0", "offsetX"], Some(TokenType::Dimension))]
    #[case(TokenType::Shadow, &["0", "offsetY"], Some(TokenType::Dimension))]
    #[case(TokenType::Shadow, &["0", "blur"], Some(TokenType::Dimension))]
    #[case(TokenType::Shadow, &["0", "spread"], Some(TokenType::Dimension))]
    #[case(TokenType::Shadow, &["0", "inset"], None)]
    #[case(TokenType::Shadow, &["color"], Some(TokenType::Color))]
    #[case(TokenType::CubicBezier, &["2"], Some(TokenType::Number))]
    #[case(TokenType::CubicBezier, &["x"], None)]
    #[case(TokenType::FontFamily, &["0"], Some(TokenType::FontFamily))]
    #[case(TokenType::Number, &["0"], None)]
    #[case(TokenType::Color, &[], None)]
    fn v6_field_type_table(
        #[case] ty: TokenType,
        #[case] segs: &[&str],
        #[case] expected: Option<TokenType>,
    ) {
        assert_eq!(field_type(ty, &path(segs)), expected);
    }

    #[test]
    fn token_type_names_round_trip() {
        for ty in TokenType::ALL {
            assert_eq!(ty.as_str().parse::<TokenType>().unwrap(), ty);
        }
        assert!("colour".parse::<TokenType>().is_err());
    }
}
