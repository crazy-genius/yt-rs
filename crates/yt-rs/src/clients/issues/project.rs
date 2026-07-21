use crate::clients::issues::MutationParams;
use crate::constants::ENDPOINT_ISSUES;
use crate::{FieldsQuery, Project, YoutrackClient};
use reqwest::Method;

pub struct IssueProjectApi<'a> {
    pub(crate) internal: &'a YoutrackClient,
    pub(crate) issue_id: &'a str,
}
impl<'a> IssueProjectApi<'a> {
    fn base(&self) -> String {
        format!("{}/{}/project", ENDPOINT_ISSUES, self.issue_id)
    }

    pub async fn get(&self, fields: Option<FieldsQuery>) -> crate::Result<Project> {
        self.internal
            .inner_send_with_serde::<_, (), Project>(
                self.base().as_str(),
                Method::GET,
                fields.as_ref(),
                None,
            )
            .await
    }

    pub async fn update(
        &self,
        project: &Project,
        params: MutationParams,
    ) -> crate::Result<Project> {
        let query = params.into_query();
        self.internal
            .inner_send_with_serde::<_, Project, Project>(
                self.base().as_str(),
                Method::POST,
                Some(&query),
                Some(project),
            )
            .await
    }
}
