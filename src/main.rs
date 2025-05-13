use std::fmt::Display;
use reqwest::header::{HeaderMap, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use chrono::prelude::*;

#[derive(Debug,Error)]
enum ProgramErrors{
    #[error("Fetch Error: {0}")]
    FetchError(#[from] reqwest::Error),
    #[error("Serde Json Error: {0}")]
    JsonParsingError(#[from] serde_json::Error),
}

#[derive(Debug,Serialize,Deserialize,Error)]
struct Repo{
    id: i32,
    name: String,
    url: String,
}

impl Display for Repo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}",self)
    }
}

#[derive(Debug,Serialize,Deserialize,Error)]
struct Actor{
    id: i32,
    login: String,
    display_login: String,
    gravitar_id: String,
    url: String,
    avatar_url: String,
}

impl Display for Actor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}",self)
    }
}

#[derive(Debug,Serialize,Deserialize,Error)]
struct User{
    id: i32,
    actor: Actor,
    repo: Repo,
    public: bool,
    created_at: DateTime<Local>
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}",self)
    }
}

async fn fetch_user_data(username: &String) -> std::result::Result<Value,ProgramErrors> {
    let url = format!("https://api.github.com/users/{}/events", username);
    let method = reqwest::Method::GET;
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, "JustNesh".parse().unwrap());
    let request = client.request(method, url).headers(headers).send().await?.text().await?;
    println!("{}", request);
    let json_user: Value = serde_json::from_str(&request.as_str())?;
    Ok(json_user)
}
#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let username = &args[1];
    match fetch_user_data(&username).await {
        Ok(data) => println!("Data : {}", data),
        Err(e) => eprintln!("{}",e),
    }
}
