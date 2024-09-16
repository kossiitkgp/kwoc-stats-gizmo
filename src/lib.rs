use chrono::DateTime;
use http::StatusCode;
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::{Client, Error};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;

pub mod database;
use database::{Project, Student};

const PER_PAGE: u32 = 100;

fn create_client() -> Result<Client, Error> {
    let access_token = match env::var("GH_ACCESS_TOKEN") {
        Ok(token) => Some(token),
        Err(_) => {
            println!("GH_ACCESS_TOKEN not found.");
            println!("Continuing without token.");
            None
        }
    };
    let client = match access_token {
        Some(token) => reqwest::Client::builder()
            .user_agent("stats-api-rust")
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    reqwest::header::AUTHORIZATION,
                    reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
                );
                headers
            })
            .build()?,
        None => reqwest::Client::builder()
            .user_agent("stats-api-rust")
            .build()?,
    };
    Ok(client)
}

lazy_static! {
    static ref CLIENT: Client = create_client().unwrap();
}

#[derive(Deserialize, Debug, Clone)]
pub struct User {
    login: Option<String>,
    id: Option<u32>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Pull {
    url: Option<String>,
    id: Option<u32>,
    state: Option<String>,
    title: Option<String>,
    body: Option<String>,
    user: Option<User>,
    merged_at: Option<String>,
    created_at: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Rate {
    limit: Option<u32>,
    remaining: Option<u32>,
    used: Option<u32>,
    reset: Option<u32>,
}

#[derive(Deserialize, Debug)]
struct RateContainer {
    core: Rate,
    search: Rate,
}

#[derive(Deserialize, Debug)]
struct RateLimit {
    resources: RateContainer,
}

#[tokio::main]
async fn get_all_pulls(owner: String, repo: String) -> Result<Vec<Pull>, Error> {
    let mut request_url =
        format!("https://api.github.com/repos/{owner}/{repo}/pulls?state=all&per_page={PER_PAGE}");

    let pattern = Regex::new(r#"<([^>]+)>; rel="next""#).unwrap();

    let mut pulls = Vec::new();
    let mut end: bool = false;

    while !end {
        let response = CLIENT.get(&request_url).send().await?;

        if response.status() != StatusCode::OK {
            println!("{request_url} STATUS : {:?}", response.status());
            return Ok(pulls);
        }

        let link_headers = &response.headers().get("LINK");

        match link_headers {
            Some(i) => match pattern.captures(i.to_str().unwrap()) {
                Some(i) => request_url = i[1].to_string(),
                None => end = true,
            },
            None => end = true,
        };

        pulls.append(&mut response.json().await?);
    }

    Ok(pulls)
}

fn split_repo_link(link: &String) -> (String, String) {
    let pattern = Regex::new(r#"https:\/\/github\.com\/([^\/]+)\/([^\/]+)"#).unwrap();
    let cap = pattern.captures(&link).unwrap();
    (cap[1].to_string(), cap[2].to_string())
}

pub fn update_users_pr_counts(
    start_time: i64,
    end_time: i64,
    kwoc_students: &mut HashMap<String, Student>,
    kwoc_projects: &Vec<Project>,
    is_end_vals: bool,
) -> Result<(), Error> {
    let pulls = kwoc_projects.iter().flat_map(|project| {
        let (owner, repo) = split_repo_link(project.repo_link.as_ref().unwrap());
        get_all_pulls(owner, repo).unwrap()
    });

    pulls.for_each(|pull| {
        let username = pull.user.clone().unwrap().login.unwrap();

        if !kwoc_students.contains_key(&username) {
            return;
        }

        if is_end_vals
            && kwoc_students
                .get(&username)
                .unwrap()
                .passed_mid_evals
                .clone()
                .unwrap()
                != "true"
        {
            return;
        }

        let create_time = DateTime::parse_from_rfc3339(pull.created_at.unwrap().as_str())
            .unwrap()
            .timestamp();

        if (create_time < start_time) || (create_time >= end_time) {
            return;
        }

        let student = kwoc_students.get_mut(&username).unwrap();

        if pull.state.unwrap() == "open" {
            *student.open_pr_count.as_mut().unwrap() += 1;
        } else if pull.merged_at.is_some() {
            *student.merged_pr_count.as_mut().unwrap() += 1;
        }
    });

    Ok(())
}

#[tokio::main]
pub async fn print_rate_limit() -> Result<(), Error> {
    let response = CLIENT
        .get("https://api.github.com/rate_limit")
        .send()
        .await?;

    let rate_limit: RateLimit = response.json().await?;
    println!("Core: {:#?}", rate_limit.resources.core);
    println!("Search: {:#?}", rate_limit.resources.search);
    Ok(())
}

pub fn print_passed_students(kwoc_students: &HashMap<String, Student>) -> Result<(), Error> {
    for (username, data) in kwoc_students {
        if data.merged_pr_count.unwrap() + data.open_pr_count.unwrap() >= 1 {
            println!("{username}");
        }
    }
    Ok(())
}
