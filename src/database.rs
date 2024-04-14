use rusqlite::{Connection, Result};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Student {
    id: u32,

    name: Option<String>,
    email: Option<String>,
    college: Option<String>,
    username: Option<String>,
    passed_mid_evals: Option<String>,
    passed_end_evals: Option<String>,
    blog_link: Option<String>,

    commit_count: Option<u32>,
    pull_count: Option<u32>,
    lines_added: Option<u32>,
    lines_removed: Option<u32>,

    pub open_pr_count: Option<u32>,
    pub merged_pr_count: Option<u32>,

    languages_used: Option<String>,
    projects_worked: Option<String>,
    pulls: Option<String>,
}

#[derive(Debug)]
pub struct Project {
    id: u32,
    name: Option<String>,
    description: Option<String>,
    tags: Option<String>,
    pub repo_link: Option<String>,
    comm_channel: Option<String>,
    readme_link: Option<String>,
    project_status: Option<String>,

    last_pull_time: Option<u32>,

    commit_count: Option<u32>,
    pull_count: Option<u32>,
    lines_added: Option<u32>,
    lines_removed: Option<u32>,

    contributers: Option<String>,
    pulls: Option<String>,

    mentor_id: Option<u32>,
    secondary_mentor_id: Option<u32>,
}

pub fn get_students(conn: &Connection) -> Result<HashMap<String, Student>> {
    let mut statement = conn.prepare("SELECT * FROM students")?;
    let iterator = statement.query_map([], |row| {
        Ok(Student {
            id: row.get(0)?,
            name: row.get(4)?,
            email: row.get(5)?,
            college: row.get(6)?,
            username: row.get(7)?,
            passed_mid_evals: row.get(8)?,
            passed_end_evals: row.get(9)?,
            blog_link: row.get(10)?,
            commit_count: row.get(11)?,
            pull_count: row.get(12)?,
            lines_added: row.get(13)?,
            lines_removed: row.get(14)?,
            languages_used: row.get(15)?,
            projects_worked: row.get(16)?,
            pulls: row.get(17)?,
            open_pr_count: Some(0),
            merged_pr_count: Some(0),
        })
    })?;
    let result: HashMap<String, Student> = iterator
        .map(|x| {
            (
                x.as_ref().unwrap().username.as_ref().unwrap().clone(),
                x.unwrap(),
            )
        })
        .collect();
    Ok(result)
}

pub fn get_projects(conn: &Connection) -> Result<Vec<Project>> {
    let mut statement = conn.prepare("SELECT * FROM projects")?;
    let iterator = statement.query_map([], |row| {
        Ok(Project {
            id: row.get(0)?,
            name: row.get(4)?,
            description: row.get(5)?,
            tags: row.get(6)?,
            repo_link: row.get(7)?,
            comm_channel: row.get(8)?,
            readme_link: row.get(9)?,
            project_status: row.get(10)?,

            last_pull_time: row.get(11)?,

            commit_count: row.get(12)?,
            pull_count: row.get(13)?,
            lines_added: row.get(14)?,
            lines_removed: row.get(15)?,

            contributers: row.get(16)?,
            pulls: row.get(17)?,

            mentor_id: row.get(18)?,
            secondary_mentor_id: row.get(19)?,
        })
    })?;
    let result: Vec<Project> = iterator.map(|x| x.unwrap()).collect();
    Ok(result)
}
