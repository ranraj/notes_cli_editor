
extern crate rusqlite;

use rusqlite::NO_PARAMS;
use rusqlite::{Connection, Result};

use crate::domain::Todo;

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
