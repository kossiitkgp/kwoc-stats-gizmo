use dotenv::dotenv;
use reqwest::Error;
use rusqlite::Connection;
use std::env;

use stats_api_rust::database::get_projects;
use stats_api_rust::database::get_students;
use stats_api_rust::print_passed_students;
use stats_api_rust::update_users_pr_counts;

fn main() -> Result<(), Error> {
    dotenv().ok();

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

    let conn = Connection::open("./devDB.db").expect("devDB.db not found.");

    let mut kwoc_students = get_students(&conn).expect("get-students failed.");
    let kwoc_projects = get_projects(&conn).expect("get-projects failed.");

    update_users_pr_counts(
        kwoc_start_time,
        kwoc_mid_evals_time,
        &mut kwoc_students,
        &kwoc_projects,
    )
    .expect("updating failed.");

    print_passed_students(&kwoc_students).expect("printing failed.");
    Ok(())
}
