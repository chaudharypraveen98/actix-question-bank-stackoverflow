use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Serialize)]
pub struct Status {
  pub status: String,
}

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "question")]
pub struct Questions {
  pub question_id: i32,
  pub title: String,
  pub q_description: String,
  pub question_link: String,
  pub votes: i32,
  pub views: String,
}

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "tag")]
pub struct Tag {
  pub tag_id: i32,
  pub tag_title: String,
}

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "tag_question")]
pub struct TagQuestion {
  pub tag_id: i32,
  pub question_id: i32,
}

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "tag_question")]
pub struct TagQuestionRelation {
  pub tag_id: i32,
  pub question_id: i32,
  pub q_title: String,
  pub q_description: String,
  pub question_link: String,
  pub votes: i32,
  pub views: String,
  pub tag_title: String,
}

#[derive(Deserialize)]
pub struct CreateTag {
  pub tag_title: String,
}
#[derive(Serialize)]
pub struct ResultResponse {
  pub message: String,
  pub success: bool,
}
