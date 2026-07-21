use crate::clients::issues::{ListParams, MutationParams};
use crate::constants::ENDPOINT_ISSUES;
use crate::{FieldsQuery, IssueTimeTracker, IssueWorkItem, YoutrackClient};
use reqwest::Method;

pub struct TimeTrackingApi<'a> {
    pub(crate) internal: &'a YoutrackClient,
    pub(crate) issue_id: &'a str,
}
impl<'a> TimeTrackingApi<'a> {
    fn base(&self) -> String {
        format!("{}/{}/timeTracking", ENDPOINT_ISSUES, self.issue_id)
    }
    fn work_items_base(&self) -> String {
        format!("{}/workItems", self.base())
    }

    pub async fn get(&self, fields: Option<FieldsQuery>) -> crate::Result<IssueTimeTracker> {
        self.internal
            .inner_send_with_serde::<_, (), IssueTimeTracker>(
                self.base().as_str(),
                Method::GET,
                fields.as_ref(),
                None,
            )
            .await
    }

    pub async fn list_work_items(&self, params: ListParams) -> crate::Result<Vec<IssueWorkItem>> {
        let query = params.into_query();
        self.internal
            .inner_send_with_serde::<_, (), Vec<IssueWorkItem>>(
                self.work_items_base().as_str(),
                Method::GET,
                Some(&query),
                None,
            )
            .await
    }

    pub async fn get_work_item(
        &self,
        work_item_id: &str,
        fields: Option<FieldsQuery>,
    ) -> crate::Result<IssueWorkItem> {
        self.internal
            .inner_send_with_serde::<_, (), IssueWorkItem>(
                format!("{}/{work_item_id}", self.work_items_base()).as_str(),
                Method::GET,
                fields.as_ref(),
                None,
            )
            .await
    }

    pub async fn create_work_item(
        &self,
        item: &IssueWorkItem,
        params: MutationParams,
    ) -> crate::Result<IssueWorkItem> {
        let query = params.into_query();
        self.internal
            .inner_send_with_serde::<_, IssueWorkItem, IssueWorkItem>(
                self.work_items_base().as_str(),
                Method::POST,
                Some(&query),
                Some(item),
            )
            .await
    }

    pub async fn update_work_item(
        &self,
        work_item_id: &str,
        item: &IssueWorkItem,
        params: MutationParams,
    ) -> crate::Result<IssueWorkItem> {
        let query = params.into_query();
        self.internal
            .inner_send_with_serde::<_, IssueWorkItem, IssueWorkItem>(
                format!("{}/{work_item_id}", self.work_items_base()).as_str(),
                Method::POST,
                Some(&query),
                Some(item),
            )
            .await
    }

    pub async fn delete_work_item(&self, work_item_id: &str) -> crate::Result<()> {
        self.internal
            .inner_send_with_serde::<(), (), ()>(
                format!("{}/{work_item_id}", self.work_items_base()).as_str(),
                Method::DELETE,
                None,
                None,
            )
            .await
    }
}
