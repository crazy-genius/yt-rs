use crate::clients::issues::{ListParams, MutationParams};
use crate::constants::ENDPOINT_ISSUES;
use crate::{FieldsQuery, IssueComment, Reaction, YoutrackClient};
use reqwest::Method;

pub struct CommentsApi<'a> {
    pub(crate) internal: &'a YoutrackClient,
    pub(crate) issue_id: &'a str,
}
impl<'a> CommentsApi<'a> {
    fn base(&self) -> String {
        format!("{}/{}/comments", ENDPOINT_ISSUES, self.issue_id)
    }

    pub fn reactions_api(&self, comment_id: &'a str) -> CommentReactionsApi<'a> {
        CommentReactionsApi { internal: self.internal, issue_id: self.issue_id, comment_id }
    }

    pub async fn list(&self, params: ListParams) -> crate::Result<Vec<IssueComment>> {
        let query = params.into_query();
        self.internal
            .inner_send_with_serde::<_, (), Vec<IssueComment>>(
                self.base().as_str(),
                Method::GET,
                Some(&query),
                None,
            )
            .await
    }

    pub async fn get(
        &self,
        comment_id: &str,
        fields: Option<FieldsQuery>,
    ) -> crate::Result<IssueComment> {
        self.internal
            .inner_send_with_serde::<_, (), IssueComment>(
                format!("{}/{comment_id}", self.base()).as_str(),
                Method::GET,
                fields.as_ref(),
                None,
            )
            .await
    }

    pub async fn create(
        &self,
        comment: &IssueComment,
        params: MutationParams,
    ) -> crate::Result<IssueComment> {
        let query = params.into_query();
        self.internal
            .inner_send_with_serde::<_, IssueComment, IssueComment>(
                self.base().as_str(),
                Method::POST,
                Some(&query),
                Some(comment),
            )
            .await
    }

    pub async fn update(
        &self,
        comment_id: &str,
        comment: &IssueComment,
        params: MutationParams,
    ) -> crate::Result<IssueComment> {
        let query = params.into_query();
        self.internal
            .inner_send_with_serde::<_, IssueComment, IssueComment>(
                format!("{}/{comment_id}", self.base()).as_str(),
                Method::POST,
                Some(&query),
                Some(comment),
            )
            .await
    }

    pub async fn delete(&self, comment_id: &str) -> crate::Result<()> {
        self.internal
            .inner_send_with_serde::<(), (), ()>(
                format!("{}/{comment_id}", self.base()).as_str(),
                Method::DELETE,
                None,
                None,
            )
            .await
    }
}

pub struct CommentReactionsApi<'a> {
    pub(crate) internal: &'a YoutrackClient,
    pub(crate) issue_id: &'a str,
    pub(crate) comment_id: &'a str,
}
impl<'a> CommentReactionsApi<'a> {
    fn base(&self) -> String {
        format!("{}/{}/comments/{}/reactions", ENDPOINT_ISSUES, self.issue_id, self.comment_id)
    }

    pub async fn list(&self, params: ListParams) -> crate::Result<Vec<Reaction>> {
        let query = params.into_query();
        self.internal
            .inner_send_with_serde::<_, (), Vec<Reaction>>(
                self.base().as_str(),
                Method::GET,
                Some(&query),
                None,
            )
            .await
    }

    pub async fn get(
        &self,
        reaction_id: &str,
        fields: Option<FieldsQuery>,
    ) -> crate::Result<Reaction> {
        self.internal
            .inner_send_with_serde::<_, (), Reaction>(
                format!("{}/{reaction_id}", self.base()).as_str(),
                Method::GET,
                fields.as_ref(),
                None,
            )
            .await
    }

    pub async fn add(
        &self,
        reaction: &Reaction,
        fields: Option<FieldsQuery>,
    ) -> crate::Result<Reaction> {
        self.internal
            .inner_send_with_serde::<_, Reaction, Reaction>(
                self.base().as_str(),
                Method::POST,
                fields.as_ref(),
                Some(reaction),
            )
            .await
    }

    pub async fn delete(&self, reaction_id: &str) -> crate::Result<()> {
        self.internal
            .inner_send_with_serde::<(), (), ()>(
                format!("{}/{reaction_id}", self.base()).as_str(),
                Method::DELETE,
                None,
                None,
            )
            .await
    }
}
