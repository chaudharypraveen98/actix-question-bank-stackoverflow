use crate::db;
use crate::models::Questions;
use crate::models::{CreateTag, ResultResponse, Tag};
use actix_web::{web, HttpResponse, Responder};
use deadpool_postgres::{Client, Pool};
use std::io::ErrorKind::Other;

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

// we can use the actix web extracter to get the param
pub async fn get_questions_by_tag(
  db_pool: web::Data<Pool>,
  path: web::Path<(i32,)>,
) -> impl Responder {
  let client: Client = db_pool
    .get()
    .await
    .expect("Error connecting to the database");

  let result = db::get_related_question(&client, path.0).await;

  match result {
    Ok(questions) => HttpResponse::Ok().json(questions),
    Err(_) => HttpResponse::InternalServerError().into(),
  }
}

// we use json extractor to extract data from body
// in the generics it contains the DTO(data transfer object) to exttract the values
pub async fn create_tag(db_pool: web::Data<Pool>, json: web::Json<CreateTag>) -> impl Responder {
  let client: Client = db_pool
    .get()
    .await
    .expect("Error connecting to the database");

  let result = db::create_tag(&client, json.tag_title.clone()).await;

  match result {
    Ok(tag) => HttpResponse::Ok().json(tag),
    Err(_) => HttpResponse::InternalServerError().into(),
  }
}

pub async fn update_tag(db_pool: web::Data<Pool>, json: web::Json<Tag>) -> impl Responder {
  let client: Client = db_pool
    .get()
    .await
    .expect("Error connecting to the database");

  let result = db::update_tag(&client, json.tag_id.clone(), json.tag_title.clone()).await;

  match result {
    Ok(()) => HttpResponse::Ok().json(ResultResponse {
      message: "updated sucessfully".to_string(),
      success: true,
    }),
    Err(ref e) if e.kind() == Other => HttpResponse::Ok().json(ResultResponse {
      message: "updated failed".to_string(),
      success: false,
    }),
    Err(_) => HttpResponse::InternalServerError().into(),
  }
}
