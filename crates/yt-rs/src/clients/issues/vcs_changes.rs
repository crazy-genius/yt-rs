use crate::clients::issues::{ListParams, MutationParams};
use crate::constants::ENDPOINT_ISSUES;
use crate::{FieldsQuery, VcsChange, YoutrackClient};
use reqwest::Method;

pub struct VcsChangesApi<'a> {
    pub(crate) internal: &'a YoutrackClient,
    pub(crate) issue_id: &'a str,
}
impl<'a> VcsChangesApi<'a> {
    fn base(&self) -> String {
        format!("{}/{}/vcsChanges", ENDPOINT_ISSUES, self.issue_id)
    }

    pub async fn list(&self, params: ListParams) -> crate::Result<Vec<VcsChange>> {
        let query = params.into_query();
        self.internal
            .inner_send_with_serde::<_, (), Vec<VcsChange>>(
                self.base().as_str(),
                Method::GET,
                Some(&query),
                None,
            )
            .await
    }

    pub async fn get(
        &self,
        vcs_change_id: &str,
        fields: Option<FieldsQuery>,
    ) -> crate::Result<VcsChange> {
        self.internal
            .inner_send_with_serde::<_, (), VcsChange>(
                format!("{}/{vcs_change_id}", self.base()).as_str(),
                Method::GET,
                fields.as_ref(),
                None,
            )
            .await
    }

    pub async fn create(
        &self,
        change: &VcsChange,
        params: MutationParams,
    ) -> crate::Result<VcsChange> {
        let query = params.into_query();
        self.internal
            .inner_send_with_serde::<_, VcsChange, VcsChange>(
                self.base().as_str(),
                Method::POST,
                Some(&query),
                Some(change),
            )
            .await
    }

    pub async fn update(
        &self,
        vcs_change_id: &str,
        change: &VcsChange,
        params: MutationParams,
    ) -> crate::Result<VcsChange> {
        let query = params.into_query();
        self.internal
            .inner_send_with_serde::<_, VcsChange, VcsChange>(
                format!("{}/{vcs_change_id}", self.base()).as_str(),
                Method::POST,
                Some(&query),
                Some(change),
            )
            .await
    }

    pub async fn delete(&self, vcs_change_id: &str) -> crate::Result<()> {
        self.internal
            .inner_send_with_serde::<(), (), ()>(
                format!("{}/{vcs_change_id}", self.base()).as_str(),
                Method::DELETE,
                None,
                None,
            )
            .await
    }
}
