// relative and with a trailing slash: joined onto the host url in
// `YoutrackClient::new`, and endpoint paths are joined onto the result
pub const REST_API_PREFIX: &str = "api/";

pub const ENDPOINT_USERS: &str = "users";

// Reserved for the clients that have not been written yet; `clients/issues.rs`,
// `clients/articles.rs` and `clients/projects.rs` are still empty. Drop the
// allow as each one starts using its endpoint.
#[allow(dead_code)]
pub const ENDPOINT_ISSUES: &str = "issues";
#[allow(dead_code)]
pub const ENDPOINT_ARTICLES: &str = "articles";
#[allow(dead_code)]
pub const ENDPOINT_PROJECTS: &str = "projects";
#[allow(dead_code)]
pub const ENDPOINT_EXTENSION: &str = "extensionEndpoints";
