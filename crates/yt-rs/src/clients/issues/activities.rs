use crate::clients::issues::{ActivityPageParams, ActivityParams};
use crate::constants::ENDPOINT_ISSUES;
use crate::{ActivityCursorPage, ActivityItem, FieldsQuery, YoutrackClient};
use reqwest::Method;

pub struct IssueActivitiesApi<'a> {
    pub(crate) internal: &'a YoutrackClient,
    pub(crate) issue_id: &'a str,
}
impl<'a> IssueActivitiesApi<'a> {
    fn base(&self) -> String {
        format!("{}/{}/activities", ENDPOINT_ISSUES, self.issue_id)
    }

    pub async fn list(&self, params: ActivityParams) -> crate::Result<Vec<ActivityItem>> {
        let query = params.into_query();
        self.internal
            .inner_send_with_serde::<_, (), Vec<ActivityItem>>(
                self.base().as_str(),
                Method::GET,
                Some(&query),
                None,
            )
            .await
    }

    pub async fn get(
        &self,
        activity_item_id: &str,
        fields: Option<FieldsQuery>,
    ) -> crate::Result<ActivityItem> {
        self.internal
            .inner_send_with_serde::<_, (), ActivityItem>(
                format!("{}/{activity_item_id}", self.base()).as_str(),
                Method::GET,
                fields.as_ref(),
                None,
            )
            .await
    }

    pub async fn page(&self, params: ActivityPageParams) -> crate::Result<ActivityCursorPage> {
        let query = params.into_query();
        self.internal
            .inner_send_with_serde::<_, (), ActivityCursorPage>(
                format!("{}/{}/activitiesPage", ENDPOINT_ISSUES, self.issue_id).as_str(),
                Method::GET,
                Some(&query),
                None,
            )
            .await
    }
}
