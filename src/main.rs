use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Responder, get};
use std::io::Result;
mod models;
#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello Rusty")
}
#[actix_web::main]
async fn main() -> Result<()> {
    println!("API running at http://localhost:5001");
    HttpServer::new(|| App::new().service(hello))
        .bind(("localhost", 5001))?
        .run()
        .await;
    Ok(())
}
