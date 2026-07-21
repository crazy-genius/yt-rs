use yt_rs::{AuthorizationFlow, FieldsQuery, IssueSearchParams, YoutrackClient};

#[tokio::main]
async fn main() {
    let host = std::env::var("YOUTRACK_HOST").expect("YOUTRACK_HOST not found");
    let token = std::env::var("YOUTRACK_TOKEN").expect("set YOUTRACK_TOKEN env variable");
    let query =
        std::env::var("YOUTRACK_QUERY").unwrap_or_else(|_| "for: me #Unresolved".to_string());
    let fields = std::env::var("YOUTRACK_FIELDS")
        .unwrap_or_else(|_| "id,summary".to_string())
        .split(',')
        .map(|x| x.to_string())
        .collect::<Vec<String>>();

    let http = reqwest::Client::new();
    let client =
        YoutrackClient::new(http, host.as_str(), AuthorizationFlow::PermanentBearerToken(token))
            .unwrap();

    let params =
        IssueSearchParams::default().query(query).top(10).fields(FieldsQuery::from(fields));

    let issues = client.issues_api().list(params).await.unwrap();
    println!("found {} issues", issues.len());
    for issue in &issues {
        println!("- {:?} {:?}", issue.id, issue.summary);
    }
}
