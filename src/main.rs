use std::sync::Mutex;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};

struct AppState {
    app_name: String,
    counter: Mutex<i32>,
}

#[get("/")]
async fn hello(data: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().body(format!(
        "Hello from {}! Counter: {}",
        data.app_name,
        data.counter.lock().unwrap()
    ))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = web::Data::new(AppState {
        app_name: "Actix-web".to_string(),
        counter: Mutex::new(0),
    });

    HttpServer::new(move || App::new().app_data(app_state.clone()).service(hello))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
