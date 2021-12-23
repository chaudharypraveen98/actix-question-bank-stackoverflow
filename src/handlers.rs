use crate::models::Status;
use actix_web::{HttpResponse, Responder};

pub async fn manual_hello() -> impl Responder {
  HttpResponse::Ok().json(Status {
    status: "UP".to_string(),
  })
}
