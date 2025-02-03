use actix_web::{web, get, App, HttpResponse, HttpServer, Responder};
use mime_guess::from_path;
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "out"]
struct Asset;

fn handle_embedded_file(path: &str) -> HttpResponse {
  match Asset::get(path) {
    Some(content) => HttpResponse::Ok()
      .content_type(from_path(path).first_or_octet_stream().as_ref())
      .body(content.data.into_owned()),
    None => HttpResponse::NotFound().body("490 Not Found"),
  }
}
#[actix_web::get("/")]
async fn index() -> impl Responder {
  handle_embedded_file("index.html")
}

#[get("/{path:.*}")]
async fn hello(path: web::Path<String>) -> impl Responder {
    handle_embedded_file(&path)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(hello)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
