use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum PropType {
    String,
    I32,
    I64,
    Bool,
    /// Untyped `object` in the spec — genuinely polymorphic at runtime.
    Json,
    Ref(String),
    Array(Box<PropType>),
    StrEnum(Vec<String>),
}

/// One entry of a discriminator mapping: the `$type` value seen on the wire,
/// and the schema describing that payload. They differ for 2 of the spec's 143
/// entries, so the two must never be conflated.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Variant {
    /// The `$type` value on the wire; becomes the Rust enum variant name.
    pub tag: String,
    /// The schema describing the payload; becomes the variant's payload type.
    pub schema: String,
}

#[derive(Debug, Clone)]
pub struct Schema {
    pub parent: Option<String>,
    /// The schema's own (inline) properties, unresolved.
    pub props: BTreeMap<String, PropType>,
    /// Discriminator mapping entries (sorted by tag); empty when the schema is not a
    /// polymorphic root.
    pub mapping: Vec<Variant>,
}

#[derive(Debug)]
pub struct Spec {
    pub schemas: BTreeMap<String, Schema>,
}

impl Spec {
    pub fn parse(raw: &str) -> Spec {
        let root: Value = serde_json::from_str(raw).expect("openapi.json is not valid JSON");
        let schemas_json =
            root["components"]["schemas"].as_object().expect("components.schemas missing");
        let mut schemas = BTreeMap::new();
        for (name, def) in schemas_json {
            schemas.insert(name.clone(), parse_schema(name, def));
        }
        Spec { schemas }
    }

    /// All properties of `name` with inheritance resolved: ancestors first,
    /// subtype declarations winning on conflict (covers `Me.id` overriding `User.id`).
    pub fn resolved_props(&self, name: &str) -> BTreeMap<String, PropType> {
        let schema =
            self.schemas.get(name).unwrap_or_else(|| panic!("unknown schema referenced: {name}"));
        let mut props = match &schema.parent {
            Some(parent) => self.resolved_props(parent),
            None => BTreeMap::new(),
        };
        props.extend(schema.props.clone());
        props
    }
}

fn parse_schema(name: &str, def: &Value) -> Schema {
    let mut parent = None;
    let mut props = BTreeMap::new();
    if let Some(all_of) = def["allOf"].as_array() {
        for part in all_of {
            if let Some(r) = part["$ref"].as_str() {
                assert!(parent.is_none(), "{name}: multiple allOf parents");
                parent = Some(ref_name(r));
            } else {
                collect_props(name, part, &mut props);
            }
        }
    } else {
        collect_props(name, def, &mut props);
    }
    let mapping = def["discriminator"]["mapping"]
        .as_object()
        .map(|m| {
            // Keep both the wire tag (the mapping key) and the target schema (the
            // $ref value) distinct: a handful of mappings (e.g. IssueCustomField's
            // `MultiValueIssueCustomField` -> `DatabaseMultiValueIssueCustomField`)
            // use a key that isn't itself a schema name.
            let mut variants: Vec<Variant> =
                m.iter()
                    .map(|(tag, v)| Variant {
                        tag: tag.clone(),
                        schema: ref_name(v.as_str().unwrap_or_else(|| {
                            panic!("{name}: mapping value is not a $ref string")
                        })),
                    })
                    .collect();
            variants.sort();
            variants
        })
        .unwrap_or_default();
    Schema { parent, props, mapping }
}

fn collect_props(schema: &str, def: &Value, out: &mut BTreeMap<String, PropType>) {
    let Some(properties) = def["properties"].as_object() else { return };
    for (prop_name, prop_def) in properties {
        out.insert(prop_name.clone(), parse_type(schema, prop_name, prop_def));
    }
}

fn parse_type(schema: &str, prop: &str, def: &Value) -> PropType {
    if let Some(r) = def["$ref"].as_str() {
        return PropType::Ref(ref_name(r));
    }
    match def["type"].as_str() {
        Some("string") => match def["enum"].as_array() {
            Some(values) => PropType::StrEnum(
                values
                    .iter()
                    .map(|v| v.as_str().expect("non-string enum value").to_owned())
                    .collect(),
            ),
            None => PropType::String,
        },
        Some("boolean") => PropType::Bool,
        Some("integer") => match def["format"].as_str() {
            Some("int64") => PropType::I64,
            Some("int32") | None => PropType::I32,
            Some(other) => panic!("{schema}.{prop}: unknown integer format {other}"),
        },
        Some("object") => PropType::Json,
        Some("array") => PropType::Array(Box::new(parse_type(schema, prop, &def["items"]))),
        other => panic!("{schema}.{prop}: unsupported type {other:?}"),
    }
}

fn ref_name(r: &str) -> String {
    r.rsplit('/').next().unwrap().to_owned()
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn load() -> Spec {
        let raw = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../docs/openapi.json"
        ))
        .expect("spec file");
        Spec::parse(&raw)
    }

    #[test]
    fn parses_all_schemas() {
        assert_eq!(load().schemas.len(), 218);
    }

    #[test]
    fn user_has_own_props_and_mapping() {
        let spec = load();
        let user = &spec.schemas["User"];
        assert!(user.parent.is_none());
        assert_eq!(user.props["login"], PropType::String);
        assert_eq!(user.props["banned"], PropType::Bool);
        assert_eq!(user.props["tags"], PropType::Array(Box::new(PropType::Ref("Tag".into()))));
        assert_eq!(user.props["profiles"], PropType::Ref("UserProfiles".into()));
        // For User, all three tags equal their schemas (no key/target mismatch here).
        assert_eq!(
            user.mapping,
            vec![
                Variant { tag: "Me".into(), schema: "Me".into() },
                Variant { tag: "User".into(), schema: "User".into() },
                Variant { tag: "VcsUnresolvedUser".into(), schema: "VcsUnresolvedUser".into() },
            ]
        );
    }

    #[test]
    fn me_inherits_from_user_and_overrides_id() {
        let spec = load();
        assert_eq!(spec.schemas["Me"].parent.as_deref(), Some("User"));
        let me = spec.resolved_props("Me");
        assert_eq!(me["login"], PropType::String); // inherited
        assert_eq!(me["id"], PropType::String); // redeclared, subtype wins
        assert!(me.contains_key("$type"));
    }

    #[test]
    fn issue_fields_have_expected_types() {
        let spec = load();
        let issue = spec.resolved_props("Issue");
        assert_eq!(issue["created"], PropType::I64);
        assert_eq!(issue["commentsCount"], PropType::I32);
        assert_eq!(issue["reporter"], PropType::Ref("User".into()));
    }

    #[test]
    fn issue_link_direction_is_string_enum() {
        let spec = load();
        let PropType::StrEnum(values) = &spec.resolved_props("IssueLink")["direction"] else {
            panic!("direction should be a string enum");
        };
        assert_eq!(values, &["OUTWARD", "INWARD", "BOTH"]);
    }

    #[test]
    fn activity_item_untyped_fields_are_json() {
        let spec = load();
        assert_eq!(spec.resolved_props("ActivityItem")["added"], PropType::Json);
    }
}
