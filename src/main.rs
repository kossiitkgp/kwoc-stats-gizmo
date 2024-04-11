use serde::Deserialize;
use reqwest::Error;
use regex::Regex;

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

fn main() -> Result<(),Error> {       
    let client = reqwest::Client::builder()
        .user_agent("Rust")
        .build()?;

    let pulls = get_all_pulls(client, "kossiitkgp", "events")?;
    println!("{:#?}", pulls);
    Ok(())
}
