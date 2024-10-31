use std::env;

use actix_files::NamedFile;
use actix_http::StatusCode;
use actix_session::Session;
use actix_web::{ get, http::header::ContentType, HttpRequest, HttpResponse, Responder, Result };

/// favicon handler
#[get("/favicon")]
async fn favicon() -> Result<impl Responder> {
    Ok(NamedFile::open("static/favicon.ico")?)
}

/// simple index handler
#[get("/welcome")]
async fn welcome(req: HttpRequest, session: Session) -> Result<HttpResponse> {
    println!("{req:?}");

    // session
    let mut counter = 1;
    if let Some(count) = session.get::<i32>("counter")? {
        println!("SESSION value: {count}");
        counter = count + 1;
    }

    // set counter to session
    session.insert("counter", counter)?;
    let static_path = env::var("PATH_STATIC").expect("PATH_STATIC Not set");

    // response
    Ok(HttpResponse::build(StatusCode::OK).content_type(ContentType::html()).body(static_path))
}
