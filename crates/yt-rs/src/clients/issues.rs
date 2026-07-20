use std::collections::HashMap;
use reqwest::Method;
use serde::Serialize;
use crate::{ActivityItem, FieldsQuery, Issue, IssueResolvedActivityItem, YoutrackClient};
use crate::constants::ENDPOINT_ISSUES;

pub struct IssuesApi<'a> {
    pub(crate) internal: &'a YoutrackClient,
}
impl<'a> IssuesApi<'a> {

    pub fn activity_api(&self, issue_id: &'a str) -> IssueActivityApi<'a> {
        IssueActivityApi {
            internal: self.internal,
            issue_id,
        }
    }



    pub async fn get_issue(&self, issue_id: &str, fields: Option<FieldsQuery>) -> crate::Result<Issue> {
        self.internal
            .inner_send_with_serde::<_, (), Issue>(
                format!("{}/{issue_id}", ENDPOINT_ISSUES).as_str(),
                Method::GET,
                fields.as_ref(),
                None,
            )
            .await
    }

    pub async fn update_issue(&self, issue_id: &str, issue: Issue, mute_update_notifications: bool,  fields: Option<FieldsQuery>) -> crate::Result<Issue> {
        let mute_update_notifications: String = match mute_update_notifications {
            true => "true".into(),
            false => "false".into(),
        };
        let mut query = HashMap::with_capacity(2);
        query.insert("muteUpdateNotifications", mute_update_notifications);
        if let Some(fields) = fields {
            query.insert("fields", fields.into());
        }

        self.internal
            .inner_send_with_serde::<HashMap<&str, String>, Issue, Issue>(
                format!("{}/{issue_id}", ENDPOINT_ISSUES).as_str(),
                Method::POST,
                Some(&query),
                Some(&issue),
            )
            .await
    }

    pub async fn delete_issue(&self, issue_id: &str, fields: Option<FieldsQuery>) -> crate::Result<Issue> {
        self.internal
            .inner_send_with_serde::<_, (), Issue>(
                format!("{}/{issue_id}", ENDPOINT_ISSUES).as_str(),
                Method::DELETE,
                fields.as_ref(),
                None,
            )
            .await
    }
}

pub struct IssueActivityApi<'a> {
    pub(crate) internal: &'a YoutrackClient,
    issue_id: &'a str,
}
impl<'a> IssueActivityApi<'a> {
    pub async fn list(&self) -> crate::Result<ActivityItem> {
        todo!()
    }
}