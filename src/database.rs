use rusqlite::{Connection, Result};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Student {
    ID: u32,

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

    LanguagesUsed: Option<String>,
    ProjectsWorked: Option<String>,
    Pulls: Option<String>,
}

pub fn get_students(conn: &Connection) -> Result<HashMap<String,Student>>{
    let mut statement = conn.prepare("SELECT * FROM students")?;
    let iterator = statement.query_map([], |row| Ok(
        Student{
            ID: row.get(0)?,
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
        }
    ))?;
    let result: HashMap<String,Student> = iterator.map(|x|
        (x.as_ref().unwrap().Username.as_ref().unwrap().clone(),x.unwrap())
    ).collect();
    Ok(result)
}