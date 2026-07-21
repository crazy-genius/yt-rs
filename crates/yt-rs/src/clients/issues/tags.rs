use crate::clients::issues::ListParams;
use crate::constants::ENDPOINT_ISSUES;
use crate::{FieldsQuery, Tag, YoutrackClient};
use reqwest::Method;

pub struct IssueTagsApi<'a> {
    pub(crate) internal: &'a YoutrackClient,
    pub(crate) issue_id: &'a str,
}
impl<'a> IssueTagsApi<'a> {
    fn base(&self) -> String {
        format!("{}/{}/tags", ENDPOINT_ISSUES, self.issue_id)
    }

    pub async fn list(&self, params: ListParams) -> crate::Result<Vec<Tag>> {
        let query = params.into_query();
        self.internal
            .inner_send_with_serde::<_, (), Vec<Tag>>(
                self.base().as_str(),
                Method::GET,
                Some(&query),
                None,
            )
            .await
    }

    pub async fn get(&self, tag_id: &str, fields: Option<FieldsQuery>) -> crate::Result<Tag> {
        self.internal
            .inner_send_with_serde::<_, (), Tag>(
                format!("{}/{tag_id}", self.base()).as_str(),
                Method::GET,
                fields.as_ref(),
                None,
            )
            .await
    }

    pub async fn add(&self, tag: &Tag, fields: Option<FieldsQuery>) -> crate::Result<Tag> {
        self.internal
            .inner_send_with_serde::<_, Tag, Tag>(
                self.base().as_str(),
                Method::POST,
                fields.as_ref(),
                Some(tag),
            )
            .await
    }

    pub async fn remove(&self, tag_id: &str) -> crate::Result<()> {
        self.internal
            .inner_send_with_serde::<(), (), ()>(
                format!("{}/{tag_id}", self.base()).as_str(),
                Method::DELETE,
                None,
                None,
            )
            .await
    }
}
