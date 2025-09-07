use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Responder, get, web::Data};
use std::io::Result;

use crate::{
    routes::{
        booking_routes::{cancel_booking, create_booking, get_bookings},
        dog_routes::create_dog,
        owner_routes::create_owner,
    },
    services::db::Database,
};
mod models;
mod routes;
mod services;
#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello Rusty")
}
#[actix_web::main]
async fn main() -> Result<()> {
    let db = Database::init().await;
    let db_data = Data::new(db);

    println!("API running at http://127.0.0.1:5001");
    HttpServer::new(move || {
        App::new()
            .app_data(db_data.clone())
            .service(hello)
            .service(create_owner)
            .service(create_dog)
            .service(create_booking)
            .service(get_bookings)
            .service(cancel_booking)
    })
    .bind(("127.0.0.1", 5001))?
    .run()
    .await
}
