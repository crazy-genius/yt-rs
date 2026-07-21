use crate::clients::issues::{ListParams, MutationParams};
use crate::constants::ENDPOINT_ISSUES;
use crate::{FieldsQuery, IssueAttachment, YoutrackClient};
use reqwest::Method;
use reqwest::multipart::{Form, Part};

pub struct AttachmentUpload {
    pub file_name: String,
    pub bytes: Vec<u8>,
}

pub struct AttachmentsApi<'a> {
    pub(crate) internal: &'a YoutrackClient,
    pub(crate) issue_id: &'a str,
}
impl<'a> AttachmentsApi<'a> {
    fn base(&self) -> String {
        format!("{}/{}/attachments", ENDPOINT_ISSUES, self.issue_id)
    }

    pub async fn list(&self, params: ListParams) -> crate::Result<Vec<IssueAttachment>> {
        let query = params.into_query();
        self.internal
            .inner_send_with_serde::<_, (), Vec<IssueAttachment>>(
                self.base().as_str(),
                Method::GET,
                Some(&query),
                None,
            )
            .await
    }

    pub async fn get(
        &self,
        attachment_id: &str,
        fields: Option<FieldsQuery>,
    ) -> crate::Result<IssueAttachment> {
        self.internal
            .inner_send_with_serde::<_, (), IssueAttachment>(
                format!("{}/{attachment_id}", self.base()).as_str(),
                Method::GET,
                fields.as_ref(),
                None,
            )
            .await
    }

    pub async fn create(
        &self,
        files: Vec<AttachmentUpload>,
        params: MutationParams,
    ) -> crate::Result<Vec<IssueAttachment>> {
        let query = params.into_query();
        let mut form = Form::new();
        for file in files {
            let part = Part::bytes(file.bytes).file_name(file.file_name.clone());
            form = form.part(file.file_name, part);
        }
        self.internal
            .inner_send_multipart::<Vec<IssueAttachment>>(self.base().as_str(), &query, form)
            .await
    }

    pub async fn update(
        &self,
        attachment_id: &str,
        attachment: &IssueAttachment,
        fields: Option<FieldsQuery>,
    ) -> crate::Result<IssueAttachment> {
        self.internal
            .inner_send_with_serde::<_, IssueAttachment, IssueAttachment>(
                format!("{}/{attachment_id}", self.base()).as_str(),
                Method::POST,
                fields.as_ref(),
                Some(attachment),
            )
            .await
    }

    pub async fn delete(&self, attachment_id: &str) -> crate::Result<()> {
        self.internal
            .inner_send_with_serde::<(), (), ()>(
                format!("{}/{attachment_id}", self.base()).as_str(),
                Method::DELETE,
                None,
                None,
            )
            .await
    }
}
