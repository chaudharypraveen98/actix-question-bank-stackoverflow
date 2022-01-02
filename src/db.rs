use crate::models::{Questions, Tag, TagQuestionRelation};
use deadpool_postgres::Client;
use std::io;
use tokio_pg_mapper::FromTokioPostgresRow;

pub async fn get_tags(client: &Client) -> Result<Vec<Tag>, io::Error> {
  let statement = client.prepare("select * from tag limit 10;").await.unwrap();

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

pub async fn get_questions(client: &Client) -> Result<Vec<Questions>, io::Error> {
  let statement = client.prepare("select * from question;").await.unwrap();
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
) -> Result<Vec<TagQuestionRelation>, io::Error> {
  let statement = client
    .prepare("select qt.tag_id,qt.question_id, t.tag_title, q.title as q_title,q.q_description, q.question_link, q.votes,q.views from tag_question qt, tag t, question q
    where qt.tag_id = $1 and qt.question_id = q.question_id and qt.tag_id=t.tag_id;")
    .await
    .unwrap();
  let questions = client
    .query(&statement, &[&tag_id])
    .await
    .expect("Error getting tags")
    .iter()
    .map(|row| TagQuestionRelation::from_row_ref(row).unwrap())
    .collect::<Vec<TagQuestionRelation>>();

  Ok(questions)
}

pub async fn create_tag(client: &Client, tag_title: String) -> Result<Tag, io::Error> {
  let statement = client
    .prepare("insert into tag (tag_title) values ($1) returning tag_id, tag_title;")
    .await
    .unwrap();
  client
    .query(&statement, &[&tag_title])
    .await
    .expect("Error creating tag")
    .iter()
    .map(|row| Tag::from_row_ref(row).unwrap())
    .collect::<Vec<Tag>>()
    .pop()
    .ok_or(io::Error::new(io::ErrorKind::Other, "Error creating tag"))
}

pub async fn update_tag(client: &Client, tag_id: i32, tag_title: String) -> Result<(), io::Error> {
  let statement = client
    .prepare("update tag set tag_title = $2 where tag_id=$1;")
    .await
    .unwrap();
  let result = client
    .execute(&statement, &[&tag_id, &tag_title])
    .await
    .expect("Error updating tag");
  match result {
    ref updated if *updated == 1 => Ok(()),
    _ => Err(io::Error::new(
      io::ErrorKind::Other,
      "Failed to updated the title",
    )),
  }
}
