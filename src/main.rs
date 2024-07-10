use std::collections::HashMap;
use std::sync::Arc;
use time::{macros::date, Date};
use uuid::Uuid;
    
    use axum::{
        extract::State, 
        handler::Handler, 
        http::StatusCode, 
        response::IntoResponse, 
        routing::{get, post}, Router
    };

pub struct Person {
    id: Uuid,
    name: String,
    nick: String,
    birth_date: Date,
    stack: Vec<String>,
}

type AppState = Arc<HashMap<Uuid, Person>>;

#[tokio::main]
async fn main() {

    let mut people: HashMap<Uuid, Person> = HashMap::new();

    let person = Person {
        id: Uuid::now_v7(),
        name: String::from("Guilherme"),
        nick: String::from("Gui"),
        birth_date: date!(1995 - 01 - 25),
        stack: vec!["Java".to_string(),"Angular".to_string(),"Ruby".to_string()],
    };

    people.insert(person.id, person);

    let app_state: AppState = Arc::new(people);

    let app = Router::new()
            .route("/pessoas", get(search_people))
            .route("/pessoa/:id", get(find_person))
            .route("/pessoas", post(create_person))
            .route("/contagem-pessoas", get(count_people))
            .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn search_people(state: State<Arc<HashMap<Uuid, Person>>>) -> impl IntoResponse {
    (StatusCode::OK, "Busca")
}

async fn find_person() -> impl IntoResponse {
    (StatusCode::OK, "By id")
}

async fn create_person() -> impl IntoResponse {
    (StatusCode::OK, "Create")
}

async fn count_people() -> impl IntoResponse {
   (StatusCode::OK, "Count") 
}

