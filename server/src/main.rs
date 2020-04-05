#[macro_use]
extern crate actix_web;

use actix_web::{App, HttpResponse, HttpServer};
use std::env;

#[get("/")]
async fn index() -> HttpResponse {
    HttpResponse::Ok().body("Hello wold!!")
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let ip = env::var("IP").unwrap_or(String::from("127.0.0.1:8080"));
    dbg!(&ip);

    HttpServer::new(move || {
        let path = env::var("DOC").unwrap();
        App::new()
            .service(index)
            .service(actix_files::Files::new("/doc", path))
    })
    .bind(ip)?
    .run()
    .await
}
