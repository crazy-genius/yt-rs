mod params;
pub use params::*;
mod comments;
pub use comments::*;
mod attachments;
pub use attachments::*;
mod links;
pub use links::*;
mod custom_fields;
pub use custom_fields::*;
mod tags;
pub use tags::*;
mod work_items;
pub use work_items::*;
mod activities;
pub use activities::*;
mod project;
pub use project::*;
mod sprints;
pub use sprints::*;
mod vcs_changes;
pub use vcs_changes::*;

use crate::constants::{ENDPOINT_ISSUES, ENDPOINT_ISSUES_COUNT};
use crate::{FieldsQuery, Issue, IssueCountResponse, YoutrackClient};
use reqwest::Method;

pub struct IssuesApi<'a> {
    pub(crate) internal: &'a YoutrackClient,
}
impl<'a> IssuesApi<'a> {
    pub fn activities_api(&self, issue_id: &'a str) -> IssueActivitiesApi<'a> {
        IssueActivitiesApi { internal: self.internal, issue_id }
    }

    pub fn comments_api(&self, issue_id: &'a str) -> CommentsApi<'a> {
        CommentsApi { internal: self.internal, issue_id }
    }

    pub fn attachments_api(&self, issue_id: &'a str) -> AttachmentsApi<'a> {
        AttachmentsApi { internal: self.internal, issue_id }
    }

    pub fn links_api(&self, issue_id: &'a str) -> LinksApi<'a> {
        LinksApi { internal: self.internal, issue_id }
    }

    pub fn custom_fields_api(&self, issue_id: &'a str) -> CustomFieldsApi<'a> {
        CustomFieldsApi { internal: self.internal, issue_id }
    }

    pub fn tags_api(&self, issue_id: &'a str) -> IssueTagsApi<'a> {
        IssueTagsApi { internal: self.internal, issue_id }
    }

    pub fn time_tracking_api(&self, issue_id: &'a str) -> TimeTrackingApi<'a> {
        TimeTrackingApi { internal: self.internal, issue_id }
    }

    pub fn project_api(&self, issue_id: &'a str) -> IssueProjectApi<'a> {
        IssueProjectApi { internal: self.internal, issue_id }
    }
    pub fn sprints_api(&self, issue_id: &'a str) -> IssueSprintsApi<'a> {
        IssueSprintsApi { internal: self.internal, issue_id }
    }
    pub fn vcs_changes_api(&self, issue_id: &'a str) -> VcsChangesApi<'a> {
        VcsChangesApi { internal: self.internal, issue_id }
    }

    pub async fn get_issue(
        &self,
        issue_id: &str,
        fields: Option<FieldsQuery>,
    ) -> crate::Result<Issue> {
        self.internal
            .inner_send_with_serde::<_, (), Issue>(
                format!("{}/{issue_id}", ENDPOINT_ISSUES).as_str(),
                Method::GET,
                fields.as_ref(),
                None,
            )
            .await
    }

    pub async fn list(&self, params: IssueSearchParams) -> crate::Result<Vec<Issue>> {
        let query = params.into_query();
        self.internal
            .inner_send_with_serde::<_, (), Vec<Issue>>(
                ENDPOINT_ISSUES,
                Method::GET,
                Some(&query),
                None,
            )
            .await
    }

    pub async fn create(&self, issue: &Issue, params: MutationParams) -> crate::Result<Issue> {
        let query = params.into_query();
        self.internal
            .inner_send_with_serde::<_, Issue, Issue>(
                ENDPOINT_ISSUES,
                Method::POST,
                Some(&query),
                Some(issue),
            )
            .await
    }

    pub async fn update_issue(
        &self,
        issue_id: &str,
        issue: Issue,
        mute_update_notifications: bool,
        fields: Option<FieldsQuery>,
    ) -> crate::Result<Issue> {
        let mut params = MutationParams::default().mute(mute_update_notifications);
        if let Some(fields) = fields {
            params = params.fields(fields);
        }
        let query = params.into_query();
        self.internal
            .inner_send_with_serde::<_, Issue, Issue>(
                format!("{}/{issue_id}", ENDPOINT_ISSUES).as_str(),
                Method::POST,
                Some(&query),
                Some(&issue),
            )
            .await
    }

    pub async fn count(
        &self,
        body: &IssueCountResponse,
        fields: Option<FieldsQuery>,
    ) -> crate::Result<IssueCountResponse> {
        let query: Vec<(&'static str, String)> =
            fields.map(|f| vec![("fields", f.into_field_value())]).unwrap_or_default();
        self.internal
            .inner_send_with_serde::<_, IssueCountResponse, IssueCountResponse>(
                ENDPOINT_ISSUES_COUNT,
                Method::POST,
                Some(&query),
                Some(body),
            )
            .await
    }

    pub async fn delete_issue(
        &self,
        issue_id: &str,
        fields: Option<FieldsQuery>,
    ) -> crate::Result<Issue> {
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
