use crate::clients::issues::{ListParams, MutationParams};
use crate::constants::ENDPOINT_ISSUES;
use crate::{FieldsQuery, IssueCustomField, YoutrackClient};
use reqwest::Method;

pub struct CustomFieldsApi<'a> {
    pub(crate) internal: &'a YoutrackClient,
    pub(crate) issue_id: &'a str,
}
impl<'a> CustomFieldsApi<'a> {
    fn base(&self) -> String {
        format!("{}/{}/customFields", ENDPOINT_ISSUES, self.issue_id)
    }

    pub async fn list(&self, params: ListParams) -> crate::Result<Vec<IssueCustomField>> {
        let query = params.into_query();
        self.internal
            .inner_send_with_serde::<_, (), Vec<IssueCustomField>>(
                self.base().as_str(),
                Method::GET,
                Some(&query),
                None,
            )
            .await
    }

    pub async fn get(
        &self,
        field_id: &str,
        fields: Option<FieldsQuery>,
    ) -> crate::Result<IssueCustomField> {
        self.internal
            .inner_send_with_serde::<_, (), IssueCustomField>(
                format!("{}/{field_id}", self.base()).as_str(),
                Method::GET,
                fields.as_ref(),
                None,
            )
            .await
    }

    pub async fn update(
        &self,
        field_id: &str,
        field: &IssueCustomField,
        params: MutationParams,
    ) -> crate::Result<IssueCustomField> {
        let query = params.into_query();
        self.internal
            .inner_send_with_serde::<_, IssueCustomField, IssueCustomField>(
                format!("{}/{field_id}", self.base()).as_str(),
                Method::POST,
                Some(&query),
                Some(field),
            )
            .await
    }
}
