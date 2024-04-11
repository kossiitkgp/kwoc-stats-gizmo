use serde::Deserialize;
use reqwest::{header, Error};
use regex::Regex;
use dotenv::dotenv;
use std::env;

const PER_PAGE: u32 = 50;

#[derive(Deserialize, Debug)]
struct User{
    login: Option<String>,
    id: Option<u32>
}

#[derive(Deserialize, Debug)]
struct Pull {
    url: Option<String>,
    id: Option<u32>,
    state: Option<String>,
    title: Option<String>,
    body: Option<String>,
    user: Option<User>
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
async fn get_all_pulls(client: reqwest::Client, owner: &str,repo: &str) -> Result<Vec<Pull>,Error>{
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

#[tokio::main]
async fn print_rate_limit(client: reqwest::Client) -> Result<(),Error>{
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

    print_rate_limit(client)?;
    // let pulls = get_all_pulls(client, "kossiitkgp", "events")?;
    // println!("{:#?}", pulls);
    Ok(())
}
