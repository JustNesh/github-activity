use std::fmt::Display;
use std::io::{stdin,stdout, Write};
use reqwest::header::{HeaderMap, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use chrono::prelude::*;

#[derive(Debug,Error)]
enum ProgramErrors {
    #[error("Fetch Error: {0}")]
    FetchError(#[from] reqwest::Error),
    #[error("Serde Json Error: {0}")]
    JsonParsingError(#[from] serde_json::Error),
    #[error("No arguments provided.")]
    NoArguments,
    #[error("No recent events were found for {0}.")]
    NoEvents(String),
    #[error("No User Found.")]
    NoUserFound,
    #[error("STD Error: {0}")]
    StdError(#[from] std::io::Error)
}
// enum EventType{
//     CreateEvent,
//     PublicEvent,
// }

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

async fn fetch_user_data(username: &String) -> std::result::Result<String,ProgramErrors> {
    let url = format!("https://api.github.com/users/{}/events", username);
    let method = reqwest::Method::GET;
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, "JustNesh".parse().unwrap());
    let request = client.request(method, url).headers(headers).send().await?.text().await?;
    println!("{}", request);
    Ok(request)
}

fn get_user_input() -> std::result::Result<String,ProgramErrors>{
    let mut buffer = String::from("");
    print!("\nGithub-Activity > ");
    stdout().flush()?;
    match stdin().read_line(&mut buffer){
        Ok(_) => {
            if buffer.trim().to_lowercase().as_str() == "q" {
                println!("\nProgram has been closed successfully.");
                std::process::exit(0);
            }
            if buffer.len() < 1 {
                return Err(ProgramErrors::NoArguments)
            }
            Ok(buffer.trim().to_owned())
        },
        Err(e) => return Err(ProgramErrors::StdError(e))
    }
}

fn process_user_data(data:String, username: &String) -> std::result::Result<Value,ProgramErrors>{
    let json_user: Value = serde_json::from_str(&data.as_str())?;
    if json_user["message"] == "Not Found"{
        return Err(ProgramErrors::NoUserFound);
    }
    if json_user.as_array().unwrap().len() < 1{
        return Err(ProgramErrors::NoEvents(username.clone()))
    }
    Ok(json_user)
}

#[tokio::main]
async fn main() {
    let program_running = true;
    println!("\nFetch events of a user by providing their username. Enter q to close program.");

    while program_running {
        let user_input = get_user_input();
        if let Err(e) = user_input {eprintln!("{}",e); continue};

        let user_input:Vec<String> = user_input.unwrap().split(' ').map(|word| word.to_owned()).collect();
        let username =  &user_input[0];

        let user_data = fetch_user_data(&username).await;
        if let Err(e) = user_data {eprintln!("{}",e); continue};
        
        let json_data = process_user_data(user_data.unwrap(), &username);
        if let Err(e) = json_data {eprintln!("{}",e); continue};
        
        let json_data = json_data.unwrap();
        let events = json_data.as_array().unwrap();
        for event in events {
            let event_type = &event["type"];
            println!("Event Type:{}", event_type);
        }
    }
}
