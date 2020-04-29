#[macro_use]
extern crate actix_web;

#[macro_use]
extern crate serde;

use actix_web::{web, App, HttpResponse, HttpServer};
use db::DbInstance;
use smallstr::SmallString;
use std::env;

#[get("/")]
async fn index() -> HttpResponse {
    HttpResponse::Ok().body("Hello wold!!")
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct CoronaSummary {
    global: Global,
    countries: Vec<Country>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Global {
    new_confirmed: i64,
    total_confirmed: u64,
    new_deaths: i64,
    total_deaths: u64,
    new_recovered: i64,
    total_recovered: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Country {
    country: String,
    country_code: SmallString<[u8; 2]>,
    slug: String,
    new_confirmed: i64,
    total_confirmed: u64,
    new_deaths: i64,
    total_deaths: u64,
    new_recovered: i64,
    total_recovered: u64,
    date: String,
}

#[post("/webhook/corona")]
async fn corona_update(
    data: web::Bytes,
    db: web::Data<DbInstance>,
) -> Result<HttpResponse, actix_web::Error> {
    dbg!("Got some");

    if let Ok(json) = serde_json::from_slice::<CoronaSummary>(&data) {
        db.put_json("corona", &json)?;
        dbg!("Added data to the database");
    } else {
        dbg!(data);
    }
    Ok(HttpResponse::Ok().body("Sucessfully cached values"))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let ip = env::var("IP").unwrap_or(String::from("127.0.0.1:8080"));
    dbg!(&ip);

    HttpServer::new(move || {
        let path = env::var("DOC").unwrap();
        let db = db::DbInstance::new("./tomodb", None).unwrap();
        App::new()
            .data(db)
            .service(index)
            .service(corona_update)
            .service(actix_files::Files::new("/doc", path))
    })
    .bind(ip)?
    .run()
    .await
}
