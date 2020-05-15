use crate::config::CONFIG_FILE;
use crate::persistence::{db_action, init_db, CrudAction, Response};
use log::{info, warn};
use mockall::*;
use serde::{Deserialize, Serialize};
use std::error;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::BufReader;

const DEFAULT_DB_NAME: &str = "todo";
const ROOT_USER: &str = "root";

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Todo {
    pub id: Option<i32>,
    pub title: String,
    pub content: String,
    pub user_name: Option<String>,
}
impl Todo {
    pub fn new(title: String, content: String) -> Self {
        Self {
            id: None,
            title,
            content,
            user_name: Some(ROOT_USER.to_owned()),
        }
    }
}
pub type ID = i64;
pub enum Action {
    Save(Todo),
    Fetch, //TODO : Pagination
    FetchById(ID),
    Delete,
    DeleteById(ID),
}

#[derive(Debug, PartialEq, Eq)]
pub enum TodoResponse {
    Done,
    One(Option<Todo>),
    All(Vec<Todo>),
    Empty,
}

#[derive(Debug)]
pub struct TodoError {
    msg: String,
    error_type: TodoErrorType,
}
#[derive(Debug)]
pub enum TodoErrorType {
    InitNotAvailable,
    UnableToInitialize,
    TestFailed,
    RecordNotFound,
}
impl TodoError {
    pub fn build(todo: TodoErrorType) -> TodoError {
        match todo {
            TodoErrorType::InitNotAvailable => TodoError {
                msg: "Please initialize application,use help".to_owned(),
                error_type: TodoErrorType::InitNotAvailable,
            },
            TodoErrorType::UnableToInitialize => TodoError {
                msg: "Unable to initizlize application, contact support".to_owned(),
                error_type: TodoErrorType::UnableToInitialize,
            },
            TodoErrorType::TestFailed => TodoError {
                msg: "Database check has failed".to_owned(),
                error_type: TodoErrorType::UnableToInitialize,
            },
            TodoErrorType::RecordNotFound => TodoError {
                msg: "Record Not Found".to_owned(),
                error_type: TodoErrorType::RecordNotFound,
            },
        }
    }
}
impl fmt::Display for TodoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid first item to double")
    }
}

// This is important for other errors to wrap this one.
impl error::Error for TodoError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

pub enum Setup {
    Init,
    Test,
}
struct ConfigurationArgument {
    db: bool,
    set: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Settings {
    pub db: String,
    pub is_saved: bool,
}

#[automock]
impl Settings {
    pub fn system_default() -> Self {
        Self {
            db: DEFAULT_DB_NAME.to_string(),
            is_saved: false,
        }
    }
    pub fn create(db: String, is_saved: bool) -> Self {
        Self { db, is_saved }
    }

    pub fn update(&self, db: String) -> Self {
        Self {
            db,
            is_saved: self.is_saved,
        }
    }
    pub fn get_db(&self) -> String {
        format!("{}.store", self.db.to_owned())
    }
    pub fn test_setup(&self, db: String) -> Result<TodoResponse, TodoError> {
        match db_action(CrudAction::HealthCheck, db) {
            Response::Success => Ok(TodoResponse::Done),
            _ => Err(TodoError::build(TodoErrorType::InitNotAvailable)),
        }
    }
    pub fn write_default_config(&self) -> Result<TodoResponse, TodoError> {
        let file_options = OpenOptions::new().write(true).open(CONFIG_FILE);

        match file_options {
            Ok(mut file) => match file.write(self.to_string().as_bytes()) {
                Ok(_) => Ok(TodoResponse::Done),
                Err(why) => {
                    info!("couldn't write to {}", why);
                    Err(TodoError::build(TodoErrorType::UnableToInitialize))
                }
            },
            Err(why) => {
                info!("couldn't write to {}", why);
                Err(TodoError::build(TodoErrorType::UnableToInitialize))
            }
        }
    }
    pub fn write_custom_config(&self) -> Result<TodoResponse, TodoError> {
        Ok(TodoResponse::Done)
    }
    //TODO : Improve with Option<Configuration> for load_config()
    pub fn is_config_available(&self) -> bool {
        self.is_saved
    }
    pub fn initalize_db(&self) -> Result<TodoResponse, TodoError> {
        let db = format!("{}.store", self.db);
        match init_db(&db) {
            Ok(_) => Ok(TodoResponse::Done),
            Err(why) => {
                warn!("Unable to initiazlize the DB {}", why);
                Err(TodoError::build(TodoErrorType::UnableToInitialize))
            }
        }
    }

    pub fn load_config() -> Result<Self, TodoError> {
        match File::open(CONFIG_FILE) {
            Ok(config_file) => {
                let buf_reader = BufReader::new(config_file);
                let mut db: String = DEFAULT_DB_NAME.to_owned();

                for (_, line) in buf_reader.lines().enumerate() {
                    let line = line.unwrap();
                    let split = line.split("=");
                    let vec = split.collect::<Vec<&str>>();

                    match vec[0] {
                        "db" => db = vec[1].trim().to_string(),
                        _ => (),
                    }
                }
                Ok(Settings::create(db, true))
            }
            Err(_) => Err(TodoError::build(TodoErrorType::InitNotAvailable)),
        }
    }
}
impl fmt::Display for Settings {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "db={} \n", &self.db.trim().replace(".store", ""))
    }
}
