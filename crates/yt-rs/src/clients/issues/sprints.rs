use crate::clients::issues::ListParams;
use crate::constants::ENDPOINT_ISSUES;
use crate::{Sprint, YoutrackClient};
use reqwest::Method;

pub struct IssueSprintsApi<'a> {
    pub(crate) internal: &'a YoutrackClient,
    pub(crate) issue_id: &'a str,
}
impl<'a> IssueSprintsApi<'a> {
    pub async fn list(&self, params: ListParams) -> crate::Result<Vec<Sprint>> {
        let query = params.into_query();
        self.internal
            .inner_send_with_serde::<_, (), Vec<Sprint>>(
                format!("{}/{}/sprints", ENDPOINT_ISSUES, self.issue_id).as_str(),
                Method::GET,
                Some(&query),
                None,
            )
            .await
    }
}
