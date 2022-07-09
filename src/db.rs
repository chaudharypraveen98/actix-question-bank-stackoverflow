use crate::{
    error::{AppError, AppErrorType},
    models::{QuestionId, Questions, ScrapedQuestion, Tag, TagQuestion, TagQuestionRelation, TagId},
};
use deadpool_postgres::Client;
use tokio_pg_mapper::FromTokioPostgresRow;

pub async fn get_tags(client: &Client) -> Result<Vec<Tag>, AppError> {
    let statement = client.prepare("select * from tag limit 10;").await?;
    // .map_err(AppError::db_error)?;

    // We dont need to explicitly convert from Error to AppError. We had implemented the From trait for Postegress Error to AppError

    // query will take the query statement and a refrence to the list of parameters

    // once we received the rows as result, we want to use the model struct so we use the iterator and map each item to the tag list

    // to convert a row we need to import a trait from pg mapper, it provide row ref
    let tags = client
        .query(&statement, &[])
        .await
        .expect("Error getting tags")
        .iter()
        .map(|row| Tag::from_row_ref(row).unwrap())
        .collect::<Vec<Tag>>();

    Ok(tags)
}
pub async fn get_questions(client: &Client) -> Result<Vec<Questions>, AppError> {
    let statement = client.prepare("select * from question;").await?;
    let questions = client
        .query(&statement, &[])
        .await
        .expect("Error getting tags")
        .iter()
        .map(|row| Questions::from_row_ref(row).unwrap())
        .collect::<Vec<Questions>>();

    Ok(questions)
}

pub async fn get_related_question(
    client: &Client,
    tag_id: i32,
) -> Result<Vec<TagQuestionRelation>, AppError> {
    let statement = client
    .prepare("select qt.tag_id,qt.question_id, t.tag_title, q.title as q_title,q.q_description, q.question_link, q.votes,q.views,q.stack_id,q.answer from tag_question qt, tag t, question q
    where qt.tag_id = $1 and qt.question_id = q.question_id and qt.tag_id=t.tag_id;")
    .await
    ?;
    let questions = client
        .query(&statement, &[&tag_id])
        .await
        .expect("Error getting tags")
        .iter()
        .map(|row| TagQuestionRelation::from_row_ref(row).unwrap())
        .collect::<Vec<TagQuestionRelation>>();

    Ok(questions)
}

pub async fn create_tag(client: &Client, tag_title: String) -> Result<Tag, AppError> {
    let statement = client
        .prepare("insert into tag (tag_title) values ($1) returning tag_id, tag_title;")
        .await?;
    client
        .query(&statement, &[&tag_title])
        .await
        .expect("Error creating tag")
        .iter()
        .map(|row| Tag::from_row_ref(row).unwrap())
        .collect::<Vec<Tag>>()
        .pop()
        .ok_or(AppError {
            cause: Some("Unknown error".to_string()),
            message: Some("Error creating todolist".to_string()),
            error_type: AppErrorType::DbError,
        })
}

pub async fn update_tag(client: &Client, tag_id: i32, tag_title: String) -> Result<bool, AppError> {
    let statement = client
        .prepare("update tag set tag_title = $2 where tag_id=$1;")
        .await?;
    let result = client
        .execute(&statement, &[&tag_id, &tag_title])
        .await
        .expect("Error updating tag");
    match result {
        ref updated if *updated == 1 => Ok(true),
        _ => Ok(false),
    }
}

// It will create or get tag id
pub async fn get_tag_id(client: &Client, tag_name: String) -> Result<TagId, AppError> {
    let statement = client
        .prepare("with s as (select tag_id from tag where tag_title = $1), i as (insert into tag (tag_title) select $1 where not exists (select 1 from s) returning tag_id) select tag_id from i union all select tag_id from s;")
        .await?;
    println!("tag name {:?}",tag_name);

    client
        .query(&statement, &[&tag_name])
        .await
        .expect("Error creating tag")
        .iter()
        .map(|row| TagId::from_row_ref(row).unwrap())
        .collect::<Vec<TagId>>()
        .pop()
        .ok_or(AppError {
            cause: Some("Unknown error".to_string()),
            message: Some("Error creating todolist".to_string()),
            error_type: AppErrorType::DbError,
        })
}

pub async fn create_or_skip(
    client: &Client,
    question: &ScrapedQuestion,
) -> Result<QuestionId, AppError> {
    let statement = client
        .prepare(
            "insert into question (title,q_description,question_link,votes,stack_id,views,answer) values ($1,$2,$3,$4,$5,$6,$7) on conflict (stack_id) do nothing returning question_id")
        .await?;
    client
        .query(
            &statement,
            &[
                &question.title,
                &question.q_description,
                &question.question_link,
                &question.votes,
                &question.stack_id,
                &question.views,
                &question.answer,
            ],
        )
        .await
        .expect("Error creating tag")
        .iter()
        .map(|row| QuestionId::from_row_ref(row).unwrap())
        .collect::<Vec<QuestionId>>()
        .pop()
        .ok_or(AppError {
            cause: Some("Unknown error".to_string()),
            message: Some("Error creating todolist".to_string()),
            error_type: AppErrorType::DbError,
        })
}
pub async fn create_tag_quest_rel(
    client: &Client,
    question: &TagQuestion,
) -> Result<bool, AppError> {
    let statement = client
        .prepare("insert into tag_question (tag_id,question_id) values ($1,$2);")
        .await?;
    let result = client
        .execute(&statement, &[&question.tag_id, &question.question_id])
        .await
        .expect("Error updating tag");
    match result {
        ref updated if *updated == 1 => Ok(true),
        _ => Ok(false),
    }
}
