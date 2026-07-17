use serde_json::json;
use yt_rs::*;

#[test]
fn me_roundtrips_and_dispatches() {
    let v = json!({"$type": "Me", "id": "1-1", "login": "root", "email": "a@b.c"});
    let u: User = serde_json::from_value(v.clone()).unwrap();
    // accessors work through the wrapper
    assert_eq!(u.id(), Some("1-1"));
    assert_eq!(u.login(), Some("root"));
    // dispatch reaches the concrete variant
    let User::Known(UserKind::Me(me)) = &u else { panic!("expected Me variant, got {u:?}") };
    assert_eq!(me.email.as_deref(), Some("a@b.c"));
    // $type survives exactly once
    assert_eq!(serde_json::to_value(&u).unwrap(), v);
}

#[test]
fn unknown_type_degrades_to_unknown_and_roundtrips_losslessly() {
    let v = json!({"$type": "HologramUser2031", "id": "1-2", "novel": true});
    let u: User = serde_json::from_value(v.clone()).unwrap();
    assert!(matches!(u, User::Unknown(_)), "got {u:?}");
    assert_eq!(u.id(), None);
    assert_eq!(serde_json::to_value(&u).unwrap(), v);
}

#[test]
fn missing_type_degrades_to_unknown() {
    let v = json!({"id": "1-3", "login": "tagless"});
    let u: User = serde_json::from_value(v).unwrap();
    assert!(matches!(u, User::Unknown(_)), "got {u:?}");
}

#[test]
fn issue_custom_fields_dispatch_to_typed_values() {
    let v = json!({
        "$type": "Issue", "id": "2-1", "idReadable": "TM-1", "summary": "test",
        "customFields": [
            {"$type": "SingleEnumIssueCustomField", "id": "c1", "name": "Priority",
             "value": {"$type": "EnumBundleElement", "id": "e1", "name": "Critical"}},
            {"$type": "PeriodIssueCustomField", "id": "c2", "name": "Estimation",
             "value": {"$type": "PeriodValue", "id": "p1", "minutes": 480}}
        ]
    });
    let issue: Issue = serde_json::from_value(v.clone()).unwrap();
    let fields = issue.custom_fields.as_ref().unwrap();

    let IssueCustomField::Known(IssueCustomFieldKind::SingleEnumIssueCustomField(f)) = &fields[0]
    else {
        panic!("expected SingleEnumIssueCustomField, got {:?}", fields[0]);
    };
    assert_eq!(f.value.as_ref().unwrap().name.as_deref(), Some("Critical"));

    let IssueCustomField::Known(IssueCustomFieldKind::PeriodIssueCustomField(f)) = &fields[1]
    else {
        panic!("expected PeriodIssueCustomField, got {:?}", fields[1]);
    };
    assert_eq!(f.value.as_ref().unwrap().minutes, Some(480));

    assert_eq!(serde_json::to_value(&issue).unwrap(), v);
}

#[test]
fn activity_item_roundtrips_with_inherited_accessors() {
    let v = json!({
        "$type": "IssueCreatedActivityItem", "id": "a1",
        "timestamp": 1710000000000_i64,
        "author": {"$type": "User", "id": "1-1", "login": "root"}
    });
    let a: ActivityItem = serde_json::from_value(v.clone()).unwrap();
    assert_eq!(a.id(), Some("a1"));
    assert_eq!(a.timestamp(), Some(1710000000000));
    assert_eq!(a.author().unwrap().login(), Some("root"));
    assert_eq!(serde_json::to_value(&a).unwrap(), v);
}

#[test]
fn issue_link_direction_enum_roundtrips_and_tolerates_unknown() {
    let v = json!({"$type": "IssueLink", "id": "l1", "direction": "OUTWARD"});
    let l: IssueLink = serde_json::from_value(v.clone()).unwrap();
    assert_eq!(l.direction, Some(IssueLinkDirection::Outward));
    assert_eq!(serde_json::to_value(&l).unwrap(), v);

    let l2: IssueLink = serde_json::from_value(json!({"direction": "SIDEWAYS"})).unwrap();
    assert_eq!(l2.direction, Some(IssueLinkDirection::Other("SIDEWAYS".into())));
    // lossless: unknown value serializes back verbatim
    assert_eq!(serde_json::to_value(&l2).unwrap(), json!({"direction": "SIDEWAYS"}));
}
