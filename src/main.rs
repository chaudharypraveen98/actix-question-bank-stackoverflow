extern crate regex;
extern crate reqwest;
extern crate select;
extern crate validator;

mod api_handlers;
mod config;
mod db;
mod error;
mod handlers;
mod models;
mod scraper;
// mod scheduler;

use std::str::FromStr;
use std::time::Duration;

use crate::api_handlers as api;
use crate::handlers::*;
use crate::models::AppState;
// use crate::scheduler::Scheduler;
// use actix::Actor;
use actix_files as fs;
use actix_rt::time;
use actix_web::{web, App, HttpServer};

use chrono::FixedOffset;
use chrono::Local;
use cron::Schedule;
use deadpool_postgres::Runtime;
use dotenv::dotenv;
use tokio_postgres::NoTls;

// IT is used as a logging middleware. We can even use the default logger with actix. keyword fuse is used to painck
use slog;
use slog::{info, o, Drain, Logger};
use slog_async;
use slog_term;

fn configure_log() -> Logger {
    let decorator = slog_term::TermDecorator::new().build();
    let console_drain = slog_term::FullFormat::new(decorator).build().fuse();
    let console_drain = slog_async::Async::new(console_drain).build().fuse();
    slog::Logger::root(console_drain, o!("v"=>env!("CARGO_PKG_VERSION")))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let config = crate::config::Config::from_env().unwrap();
    let pool = config.pg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();

    let log = configure_log();

    info!(
        log,
        "Starting the server at http://{}:{}/", config.server.host, config.server.port
    );
    // let scheduler_obj = Scheduler {pool:pool.clone(),log:log.clone()};
    // Scheduler::start(scheduler_obj);
    
    // actix_rt::spawn(async move {
    //     let mut interval = time::interval(Duration::from_secs(120));
    //     let new_pool = config.pg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();

    //     let new_log = configure_log();
    //     loop {
    //         interval.tick().await;
    //         println!("120 seconds");
    //         scrape_questions(new_pool.clone(), new_log.clone()).await.unwrap();
    //     }
    // });

    actix_rt::spawn(async move {
        let expression = "1/10   *   *     *       *  *  *";
        let schedule = Schedule::from_str(expression).unwrap();
        let offset  = Some(FixedOffset::east(0)).unwrap();
        // let new_pool = config.pg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();
        // let new_log = configure_log();

        loop {
            let mut upcoming = schedule.upcoming(offset).take(1);
            actix_rt::time::sleep(Duration::from_millis(500)).await;
            let local = &Local::now();

            if let Some(datetime) = upcoming.next() {
                if datetime.timestamp() <= local.timestamp() {
                    println!("120 seconds");
                }
            }
            
            // scrape_questions(new_pool.clone(), new_log.clone())
            //     .await
            //     .unwrap();
        }
    });

    info!(log, "Testing");

    // we need to pass the ownership so we use the move
    // AS the web server make instance for each thread to we need to pass the pool

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                pool: pool.clone(),
                log: log.clone(),
            }))
            .service(fs::Files::new("/static", "./static").show_files_listing())
            .route("/", web::get().to(home_page))
            // .route("/scrape{_:/?}", web::get().to(scrape_questions))
            .route("/tags{_:/?}", web::get().to(get_tags))
            .route("/tags{_:/?}", web::post().to(create_tag))
            .route("/tags/update/{tag_id}{_:/?}", web::post().to(update_tag))
            .route("/questions{_:/?}", web::get().to(get_questions))
            .route(
                "/questions/{tag_id}{_:/?}",
                web::get().to(get_questions_by_tag),
            )
            .route("/api/tags{_:/?}", web::put().to(api::update_tag))
            .route("/api/tags{_:/?}", web::get().to(api::get_tags))
            .route("/api/tags{_:/?}", web::post().to(api::create_tag))
            .route("/api/questions{_:/?}", web::get().to(api::get_questions))
            .route(
                "/api/questions/{tag_id}{_:/?}",
                web::get().to(api::get_questions_by_tag),
            )
    })
    .bind(format!("{}:{}", config.server.host, config.server.port))?
    .run()
    .await
}

// sudo service postgresql stop
// sudo update-rc.d postgresql disable
// sudo docker-compose up -d
// sudo psql -h 127.0.0.1 -p 5432 -U actix actix < database.sql
