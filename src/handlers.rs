use crate::db;
use crate::models::Questions;
use crate::models::TagQuestionRelation;
use crate::models::{CreateTag, ResultResponse, Tag};
use actix_web::{web, HttpResponse, Responder};
use deadpool_postgres::{Client, Pool};
use sailfish::TemplateOnce;
use std::io::ErrorKind::Other;

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

// we receive the db pool which extracting from the application data and specify pool type
pub async fn get_tags(db_pool: web::Data<Pool>) -> impl Responder {
  let client: Client = db_pool
    .get()
    .await
    .expect("Error connecting to the database");

  let result = db::get_tags(&client).await;

  match result {
    Ok(tags) => {
      let ctx = TagsTemplate { tags_list: tags }.render_once().unwrap();
      HttpResponse::Ok().body(ctx)
    }
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
    Ok(questions) => {
      let ctx = QuestionTemplate {
        questions_list: questions,
      }
      .render_once()
      .unwrap();
      HttpResponse::Ok().body(ctx)
    }
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
    Ok(questions) => {
      let ctx = QuestionByIdTemplate {
        questions_list: questions,
      }
      .render_once()
      .unwrap();
      HttpResponse::Ok().body(ctx)
    }
    Err(_) => HttpResponse::InternalServerError().into(),
  }
}

// we use json extractor to extract data from body
// in the generics it contains the DTO(data transfer object) to exttract the values
pub async fn create_tag(db_pool: web::Data<Pool>, form: web::Form<CreateTag>) -> impl Responder {
  let client: Client = db_pool
    .get()
    .await
    .expect("Error connecting to the database");

  let result = db::create_tag(&client, form.tag_title.clone()).await;

  match result {
    Ok(tag) => {
      let ctx = CreateTagTemplate { tag: tag }.render_once().unwrap();
      HttpResponse::Ok().body(ctx)
    }
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
