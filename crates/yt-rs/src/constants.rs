// relative and with a trailing slash: joined onto the host url in
// `YoutrackClient::new`, and endpoint paths are joined onto the result
pub const REST_API_PREFIX: &str = "api/";

pub const ENDPOINT_USERS: &str = "users";
pub const ENDPOINT_ISSUES: &str = "issues";

#[allow(dead_code)]
pub const ENDPOINT_ARTICLES: &str = "articles";
#[allow(dead_code)]
pub const ENDPOINT_PROJECTS: &str = "projects";
#[allow(dead_code)]
pub const ENDPOINT_EXTENSION: &str = "extensionEndpoints";
