use crate::spec::{PropType, Spec, Variant};
use std::collections::{BTreeMap, BTreeSet};

/// Spec defects patched before code generation. Currently one known JetBrains
/// exporter bug: `Project.customFields` is emitted as a bare `object` but is
/// actually an array of `ProjectCustomField`.
fn overrides() -> Vec<(&'static str, &'static str, PropType)> {
    vec![(
        "Project",
        "customFields",
        PropType::Array(Box::new(PropType::Ref("ProjectCustomField".into()))),
    )]
}

pub fn apply_overrides(spec: &mut Spec) {
    for (schema, prop, ty) in overrides() {
        let s = spec
            .schemas
            .get_mut(schema)
            .unwrap_or_else(|| panic!("override targets unknown schema {schema}"));
        let slot = s
            .props
            .get_mut(prop)
            .unwrap_or_else(|| panic!("override targets unknown property {schema}.{prop}"));
        *slot = ty;
    }
}

pub struct Classified {
    /// Root name -> sorted variants (tag + schema pairs; the spec's flat transitive
    /// mapping, which always includes the root itself).
    pub roots: BTreeMap<String, Vec<Variant>>,
    /// Schema name -> its root. Keyed by schema (not wire tag): this is used
    /// downstream to decide whether a struct is a discriminator variant and must
    /// have `$type` stripped, which is a per-schema property.
    pub variant_of: BTreeMap<String, String>,
    /// Schema names that appear as the concrete type of some OTHER schema's
    /// property — directly, or as an array element (e.g.
    /// `SingleEnumIssueCustomField.value: EnumBundleElement`). A schema in this
    /// set is reachable *without* going through its root's `#[serde(tag =
    /// "$type")]` enum, so serde never supplies the tag for it on that path; it
    /// must carry `$type` as an ordinary field or the value is lost on
    /// serialize. The `allOf` parent link is deliberately NOT counted here —
    /// that is inheritance, not a field reference, and is never a way to reach
    /// a schema as a standalone value.
    pub directly_referenced: BTreeSet<String>,
}

pub fn classify(spec: &Spec) -> Classified {
    let mut roots = BTreeMap::new();
    let mut variant_of = BTreeMap::new();
    for (name, schema) in &spec.schemas {
        if schema.mapping.is_empty() {
            continue;
        }
        for v in &schema.mapping {
            assert!(
                spec.schemas.contains_key(&v.schema),
                "{name}: mapping references unknown schema {}",
                v.schema
            );
            let prev = variant_of.insert(v.schema.clone(), name.clone());
            assert!(prev.is_none(), "{} appears in two discriminator mappings", v.schema);
        }
        roots.insert(name.clone(), schema.mapping.clone());
    }
    let directly_referenced = compute_directly_referenced(spec);
    Classified { roots, variant_of, directly_referenced }
}

/// Scans every schema's OWN (unresolved, post-override) properties for `$ref`
/// targets, including refs nested one level inside an array. Own props (rather
/// than resolved/inherited props) are sufficient: any `$ref` field declared on
/// a parent schema is found when that parent itself is visited, so nothing is
/// missed by not resolving inheritance here.
fn compute_directly_referenced(spec: &Spec) -> BTreeSet<String> {
    let mut refs = BTreeSet::new();
    for schema in spec.schemas.values() {
        for ty in schema.props.values() {
            collect_ref_targets(ty, &mut refs);
        }
    }
    refs
}

fn collect_ref_targets(ty: &PropType, out: &mut BTreeSet<String>) {
    match ty {
        PropType::Ref(name) => {
            out.insert(name.clone());
        }
        PropType::Array(inner) => collect_ref_targets(inner, out),
        _ => {}
    }
}

pub fn domain_of(name: &str) -> &'static str {
    for (domain, names) in DOMAINS {
        if names.contains(&name) {
            return domain;
        }
    }
    panic!("schema {name} has no domain assignment — add it to DOMAINS in classify.rs");
}

/// Explicit schema -> module assignment. A new spec type with no entry here is a
/// hard error, forcing a conscious placement decision instead of a silent default.
pub const DOMAINS: &[(&str, &[&str])] = &[
    (
        "activity",
        &[
            "ActivityCategory",
            "ActivityCursorPage",
            "ActivityItem",
            "AttachmentActivityItem",
            "CommentActivityItem",
            "CommentAttachmentsActivityItem",
            "CreatedDeletedActivityItem",
            "CustomFieldActivityItem",
            "IssueCreatedActivityItem",
            "IssueResolvedActivityItem",
            "LinksActivityItem",
            "MultiValueActivityItem",
            "ProjectActivityItem",
            "SimpleValueActivityItem",
            "SingleValueActivityItem",
            "SprintActivityItem",
            "TagsActivityItem",
            "TextCustomFieldActivityItem",
            "TextMarkupActivityItem",
            "UsesMarkupActivityItem",
            "VcsChangeActivityItem",
            "VisibilityActivityItem",
            "VisibilityGroupActivityItem",
            "VisibilityUserActivityItem",
            "VotersActivityItem",
            "WorkItemActivityItem",
            "WorkItemAuthorActivityItem",
            "WorkItemDurationActivityItem",
            "WorkItemTypeActivityItem",
        ],
    ),
    (
        "admin",
        &[
            "AppearanceSettings",
            "BackupError",
            "BackupFile",
            "BackupStatus",
            "BitBucketChangesProcessor",
            "BitBucketServer",
            "BitbucketStandaloneChangesProcessor",
            "BitbucketStandaloneServer",
            "ChangesProcessor",
            "DatabaseAttributeValue",
            "DatabaseBackupSettings",
            "EmailSettings",
            "GitHubChangesProcessor",
            "GitHubServer",
            "GitLabChangesProcessor",
            "GitLabServer",
            "GiteaChangesProcessor",
            "GiteaServer",
            "GlobalSettings",
            "GlobalTimeTrackingSettings",
            "GogsChangesProcessor",
            "GogsServer",
            "JenkinsChangesProcessor",
            "JenkinsServer",
            "License",
            "LocaleDescriptor",
            "LocaleSettings",
            "NotificationSettings",
            "RestCorsSettings",
            "SpaceChangesProcessor",
            "SpaceServer",
            "SystemSettings",
            "TeamcityChangesProcessor",
            "TeamcityServer",
            "Telemetry",
            "VcsChange",
            "VcsHostingChangesProcessor",
            "VcsHostingServer",
            "VcsServer",
        ],
    ),
    (
        "agile",
        &[
            "Agile",
            "AgileColumn",
            "AgileColumnFieldValue",
            "AgileSharingSettings",
            "AgileStatus",
            "AttributeBasedSwimlaneSettings",
            "ColumnSettings",
            "IssueBasedSwimlaneSettings",
            "Sprint",
            "SprintsSettings",
            "SwimlaneEntityAttributeValue",
            "SwimlaneSettings",
            "SwimlaneValue",
        ],
    ),
    (
        "article",
        &["Article", "ArticleAttachment", "ArticleComment", "BaseArticle", "ExternalArticle"],
    ),
    (
        "common",
        &[
            "BaseBundle",
            "BaseWorkItem",
            "BuildBundle",
            "BuildBundleCustomFieldDefaults",
            "BuildBundleElement",
            "Bundle",
            "BundleCustomFieldDefaults",
            "BundleElement",
            "ColorCoding",
            "CommandLimitedVisibility",
            "CommandList",
            "CommandUnlimitedVisibility",
            "CommandVisibility",
            "CustomField",
            "CustomFieldCondition",
            "CustomFieldDefaults",
            "CustomFilterField",
            "DateFormatDescriptor",
            "DuplicateVote",
            "EnumBundle",
            "EnumBundleCustomFieldDefaults",
            "EnumBundleElement",
            "Event",
            "FieldBasedColorCoding",
            "FieldBasedCondition",
            "FieldStyle",
            "FieldType",
            "FilterField",
            "LimitedVisibility",
            "LocalizableBundleElement",
            "Logo",
            "NestedGroup",
            "OwnedBundle",
            "OwnedBundleCustomFieldDefaults",
            "OwnedBundleElement",
            "ParsedCommand",
            "PeriodFieldFormat",
            "PredefinedFilterField",
            "Reaction",
            "SavedQuery",
            "SearchSuggestions",
            "StateBundle",
            "StateBundleCustomFieldDefaults",
            "StateBundleElement",
            "StorageEntry",
            "Suggestion",
            "Tag",
            "TagSharingSettings",
            "TimeZoneDescriptor",
            "UnlimitedVisibility",
            "VersionBundle",
            "VersionBundleCustomFieldDefaults",
            "VersionBundleElement",
            "Visibility",
            "WIPLimit",
            "WatchFolder",
            "WatchFolderSharingSettings",
            "WorkItemAttribute",
            "WorkItemAttributePrototype",
            "WorkItemAttributeValue",
            "WorkTimeSettings",
        ],
    ),
    (
        "issue",
        &[
            "DatabaseMultiValueIssueCustomField",
            "DatabaseSingleValueIssueCustomField",
            "DateIssueCustomField",
            "DurationValue",
            "ExternalIssue",
            "Issue",
            "IssueAttachment",
            "IssueComment",
            "IssueCountResponse",
            "IssueCustomField",
            "IssueFolder",
            "IssueKey",
            "IssueLink",
            "IssueLinkType",
            "IssueTag",
            "IssueTimeTracker",
            "IssueVoters",
            "IssueWatcher",
            "IssueWatchers",
            "IssueWorkItem",
            "MultiBuildIssueCustomField",
            "MultiEnumIssueCustomField",
            "MultiGroupIssueCustomField",
            "MultiOwnedIssueCustomField",
            "MultiUserIssueCustomField",
            "MultiVersionIssueCustomField",
            "PeriodIssueCustomField",
            "PeriodValue",
            "SimpleIssueCustomField",
            "SingleBuildIssueCustomField",
            "SingleEnumIssueCustomField",
            "SingleGroupIssueCustomField",
            "SingleOwnedIssueCustomField",
            "SingleUserIssueCustomField",
            "SingleVersionIssueCustomField",
            "StateIssueCustomField",
            "StateMachineIssueCustomField",
            "TextFieldValue",
            "TextIssueCustomField",
            "WorkItemType",
        ],
    ),
    (
        "project",
        &[
            "BuildProjectCustomField",
            "BundleProjectCustomField",
            "EnumProjectCustomField",
            "GroupProjectCustomField",
            "OwnedProjectCustomField",
            "PeriodProjectCustomField",
            "Project",
            "ProjectBasedColorCoding",
            "ProjectColor",
            "ProjectCustomField",
            "ProjectTeam",
            "ProjectTimeTrackingSettings",
            "SimpleProjectCustomField",
            "StateProjectCustomField",
            "TextProjectCustomField",
            "VersionProjectCustomField",
            "WorkItemProjectAttribute",
        ],
    ),
    (
        "user",
        &[
            "AllUsersGroup",
            "GeneralUserProfile",
            "Me",
            "NotificationsUserProfile",
            "OnlineUsers",
            "RegisteredUsersGroup",
            "TimeTrackingUserProfile",
            "User",
            "UserBundle",
            "UserCustomFieldDefaults",
            "UserGroup",
            "UserProfiles",
            "UserProjectCustomField",
            "VcsUnresolvedUser",
        ],
    ),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::PropType;
    use crate::spec::Variant;
    use crate::spec::tests::load;

    #[test]
    fn twenty_roots_143_variants() {
        let cls = classify(&load());
        assert_eq!(cls.roots.len(), 20);
        assert_eq!(cls.variant_of.len(), 143);
        assert_eq!(
            cls.roots["User"],
            vec![
                Variant { tag: "Me".into(), schema: "Me".into() },
                Variant { tag: "User".into(), schema: "User".into() },
                Variant { tag: "VcsUnresolvedUser".into(), schema: "VcsUnresolvedUser".into() },
            ]
        );
        assert_eq!(cls.roots["ActivityItem"].len(), 27);
        assert_eq!(cls.variant_of["Me"], "User");
        // the root itself is concrete and appears in its own mapping
        assert_eq!(cls.variant_of["User"], "User");
        // Tag has subtypes but no mapping -> not a root
        assert!(!cls.roots.contains_key("Tag"));
    }

    #[test]
    fn no_root_is_a_variant_of_another_root() {
        let cls = classify(&load());
        for root in cls.roots.keys() {
            assert_eq!(&cls.variant_of[root], root, "{root} nests under another root");
        }
    }

    /// IssueCustomField's mapping has the spec's only 2 entries where the
    /// discriminator KEY (the `$type` value actually sent on the wire) differs
    /// from the TARGET schema (the `$ref` describing the payload shape).
    /// Conflating the two — e.g. by using the schema name as the enum variant's
    /// serde tag — makes serde expect `$type: "DatabaseSingleValueIssueCustomField"`
    /// when the server actually sends `$type: "SingleValueIssueCustomField"`. That
    /// mismatch doesn't error: an untagged `Known`/`Unknown(Value)` wrapper above it
    /// silently swallows it into `Unknown`. This test pins both halves so a
    /// regression shows up as a loud assertion failure instead of a silent runtime
    /// degradation.
    #[test]
    fn mismatched_discriminator_tags_keep_both_halves() {
        let cls = classify(&load());
        let issue_custom_field = &cls.roots["IssueCustomField"];
        assert!(
            issue_custom_field.contains(&Variant {
                tag: "SingleValueIssueCustomField".into(),
                schema: "DatabaseSingleValueIssueCustomField".into(),
            }),
            "missing SingleValueIssueCustomField -> DatabaseSingleValueIssueCustomField variant"
        );
        assert!(
            issue_custom_field.contains(&Variant {
                tag: "MultiValueIssueCustomField".into(),
                schema: "DatabaseMultiValueIssueCustomField".into(),
            }),
            "missing MultiValueIssueCustomField -> DatabaseMultiValueIssueCustomField variant"
        );

        // variant_of is keyed by SCHEMA, not tag.
        assert_eq!(cls.variant_of["DatabaseSingleValueIssueCustomField"], "IssueCustomField");
        assert_eq!(cls.variant_of["DatabaseMultiValueIssueCustomField"], "IssueCustomField");
        assert!(!cls.variant_of.contains_key("SingleValueIssueCustomField"));
        assert!(!cls.variant_of.contains_key("MultiValueIssueCustomField"));
    }

    #[test]
    fn every_schema_has_a_domain_and_no_stale_entries() {
        let spec = load();
        for name in spec.schemas.keys() {
            domain_of(name); // panics if unassigned
        }
        let mut seen = std::collections::BTreeSet::new();
        for (_, names) in DOMAINS {
            for n in *names {
                assert!(spec.schemas.contains_key(*n), "{n} is in DOMAINS but not in the spec");
                assert!(seen.insert(*n), "{n} assigned to two domains");
            }
        }
        assert_eq!(seen.len(), spec.schemas.len());
    }

    /// Load-bearing regression pin: without this test a future refactor could
    /// silently flip the dual-use/enum-only split back to "keep on all 143" or
    /// "strip on all 143", both of which were tried and rejected (see emit.rs).
    #[test]
    fn dual_use_split_is_49_and_94() {
        let mut spec = load();
        apply_overrides(&mut spec);
        let cls = classify(&spec);
        let dual_use =
            cls.variant_of.keys().filter(|s| cls.directly_referenced.contains(*s)).count();
        let enum_only = cls.variant_of.len() - dual_use;
        assert_eq!(dual_use, 49, "dual-use (keep $type) variant count regressed");
        assert_eq!(enum_only, 94, "enum-only (strip $type) variant count regressed");
    }

    #[test]
    fn direct_reference_examples() {
        let mut spec = load();
        apply_overrides(&mut spec);
        let cls = classify(&spec);

        // SingleEnumIssueCustomField.value: EnumBundleElement — a plain $ref field,
        // bypassing the BundleElement root wrapper entirely.
        assert!(cls.directly_referenced.contains("EnumBundleElement"));
        // AttachmentActivityItem is only ever reached through ActivityItemKind;
        // no other schema names it directly as a field type.
        assert!(!cls.directly_referenced.contains("AttachmentActivityItem"));

        // The allOf parent link must not count as a direct reference.
        // EnumBundleElement's allOf parent is LocalizableBundleElement, which is
        // never itself the type of any $ref field in the spec.
        assert!(!cls.directly_referenced.contains("LocalizableBundleElement"));

        // IssueWorkItem has its own `type` prop (-> type_) that collides with a
        // kept `$type` field; it is ALSO dual-use — e.g. `IssueTimeTracker.workItems`
        // is `Array<IssueWorkItem>`, a plain $ref field bypassing BaseWorkItem's
        // tagged wrapper. The collision rule must still win at emission time (see
        // emit.rs), independent of this fact.
        assert!(cls.directly_referenced.contains("IssueWorkItem"));
    }

    #[test]
    fn project_custom_fields_override_applies() {
        let mut spec = load();
        apply_overrides(&mut spec);
        assert_eq!(
            spec.schemas["Project"].props["customFields"],
            PropType::Array(Box::new(PropType::Ref("ProjectCustomField".into())))
        );
    }
}
