use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use slog::Logger;
use tokio_pg_mapper_derive::PostgresMapper;
use validator::Validate;

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

#[derive(Serialize, Deserialize, PostgresMapper,Debug)]
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

#[derive(Validate, Deserialize)]
pub struct CreateTag {
    #[validate(length(min = 1))]
    pub tag_title: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateQuestion {
    pub title: String,
    pub q_description: String,
    pub question_link: String,
    pub votes: i32,
    pub views: String,
}

#[derive(Serialize)]
pub struct ResultResponse {
    pub message: String,
    pub success: bool,
}

pub struct AppState {
    pub pool: Pool,
    pub log: Logger,
}
