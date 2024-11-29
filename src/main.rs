use actix_web::{App, Error, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use actix_web::error::ErrorNotFound;

#[derive(Serialize, Deserialize)]
struct User {
    name: String,
}

#[derive(Serialize)]
struct CreateUserDTO {
    id: u32,
    name: String,
}

type UserDB = Arc<Mutex<HashMap<u32, User>>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = 8080;
    println!("Starting server on {port}");

    let user_db = Arc::new(Mutex::new(HashMap::<u32, User>::new()));

    HttpServer::new(move || {
        let app_data = actix_web::web::Data::new(user_db.clone());
        App::new().app_data(app_data)
            .service(greet)
            .service(create_user)
            .service(get_user)
    })
    .bind(("127.0.0.1", port))?
    .workers(2)
    .run()
    .await
}

#[actix_web::get("/greet/{id}")]
async fn greet(user_id: actix_web::web::Path<u32>) -> impl Responder {
    format!("Hello, {user_id}!")
}

#[actix_web::post("/create-user")]
async fn create_user(user_data: actix_web::web::Json<User>, db: actix_web::web::Data<UserDB>) -> impl Responder {
    let mut db = db.lock().unwrap();
    let new_id = db.keys().max().unwrap_or(&0) + 1;

    let name = user_data.name.clone();
    db.insert(new_id, user_data.into_inner());

    HttpResponse::Created().json(CreateUserDTO { name, id: new_id })
}

#[actix_web::get("/user/{id}")]
async fn get_user(
    user_id: actix_web::web::Path<u32>,
    db: actix_web::web::Data<UserDB>,
) -> Result<impl Responder, Error> {
    let user_id = user_id.into_inner();
    let db = db.lock().unwrap();

    match db.get(&user_id) {
        Some(user) => Ok(HttpResponse::Ok().json(user)),
        None => Err(ErrorNotFound("User not found")),
    }
}