use crate::clients::issues::{ListParams, MutationParams};
use crate::constants::ENDPOINT_ISSUES;
use crate::{FieldsQuery, Issue, IssueLink, YoutrackClient};
use reqwest::Method;

pub struct LinksApi<'a> {
    pub(crate) internal: &'a YoutrackClient,
    pub(crate) issue_id: &'a str,
}
impl<'a> LinksApi<'a> {
    fn base(&self) -> String {
        format!("{}/{}/links", ENDPOINT_ISSUES, self.issue_id)
    }

    pub async fn list(&self, params: ListParams) -> crate::Result<Vec<IssueLink>> {
        let query = params.into_query();
        self.internal
            .inner_send_with_serde::<_, (), Vec<IssueLink>>(
                self.base().as_str(),
                Method::GET,
                Some(&query),
                None,
            )
            .await
    }

    pub async fn get(
        &self,
        link_id: &str,
        fields: Option<FieldsQuery>,
    ) -> crate::Result<IssueLink> {
        self.internal
            .inner_send_with_serde::<_, (), IssueLink>(
                format!("{}/{link_id}", self.base()).as_str(),
                Method::GET,
                fields.as_ref(),
                None,
            )
            .await
    }

    pub async fn list_issues(
        &self,
        link_id: &str,
        params: ListParams,
    ) -> crate::Result<Vec<Issue>> {
        let query = params.into_query();
        self.internal
            .inner_send_with_serde::<_, (), Vec<Issue>>(
                format!("{}/{link_id}/issues", self.base()).as_str(),
                Method::GET,
                Some(&query),
                None,
            )
            .await
    }

    pub async fn add_issue(
        &self,
        link_id: &str,
        issue: &Issue,
        params: MutationParams,
    ) -> crate::Result<Issue> {
        let query = params.into_query();
        self.internal
            .inner_send_with_serde::<_, Issue, Issue>(
                format!("{}/{link_id}/issues", self.base()).as_str(),
                Method::POST,
                Some(&query),
                Some(issue),
            )
            .await
    }

    pub async fn remove_issue(&self, link_id: &str, issue_id: &str) -> crate::Result<()> {
        self.internal
            .inner_send_with_serde::<(), (), ()>(
                format!("{}/{link_id}/issues/{issue_id}", self.base()).as_str(),
                Method::DELETE,
                None,
                None,
            )
            .await
    }
}
