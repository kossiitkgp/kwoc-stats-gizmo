use rusqlite::Connection;
use serde::Deserialize;
use reqwest::Error;
use regex::Regex;
use dotenv::dotenv;
use std::{collections::HashMap, env};
use chrono::DateTime;

mod database;
use database::{Student,get_students};

const PER_PAGE: u32 = 50;

#[derive(Deserialize, Debug)]
struct User{
    login: Option<String>,
    id: Option<u32>,
    open_pr_count: Option<u32>,
    merged_pr_count: Option<u32>
}

#[derive(Deserialize, Debug)]
struct Pull {
    url: Option<String>,
    id: Option<u32>,
    state: Option<String>,
    title: Option<String>,
    body: Option<String>,
    user: Option<User>,
    merged_at: Option<String>,
    created_at: Option<String>
}

#[derive(Deserialize, Debug)]
struct Rate {
    limit: Option<u32>,
    remaining: Option<u32>,
    used: Option<u32>,
    reset: Option<u32>
}

#[derive(Deserialize, Debug)]
struct RateContainer {
    core: Rate,
    search: Rate
}

#[derive(Deserialize, Debug)]
struct RateLimit {
    resources: RateContainer
}

#[tokio::main]
async fn get_all_pulls(client: &reqwest::Client, owner: &str, repo: &str) -> Result<Vec<Pull>,Error>{
    let mut request_url = format!("https://api.github.com/repos/{owner}/{repo}/pulls?state=all&per_page={PER_PAGE}");
    let pattern = Regex::new(r#"<([^>]+)>; rel="next""#).unwrap();

    let mut result = Vec::new();
    let mut end: bool = false;

    while !end {
        let response = client.get(&request_url)
            .send()
            .await?;

        let link_headers= &response.headers().get("LINK");

        match link_headers{
            Some(i) => match pattern.captures(i.to_str().unwrap()) {
                Some(i) => request_url = i[1].to_string(),
                None => end = true,
            },
            None => end = true,
        };

        let mut pulls: Vec<Pull> = response.json().await?;
        result.append(&mut pulls);
    }

    Ok(result)
}

//probably a good idea to make the map key the username
fn get_users_pr_counts(client: &reqwest::Client, owner: &str, repo: &str, start_time: i64, end_time: i64) -> Result<HashMap<u32,User>,Error>{
    let mut result = HashMap::new();
    let pulls = get_all_pulls(client, owner, repo)?;

    for pull in pulls {
        //ensure pull made by kwoc student
        let create_time = DateTime::parse_from_rfc3339(pull.created_at.unwrap().as_str())
            .unwrap()
            .timestamp();

        if (create_time < start_time) || (create_time > end_time) {
            continue;
        }

        let key = pull.user.as_ref().unwrap().id.unwrap();

        result.entry(key).or_insert({
            let user ={
                User {
                    open_pr_count: Some(0),
                    merged_pr_count: Some(0),
                    ..pull.user.unwrap()
                }
            };
            user
        });
        
        if pull.state.unwrap() == "open" {
            result.entry(key).and_modify(|i: &mut User| (*i).open_pr_count=Some((*i).open_pr_count.unwrap()+1));
        }else if pull.merged_at.is_some() {
            result.entry(key).and_modify(|i: &mut User| (*i).merged_pr_count=Some((*i).merged_pr_count.unwrap()+1));
        }
    }
    Ok(result)
}

#[tokio::main]
async fn print_rate_limit(client: &reqwest::Client) -> Result<(),Error>{
    let response = client.get("https://api.github.com/rate_limit")
        .send()
        .await?;

    let rate_limit : RateLimit = response.json().await?;
    println!("Core: {:#?}",rate_limit.resources.core);
    println!("Search: {:#?}",rate_limit.resources.search);
    Ok(())
}

fn main() -> Result<(),Error> {

    dotenv().ok();

    let access_token = match env::var("GH_ACCESS_TOKEN") {
        Ok(token) => Some(token),
        Err(err) => {
            println!("GH_ACCESS_TOKEN not found.");
            None
        }
    };

    let kwoc_start_time = env::var("KWOC_START_TIME").expect("KWOC_START_TIME not found.").parse::<i64>().unwrap();
    let kwoc_mid_evals_time = env::var("KWOC_MID_EVALS_TIME").expect("KWOC_MID_EVALS_TIME not found.").parse::<i64>().unwrap();
    let kwoc_end_evals_time = env::var("KWOC_END_EVALS_TIME").expect("KWOC_END_EVALS_TIME not found.").parse::<i64>().unwrap();

    let client = match access_token {
        Some(token) => {
            reqwest::Client::builder()
                .user_agent("Rust") 
                .default_headers({
                    let mut headers = reqwest::header::HeaderMap::new();
                    headers.insert(
                        reqwest::header::AUTHORIZATION,
                        reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
                    );
                    headers
                })
                .build()?
        }
        None => {
            reqwest::Client::builder()
                .user_agent("Rust") 
                .build()?
        }
    };

    let conn = Connection::open("./devDB.db").unwrap();

    let kwoc_students = get_students(&conn).unwrap();

    // print_rate_limit(&client)?;
    // let users = get_users_pr_counts(&client, "kossiitkgp", "events",kwoc_start_time,kwoc_mid_evals_time).unwrap();

    // for (key, value) in users.iter() {
    //     println!("{key} {:?}",value);
    // }


    // let pulls = get_all_pulls(&client, "kossiitkgp", "events")?;
    // println!("{:#?}", pulls);
    Ok(())
}
