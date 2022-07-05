use crate::db;
use crate::error::AppError;
use crate::models::{AppState, CreateTag, ResultResponse, Tag};
use actix_web::{web, HttpResponse, Responder};
use deadpool_postgres::{Client, Pool};
use slog::{crit, info, o, Logger};
use validator::Validate;

async fn configure_pool(pool: Pool, log: Logger) -> Result<Client, AppError> {
    pool.get().await.map_err(|err| {
        // Creating a child logger
        let sublog = log.new(o!("cause"=>err.to_string()));
        crit!(sublog, "Error creating client");
        AppError::from(err)
    })
}

// we receive the db pool which extracting from the application data and specify pool type
pub async fn get_tags(state: web::Data<AppState>) -> Result<impl Responder, AppError> {
    // METHOD 1 is to explicitly convert the Deadpool Error to App Error in the handler Like the below

    // let client: Client = state.pool.get().await.map_err(|err|  AppError {
    //     cause: None,
    //     message: Some(err.to_string()),
    //     error_type: AppErrorType::DbError,
    // })?;

    // METHOD 2 is to implement a db_error trait which can shared between handlers like.The `?` operator transparently invokes the `Into` trait

    //let client: Client = state.pool.get().await.map_err(AppError::db_error)?;

    // METHOD 3 is to implement a From trait to convert from one type to another
    // The From and Into traits are inherently linked, used for converting between several types but Using the Into trait will typically require specification of the type to convert into as the compiler is unable to determine this most of the time.
    let sublog = state.log.new(o!("handler" => "get_tags"));
    let client: Client = configure_pool(state.pool.clone(), sublog.clone()).await?;

    let result = db::get_tags(&client).await;

    result.map(|tags| HttpResponse::Ok().json(tags))
}

pub async fn get_questions(state: web::Data<AppState>) -> Result<impl Responder, AppError> {
    let sublog = state.log.new(o!("handler" => "get_questions"));
    let client: Client = configure_pool(state.pool.clone(), sublog.clone()).await?;

    let result = db::get_questions(&client).await;

    result.map(|questions| HttpResponse::Ok().json(questions))
}

// we can use the actix web extracter to get the param
pub async fn get_questions_by_tag(
    state: web::Data<AppState>,
    path: web::Path<(i32,)>,
) -> Result<impl Responder, AppError> {
    let sublog = state.log.new(o!("handler" => "get_questions_by_tag"));
    let client: Client = configure_pool(state.pool.clone(), sublog.clone()).await?;

    let result = db::get_related_question(&client, path.0).await;

    result.map(|questions| HttpResponse::Ok().json(questions))
}

// we use json extractor to extract data from body
// in the generics it contains the DTO(data transfer object) to exttract the values
pub async fn create_tag(
    state: web::Data<AppState>,
    json: web::Json<CreateTag>,
) -> Result<impl Responder, AppError> {
    let sublog = state.log.new(o!("handler" => "create_tag"));

    let client: Client = configure_pool(state.pool.clone(), sublog.clone()).await?;
    let is_valid = json.validate().map_err(|err| AppError::from(err));
    match is_valid {
        Ok(_) => {
            let result = db::create_tag(&client, json.tag_title.clone()).await;
            info!(sublog, "{:?}", result);
            result.map(|tag| HttpResponse::Ok().json(tag))
        }
        Err(err) => {
            crit!(sublog, "{:?}", err);
            Err(err)
        },
    }
}

pub async fn update_tag(
    state: web::Data<AppState>,
    json: web::Json<Tag>,
) -> Result<impl Responder, AppError> {
    let sublog = state.log.new(o!("handler" => "update_tag"));
    let client: Client = configure_pool(state.pool.clone(), sublog.clone()).await?;

    let result = db::update_tag(&client, json.tag_id.clone(), json.tag_title.clone()).await;

    result.map(|updated| {
        HttpResponse::Ok().json(ResultResponse {
            message: "operation completed".to_string(),
            success: updated,
        })
    })
}
