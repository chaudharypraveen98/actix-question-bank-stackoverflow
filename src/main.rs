mod config;
mod handlers;
mod models;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

use crate::handlers::*;
use deadpool_postgres::Runtime;
use dotenv::dotenv;
use tokio_postgres::NoTls;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let config = crate::config::Config::from_env().unwrap();
    let pool = config.pg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();

    println!(
        "Starting the server at http://{}:{}/",
        config.server.host, config.server.port
    );

    // we need to pass the ownership so we use the move
    // AS the web server make instance for each thread to we need to pass the pool

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(format!("{}:{}", config.server.host, config.server.port))?
    .run()
    .await
}

// sudo service postgresql stop
// sudo update-rc.d postgresql disable
// sudo docker-compose up -d
// sudo psql -h 127.0.0.1 -p 5432 -U actix actix < database.sql
