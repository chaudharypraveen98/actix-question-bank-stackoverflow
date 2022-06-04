use crate::db;
use crate::error::AppError;
use crate::models::{AppState, CreateTag, ResultResponse, Tag};
use actix_web::{web, HttpResponse, Responder};
use deadpool_postgres::Client;

// we receive the db pool which extracting from the application data and specify pool type
pub async fn get_tags(state: web::Data<AppState>) -> Result<impl Responder, AppError> {
    let client: Client = state.pool.get().await.map_err(AppError::db_error)?;

    let result = db::get_tags(&client).await;

    result.map(|tags| HttpResponse::Ok().json(tags))
}

pub async fn get_questions(state: web::Data<AppState>) -> Result<impl Responder, AppError> {
    let client: Client = state.pool.get().await.map_err(AppError::db_error)?;

    let result = db::get_questions(&client).await;

    result.map(|questions| HttpResponse::Ok().json(questions))
}

// we can use the actix web extracter to get the param
pub async fn get_questions_by_tag(
    state: web::Data<AppState>,
    path: web::Path<(i32,)>,
) -> Result<impl Responder, AppError> {
    let client: Client = state.pool.get().await.map_err(AppError::db_error)?;

    let result = db::get_related_question(&client, path.0).await;

    result.map(|questions| HttpResponse::Ok().json(questions))
}

// we use json extractor to extract data from body
// in the generics it contains the DTO(data transfer object) to exttract the values
pub async fn create_tag(
    state: web::Data<AppState>,
    json: web::Json<CreateTag>,
) -> Result<impl Responder, AppError> {
    let client: Client = state.pool.get().await.map_err(AppError::db_error)?;

    let result = db::create_tag(&client, json.tag_title.clone()).await;

    result.map(|tag| HttpResponse::Ok().json(tag))
}

pub async fn update_tag(
    state: web::Data<AppState>,
    json: web::Json<Tag>,
) -> Result<impl Responder, AppError> {
    let client: Client = state.pool.get().await.map_err(AppError::db_error)?;

    let result = db::update_tag(&client, json.tag_id.clone(), json.tag_title.clone()).await;

    result.map(|updated| {
        HttpResponse::Ok().json(ResultResponse {
            message: "operation completed".to_string(),
            success: updated,
        })
    })
}
