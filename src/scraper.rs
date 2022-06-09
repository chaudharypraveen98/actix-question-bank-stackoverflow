use rand::seq::SliceRandom;
use select::document::Document;
use slog::{Logger, info};
use crate::models::CreateQuestion;
use select::predicate::{Class, Name, Predicate};
use regex::{Captures, Regex};

#[warn(unused_imports,dead_code)]
fn views_count(views:&str)->i32{
    let re = Regex::new("(or|e)").unwrap();
    let result = re.replace_all(views, |cap: &Captures| {
        match &cap[0] {
            "or" => "str",
            "e" => "er",
            _ => panic!("We should never get here"),
        }.to_string()
    });
    result.into_owned().parse::<i32>().unwrap()
}

pub async fn hacker_news(log:&Logger,url: &str, count: usize) -> Result<Vec<CreateQuestion>, reqwest::Error> {
    info!(log,"check 1");
    let resp = reqwest::get(url).await?;
    // info!(log,"body = {:?}", resp.text().await?);
    // assert!(resp.status().is_success());
    let document = Document::from(&*resp.text().await?);
    
    let mut res=Vec::new();

    for node in document.select(Class("s-post-summary")).take(count) {
        let question = node.select(Class("s-post-summary--content-excerpt")).next().unwrap().text();
        let title_element = node.select(Class("s-post-summary--content-title").child(Name("a"))).next().unwrap();
        let title = title_element.text();
        let question_link = title_element.attr("href").unwrap();
        let stats = node
        .select(Class("s-post-summary--stats-item-number"))
        .map(|stat| stat.text())
        .collect::<Vec<_>>();
        let votes = &stats[0];
        let answer = &stats[1];
        let views = &stats[2];
        let tags = node
            .select(Class("post-tag"))
            .map(|tag| tag.text())
            .collect::<Vec<_>>();
        // let mut tags=Vec::new();
        // let tag_element = node
        //     .select(Class("tags").child(Class("mt0")));
        // for tag in tag_element{
        //     let tag_text = tag.text();
        //     if !tags.contains(&tag_text){
        //         tags.push(tag_text);
        //     }
        // }        
        info!(log,"Tags           => {:?}", tags);
        info!(log,"Answers        => {}", answer);
        let question = CreateQuestion{
            title:title,
            q_description:question,
            question_link:question_link.to_owned(),
            votes:votes.parse::<i32>().unwrap(),
            views:views.to_owned()
        };
        res.push(question);
    }
    Ok(res)
}

// Getting random tag
pub fn get_random_url(log:&Logger) -> String {
    let default_tags = vec!["python", "rust", "c#", "android", "html", "javascript"];
    let random_tag = default_tags.choose(&mut rand::thread_rng()).unwrap();
    let url = format!(
        "https://stackoverflow.com/questions/tagged/{}?tab=Votes",
        random_tag
    );
    info!(log,"Url           => {}", &url);
    url.to_string()
}