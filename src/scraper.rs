use crate::models::{ScrapedQuestion, ScraperResult};
use rand::seq::SliceRandom;
use regex::{Captures, Regex};
use select::document::Document;
use select::predicate::{Class, Name, Predicate};
use slog::{info, Logger};
use std::collections::{HashMap, HashSet};

#[warn(unused_imports, dead_code)]
fn views_count(views: &str) -> i32 {
    let re = Regex::new("(or|e)").unwrap();
    let result = re.replace_all(views, |cap: &Captures| {
        match &cap[0] {
            "or" => "str",
            "e" => "er",
            _ => panic!("We should never get here"),
        }
        .to_string()
    });
    result.into_owned().parse::<i32>().unwrap()
}

pub async fn hacker_news(
    log: &Logger,
    url: &str,
    count: usize,
) -> Result<ScraperResult, reqwest::Error> {
    info!(log, "check 1");
    let resp = reqwest::get(url).await?;
    // info!(log,"body = {:?}", resp.text().await?);
    // assert!(resp.status().is_success());
    let document = Document::from(&*resp.text().await?);

    let mut res = Vec::new();
    let mut tag_set = HashMap::new();

    for node in document.select(Class("s-post-summary")).take(count) {
        let question = node
            .select(Class("s-post-summary--content-excerpt"))
            .next()
            .unwrap()
            .text();
        let title_element = node
            .select(Class("s-post-summary--content-title").child(Name("a")))
            .next()
            .unwrap();
        let title = title_element.text();
        let question_link = title_element.attr("href").unwrap();
        let stats = node
            .select(Class("s-post-summary--stats-item-number"))
            .map(|stat| stat.text())
            .collect::<Vec<_>>();
        let votes = &stats[0];
        let answer = &stats[1];
        let views = &stats[2];
        let tags_vec:HashSet<String> = node
            .select(Class("post-tag"))
            .map(|tag| tag.text())
            .collect();
        for tag in &tags_vec {
            *tag_set.entry(tag.clone()).or_insert(0) += 1;
        }

        let post_id = node.attr("data-post-id").unwrap();
        // let mut tags=Vec::new();
        // let tag_element = node
        //     .select(Class("tags").child(Class("mt0")));
        // for tag in tag_element{
        //     let tag_text = tag.text();
        //     if !tags.contains(&tag_text){
        //         tags.push(tag_text);
        //     }
        // }
        let question = ScrapedQuestion {
            title,
            q_description: question,
            question_link: question_link.to_owned(),
            votes: votes.parse::<i32>().unwrap(),
            views: views.to_owned(),
            stack_id: post_id.parse::<i32>().unwrap(),
            tags:tags_vec,
            answer: answer.parse::<i32>().unwrap(),
        };
        res.push(question);
    }
    Ok(ScraperResult {
        questions: res,
        unique_tags: tag_set,
    })
}

// Getting random tag
pub fn get_random_url(log: &Logger) -> String {
    let default_tags = vec!["python", "rust", "c#", "android", "html", "javascript"];
    let random_tag = default_tags.choose(&mut rand::thread_rng()).unwrap();
    let url = format!(
        "https://stackoverflow.com/questions/tagged/{}?tab=Votes",
        random_tag
    );
    info!(log, "Url           => {}", &url);
    url.to_string()
}
