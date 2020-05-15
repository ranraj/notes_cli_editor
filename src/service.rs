use crate::domain::{Action, Todo, TodoError, TodoErrorType, TodoResponse, ID};
use crate::persistence::{db_action, CrudAction, Response};
use cfg_if::*;

cfg_if! {
    if #[cfg(test)] {
        use super::domain::MockSettings as Settings;
    } else {
        use super::domain::Settings;
    }
}

pub fn action_router(configuration: &Settings, action: Action) -> Result<TodoResponse, TodoError> {
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
