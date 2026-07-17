use crate::classify::Classified;
use crate::spec::{PropType, Spec};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

// Rust reserved and weak keywords, including edition-2024 additions (`gen`) and
// reserved-for-future-use words (`abstract`, `become`, `box`, `do`, `final`,
// `override`, `priv`, `typeof`, `unsized`, `virtual`, `yield`). Kept alphabetically
// sorted.
const RUST_KEYWORDS: &[&str] = &[
    "abstract", "as", "async", "await", "become", "box", "break", "const", "continue", "crate",
    "do", "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "gen", "if", "impl",
    "in", "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref",
    "return", "self", "static", "struct", "super", "trait", "true", "try", "type", "typeof",
    "unsafe", "unsized", "use", "virtual", "where", "while", "yield",
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

/// "mail_protocol" -> "MailProtocol" (for enum type names derived from the
/// snake_case output of `rust_field_name`; splits on `_` and pascalizes each
/// segment, so e.g. the keyword-escaped "type_" -> "Type").
fn type_pascal(field: &str) -> String {
    field
        .split('_')
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut c = w.chars();
            c.next().map(|f| f.to_ascii_uppercase().to_string() + c.as_str()).unwrap_or_default()
        })
        .collect()
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

/// Resolved properties as they should appear on the emitted struct.
///
/// `$type` is stripped only when keeping it would collide with another property
/// that maps to the same Rust field name (`type_`) — e.g. a schema with its own
/// `type` property. In that case the discriminator can't be represented as a
/// struct field anyway, and when this schema is used as a variant payload the
/// enclosing `#[serde(tag = "$type")]` enum already round-trips it without help:
/// on deserialize the tag is consumed by the enum and never reaches the
/// payload; on serialize the enum always (re)injects its own tag, ignoring
/// whatever the payload itself may hold for that field.
///
/// When there is no collision, `$type` is kept as an ordinary field. This
/// matters because a variant schema can *also* be referenced directly by
/// another schema's property, bypassing the root's tagged wrapper entirely
/// (e.g. `SingleEnumIssueCustomField.value: EnumBundleElement` — `value` names
/// the concrete variant schema, not the `BundleElement` root). In that context
/// the struct is serialized/deserialized on its own, with no enum around to
/// supply the tag, so the field must carry it for the round-trip to stay
/// lossless.
fn struct_props(spec: &Spec, cls: &Classified, schema_name: &str) -> BTreeMap<String, PropType> {
    let mut props = spec.resolved_props(schema_name);
    if cls.variant_of.contains_key(schema_name) {
        let collides = props.keys().any(|k| k != "$type" && rust_field_name(k) == "type_");
        if collides {
            props.remove("$type");
        }
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

const HEADER: &str = "// @generated by `cargo xtask codegen` \u{2014} do not edit.\n\n";

pub fn emit_all(spec: &Spec, cls: &Classified) -> BTreeMap<String, String> {
    let mut by_domain: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for name in spec.schemas.keys() {
        by_domain.entry(crate::classify::domain_of(name)).or_default().push(name);
    }

    let mut files = BTreeMap::new();
    for (domain, names) in &by_domain {
        let mut src = String::from(HEADER);
        src.push_str("use serde::{Deserialize, Serialize};\n");
        src.push_str("use super::*;\n\n");
        for name in names {
            if cls.roots.contains_key(*name) {
                emit_root(spec, cls, name, &mut src);
            } else {
                emit_struct(spec, cls, name, name, &mut src);
            }
        }
        files.insert(format!("{domain}.rs"), src);
    }

    let mut mod_rs = String::from(HEADER);
    mod_rs.push_str("#![allow(clippy::large_enum_variant)]\n\n");
    for domain in by_domain.keys() {
        writeln!(mod_rs, "pub mod {domain};").unwrap();
    }
    mod_rs.push('\n');
    for domain in by_domain.keys() {
        writeln!(mod_rs, "pub use {domain}::*;").unwrap();
    }
    files.insert("mod.rs".into(), mod_rs);
    files
}

fn emit_root(spec: &Spec, cls: &Classified, name: &str, out: &mut String) {
    let variants = &cls.roots[name];
    let data_name = format!("{name}Data");

    writeln!(out, "/// Polymorphic `{name}`; unrecognized `$type`s from newer YouTrack").unwrap();
    writeln!(out, "/// versions degrade to `Unknown` instead of failing the whole parse.").unwrap();
    writeln!(out, "#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]").unwrap();
    writeln!(out, "#[serde(untagged)]").unwrap();
    writeln!(out, "pub enum {name} {{").unwrap();
    writeln!(out, "    Known({name}Kind),").unwrap();
    writeln!(out, "    Unknown(serde_json::Value),").unwrap();
    writeln!(out, "}}\n").unwrap();

    writeln!(out, "#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]").unwrap();
    writeln!(out, r##"#[serde(tag = "$type")]"##).unwrap();
    writeln!(out, "pub enum {name}Kind {{").unwrap();
    for v in variants {
        // The enum variant name must be the WIRE TAG (serde uses the variant name as
        // the `$type` value); the payload type must be the TARGET SCHEMA. These
        // differ for 2 of 143 mapping entries (both under IssueCustomField), so the
        // two must never be conflated.
        let payload = if v.schema == name { data_name.as_str() } else { v.schema.as_str() };
        writeln!(out, "    {}({payload}),", v.tag).unwrap();
    }
    writeln!(out, "}}\n").unwrap();

    // the root itself is concrete and appears in its own mapping
    emit_struct(spec, cls, name, &data_name, out);

    emit_accessors(spec, cls, name, out);
}

fn emit_accessors(spec: &Spec, cls: &Classified, root: &str, out: &mut String) {
    let props = struct_props(spec, cls, root); // root is its own variant
    let variants = &cls.roots[root];

    // Accessors are only sound if every variant declares the field with the same
    // type as the root. Some props (e.g. ActivityItem.added/removed/target,
    // IssueCustomField.value, ChangesProcessor.server) are declared broadly on the
    // root and narrowed to incompatible concrete types per variant by design; a
    // uniform accessor is impossible for those, so they are skipped entirely rather
    // than causing a panic. Callers match on the `{Root}Kind` enum to get the
    // properly typed value for those props.
    let uniform_props: Vec<(&String, &PropType)> = props
        .iter()
        // `$type` is excluded even when `struct_props` kept it as a field (see its
        // doc comment): when reached through `{Root}Kind`, the tag is consumed by
        // the enum during deserialize and never populates the payload struct, so
        // an accessor here would silently and misleadingly always return `None`.
        .filter(|(orig, _)| orig.as_str() != "$type")
        .filter(|(orig, ty)| {
            variants.iter().all(|v| {
                spec.resolved_props(&v.schema).get(orig.as_str()).is_some_and(|vt| vt == *ty)
            })
        })
        .collect();

    writeln!(out, "impl {root} {{").unwrap();
    for (orig, ty) in &uniform_props {
        let field = rust_field_name(orig);
        let (ret, _) = accessor_parts(root, orig, ty);
        writeln!(out, "    pub fn {field}(&self) -> {ret} {{").unwrap();
        writeln!(out, "        match self {{").unwrap();
        writeln!(out, "            Self::Known(k) => k.{field}(),").unwrap();
        writeln!(out, "            Self::Unknown(_) => None,").unwrap();
        writeln!(out, "        }}").unwrap();
        writeln!(out, "    }}").unwrap();
    }
    writeln!(out, "}}\n").unwrap();

    writeln!(out, "impl {root}Kind {{").unwrap();
    for (orig, ty) in &uniform_props {
        let field = rust_field_name(orig);
        let (ret, access) = accessor_parts(root, orig, ty);
        writeln!(out, "    pub fn {field}(&self) -> {ret} {{").unwrap();
        writeln!(out, "        match self {{").unwrap();
        for v in variants {
            writeln!(out, "            Self::{}(x) => x.{field}{access},", v.tag).unwrap();
        }
        writeln!(out, "        }}").unwrap();
        writeln!(out, "    }}").unwrap();
    }
    writeln!(out, "}}\n").unwrap();
}

/// (return type, access-expression suffix applied to `x.field`)
fn accessor_parts(root: &str, orig: &str, ty: &PropType) -> (String, &'static str) {
    match ty {
        PropType::String => ("Option<&str>".into(), ".as_deref()"),
        PropType::I32 => ("Option<i32>".into(), ""),
        PropType::I64 => ("Option<i64>".into(), ""),
        PropType::Bool => ("Option<bool>".into(), ""),
        PropType::Json => ("Option<&serde_json::Value>".into(), ".as_ref()"),
        PropType::Ref(t) => (format!("Option<&{t}>"), ".as_deref()"),
        PropType::Array(inner) => {
            (format!("Option<&[{}]>", element_type(root, orig, inner)), ".as_deref()")
        }
        PropType::StrEnum(_) => {
            panic!("{root}.{orig}: string enums on polymorphic roots are not supported")
        }
    }
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

        // The enum's variant LIST, asserted distinctly from any match arm: this exact
        // substring (open brace, newline, 4-space-indented "Outward,") can only occur
        // in the `pub enum IssueLinkDirection { ... }` declaration itself. Match arms
        // referencing the same variant are always qualified (`Self::Outward,` /
        // `IssueLinkDirection::Outward,`), so they never produce a bare 4-space
        // "Outward," line and cannot satisfy this assertion. This would fail if the
        // variant list were emitted empty.
        assert!(src.contains("pub enum IssueLinkDirection {\n    Outward,\n"));

        // Both conversion impls required by `#[serde(from = "String", into = "String")]`
        // must actually be emitted; deleting either would leave code that does not
        // compile even though the weaker assertions above would still pass.
        assert!(src.contains("impl From<String> for IssueLinkDirection {"));
        assert!(src.contains("impl From<IssueLinkDirection> for String {"));

        // Round-trip arms, both directions.
        assert!(src.contains(r#""OUTWARD" => Self::Outward,"#));
        assert!(src.contains(r#"IssueLinkDirection::Outward => "OUTWARD".to_owned(),"#));

        // Lossless `Other` catch-all, both directions.
        assert!(src.contains("_ => Self::Other(s),"));
        assert!(src.contains("IssueLinkDirection::Other(s) => s,"));
    }

    // 2 of the spec's 3 string enums (the third, `IssueLink.direction`, is a single
    // word and so cannot catch this class of bug): `type_pascal` used to only
    // uppercase the first character of the already-snake_case field name, producing
    // e.g. `EmailSettingsMail_protocol` instead of `EmailSettingsMailProtocol`.
    #[test]
    fn str_enum_names_are_camel_case() {
        let src = emitted("EmailSettings");
        assert!(src.contains("pub enum EmailSettingsMailProtocol {"));
        assert!(src.contains("pub mail_protocol: Option<EmailSettingsMailProtocol>,"));
        assert!(!enum_type_name(&src).contains('_'));

        let src = emitted("DatabaseBackupSettings");
        assert!(src.contains("pub enum DatabaseBackupSettingsArchiveFormat {"));
        assert!(!enum_type_name(&src).contains('_'));
    }

    /// Extracts the type name out of the (sole, in these two fixtures) `pub enum
    /// NAME {` declaration, so the "no underscore" check inspects the actual emitted
    /// name rather than restating a hardcoded expectation.
    fn enum_type_name(src: &str) -> &str {
        let after = src.split("pub enum ").nth(1).expect("no `pub enum` in source");
        after.split(" {").next().expect("malformed enum declaration")
    }

    fn all_files() -> std::collections::BTreeMap<String, String> {
        let mut spec = load();
        apply_overrides(&mut spec);
        let cls = classify(&spec);
        emit_all(&spec, &cls)
    }

    #[test]
    fn root_emits_wrapper_kind_data_and_accessors() {
        let src = &all_files()["user.rs"];
        // untagged forward-compat wrapper keeps the spec name
        assert!(src.contains("pub enum User {"));
        assert!(src.contains("Known(UserKind),"));
        assert!(src.contains("Unknown(serde_json::Value),"));
        assert!(src.contains("#[serde(untagged)]"));
        // tagged dispatch
        assert!(src.contains(r##"#[serde(tag = "$type")]"##));
        assert!(src.contains("pub enum UserKind {"));
        assert!(src.contains("Me(Me),"));
        assert!(src.contains("User(UserData),"));
        assert!(src.contains("VcsUnresolvedUser(VcsUnresolvedUser),"));
        // the root's own concrete struct
        assert!(src.contains("pub struct UserData {"));
        // accessors on wrapper and kind
        assert!(src.contains("pub fn login(&self) -> Option<&str>"));
        assert!(src.contains("pub fn tags(&self) -> Option<&[Tag]>"));
        assert!(src.contains("pub fn profiles(&self) -> Option<&UserProfiles>"));
        assert!(src.contains("Self::Unknown(_) => None"));
    }

    #[test]
    fn activity_root_emits_kind_and_typed_accessors() {
        let src = &all_files()["activity.rs"];
        assert!(src.contains("pub enum ActivityItemKind {"));
        assert!(src.contains("WorkItemActivityItem(WorkItemActivityItem),"));
        assert!(src.contains("pub fn timestamp(&self) -> Option<i64>"));
    }

    /// `ActivityItem.added`/`.removed`/`.target`, `IssueCustomField.value`, and
    /// `ChangesProcessor.server` are declared broadly on the root but narrowed to
    /// concrete, mutually incompatible types by each variant (e.g.
    /// `AttachmentActivityItem.added` is `Vec<IssueAttachment>`, while
    /// `AttachmentActivityItem` and other variants disagree with the root's `object`).
    /// No single return type could soundly represent all variants, so these props
    /// get no uniform accessor at all — callers must match on the `Kind` enum to get
    /// the properly typed value.
    #[test]
    fn divergent_props_get_no_accessor() {
        let files = all_files();

        let activity = &files["activity.rs"];
        assert!(!activity.contains("fn added("));
        assert!(!activity.contains("fn removed("));
        assert!(!activity.contains("fn target("));
        // uniform prop on the same root still gets its accessor
        assert!(activity.contains("pub fn timestamp(&self) -> Option<i64>"));

        let issue = &files["issue.rs"];
        assert!(!issue.contains("fn value("));

        let admin = &files["admin.rs"];
        assert!(!admin.contains("fn server("));
    }

    #[test]
    fn all_eight_domains_plus_mod_rs() {
        let files = all_files();
        let names: Vec<&str> = files.keys().map(String::as_str).collect();
        assert_eq!(
            names,
            [
                "activity.rs",
                "admin.rs",
                "agile.rs",
                "article.rs",
                "common.rs",
                "issue.rs",
                "mod.rs",
                "project.rs",
                "user.rs"
            ]
        );
        let m = &files["mod.rs"];
        assert!(m.contains("// @generated"));
        assert!(m.contains("#![allow(clippy::large_enum_variant)]"));
        assert!(m.contains("pub mod user;"));
        assert!(m.contains("pub use user::*;"));
    }

    #[test]
    fn every_file_has_generated_header() {
        for (name, src) in all_files() {
            assert!(src.starts_with("// @generated"), "{name} missing header");
        }
    }
}
