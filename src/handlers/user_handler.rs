use actix_web::{ get, post, put, web, HttpResponse, Responder };
use mongodb::{ bson::doc, Collection };
use log::error;
use serde::{ Deserialize, Serialize };
use futures::stream::TryStreamExt;

use crate::{
    configs::db::AppStates,
    constants,
    models::{ error_model::ApiErrorType, user_model::User },
};

#[derive(Deserialize)]
pub enum OrderQuery {
    NEW,
    OLD,
}

#[derive(Deserialize)]
pub struct ListQuery {
    search: Option<String>,
    per_page: Option<i64>,
    page: Option<i64>,
    order: Option<OrderQuery>,
}
impl Default for ListQuery {
    fn default() -> Self {
        ListQuery {
            page: Some(1),
            per_page: Some(10),
            search: Some("".to_string()),
            order: Some(OrderQuery::NEW),
        }
    }
}

#[derive(Serialize)]
pub struct ResultData {
    data: Vec<User>,
    total: u64,
    page: i64,
    per_page: i64,
}

/// Adds a new user to the "users" collection in the database.
#[post("/add_user")]
async fn add_user(cfg: web::Data<AppStates>, json: web::Json<User>) -> HttpResponse {
    let collection: Collection<User> = cfg.mongo_db.collection("users");
    let result = collection.insert_one(json.into_inner()).await;
    match result {
        Ok(_) => HttpResponse::Ok().body("user added"),
        Err(err) => {
            error!("Error: {}", err);
            HttpResponse::InternalServerError().body(err.to_string())
        }
    }
}

/// Gets the user with the supplied username.
#[get("/user/{username}")]
async fn get_user(cfg: web::Data<AppStates>, username: web::Path<String>) -> HttpResponse {
    let username = username.into_inner();
    let collection: Collection<User> = cfg.mongo_db.collection("users");
    match collection.find_one(doc! { "username": &username }).await {
        Ok(Some(user)) => HttpResponse::Ok().json(user),
        Ok(None) => {
            HttpResponse::NotFound().body(format!("No user found with username {username}"))
        }
        Err(err) => {
            error!("Error: {}", err);
            HttpResponse::InternalServerError().body(err.to_string())
        }
    }
}

#[get("/users")]
async fn get_users(
    cfg: web::Data<AppStates>,
    query: Option<web::Query<ListQuery>>
) -> impl Responder {
    let query = query.unwrap();
    let search = query.search.clone().unwrap_or_else(|| "".to_string());
    let per_page = query.per_page.unwrap_or(10);
    let page = query.page.unwrap_or(1);

    let limit = per_page.try_into().unwrap_or(constants::DEFAULT_LIMIT_SIZE);
    let offset = ((page - 1) * per_page).try_into().unwrap_or(constants::DEFAULT_OFFSET_SIZE);
    let collection: Collection<User> = cfg.mongo_db.collection("users");

    let mut cursor = collection
        .find(doc! { "username": {"$regex": search, "$options": "i"} })
        .limit(limit.try_into().unwrap())
        .skip(offset.try_into().unwrap()).await
        .expect("Failed to execute find.");

    let count = collection.count_documents(doc! {}).await.unwrap();

    let mut users: Vec<User> = Vec::new();
    while let Some(result) = cursor.try_next().await.expect("Failed to fetch next document.") {
        users.push(result);
    }

    let data = ResultData {
        data: users,
        total: count,
        page: page,
        per_page: per_page,
    };

    HttpResponse::Ok().json(data)
}

#[put("/user/{username}")]
async fn update_user(
    cfg: web::Data<AppStates>,
    username: web::Path<String>,
    json: web::Json<User>
) -> HttpResponse {
    let username = username.into_inner();
    if username.is_empty() {
        return HttpResponse::BadRequest().body("Invalid username");
    }
    let collection: Collection<User> = cfg.mongo_db.collection("users");
    let result = collection.find_one_and_update(
        doc! { "username": &username },
        doc! {"$set":{
                        "first_name": json.first_name.to_owned(),
                        "last_name": json.last_name.to_owned(),
                        "username": json.username.to_owned(),
                        "email": json.email.to_owned(),
                     }}
    ).await;

    match result {
        Ok(Some(_)) => HttpResponse::Ok().body(format!("success update user")),
        Ok(None) => HttpResponse::NotFound().body(format!("User {username} not found!")),
        Err(err) => {
            error!("Error: {}", err);
            HttpResponse::InternalServerError().body(err.to_string())
        }
    }
}
