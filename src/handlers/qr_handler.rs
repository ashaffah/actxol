use actix_http::StatusCode;
use actix_web::{ get, http::header::ContentType, post, web, HttpResponse, Result };
use qirust::helper::generate_svg_string;
use serde::Deserialize;

#[derive(Deserialize)]
struct Info {
    data: String,
}

// this handler gets called if the query deserializes into `Info` successfully
// otherwise a 400 Bad Request error response is returned
#[get("/qr")]
async fn generate_qr(info: web::Query<Info>) -> Result<HttpResponse> {
    let svg_string = generate_svg_string(&info.data);
    let response =
        format!("<body><div style='width: 500; height: 500; margin-left: auto; margin-right: auto;'>{}</div></body>", svg_string);
    Ok(HttpResponse::build(StatusCode::OK).content_type(ContentType::html()).body(response))
}

#[post("/svg")]
async fn get_svg(info: web::Json<Info>) -> Result<HttpResponse> {
    let svg_string = generate_svg_string(&info.data);

    Ok(HttpResponse::build(StatusCode::OK).content_type(ContentType::json()).body(svg_string))
}
