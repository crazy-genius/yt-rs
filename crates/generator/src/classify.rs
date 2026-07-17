use crate::spec::{PropType, Spec, Variant};
use std::collections::BTreeMap;

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
    Classified { roots, variant_of }
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
