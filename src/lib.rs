use regex::Regex;
use chrono::DateTime;
use serde::Deserialize;
use reqwest::Error;
use std::collections::HashMap;

pub mod database;
use database::Student;

const PER_PAGE: u32 = 50;

#[derive(Deserialize, Debug)]
pub struct User{
    login: Option<String>,
    id: Option<u32>,
    open_pr_count: Option<u32>,
    merged_pr_count: Option<u32>
}

#[derive(Deserialize, Debug)]
pub struct Pull {
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

pub fn get_users_pr_counts(client: &reqwest::Client, owner: &str, repo: &str, start_time: i64, end_time: i64, kwoc_students: &HashMap<String,Student>) -> Result<HashMap<String,User>,Error>{
    let mut result = HashMap::new();
    let pulls = get_all_pulls(client, owner, repo)?;

    for pull in pulls {
        if !kwoc_students.contains_key(pull.user.as_ref().unwrap().login.as_ref().unwrap()) {
            continue;
        }

        let create_time = DateTime::parse_from_rfc3339(pull.created_at.unwrap().as_str())
            .unwrap()
            .timestamp();

        if (create_time < start_time) || (create_time > end_time) {
            continue;
        }

        let key = pull.user.as_ref().unwrap().login.clone().unwrap();

        result.entry(key.clone()).or_insert({
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
pub async fn print_rate_limit(client: &reqwest::Client) -> Result<(),Error>{
    let response = client.get("https://api.github.com/rate_limit")
        .send()
        .await?;

    let rate_limit : RateLimit = response.json().await?;
    println!("Core: {:#?}",rate_limit.resources.core);
    println!("Search: {:#?}",rate_limit.resources.search);
    Ok(())
}

