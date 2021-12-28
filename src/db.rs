use crate::models::{Questions, Tag};
use deadpool_postgres::Client;
use std::io;
use tokio_pg_mapper::FromTokioPostgresRow;

pub async fn get_tags(client: &Client) -> Result<Vec<Tag>, io::Error> {
  let statement = client.prepare("select * from tag;").await.unwrap();

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
