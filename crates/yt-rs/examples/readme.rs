use yt_rs::{AuthorizationFlow, YoutrackClient};

#[tokio::main]
async fn main() {
    let token = std::env::var("YOUTRACK_TOKEN").expect("set YOUTRACK_TOKEN env variable");

    let http = reqwest::Client::new();
    let client = YoutrackClient::new(
        http,
        "https://youtrack.gbfa.dev",
        AuthorizationFlow::PermanentBearerToken(token),
    )
    .unwrap();
    let users_api = client.users_api();

    let fields_query = vec!["login".into(), "id".into()].into();
    let me = users_api.me(Some(fields_query)).await.unwrap();
    println!("{:?}", me)
}
