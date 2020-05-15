
use cfg_if::cfg_if;
use clap::ArgMatches;
use log::info;
use std::io::{stdin, stdout, Write};

use crate::service::action_router;
use crate::config::config_router;
use crate::domain::{Action, Setup, Todo, TodoResponse};

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
        match config_router(settings, Setup::Init) {
            Ok(_) => println!("Initialization completed successful"),
            Err(why) => println!("Initialization has failed - Reason : {}", why),
        }
    }
}

pub fn handle_test(matches: &ArgMatches, settings: &Settings) {
    if matches.is_present("test") {
        match config_router(settings, Setup::Test) {
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
