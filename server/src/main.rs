#[macro_use]
extern crate actix_web;

use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use std::env;
use rand::prelude::*;
use std::fs;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

struct App {
    pub rgb_evidence: Option<PathBuf>,
}

impl App {
    fn new<P, R>(rgb: R) -> Self 
    where
        P: AsRef<Path>,
        R: Into<Option<P>>,
    {
        let rgb_evidence = rgb.into()
            .map(|v| v.as_ref().to_path_buf());
            
        Self {
            rgb_evidence
        }
    }    
}

#[get("/")]
async fn index() -> HttpResponse {
    HttpResponse::Ok().body("Hello")    
}

#[get("/rgb")]
async fn rgb(
    data: web::Data<App>,
    rng: web::Data<ThreadRng>
) -> HttpResponse {
    data.rgb_evidence
        .and_then(|path| {
            fs::read_dir(path)
                .filter_map(|v| v.ok().map(|x| !x.is_dir()))
                .choose(&mut rng)
        })
        .map_or_else(
            || HttpResponse::NotFound().finish(),
            |e| HttpResponse::Ok().body(format!(r#"<img src="/rgb/{}">"#, e.file_name());)
        )
    }
}

#[actix_rt::main]
async fn main() -> Result<()> {
    let ip = env::var("IP").unwrap_or(String::from("127.0.0.1:8080"));
    let mut rng = ThreadRng::new();
    let rng_ref = wed::data::new(rng);
    
    let rgb_path = env::var("RGB_EVIDENCE");
    let my_app = App::new(&rgb_path);
    
    HttpServer::new(move || {
        let mut app = App::new()
            .data(my_app)
            .app_data(rng_ref)
            .service(index);
        
        if let Some(path) = rgb_path {
            app = app.service(actix_files::Files::new("/rgb", path));
        }
            
        app
    })
    .bind(ip)?
    .start()
    .await
}