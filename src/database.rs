use rusqlite::{Connection, Result};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Student {
    id: u32,

    Name: Option<String>,
    Email: Option<String>,
    College: Option<String>,
    Username: Option<String>,
    PassedMidEvals: Option<String>,
    PassedEndEvals: Option<String>,
    BlogLink: Option<String>,

    CommitCount: Option<u32>,
    PullCount: Option<u32>,
    LinesAdded: Option<u32>,
    LinesRemoved: Option<u32>,
    
    pub open_pr_count: Option<u32>,
    pub merged_pr_count: Option<u32>,

    LanguagesUsed: Option<String>,
    ProjectsWorked: Option<String>,
    Pulls: Option<String>,
}

#[derive(Debug)]
pub struct Project {
    id: u32,
    Name: Option<String>,
    Description: Option<String>,
    Tags: Option<String>,
    pub RepoLink: Option<String>,
    CommChannel: Option<String>,
    ReadmeLink: Option<String>,
    ProjectStatus: Option<String>,

    LastPullTime: Option<u32>,

    CommitCount: Option<u32>,
    PullCount: Option<u32>,
    LinesAdded: Option<u32>,
    LinesRemoved: Option<u32>,

    Contributers: Option<String>,
    Pulls: Option<String>,

    MentorId: Option<u32>,
    SecondaryMentorId: Option<u32>
}

pub fn get_students(conn: &Connection) -> Result<HashMap<String,Student>>{
    let mut statement = conn.prepare("SELECT * FROM students")?;
    let iterator = statement.query_map([], |row| Ok(
        Student{
            id: row.get(0)?,
            Name: row.get(4)?,
            Email: row.get(5)?,
            College: row.get(6)?,
            Username: row.get(7)?,
            PassedMidEvals: row.get(8)?,
            PassedEndEvals: row.get(9)?,
            BlogLink: row.get(10)?,
            CommitCount: row.get(11)?,
            PullCount: row.get(12)?,
            LinesAdded: row.get(13)?,
            LinesRemoved: row.get(14)?,
            LanguagesUsed: row.get(15)?,
            ProjectsWorked: row.get(16)?,
            Pulls: row.get(17)?,
            open_pr_count: Some(0),
            merged_pr_count: Some(0),
        }
    ))?;
    let result: HashMap<String,Student> = iterator.map(|x|
        (x.as_ref().unwrap().Username.as_ref().unwrap().clone(),x.unwrap())
    ).collect();
    Ok(result)
}

pub fn get_projects(conn: &Connection) -> Result<Vec<Project>>{
    let mut statement = conn.prepare("SELECT * FROM projects")?;
    let iterator = statement.query_map([], |row| Ok(
        Project{
            id: row.get(0)?,
            Name: row.get(4)?,
            Description: row.get(5)?,
            Tags: row.get(6)?,
            RepoLink: row.get(7)?,
            CommChannel: row.get(8)?,
            ReadmeLink: row.get(9)?,
            ProjectStatus: row.get(10)?,

            LastPullTime: row.get(11)?,

            CommitCount: row.get(12)?,
            PullCount: row.get(13)?,
            LinesAdded: row.get(14)?,
            LinesRemoved: row.get(15)?,

            Contributers: row.get(16)?,
            Pulls: row.get(17)?,

            MentorId: row.get(18)?,
            SecondaryMentorId: row.get(19)?
        }
    ))?;
    let result: Vec<Project> = iterator.map(|x|
        x.unwrap()
    ).collect();
    Ok(result)
}