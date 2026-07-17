use crate::clients::users::UsersApi;
use crate::constants::REST_API_PREFIX;
use crate::models::ApiError;
use crate::YoutrackError;
use reqwest::{Client, Method, RequestBuilder, Response, Url};
use serde::Serialize;
use serde::de::DeserializeOwned;

mod articles;
mod issues;
mod projects;
mod users;

#[derive(Clone)]
pub enum AuthorizationFlow {
    PermanentBearerToken(String),
    OAuthFlow,
}

#[derive(Clone)]
pub struct YoutrackClient {
    pub(crate) http: Client,
    pub(crate) base_uri: Url,
    authorization_flow: AuthorizationFlow,
}
impl YoutrackClient {
    /// `host_uri` is the YouTrack host only, e.g. `https://youtrack.example.com`
    /// or `https://example.com/youtrack` for installations under a subpath.
    /// The `api/` prefix is appended internally.
    pub fn new(
        http: Client,
        host_uri: &str,
        auth_flow: AuthorizationFlow,
    ) -> crate::Result<Self> {
        let mut base_uri: Url = host_uri.parse()?;
        // Url::join treats the last segment of a slash-less path as a file
        // name and would replace it, so make the path explicitly a directory
        if !base_uri.path().ends_with('/') {
            let path = format!("{}/", base_uri.path());
            base_uri.set_path(&path);
        }
        let base_uri = base_uri.join(REST_API_PREFIX)?;

        Ok(Self { http, base_uri, authorization_flow: auth_flow })
    }

    pub(crate) async fn inner_send_with_serde<Q, B, R>(
        &self,
        path: &str,
        method: Method,
        query: Option<&Q>,
        payload: Option<&B>,
    ) -> crate::Result<R>
    where
        Q: Serialize + ?Sized,
        B: Serialize + ?Sized,
        R: DeserializeOwned,
    {
        let endpoint = self.base_uri.join(path)?;
        let mut request_builder = self.http.request(method, endpoint);

        if let Some(query) = query {
            request_builder = request_builder.query(query);
        }
        if let Some(payload) = payload {
            request_builder = request_builder.json(payload);
        }

        let response = self.inner_send_authorized(request_builder).await?;
        let status = response.status();
        let bytes = response.bytes().await?;

        if status.is_success() {
            // empty bodies (204 No Content, DELETE responses) deserialize
            // as `null`, so `R = ()` and `R = Option<T>` keep working
            let data = if bytes.is_empty() {
                serde_json::from_slice(b"null")?
            } else {
                serde_json::from_slice(&bytes)?
            };

            return Ok(data);
        }

        if status.is_client_error()
            && let Ok(err) = serde_json::from_slice::<ApiError>(&bytes)
        {
            return Err(YoutrackError::ApiError(err));
        }

        // server errors and client errors whose body is not a youtrack
        // ApiError json (e.g. html from a reverse proxy)
        Err(YoutrackError::UnexpectedStatus {
            status,
            body: String::from_utf8_lossy(&bytes).into_owned(),
        })
    }

    pub(crate) async fn inner_send_authorized(
        &self,
        req: RequestBuilder,
    ) -> reqwest::Result<Response> {
        let req = match &self.authorization_flow {
            AuthorizationFlow::PermanentBearerToken(token) => req.bearer_auth(token),
            AuthorizationFlow::OAuthFlow => unreachable!(),
        };

        let request = req.build()?;
        self.http.execute(request).await
    }
}
impl YoutrackClient {
    pub fn users_api(&self) -> UsersApi<'_> {
        UsersApi { internal: self }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn client(host: &str) -> YoutrackClient {
        YoutrackClient::new(
            Client::new(),
            host,
            AuthorizationFlow::PermanentBearerToken(String::new()),
        )
        .unwrap()
    }

    #[test]
    fn base_uri_appends_api_prefix() {
        for (host, expected) in [
            ("https://yt.example.com", "https://yt.example.com/api/"),
            ("https://yt.example.com/", "https://yt.example.com/api/"),
            ("https://example.com/youtrack", "https://example.com/youtrack/api/"),
            ("https://example.com/youtrack/", "https://example.com/youtrack/api/"),
        ] {
            assert_eq!(client(host).base_uri.as_str(), expected, "host: {host}");
        }
    }

    #[test]
    fn endpoint_paths_join_under_api_prefix() {
        let endpoint = client("https://yt.example.com").base_uri.join("users/me").unwrap();
        assert_eq!(endpoint.as_str(), "https://yt.example.com/api/users/me");
    }
}
