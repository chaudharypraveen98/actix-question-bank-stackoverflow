use crate::db;
use crate::error::AppError;
use crate::models::{AppState, CreateTag, Questions, ResultResponse, Tag, TagQuestionRelation};
use crate::scraper;
use actix_web::{web, HttpResponse, Responder};
use deadpool_postgres::{Client, Pool};
use sailfish::TemplateOnce;
use slog::{crit, info, o, Logger};

//  Templates Data
#[derive(TemplateOnce)]
#[template(path = "home.stpl")]
struct Home {}

#[derive(TemplateOnce)]
#[template(path = "tags.stpl")]
struct TagsTemplate {
    tags_list: Vec<Tag>,
}

#[derive(TemplateOnce)]
#[template(path = "questions.stpl")]
struct QuestionTemplate {
    questions_list: Vec<Questions>,
}

#[derive(TemplateOnce)]
#[template(path = "question_by_tag.stpl")]
struct QuestionByIdTemplate {
    questions_list: Vec<TagQuestionRelation>,
}

#[derive(TemplateOnce)]
#[template(path = "create_success.stpl")]
struct CreateTagTemplate {
    tag: Tag,
}

pub async fn home_page() -> impl Responder {
    HttpResponse::Ok().body(Home {}.render_once().unwrap())
}

async fn configure_pool(pool: Pool, log: Logger) -> Result<Client, AppError> {
    pool.get().await.map_err(|err| {
        let sublog = log.new(o!("cause"=>err.to_string()));
        crit!(sublog, "Error creating client");
        AppError::from(err)
    })
}

pub async fn get_tags(state: web::Data<AppState>) -> Result<impl Responder, AppError> {
    let sublog = state.log.new(o!("handler" => "get_tags"));
    let client: Client = configure_pool(state.pool.clone(), sublog.clone()).await?;

    let result = db::get_tags(&client).await;

    result.map(|tags| {
        let ctx = TagsTemplate { tags_list: tags }.render_once().unwrap();
        HttpResponse::Ok().body(ctx)
    })
}

pub async fn scrape_questions(state: web::Data<AppState>) -> impl Responder {
    info!(state.log, "check done");
    let url = scraper::get_random_url(&state.log);
    let result = scraper::hacker_news(&state.log, &url, 10).await;

    match result {
        Ok(questions) => HttpResponse::Ok().json(questions),
        Err(_) => HttpResponse::InternalServerError().into(),
    }
}

pub async fn get_questions(state: web::Data<AppState>) -> Result<impl Responder, AppError> {
    let sublog = state.log.new(o!("handler" => "get_questions"));
    let client: Client = configure_pool(state.pool.clone(), sublog.clone()).await?;

    let result = db::get_questions(&client).await;

    result.map(|questions| {
        let ctx = QuestionTemplate {
            questions_list: questions,
        }
        .render_once()
        .unwrap();
        HttpResponse::Ok().body(ctx)
    })
}

// we can use the actix web extracter to get the param
pub async fn get_questions_by_tag(
    state: web::Data<AppState>,
    path: web::Path<(i32,)>,
) -> Result<impl Responder, AppError> {
    let sublog = state.log.new(o!("handler" => "get_questions_by_tag"));
    let client: Client = configure_pool(state.pool.clone(), sublog.clone()).await?;

    let result = db::get_related_question(&client, path.0).await;

    result.map(|questions| {
        let ctx = QuestionByIdTemplate {
            questions_list: questions,
        }
        .render_once()
        .unwrap();
        HttpResponse::Ok().body(ctx)
    })
}

// we use json extractor to extract data from body
// in the generics it contains the DTO(data transfer object) to exttract the values
pub async fn create_tag(
    state: web::Data<AppState>,
    form: web::Form<CreateTag>,
) -> Result<impl Responder, AppError> {
    let sublog = state.log.new(o!("handler" => "create_tag"));
    let client: Client = configure_pool(state.pool.clone(), sublog.clone()).await?;

    let result = db::create_tag(&client, form.tag_title.clone()).await;

    result.map(|tag| {
        let ctx = CreateTagTemplate { tag: tag }.render_once().unwrap();
        HttpResponse::Ok().body(ctx)
    })
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
            message: "operation completed sucessfully".to_string(),
            success: updated,
        })
    })
}
