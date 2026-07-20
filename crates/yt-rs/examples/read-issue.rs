use yt_rs::{AuthorizationFlow, YoutrackClient};

#[tokio::main]
async fn main() {
    let host = std::env::var("YOUTRACK_HOST").expect("YOUTRACK_HOST not found");
    let token = std::env::var("YOUTRACK_TOKEN").expect("set YOUTRACK_TOKEN env variable");
    let issue_id = std::env::var("YOUTRACK_ISSUE").expect("YOUTRACK_ISSUE not found");
    let fields = std::env::var("YOUTRACK_FIELDS").unwrap_or("id,summary,description".to_string())
        .split(',').map(|x| x.into()).collect::<Vec<String>>();

    let http = reqwest::Client::new();
    let client =
        YoutrackClient::new(http, host.as_str(), AuthorizationFlow::PermanentBearerToken(token))
            .unwrap();
    let issues_api = client.issues_api();

    let fields_query = fields.into();
    let issue = issues_api.get_issue(&issue_id, Some(fields_query)).await.unwrap();
    println!("issue {} has been read {:#?} {:#?}", issue_id, issue.summary, issue.description);
}
