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
    StdError(#[from] std::io::Error),
    #[error("Event type was not accounted for")]
    UncheckedEventType,
}
enum EventType{
    CreateEvent,
    ForkEvent,
    IssueCommentEvent,
    IssuesEvent,
    PublicEvent,
    PullRequestEvent,
    PushEvent,
    WatchEvent,
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

async fn fetch_user_data(username: &String) -> std::result::Result<String,ProgramErrors> {
    let url = format!("https://api.github.com/users/{}/events", username);
    let method = reqwest::Method::GET;
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, "JustNesh".parse().unwrap());
    let request = client.request(method, url).headers(headers).send().await?.text().await?;
    // println!("{:?}", request);
    Ok(request)
}

fn get_user_input() -> std::result::Result<String,ProgramErrors>{
    let mut buffer = String::from("");
    print!("\nGithub-Activity > ");
    stdout().flush()?;
    match stdin().read_line(&mut buffer){
        Ok(_) => {
            if (buffer.trim().to_lowercase().as_str() == "q") | (buffer.trim().to_lowercase().as_str() == "quit") {
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

fn process_event_type(event_type:&str, event:&Value) -> std::result::Result<(), ProgramErrors>{
    let event_type = match event_type {
        "CreateEvent" => EventType::CreateEvent,
        "ForkEvent" => EventType::ForkEvent,
        "IssueCommentEvent" => EventType::IssueCommentEvent,
        "IssuesEvent" => EventType::IssuesEvent,
        "PublicEvent" => EventType::PublicEvent,
        "PullRequestEvent" => EventType::PullRequestEvent,
        "PushEvent" => EventType::PushEvent,
        "WatchEvent" => EventType::WatchEvent,
        _ => return Err(ProgramErrors::UncheckedEventType),
    };

    match event_type{
        EventType::CreateEvent => {
            let repo_name = &event["repo"]["name"];
            let user_name = &event["actor"]["display_login"].as_str().unwrap();
            let ref_type = &event["payload"]["ref_type"].as_str().unwrap();
            if ref_type == &"repository"{
                println!("{} created a new repository named {}.", user_name,repo_name);
            } else if ref_type == &"branch" {
                let branch_name = &event["payload"]["ref"];
                println!("{} created a new branch named {} in {}.", user_name,branch_name,repo_name);
            } else {
                println!("JUSTIN THIS IS A NEW CREATE EVENT YOU NEED TO ADD: {}", ref_type);
            }
        },
        EventType::ForkEvent => {
            let og_repo = &event["repo"]["name"];
            let new_repo = &event["payload"]["forkee"]["full_name"];
            let user_name = &event["actor"]["display_login"].as_str().unwrap();
            println!("{} forked from {} to {}", user_name, og_repo, new_repo);
        },
        EventType::IssueCommentEvent => {
            let repo_name = &event["payload"]["forkee"]["full_name"];
            let user_name = &event["actor"]["display_login"].as_str().unwrap();
            let comment_title = &event["payload"]["issue"]["title"];
            println!("{} commented an issue in the repo {} with the title {}", user_name, repo_name, comment_title);
        },
        EventType::IssuesEvent => {
            let repo_name = &event["repo"]["name"];
            let user_name = &event["actor"]["display_login"].as_str().unwrap();
            let issue_title = &event["payload"]["issue"]["title"];
            println!("{} opened an issue in {} with the title {}", user_name, repo_name, issue_title);
        }
        EventType::PublicEvent => {
            let repo_name = &event["repo"]["name"];
            let user_name = &event["actor"]["display_login"].as_str().unwrap();
            println!("{} made {} public", user_name,repo_name); 
        },
        EventType::PullRequestEvent => {
            let repo_name = &event["repo"]["name"];
            let user_name = &event["actor"]["display_login"].as_str().unwrap();
            let pull_title = &event["payload"]["pull_request"]["title"];
            println!("{} opened an pull request in {} with the title {}", user_name, repo_name, pull_title);            
        }
        EventType::PushEvent => {
            let repo_name = &event["repo"]["name"];
            let user_name = &event["actor"]["display_login"].as_str().unwrap();
            let commits_array = &event["payload"]["commits"].as_array().unwrap();
            for commit in commits_array.iter() {
                let push_message = &commit["message"];
                println!("{} pushed new content to {} with the message {}", user_name, repo_name, push_message);
            }
            println!("{} made {} public", user_name,repo_name);            
        },
        EventType::WatchEvent => {
            let repo_name = &event["repo"]["name"];
            let user_name = &event["actor"]["display_login"].as_str().unwrap();
            println!("{} is watching {}", user_name, repo_name);
        }
    }

    Ok(())
}

fn process_events(events: &Vec<Value>) -> std::result::Result<(), ProgramErrors>{
        for event in events {
            let event_type = &event["type"].as_str().unwrap();
            process_event_type(event_type, event)?
        }
    Ok(())    
}

#[tokio::main]
async fn main() {
    let program_running = true;
    println!("\nFetch events of a user by providing their username. Enter q to close program.");

    while program_running {
        let user_input = get_user_input();
        if let Err(e) = user_input {eprintln!("{}",e); continue};
        println!("\n");

        let user_input:Vec<String> = user_input.unwrap().split(' ').map(|word| word.to_owned()).collect();
        let username =  &user_input[0];

        let user_data = fetch_user_data(&username).await;
        if let Err(e) = user_data {eprintln!("{}",e); continue};
        
        let json_data = process_user_data(user_data.unwrap(), &username);
        if let Err(e) = json_data {eprintln!("{}",e); continue};
        
        let json_data = json_data.unwrap();
        let events = json_data.as_array().unwrap();
        let processed_events = process_events(events);
        if let Err(e) = processed_events {eprintln!("{}",e); continue};
        println!("\n");
    }
}
