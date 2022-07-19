use std::collections::{HashMap};

use crate::db;
use crate::error::AppError;
use crate::models::{
    AppState, CreateTag, Questions, ResultResponse, ScrapedQuestion, Tag, TagQuestionRelation, TagQuestion,
};
use crate::scraper::{get_random_url, hacker_news};
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

pub async fn scrape_questions(pool:Pool,log:Logger) -> Result<(), AppError> {
    let sublog = log.new(o!("handler" => "scrape_questions"));
    let client: Client = configure_pool(pool.clone(), sublog.clone()).await?;
    let url = get_random_url(&log);
    let mut result = hacker_news(&log, &url, 10).await.unwrap();

    // IT will contains the count of occurence of tag
    let tags_hashmap = &mut result.unique_tags;
    let questions = &result.questions;

    // IT contains the tag id as value
    let mut index_table:HashMap<String, i32> = HashMap::new();
    
    for question in questions {

        // creating or getting question from database
        let question_id_res = db::create_or_skip(&client, &question).await;
        if let Ok(question_id_res) = question_id_res {
            let question_id = question_id_res.question_id;

            // iterating over all the tags in a question
            for tag in &question.tags {
                if tags_hashmap.get(tag) != Some(&0) {
                    (*tags_hashmap.entry(tag.clone()).or_insert(0)) -= 1;
                    let tag_id:i32;

                    // checking id of tag if it exists
                    if index_table.contains_key(tag){
                         match index_table.get(tag) {
                            Some(val)=> {tag_id = val.clone()},
                            None => {tag_id=-1}
                         }
                    } else {
                        // creating tag
                        let res = db::get_tag_id(&client, tag.clone()).await.unwrap();
                        tag_id = res.tag_id as i32;
                        index_table.insert(tag.to_owned(), tag_id);
                    }

                    // setting relationship btw question and tag
                    let tag_question = TagQuestion{
                        tag_id:i32::from(tag_id),
                        question_id
                    };
                    match db::create_tag_quest_rel(&client, &tag_question).await  {
                        Ok(_)=>println!("created relationship btw tag id {:?} and question id {:?}",tag_id,question_id),
                        Err(_) => println!("Failed"),
                    }
                }
            }
        }
    }
    Ok(())
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
