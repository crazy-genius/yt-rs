use yt_rs::{AuthorizationFlow, YoutrackClient};

#[tokio::main]
async fn main() {
    let host = std::env::var("YOUTRACK_HOST").expect("YOUTRACK_HOST not found");
    let token = std::env::var("YOUTRACK_TOKEN").expect("set YOUTRACK_TOKEN env variable");

    let http = reqwest::Client::new();
    let client =
        YoutrackClient::new(http, host.as_str(), AuthorizationFlow::PermanentBearerToken(token))
            .unwrap();
    let users_api = client.users_api();

    let fields_query = vec!["login".into(), "id".into()].into();
    let me = users_api.me(Some(fields_query)).await.unwrap();
    println!("logged in as {:?} (id {:?})", me.login(), me.id());
}
