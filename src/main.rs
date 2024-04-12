use rusqlite::Connection;
use reqwest::Error;
use dotenv::dotenv;
use std::env;

use stats_api_rust::database::get_students;
use stats_api_rust::get_users_pr_counts;

fn main() -> Result<(),Error> {

    dotenv().ok();

    let access_token = match env::var("GH_ACCESS_TOKEN") {
        Ok(token) => Some(token),
        Err(err) => {
            println!("GH_ACCESS_TOKEN not found.");
            None
        }
    };

    let kwoc_start_time = env::var("KWOC_START_TIME")
        .expect("KWOC_START_TIME not found.")
        .parse::<i64>()
        .expect("KWOC_START_TIME not valid.");
    let kwoc_mid_evals_time = env::var("KWOC_MID_EVALS_TIME")
        .expect("KWOC_MID_EVALS_TIME not found.")
        .parse::<i64>()
        .expect("KWOC_MID_EVALS_TIME not valid.");
    let kwoc_end_evals_time = env::var("KWOC_END_EVALS_TIME")
        .expect("KWOC_END_EVALS_TIME not found.")
        .parse::<i64>()
        .expect("KWOC_END_EVALS_TIME not valid.");

    let client = match access_token {
        Some(token) => {
            reqwest::Client::builder()
                .user_agent("stats-api-rust") 
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
                .user_agent("stats-api-rust") 
                .build()?
        }
    };

    let conn = Connection::open("./devDB.db").expect("devDB.db not found.");

    let kwoc_students = get_students(&conn).expect("get-students failed.");

    let users = get_users_pr_counts(&client, "nik132-eng", "chrome-extensions",kwoc_start_time,kwoc_mid_evals_time,&kwoc_students).unwrap();

    for (key, value) in users.iter() {
        println!("{key} {:?}",value);
    }
    Ok(())
}
