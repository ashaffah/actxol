use actix_web::{ get, post, web, HttpResponse };
use mongodb::{ bson::doc, Collection };
use crate::{ configs::db::AppStates, models::user_model::User };

/// Adds a new user to the "users" collection in the database.
#[post("/add_user")]
async fn add_user(cfg: web::Data<AppStates>, json: web::Json<User>) -> HttpResponse {
    let collection: Collection<User> = cfg.mongo_db.collection("users");
    let result = collection.insert_one(json.into_inner()).await;
    match result {
        Ok(_) => HttpResponse::Ok().body("user added"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

/// Gets the user with the supplied username.
#[get("/get_user/{username}")]
async fn get_user(cfg: web::Data<AppStates>, username: web::Path<String>) -> HttpResponse {
    let username = username.into_inner();
    let collection: Collection<User> = cfg.mongo_db.collection("users");
    match collection.find_one(doc! { "username": &username }).await {
        Ok(Some(user)) => HttpResponse::Ok().json(user),
        Ok(None) => {
            HttpResponse::NotFound().body(format!("No user found with username {username}"))
        }
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}
