fn main() {}

mod domain {
    use std::error;
    use std::fmt;
    #[derive(Debug, PartialEq, Eq)]
    pub struct Todo {
        id: Option<i32>,
        title: String,
        content: String,
    }
    impl Todo {
        pub fn new(title: String, content: String) -> Self {
            Self {
                id: None,
                title,
                content,
            }
        }
    }
    pub type ID = i32;
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
        One(Todo),
        All(Vec<Todo>),
    }

    #[derive(Debug)]
    pub struct TodoError {
        msg: String,
        error_type: TodoErrorType,
    }
    #[derive(Debug)]
    pub enum TodoErrorType {
        InitNotAvailable,
    }
    impl TodoError {
        pub fn build(todo: TodoErrorType) -> TodoError {
            match todo {
                TodoErrorType::InitNotAvailable => TodoError {
                    msg: "Please initialize application,use help".to_owned(),
                    error_type: TodoErrorType::InitNotAvailable,
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
}

mod config {
    use super::domain::{Setup, TodoError, TodoErrorType, TodoResponse};
    use mockall::predicate::*;
    use mockall::*;

    fn initialize_setup() -> Result<TodoResponse, TodoError> {
        Ok(TodoResponse::Done)
    }
    
    fn setup_router(configuraiton:&impl Configuration, setup: Setup) -> Result<TodoResponse, TodoError> {
                    
        match setup {
            Setup::Init => initialize_setup(),
            Setup::Test => {
                if configuraiton.is_config_available() {                                                           
                    Ok(TodoResponse::Done)
                } else {
                    Err(TodoError::build(TodoErrorType::InitNotAvailable))
                }
            }
        }
    }

    #[automock]
    trait Configuration{        
        fn test_setup(&self) -> Result<TodoResponse, TodoError>;
        fn load_config(&self) -> Result<Settings,TodoError> ;
        fn write_default_config(&self) -> Result<TodoResponse, TodoError>;
        fn write_custom_config(&self) -> Result<TodoResponse, TodoError>;
        fn is_config_available(&self) -> bool;
    }
    #[derive(Debug, PartialEq, Eq)]
    pub struct Settings {
        pub db: Option<String>,
    }
    #[automock]
    impl Settings{
        fn new() -> Self {
            Self { db: None }
        }                    
    }
    impl Configuration for Settings{
        
        fn test_setup(&self) -> Result<TodoResponse, TodoError> {
            Ok(TodoResponse::Done)
        }
        fn load_config(&self) -> Result<Settings,TodoError> {
            Ok(Settings::new())
        }
        fn write_default_config(&self) -> Result<TodoResponse, TodoError> {
            Ok(TodoResponse::Done)
        }
        fn write_custom_config(&self) -> Result<TodoResponse, TodoError> {
            Ok(TodoResponse::Done)
        }
        //TODO : Improve with Option<Configuration> for load_config()
        fn is_config_available(&self) -> bool {
            true
        }
    }
    
    #[test]
    fn setup_router_test() {        
        let configuraiton = Settings::new();
        let response = setup_router(&configuraiton,Setup::Init);
        assert!(matches!(response, Ok(TodoResponse::Done)));
    }
    #[test]
    fn setup_router_configuration_not_available_test() {
        let mut mock = MockConfiguration::new();
        
        mock.expect_is_config_available().returning(||true);
        assert_eq!(true,mock.is_config_available());
        let response = setup_router(&mock,Setup::Test);
        assert!(matches!(response, Ok(TodoResponse::Done)));
    }
    #[test]
    fn initialize_setup_test() {
        let response = initialize_setup();
        assert!(matches!(response, Ok(TodoResponse::Done)));
    }

    // #[test]
    // fn load_config_test() {
    //     let configuration = MockConfiguration::new();
    //     let mock_settings = MockSettings::new();

    //     let join = configuration.expect_load_config().returning(||mock_settings);
    //     let response = configuration.load_config();
    //     assert!(matches!(response, Ok(Settings)));
    // }
    #[test]
    fn write_default_config_test() {
        let config = Settings::new();
        let response = config.write_default_config();
        assert!(matches!(response, Ok(TodoResponse::Done)));
    }
    #[test]
    fn write_custom_config_test() {
        let config = Settings::new();
        let response = config.write_custom_config();
        assert!(matches!(response, Ok(TodoResponse::Done)));
    }
    #[test]
    fn is_config_available_test() {
        let config = Settings::new();
        let response = config.is_config_available();
        assert!(response, true);
    }
}

mod action {
    use super::domain::{Action, Todo, TodoError, TodoResponse, ID};

    fn action_router(action: Action) -> Result<TodoResponse, TodoError> {
        match action {
            Action::Save(todo) => save(todo),
            Action::Fetch => fetch(),
            Action::FetchById(id) => fetch_by_id(id),
            Action::Delete => delete(),
            Action::DeleteById(id) => delete_by_id(id),
        }
    }
    fn save(todo: Todo) -> Result<TodoResponse, TodoError> {
        Ok(TodoResponse::Done)
    }
    fn fetch() -> Result<TodoResponse, TodoError> {
        Ok(TodoResponse::Done)
    }
    fn fetch_by_id(id: ID) -> Result<TodoResponse, TodoError> {
        Ok(TodoResponse::Done)
    }
    fn delete() -> Result<TodoResponse, TodoError> {
        Ok(TodoResponse::Done)
    }
    fn delete_by_id(id: ID) -> Result<TodoResponse, TodoError> {
        Ok(TodoResponse::Done)
    }
    #[test]
    fn save_test() {
        let todo = Todo::new("".to_owned(), "".to_owned());
        let response = action_router(Action::Save(todo));
        assert!(matches!(response, Ok(TodoResponse::Done)));
    }
    #[test]
    fn fetch_test() {
        let response = action_router(Action::Fetch);
        assert!(matches!(response, Ok(TodoResponse::Done)));
    }
    #[test]
    fn fetch_by_id_test() {
        let id = 0;
        let response = action_router(Action::FetchById(id));
        assert!(matches!(response, Ok(TodoResponse::Done)));
    }
    #[test]
    fn delete_test() {
        let response = action_router(Action::Delete);
        assert!(matches!(response, Ok(TodoResponse::Done)));
    }
    #[test]
    fn delete_by_id_test() {
        let id = 0;
        let response = action_router(Action::DeleteById(id));
        assert!(matches!(response, Ok(TodoResponse::Done)));
    }
}
mod setup {
    use super::domain::{TodoError, TodoResponse};
    fn init() -> Result<TodoResponse, TodoError> {
        Ok(TodoResponse::Done)
    }
    fn create_db() -> Result<TodoResponse, TodoError> {
        Ok(TodoResponse::Done)
    }
    fn create_default_user() -> Result<TodoResponse, TodoError> {
        Ok(TodoResponse::Done)
    }
    fn check_db() -> Result<TodoResponse, TodoError> {
        Ok(TodoResponse::Done)
    }
    #[test]
    fn init_test() {
        let response = init();
        assert!(matches!(response, Ok(TodoResponse::Done)));
    }
    #[test]
    fn create_db_test() {
        let response = create_db();
        assert!(matches!(response, Ok(TodoResponse::Done)));
    }
    #[test]
    fn create_default_user_test() {
        let response = create_default_user();
        assert!(matches!(response, Ok(TodoResponse::Done)));
    }
    #[test]
    fn check_db_test() {
        let response = check_db();
        assert!(matches!(response, Ok(TodoResponse::Done)));
    }
}
