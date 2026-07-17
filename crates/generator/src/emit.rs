use crate::classify::Classified;
use crate::spec::{PropType, Spec};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

const RUST_KEYWORDS: &[&str] = &[
    "as", "box", "break", "const", "continue", "crate", "do", "dyn", "else", "enum", "extern",
    "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
    "return", "self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use",
    "where", "while", "yield",
];

/// camelCase (or $-prefixed) property name -> snake_case Rust field name.
pub fn rust_field_name(orig: &str) -> String {
    let stripped = orig.strip_prefix('$').unwrap_or(orig);
    let chars: Vec<char> = stripped.chars().collect();
    let mut out = String::new();
    for (i, &c) in chars.iter().enumerate() {
        if c.is_ascii_uppercase() {
            let prev_lower =
                i > 0 && (chars[i - 1].is_ascii_lowercase() || chars[i - 1].is_ascii_digit());
            let next_lower = chars.get(i + 1).is_some_and(|n| n.is_ascii_lowercase());
            let prev_upper = i > 0 && chars[i - 1].is_ascii_uppercase();
            if prev_lower || (prev_upper && next_lower) {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    if RUST_KEYWORDS.contains(&out.as_str()) {
        out.push('_');
    }
    out
}

/// "mailProtocol" -> "MailProtocol" (for enum type names derived from field names).
fn type_pascal(field: &str) -> String {
    let mut c = field.chars();
    c.next().map(|f| f.to_ascii_uppercase().to_string() + c.as_str()).unwrap_or_default()
}

/// "MS_GRAPH_API" -> "MsGraphApi" (for enum variant names derived from values).
fn variant_pascal(value: &str) -> String {
    value
        .split(['_', '-'])
        .filter(|w| !w.is_empty())
        .map(|w| {
            let w = w.to_ascii_lowercase();
            let mut c = w.chars();
            c.next().map(|f| f.to_ascii_uppercase().to_string() + c.as_str()).unwrap_or_default()
        })
        .collect()
}

fn str_enum_name(struct_name: &str, field: &str) -> String {
    format!("{struct_name}{}", type_pascal(&rust_field_name(field)))
}

fn owned_type(struct_name: &str, orig: &str, ty: &PropType) -> String {
    match ty {
        PropType::String => "String".into(),
        PropType::I32 => "i32".into(),
        PropType::I64 => "i64".into(),
        PropType::Bool => "bool".into(),
        PropType::Json => "serde_json::Value".into(),
        PropType::Ref(t) => format!("Box<{t}>"),
        PropType::Array(inner) => format!("Vec<{}>", element_type(struct_name, orig, inner)),
        PropType::StrEnum(_) => str_enum_name(struct_name, orig),
    }
}

fn element_type(struct_name: &str, orig: &str, inner: &PropType) -> String {
    match inner {
        PropType::Ref(t) => t.clone(), // Vec already provides indirection; never Box elements
        PropType::String => "String".into(),
        PropType::I32 => "i32".into(),
        PropType::I64 => "i64".into(),
        PropType::Bool => "bool".into(),
        PropType::Json => "serde_json::Value".into(),
        other => panic!("{struct_name}.{orig}: unsupported array element type {other:?}"),
    }
}

/// Resolved properties as they should appear on the emitted struct:
/// `$type` stripped when the schema is a discriminator-mapping variant.
fn struct_props(spec: &Spec, cls: &Classified, schema_name: &str) -> BTreeMap<String, PropType> {
    let mut props = spec.resolved_props(schema_name);
    if cls.variant_of.contains_key(schema_name) {
        props.remove("$type"); // serde consumes the tag for enum variants
    }
    props
}

pub fn emit_struct(
    spec: &Spec,
    cls: &Classified,
    schema_name: &str,
    struct_name: &str,
    out: &mut String,
) {
    let props = struct_props(spec, cls, schema_name);

    // adjacent string-enum types first
    for (orig, ty) in &props {
        if let PropType::StrEnum(values) = ty {
            emit_str_enum(&str_enum_name(struct_name, orig), values, out);
        }
    }

    // guard against field-name collisions after renaming
    let mut seen = BTreeSet::new();
    for orig in props.keys() {
        let field = rust_field_name(orig);
        assert!(seen.insert(field.clone()), "{schema_name}: field name collision on {field}");
    }

    writeln!(out, "#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]").unwrap();
    writeln!(out, "pub struct {struct_name} {{").unwrap();
    for (orig, ty) in &props {
        let field = rust_field_name(orig);
        if &field == orig {
            writeln!(out, r#"    #[serde(skip_serializing_if = "Option::is_none")]"#).unwrap();
        } else {
            writeln!(
                out,
                r#"    #[serde(rename = "{orig}", skip_serializing_if = "Option::is_none")]"#
            )
            .unwrap();
        }
        writeln!(out, "    pub {field}: Option<{}>,", owned_type(struct_name, orig, ty)).unwrap();
    }
    writeln!(out, "}}\n").unwrap();
}

fn emit_str_enum(name: &str, values: &[String], out: &mut String) {
    writeln!(out, "#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]").unwrap();
    writeln!(out, r#"#[serde(from = "String", into = "String")]"#).unwrap();
    writeln!(out, "pub enum {name} {{").unwrap();
    for v in values {
        writeln!(out, "    {},", variant_pascal(v)).unwrap();
    }
    writeln!(out, "    /// Value not present in the spec this client was generated from.").unwrap();
    writeln!(out, "    Other(String),").unwrap();
    writeln!(out, "}}\n").unwrap();

    writeln!(out, "impl From<String> for {name} {{").unwrap();
    writeln!(out, "    fn from(s: String) -> Self {{").unwrap();
    writeln!(out, "        match s.as_str() {{").unwrap();
    for v in values {
        writeln!(out, r#"            "{v}" => Self::{},"#, variant_pascal(v)).unwrap();
    }
    writeln!(out, "            _ => Self::Other(s),").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out, "}}\n").unwrap();

    writeln!(out, "impl From<{name}> for String {{").unwrap();
    writeln!(out, "    fn from(v: {name}) -> Self {{").unwrap();
    writeln!(out, "        match v {{").unwrap();
    for v in values {
        writeln!(out, r#"            {name}::{} => "{v}".to_owned(),"#, variant_pascal(v)).unwrap();
    }
    writeln!(out, "            {name}::Other(s) => s,").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out, "}}\n").unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classify::{apply_overrides, classify};
    use crate::spec::tests::load;

    fn emitted(schema: &str) -> String {
        let mut spec = load();
        apply_overrides(&mut spec);
        let cls = classify(&spec);
        let mut out = String::new();
        emit_struct(&spec, &cls, schema, schema, &mut out);
        out
    }

    #[test]
    fn field_name_conversion() {
        assert_eq!(rust_field_name("login"), "login");
        assert_eq!(rust_field_name("fullName"), "full_name");
        assert_eq!(rust_field_name("isRTL"), "is_rtl");
        assert_eq!(rust_field_name("thumbnailURL"), "thumbnail_url");
        assert_eq!(rust_field_name("daysAWeek"), "days_a_week");
        assert_eq!(rust_field_name("$type"), "type_");
        assert_eq!(rust_field_name("type"), "type_");
    }

    #[test]
    fn plain_struct_shape() {
        let src = emitted("Agile");
        assert!(src.contains("pub struct Agile {"));
        assert!(
            src.contains("#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]")
        );
        // Agile is NOT in any discriminator mapping, so it keeps $type as a field.
        // (One of the 75 such schemas; the other 143 are mapping variants where serde
        // consumes the tag. Tag itself is NOT a valid example here despite the plan
        // saying so — it is a variant of IssueFolder.)
        assert!(
            src.contains(r#"#[serde(rename = "$type", skip_serializing_if = "Option::is_none")]"#)
        );
        assert!(src.contains("pub type_: Option<String>,"));
        // direct refs boxed
        assert!(src.contains("pub owner: Option<Box<User>>,"));
    }

    #[test]
    fn variant_struct_strips_type_tag_and_boxes_refs() {
        // IssueWorkItem is a variant of BaseWorkItem AND has a plain `type` property
        let src = emitted("IssueWorkItem");
        assert!(!src.contains(r#"rename = "$type""#), "variant must not keep the tag field");
        assert!(
            src.contains(r#"#[serde(rename = "type", skip_serializing_if = "Option::is_none")]"#)
        );
        assert!(src.contains("pub type_: Option<Box<WorkItemType>>,"));
        // inherited from BaseWorkItem
        assert!(src.contains("pub author: Option<Box<User>>,"));
    }

    #[test]
    fn arrays_are_unboxed() {
        let src = emitted("Issue");
        assert!(src.contains("pub custom_fields: Option<Vec<IssueCustomField>>,"));
    }

    #[test]
    fn untyped_object_is_json_value() {
        let mut spec = load();
        apply_overrides(&mut spec);
        let cls = classify(&spec);
        let mut out = String::new();
        emit_struct(&spec, &cls, "ActivityItem", "ActivityItemData", &mut out);
        assert!(out.contains("pub added: Option<serde_json::Value>,"));
    }

    #[test]
    fn override_reaches_emission() {
        let src = emitted("Project");
        assert!(src.contains("pub custom_fields: Option<Vec<ProjectCustomField>>,"));
    }

    #[test]
    fn string_enum_emitted_with_lossless_other() {
        let src = emitted("IssueLink");
        assert!(src.contains("pub enum IssueLinkDirection {"));
        assert!(src.contains("Outward,"));
        assert!(src.contains("Other(String),"));
        assert!(src.contains(r#"#[serde(from = "String", into = "String")]"#));
        assert!(src.contains("pub direction: Option<IssueLinkDirection>,"));
    }
}
