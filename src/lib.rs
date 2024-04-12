use regex::Regex;
use chrono::DateTime;
use serde::Deserialize;
use reqwest::Error;
use std::collections::HashMap;
use http::StatusCode;

pub mod database;
use database::Student;
use database::Project;

const PER_PAGE: u32 = 50;

#[derive(Deserialize, Debug)]
pub struct User{
    login: Option<String>,
    id: Option<u32>,
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
async fn get_all_pulls(client: &reqwest::Client, owner: String, repo: String) -> Result<Vec<Pull>,Error>{
    let mut request_url = format!("https://api.github.com/repos/{owner}/{repo}/pulls?state=all&per_page={PER_PAGE}");
    
    let pattern = Regex::new(r#"<([^>]+)>; rel="next""#).unwrap();

    let mut result = Vec::new();
    let mut end: bool = false;

    while !end {
        let response = client.get(&request_url)
            .send()
            .await?;

        if response.status() != StatusCode::OK {
            return Ok(result);
        }

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

fn split_repo_link(link: &String) -> (String,String) {
    let pattern = Regex::new(r#"https:\/\/github\.com\/([^\/]+)\/([^\/]+)"#).unwrap();
    let cap = pattern.captures(&link).unwrap();
    (cap[1].to_string(),cap[2].to_string())
}

pub fn update_users_pr_counts(client: &reqwest::Client, start_time: i64, end_time: i64, kwoc_students: &mut HashMap<String,Student>, kwoc_projects: &Vec<Project>) -> Result<(),Error>{
    let mut pulls: Vec<Pull> = Vec::new();
    
    for project in kwoc_projects.iter() {
        //possibly concurrent calls here
        let (owner,repo) = split_repo_link(project.RepoLink.as_ref().unwrap());
        pulls.append(&mut get_all_pulls(client, owner, repo)?);
    }

    for pull in pulls {

        let username = pull.user.as_ref().unwrap().login.as_ref().unwrap();

        if !kwoc_students.contains_key(username) {
            continue;
        }

        let create_time = DateTime::parse_from_rfc3339(pull.created_at.unwrap().as_str())
            .unwrap()
            .timestamp();

        if (create_time < start_time) || (create_time >= end_time) {
            continue;
        }
        
        let student = kwoc_students.get_mut(username).unwrap();
        
        if pull.state.unwrap() == "open" {
            *student.open_pr_count.as_mut().unwrap()+=1;
        }else if pull.merged_at.is_some() {
            *student.merged_pr_count.as_mut().unwrap()+=1;
        }
    }
    Ok(())
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

pub fn print_passed_students(kwoc_students: &HashMap<String,Student>) -> Result<(),Error>{
    for (username,data) in kwoc_students {
        if data.merged_pr_count.unwrap() + data.merged_pr_count.unwrap() >= 1 {
            println!("{username}");
        }
    }
    Ok(())
}

