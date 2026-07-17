use super::YoutrackClient;
use crate::constants::ENDPOINT_USERS;
use crate::models::{FieldsQuery, user::User};
use reqwest::Method;

pub struct UsersApi<'a> {
    pub(crate) internal: &'a YoutrackClient,
}
impl<'a> UsersApi<'a> {
    pub async fn me(&self, fields: Option<FieldsQuery>) -> crate::Result<User> {
        self.internal
            .inner_send_with_serde::<_, (), User>(
                format!("{}/me", ENDPOINT_USERS).as_str(),
                Method::GET,
                fields.as_ref(),
                None,
            )
            .await
    }
}
