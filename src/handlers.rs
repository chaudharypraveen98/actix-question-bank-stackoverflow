use crate::db;
use crate::models::Status;
use actix_web::{web, HttpResponse, Responder};
use deadpool_postgres::{Client, Pool};

pub async fn manual_hello() -> impl Responder {
  HttpResponse::Ok().json(Status {
    status: "UP".to_string(),
  })
}

// we receive the db pool which extracting from the application data and specify pool type
pub async fn get_tags(db_pool: web::Data<Pool>) -> impl Responder {
  let client: Client = db_pool
    .get()
    .await
    .expect("Error connecting to the database");

  let result = db::get_tags(&client).await;

  match result {
    Ok(tags) => HttpResponse::Ok().json(tags),
    Err(_) => HttpResponse::InternalServerError().into(),
  }
}

pub async fn get_questions(db_pool: web::Data<Pool>) -> impl Responder {
  let client: Client = db_pool
    .get()
    .await
    .expect("Error connecting to the database");

  let result = db::get_questions(&client).await;

  match result {
    Ok(questions) => HttpResponse::Ok().json(questions),
    Err(_) => HttpResponse::InternalServerError().into(),
  }
}
