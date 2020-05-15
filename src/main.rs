use clap::{load_yaml, App, ArgMatches};
use handler::{handle_init,handle_test,handle_add,handle_list,handle_remove,handle_config_argument};

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from(yaml).get_matches();
    application(&matches);
}

fn application(matches: &ArgMatches) {
    let settings = handle_config_argument(matches);
    handle_init(matches, &settings);
    handle_test(matches, &settings);
    handle_add(matches, &settings);
    handle_list(matches, &settings);
    handle_remove(matches, &settings);
}
mod handler{
    use std::io::{stdin, stdout, Write};
    use cfg_if::cfg_if;
    use log::info;
    use clap::{ArgMatches};
    use crate::domain::{Action, Setup, Todo, TodoResponse};
    use crate::config::{setup_router};
    use crate::action::action_router;

    const DELIMETER: &str = "$";

    cfg_if! {
        if #[cfg(test)] {
            use crate::domain::MockSettings as Settings;
        } else {
            use crate::domain::Settings;
        }
    }

    pub fn handle_config_argument(matches: &ArgMatches) -> Settings {
        let base_settings = match Settings::load_config() {
            Ok(settings) => settings,
            Err(why) => {
                info!("Unable to load configuraiton, setting default : {}", why);            
                Settings::system_default()
            }
        };
        if matches.is_present("db") {
            let db = matches
                .value_of("db")
                .unwrap_or(base_settings.get_db().as_str())
                .trim()
                .to_lowercase();
            base_settings.update(db)
        } else {
            base_settings
        }
    }
    
    pub fn handle_init(matches: &ArgMatches, settings: &Settings) {
        if matches.is_present("init") {
            match setup_router(settings, Setup::Init) {
                Ok(_) => println!("Initialization completed successful"),
                Err(why) => println!("Initialization has failed - Reason : {}", why),
            }
        }
    }
    
    pub fn handle_test(matches: &ArgMatches, settings: &Settings) {
        if matches.is_present("test") {
            match setup_router(settings, Setup::Test) {
                Ok(_) => println!("Test completed successful"),
                Err(_) => println!("Test has failed, Please initalize"),
            }
        }
    }
    
    pub fn handle_add(matches: &ArgMatches, settings: &Settings) {
        if let Some(_) = matches.subcommand_matches("add") {
            let (title, content) = read_add_input();
            let todo = Todo {
                id: Option::None,
                title,
                content,
                user_name: Option::None,
            };
            match action_router(&settings, Action::Save(todo)) {
                Ok(_) => println!("Saved successful"),
                Err(_) => println!("Save has failed, Please use test command"),
            }
        }
    }
    pub fn handle_list(matches: &ArgMatches, settings: &Settings) {
        if let Some(matches) = matches.subcommand_matches("list") {
            if let Some(id) = matches.value_of("input").map(|id| id.trim().parse::<i64>()) {
                match id {
                    Ok(record_id) => {
                        if let Ok(response) = action_router(&settings, Action::FetchById(record_id)) {
                            match response {
                                TodoResponse::One(todo) => {
                                    if let Some(record) = todo {
                                        let serialized_todo = serde_json::to_string(&record).unwrap();
                                        println!("{}", serialized_todo);
                                    } else {
                                        println!("Record not found")
                                    }
                                }
                                _ => println!("Record not found"),
                            }
                        } else {
                            //Todo
                        }
                    }
                    Err(_) => println!("Not a valid integer"),
                }
            } else {
                if let Ok(response) = action_router(&settings, Action::Fetch) {
                    match response {
                        TodoResponse::All(todos) => {
                            for todo in todos {
                                let serialized_todo = serde_json::to_string(&todo).unwrap();
                                println!("{}", serialized_todo);
                            }
                        }
                        _ => println!("Records not found"),
                    }
                } else {
                    //TODO
                }
            }
        }
    }
    
    pub fn handle_remove(matches: &ArgMatches, settings: &Settings) {
        if let Some(matches) = matches.subcommand_matches("remove") {
            if let Some(id) = matches.value_of("input").map(|id| id.trim().parse::<i64>()) {
                match id {
                    Ok(record_id) => {
                        let message = format!("a record id : {}", record_id);
                        if remove_confirmation(&message) {
                            if let Ok(response) = action_router(settings, Action::DeleteById(record_id))
                            {
                                match response {
                                    TodoResponse::Done => {
                                        println!("Successfuly removed a record id {}", record_id)
                                    }
                                    _ => println!("Record not found"),
                                }
                            } else {
                                //TODO
                            }
                        }
                    }
                    Err(_) => println!("Not a valid integer"),
                }
            } else {
                if remove_confirmation("all records") {
                    if let Ok(response) = action_router(settings, Action::Delete) {
                        match response {
                            TodoResponse::Done => println!("Remove all successful "),
                            _ => println!("Record not found"),
                        }
                    }
                } else {
                    //TODO : ignore
                }
            }
        }
    }
    fn read_add_input() -> (String, String) {
        let mut title = String::new();
        let mut content = String::new();
        print!("Title {} ", DELIMETER);
        let _ = stdout().flush();
        stdin()
            .read_line(&mut title)
            .expect("Did not enter a correct string");
        clean_input(&mut title);
    
        print!("Content {} ", DELIMETER);
        let _ = stdout().flush();
        stdin()
            .read_line(&mut content)
            .expect("Did not enter a correct string");
        clean_input(&mut content);
        (title, content)
    }
    fn clean_input(s: &mut String) {
        if let Some('\n') = s.chars().next_back() {
            s.pop();
        }
        if let Some('\r') = s.chars().next_back() {
            s.pop();
        }
    }
    
    fn remove_confirmation(message: &str) -> bool {
        let mut confirmation = String::new();
        print!(
            "Do you want to remove {} (press enter to continue or type (N/n)) {} ",
            message, DELIMETER
        );
        let _ = stdout().flush();
        stdin()
            .read_line(&mut confirmation)
            .expect("Did not enter a correct string");
        clean_input(&mut confirmation);
        if confirmation.eq_ignore_ascii_case("N") || confirmation.eq_ignore_ascii_case("n") {
            return false;
        } else {
            return true;
        }
    }
}

mod domain {
    use log::{info, warn};
    use mockall::*;
    use std::error;
    use std::fmt;
    use crate::config::CONFIG_FILE;
    use crate::persistence::{db_action, init_db, CrudAction, Response};
    use serde::{Deserialize, Serialize};
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
        pub fn create(db: String,is_saved: bool) -> Self {
            Self {
                db,
                is_saved,
            }
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
                    Ok(Settings::create(db,true))
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
}

mod config {
    use super::domain::{Setup, TodoError, TodoErrorType, TodoResponse};    
    use cfg_if::*;
    use log::{info, warn};
    use mockall::predicate::*;
    
    use std::fs::{File};
    
    cfg_if! {
        if #[cfg(test)] {
            use super::domain::MockSettings as Settings;            
        } else {
            use super::domain::Settings;            
        }
    }
    pub const CONFIG_FILE: &str = "app.conf";
    
    

    fn initialize_setup(settings: &Settings) -> Result<TodoResponse, TodoError> {
        let result_error = initialize_config_file();
        if result_error.is_err() {
            result_error
        } else {
            let write_error = settings.write_default_config();
            if write_error.is_err() {
                write_error
            } else {
                let db_error = settings.initalize_db();
                if db_error.is_err() {
                    db_error
                } else {
                    Ok(TodoResponse::Done)
                }
            }
        }
    }

    fn initialize_config_file() -> Result<TodoResponse, TodoError> {
        if File::open(CONFIG_FILE).is_err() {
            match File::create(CONFIG_FILE) {
                Ok(_) => Ok(TodoResponse::Done),
                _ => Err(TodoError::build(TodoErrorType::UnableToInitialize)),
            }
        } else {
            info!("Config initialized");
            Ok(TodoResponse::Done)
        }
    }

    pub fn setup_router(configuration: &Settings, setup: Setup) -> Result<TodoResponse, TodoError> {
        match setup {
            Setup::Init => initialize_setup(configuration),
            Setup::Test => {
                if configuration.is_config_available() {
                    configuration.test_setup(configuration.get_db())
                } else {
                    Err(TodoError::build(TodoErrorType::TestFailed))
                }
            }
        }
    }

    #[test]
    fn initialize_setup_test() {
        let mut mock = Settings::new();
        mock.expect_write_default_config().returning(||Ok(TodoResponse::Done));
        mock.expect_initalize_db().returning(||Ok(TodoResponse::Done));
        let response = initialize_setup(&mock);
        assert!(matches!(response, Ok(TodoResponse::Done)));
    }

    #[test]
    fn load_config_test() {
        
         
        let settings_ctx = Settings::load_config_context();

        let _settings = settings_ctx.expect().returning(|| Ok(Settings::new()));
        let response = Settings::load_config();
        assert!(matches!(response, _settings));
    }
     
    // #[test]
    // fn write_custom_config_test() {
    //     let config = Settings::new();
    //     let response = config.write_custom_config();
    //     assert!(matches!(response, Ok(TodoResponse::Done)));
    // }
    // #[test]
    // fn is_config_available_test() {
    //     let config = Settings::system_default();
    //     let response = config.is_config_available();
    //     assert!(response, true);
    // }
}

#[cfg(test)]
mod tests {
    // use cfg_if::*;
    // use crate::config::{setup_router};
    // use crate::domain::{Setup,TodoResponse,MockSet};
    // cfg_if! {
    //     if #[cfg(test)] {
    //         use super::domain::MockSettings as Settings;
    //     } else {
    //         use super::domain::Settings;
    //     }
    // }
    // #[test]
    // fn setup_router_configuration_not_available_test() {
    //     MockSet::new();
    // }
    use mockall::*;    
     
    use super::domain::{MockSettings,TodoResponse};
 
    #[test]
    fn settings() {
        let mut mock = MockSettings::new();
        mock.expect_test_setup().returning(|x| Ok(TodoResponse::Done));
        println!("{:?}", mock.test_setup("".to_owned()));
    }
}
mod action {
    use super::domain::{Action, Todo, TodoError, TodoErrorType, TodoResponse, ID};
    use super::persistence::{db_action, CrudAction, Response};
    use cfg_if::*;
    cfg_if! {
        if #[cfg(test)] {
            use super::domain::MockSettings as Settings;
        } else {
            use super::domain::Settings;
        }
    }

    pub fn action_router(
        configuration: &Settings,
        action: Action,
    ) -> Result<TodoResponse, TodoError> {
        let db = configuration.get_db();
        if configuration.is_config_available() {
            match action {
                Action::Save(todo) => save(todo, db),
                Action::Fetch => fetch(db),
                Action::FetchById(id) => fetch_by_id(id, db),
                Action::Delete => delete(db),
                Action::DeleteById(id) => delete_by_id(id, db),
            }
        } else {
            Err(TodoError::build(TodoErrorType::InitNotAvailable))
        }
    }
    fn save(todo: Todo, db: String) -> Result<TodoResponse, TodoError> {
        db_action(CrudAction::Save(todo), db);
        //TODO : Handle error
        Ok(TodoResponse::Done)
    }
    fn fetch(db: String) -> Result<TodoResponse, TodoError> {
        Ok(match db_action(CrudAction::FindAll, db) {
            Response::List(result) => TodoResponse::All(result),
            _ => TodoResponse::Empty,
        })
    }
    fn fetch_by_id(id: ID, db: String) -> Result<TodoResponse, TodoError> {
        Ok(match db_action(CrudAction::Find(id), db) {
            Response::One(result) => TodoResponse::One(result),
            _ => TodoResponse::Empty,
        })
    }

    fn delete(db: String) -> Result<TodoResponse, TodoError> {
        match db_action(CrudAction::RemoveAll, db) {
            Response::Success => Ok(TodoResponse::Done),
            _ => Err(TodoError::build(TodoErrorType::InitNotAvailable)),
        }
    }

    fn delete_by_id(id: ID, db: String) -> Result<TodoResponse, TodoError> {
        match db_action(CrudAction::Remove(id), db) {
            Response::Success => Ok(TodoResponse::Done),
            _ => Err(TodoError::build(TodoErrorType::InitNotAvailable)),
        }
    }
    // #[test]
    // fn save_test() {
    //     let todo = Todo::new("".to_owned(), "".to_owned());
    //     let settings = Settings::new();
    //     let response = action_router(&settings,Action::Save(todo));
    //     assert!(matches!(response, Ok(TodoResponse::Done)));
    // }
    // #[test]
    // fn fetch_test() {
    //     let settings = MockSettings::new();
    //     let response = action_router(settings,Action::Fetch);
    //     assert!(matches!(response, Ok(TodoResponse::Done)));
    // }
    // #[test]
    // fn fetch_by_id_test() {
    //     let id = 0;
    //     let settings = MockSettings::new();
    //     let response = action_router(settings,Action::FetchById(id));
    //     assert!(matches!(response, Ok(TodoResponse::Done)));
    // }
    // #[test]
    // fn delete_test() {
    //     let settings = MockSettings::new();
    //     let response = action_router(&settings,Action::Delete);
    //     assert!(matches!(response, Ok(TodoResponse::Done)));
    // }
    // #[test]
    // fn delete_by_id_test() {
    //     let id = 0;
    //     let settings = MockSettings::new;
    //     let response = action_router(settings,Action::DeleteById(id));
    //     assert!(matches!(response, Ok(TodoResponse::Done)));
    // }
}
mod setup {
    use super::domain::{TodoError, TodoResponse};
    use crate::persistence::CrudAction;

    // fn init() -> Result<TodoResponse, TodoError> {
    //     Ok(TodoResponse::Done)
    // }
    // fn create_db() -> Result<TodoResponse, TodoError> {
    //     Ok(TodoResponse::Done)
    // }
    // fn create_default_user() -> Result<TodoResponse, TodoError> {
    //     Ok(TodoResponse::Done)
    // }
    // fn check_db() -> Result<TodoResponse, TodoError> {
    //     Ok(TodoResponse::Done)
    // }
    // #[test]
    // fn init_test() {
    //     let response = init();
    //     assert!(matches!(response, Ok(TodoResponse::Done)));
    // }
    // #[test]
    // fn create_db_test() {
    //     let response = create_db();
    //     assert!(matches!(response, Ok(TodoResponse::Done)));
    // }
    // #[test]
    // fn create_default_user_test() {
    //     let response = create_default_user();
    //     assert!(matches!(response, Ok(TodoResponse::Done)));
    // }
    // #[test]
    // fn check_db_test() {
    //     let response = db_action(CrudAction::Save,"".to_owned());
    //     assert!(matches!(response, Ok(TodoResponse::Done)));
    // }
}

mod persistence {
    extern crate rusqlite;
    use crate::domain::Todo;
    use rusqlite::NO_PARAMS;
    use rusqlite::{Connection, Result};

    use serde::{Deserialize, Serialize};

    static DEFAULT_USER: &str = "Root";

    pub fn init_db(db: &String) -> Result<Response> {
        let conn = Connection::open(db)?;

        conn.execute(
            "create table if not exists user (
             id integer primary key,
             name text not null unique
         )",
            NO_PARAMS,
        )?;
        conn.execute(
            "create table if not exists todo (
             id integer primary key,
             title text not null,
             content text not null,
             user_id integer not null references user(id)
         )",
            NO_PARAMS,
        )?;

        conn.execute(
            "create table if not exists health (             
             name text not null             
         )",
            NO_PARAMS,
        )?;

        Ok(match insert_user(DEFAULT_USER, &conn) {
            Ok(_) => Response::Success,
            Err(_) => {
                //TODO : ignore unique constraint error
                //println!("Init {}",e);
                Response::Success
            }
        })
    }

    pub fn check(conn: &Connection) -> Result<Response> {
        #[derive(Debug)]
        struct Health {
            name: String,
        }

        let name = String::from("health str");
        conn.execute("INSERT INTO health (name) values (?1)", &[&name])?;
        let mut stmt = conn.prepare("SELECT * FROM health;")?;

        let health = stmt.query_map(NO_PARAMS, |row| Ok(Health { name: row.get(0)? }))?;
        conn.execute("DELETE FROM health", NO_PARAMS)?;
        Ok(Response::Success)
    }

    pub enum CrudAction {
        Save(Todo),
        Find(i64),
        Remove(i64),
        FindAll,
        RemoveAll,
        HealthCheck,
    }
    pub enum Response {
        List(Vec<Todo>),
        One(Option<Todo>),
        Success,
        Error(String),
    }

    pub fn db_action(action: CrudAction, db: String) -> Response {
        if let Ok(conn) = Connection::open(db) {
            match action {
                CrudAction::Save(todo) => insert_todo(todo, &conn).unwrap(),
                CrudAction::Find(id) => match read_one(id, &conn) {
                    Ok(resp) => resp,
                    Err(_) => Response::Error("Failure".to_string()),
                },
                CrudAction::FindAll => read_all(&conn).unwrap(),
                CrudAction::Remove(id) => remove_record(id, &conn).unwrap(),
                CrudAction::RemoveAll => remove_all_records(&conn).unwrap(),
                CrudAction::HealthCheck => match check(&conn) {
                    Ok(resp) => resp,
                    Err(why) => {
                        println!("{}", why);
                        panic!("{}", why)
                    }
                },
            }
        } else {
            println!("Db store is not found, Please setup application");
            Response::Error("Db store is not found, Please setup application".to_string())
        }
    }

    fn insert_user(name: &str, conn: &Connection) -> Result<Response> {
        let last_id: String = conn.last_insert_rowid().to_string();
        conn.execute(
            "INSERT INTO user (id,name) values (?1,?2)",
            &[&last_id, &name.to_string()],
        )?;
        Ok(Response::Success)
    }

    fn insert_todo(todo: Todo, conn: &Connection) -> Result<Response> {
        conn.execute(
        "INSERT INTO todo (title,content,user_id) values (?1,?2, (SELECT id FROM user where name = ?3));",
        &[&todo.title.to_string(),&todo.content.to_string(), &DEFAULT_USER.to_string()],
    )?;

        Ok(Response::Success)
    }
    fn read_one(id: i64, conn: &Connection) -> Result<Response> {
        let mut stmt = conn.prepare(
            "SELECT t.id,t.title,t.content,u.name from todo t
        INNER JOIN user u
        ON u.id = t.user_id where t.id = :id and u.id = (SELECT id FROM user where name = :name)",
        )?;

        let mut rows = stmt.query_named(&[(":id", &id), (":name", &DEFAULT_USER)])?;
        let mut result: Option<Todo> = None;
        while let Some(row) = rows.next()? {
            result = Some(Todo {
                id: Option::Some(row.get(0)?),
                title: row.get(1)?,
                content: row.get(2)?,
                user_name: Option::Some(row.get(3)?),
            })
        }
        Ok(Response::One(result))
    }

    fn read_all(conn: &Connection) -> Result<Response> {
        let mut stmt = conn.prepare(
            "SELECT t.id,t.title,t.content,u.name from todo t
        INNER JOIN user u
        ON u.id = t.user_id;",
        )?;
        let todos = stmt.query_map(NO_PARAMS, |row| {
            Ok(Todo {
                id: Option::Some(row.get(0)?),
                title: row.get(1)?,
                content: row.get(2)?,
                user_name: Option::Some(row.get(3)?),
            })
        })?;
        let collected: rusqlite::Result<Vec<Todo>> = todos.collect();
        let result = match collected {
            Ok(list) => list,
            Err(_) => Vec::<Todo>::new(),
        };
        Ok(Response::List(result))
    }

    fn remove_all_records(conn: &Connection) -> Result<Response> {
        conn.execute("DELETE FROM todo", NO_PARAMS)?;
        Ok(Response::Success)
    }
    fn remove_record(id: i64, conn: &Connection) -> Result<Response> {
        conn.execute("DELETE FROM todo where id =?", &[&id])?;
        Ok(Response::Success)
    }
}
