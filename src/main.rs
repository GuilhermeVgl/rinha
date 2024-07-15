use axum::{
    extract::{Path, State, Json}, 
    http::StatusCode, 
    response::IntoResponse, 
    routing::{get, post}, 
    Router
};
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use time::{macros::date, Date};
use uuid::Uuid;
use serde::Serialize;
use serde::Deserialize;
time::serde::format_description!(date_format, Date, "[year]-[month]-[day]");

#[derive(Clone, Serialize)]
pub struct Person {
    pub id: Uuid,
    #[serde(rename = "nome")]
    pub name: String,
    #[serde(rename = "apelido")]
    pub nick: String,
    #[serde(rename = "nascimento", with = "date_format")]
    pub birth_date: Date,
    pub stack: Option<Vec<String>>,
}

#[derive(Clone, Deserialize)]
#[serde(try_from = "String")]
pub struct PersonName (String);

impl TryFrom<String> for PersonName {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() <= 100 {
            Ok(Self(value))
        } else {
            Err("Name too big")
        }
    }
}

#[derive(Clone, Deserialize)]
#[serde(try_from = "String")]
pub struct Nick (String);

impl TryFrom<String> for Nick {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() <= 32 {
            Ok(Self(value))
        } else {
            Err("Nick too big")
        }
    }
}

#[derive(Clone, Deserialize)]
#[serde(try_from = "String")]
pub struct Tech (String);

impl TryFrom<String> for Tech {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() <= 32 {
            Ok(Self(value))
        } else {
            Err("Some tech is too big")
        }
    }
}

impl From<Tech> for String {
    fn from(value: Tech) -> Self {
        value.0
    }
}

#[derive(Clone, Deserialize)]
pub struct NewPerson {
    #[serde(rename = "nome")]
    pub name: PersonName,
    #[serde(rename = "apelido")]
    pub nick: Nick, 
    #[serde(rename = "nascimento", with = "date_format")]
    pub birth_date: Date,
    pub stack: Option<Vec<Tech>>,
}

type AppState = Arc<Mutex<HashMap<Uuid, Person>>>;

#[tokio::main]
async fn main() {

    let mut people: HashMap<Uuid, Person> = HashMap::new();

    let person = Person {
        id: Uuid::now_v7(),
        name: String::from("Guilherme"),
        nick: String::from("Gui"),
        birth_date: date!(1995 - 01 - 25),
        stack: None,
    };

    println!("{}", person.id);

    people.insert(person.id, person);

    let app_state: AppState = Arc::new(Mutex::new(people));

    let app = Router::new()
            .route("/pessoas", get(search_people))
            .route("/pessoa/:id", get(find_person))
            .route("/pessoas", post(create_person))
            .route("/contagem-pessoas", get(count_people))
            .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn search_people() -> impl IntoResponse {
    (StatusCode::OK, "Busca")
}

async fn find_person(
    State(people): State<AppState>, 
    Path(person_id): Path<Uuid>
) -> impl IntoResponse {

    let my_people = people.lock().await;

    match my_people.get(&person_id) {
        Some(person) => Ok(Json(person.clone())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn create_person(
    State(people): State<AppState>,
    Json(new_person): Json<NewPerson>, 
) -> impl IntoResponse{

    let id = Uuid::now_v7();
    let person = Person {
        id,
        name: new_person.name.0,
        birth_date: new_person.birth_date,
        nick: new_person.nick.0,
        stack: new_person
            .stack
            .map(|stack| stack.into_iter().map(String::from).collect()),
    };

    people.lock().await.insert(id, person.clone());

    (StatusCode::CREATED, Json(person))
}

async fn count_people(
    State(people): State<AppState>
)-> impl IntoResponse {
    let count = people.lock().await.len();

   (StatusCode::OK, Json(count)) 
}

