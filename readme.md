## Actix Question Bank Stackoverflow
This api serves the purpose of creating, updating, and reading the questions and tags from the database. We will soon integrate the stackoverflow scraper.

#### How to run : 
1. Set up Postgres database by installing postgress docker image and change the docker-compose.yaml with the database version type. I have used the 12.9 apline version. Docker must be present in your system <a href="https://www.digitalocean.com/community/tutorials/how-to-install-postgresql-on-ubuntu-20-04-quickstart">More Detail</a>
2. Run the database initiate file by following command
   ```sudo psql -h 127.0.0.1 -p 5432 -U actix actix < database.sql```
3. Run the database by `sudo docker-compose up -d`. Be sure to stop the docker after use by using `docker ps` to get the container id then `docker stop <container_id>` to stop the database instance.
4. Run the server by following `cargo run`

#### Database Access
use the following command by `sudo psql -h 127.0.0.1 -p 5432 -U actix actix` .password is actix. You can configure it in **docker-compose.yaml**

#### Api Endpoints :
1. Hello world : GET REQUEST `http://127.0.0.1:8000/`
2. Get Tags :  GET REQUEST `http://127.0.0.1:8000/tags/`
3. Get Questions : GET REQUEST `http://127.0.0.1:8000/questions/`
4. Get Questions By Tag : 
  * Api structure : GET REQUEST `http://127.0.0.1:8000/questions/<tag_id/>`
  * Sample Api Endpoint : GET REQUEST `http://127.0.0.1:8000/questions/2/`
5. Create Tag : POST REQUEST
  * Api endpoint : `http://127.0.0.1:8000/tags/`
  * Sample Body : 
  * ```{    "tag_title":"c++"    }```
  * Note : Content-Type must be **application/json** in request header
6. Update Tag : PUT REQUEST
   * Api endpoint : `http://127.0.0.1:8000/tags/`
   * Sample body
   * ```{    "tag_title":"golang",    "tag_id":3}```

#### Templating
We have used the <a href="https://crates.io/crates/sailfish">Sailfish</a> templating engine (Simple, small, and extremely fast template engine for Rust).

#### Error Handling
Errors are the part of software development. There are basically two types of errors in Rust i.e recoverable and unrecoverable. 

**Recoverable errors** have type `Result<T, E>` and **Unrecoverable errors** have `panic!` macro that stops execution.

Errors serve two main purposes:
* Control flow (i.e. determine what do next);
* Reporting (e.g. investigate, after the fact, what went wrong on).


We can also distinguish errors based on their location:
* Internal (i.e. a function calling another function within our application);
* At the edge (i.e. an API request that we failed to fulfill).

|              |        Internal        | At the edge   |
|:------------:|:----------------------:|---------------|
| Control Flow | Types, methods, fields |  Status codes |
|   Reporting  |       Logs/traces      | Response body |


You can refer more <a href="https://www.lpalmieri.com/posts/error-handling-rust/">here</a>

The first place where we are likely to get an error is `pool.get()` . When we are unable to get the pool instance whatever will be the reason like wrong credentials, database instance not running etc.

Lets try how to handle it.

## Using Unwrap to create panic.
Creating Panic is usefull during the prototyping phase when we are more focused on logic implementation. 

<a href="https://github.com/chaudharypraveen98/actix-question-bank-stackoverflow/blob/master/src/api_handlers.rs">'/src/api_handlers.rs'</a>

```
pub async fn get_tags(state: web::Data<AppState>) ->impl Responder  {
    // Just not handle error and let the system to Panic (unrecoverable error)
    let client = state.pool
        .get()
        .await.unwrap();
    let result = db::get_tags_unwrap(&client).await;

    match result {
        Ok(tags) => HttpResponse::Ok().json(tags),
        Err(_) => HttpResponse::InternalServerError().into(),
    }
}
```

Calling unwrap will create panic and the execution stops. So, we have discovered that handling errors like this is not ideal in the production server.

Lets move to another way

## Using the Result<T, E>

### Handling Single Error

The caller of the execute must be aware whether the program completed successfully or failed. For this use case, we can use simple `ResultSignal` enum

<a href="https://github.com/chaudharypraveen98/actix-question-bank-stackoverflow/blob/master/src/error.rs">'./src/error.rs'</a>
```
pub enum ResultSignal<Success> {
    Ok(Success),
    Err
}
```

It will return Ok status on success and Error on failure. It is helpful now that our user is aware that something mishappend has occured. It is suitable for a single kind of error but our system consists of different services and they can fail in different ways.

**what is the reason? where has failure occured?**
To answer this, we need to handle various errors according to our need like database error, filter errors etc

### Handling Multiple Errors

Lets create a enum for different types of errors. For the sake of simplicity, we just considereing two cases only Db Error and Not Found Error

<a href="https://github.com/chaudharypraveen98/actix-question-bank-stackoverflow/blob/master/src/error.rs">'./src/error.rs'</a>
```
pub enum AppErrorType {
    DbError,
    NotFoundError,
}
```

Then we need to implement the **Debug and Display** trait. It is used to print errors using *println* command.

```
#[derive(Debug)]
pub enum AppErrorType {
    DbError,
    NotFoundError,
}

impl fmt::Display for AppErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
```

* **#[derive(Debug)]** :  This macro will enable the Debug trait for the enum AppErrorType.

* **fn fmt** : Display is for user-facing output.
* **write!** and **writeln!** are two macros which are used to emit the format string to a specified stream. This is used to prevent intermediate allocations of format strings and instead directly write the output.

Let's implement the **From** trait to convert from one type to another like PoolError to AppErrorType.

```
impl From<PoolError> for AppErrorType {
    fn from(_error: PoolError) -> AppErrorType {
        AppErrorType::DbError
    }
}
impl From<Error> for AppErrorType {
    fn from(_error: Error) -> AppErrorType {
        AppErrorType::DbError
    }
}
```

Since, we will be using Result<HttpResponse,AppErrorType>  return type in api handlers then we need overwrite the **ResponseError** Trait.

Actix provide two methods **error_response** and **status_code** to handle errors response.

```
impl ResponseError for AppErrorType {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).finish()
    }
}
```

Then, we need to change the return type from **impl Responder** or **HttpResponse** to **Result<HttpResponse,AppErrorType>**. 

We have used the `?` trait already implemented above in error.rs instead to unwrap.

<a href="https://github.com/chaudharypraveen98/actix-question-bank-stackoverflow/blob/master/src/api_handlers.rs">'./src/api_handlers.rs'</a>
```
pub async fn get_tags(state: web::Data<AppState>) ->Result<HttpResponse,AppErrorType> {
    let client: Client = state.pool.get().await?;
    let result = db::get_tags(&client).await;
    result.map(|tags| HttpResponse::Ok().json(tags))
}
```

Instead of using unwrap in `client.prepare("select * from tag limit 10;").await.unwrap()`, we can now use the `?` as we have implemented the From trait and update the return type too `Result<Vec<Tag>, AppErrorType>` 

<a href="https://github.com/chaudharypraveen98/actix-question-bank-stackoverflow/blob/master/src/db.rs">'./src/db.rs'</a>

```
pub async fn get_tags(client: &Client) -> Result<Vec<Tag>, AppErrorType> {
    let statement = client.prepare("select * from tag limit 10;").await?;
    let tags = client
        .query(&statement, &[])
        .await
        .expect("Error getting tags")
        .iter()
        .map(|row| Tag::from_row_ref(row).unwrap())
        .collect::<Vec<Tag>>();

    Ok(tags)
}
```

Lets spin our server and hit the endpoint. 

![Returned Response](https://dev-to-uploads.s3.amazonaws.com/uploads/articles/lf422vtpmkv6z2te1y6d.png)
The request returned a `500` status code. Instead of just panicking we are getting a status code which will help to debug.

#### But is only status code really helpful??
Not, not at all, A good error which contains cause of error, error status code and a message for the client user which is human readable

#### Let's try to improve our error handling
Let's implement AppError struct which contains our three fields cause, error type and message.

<a href="https://github.com/chaudharypraveen98/actix-question-bank-stackoverflow/blob/master/src/error.rs">'./src/error.rs'</a>

```
pub struct AppError {
    pub cause: Option<String>,
    pub message: Option<String>,
    pub error_type: AppErrorType,
}
```

Just like AppErrorType, Let's implement Debug and Display trait
```
#[derive(Debug)]
pub struct AppError {
    pub cause: Option<String>,
    pub message: Option<String>,
    pub error_type: AppErrorType,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
```

Once we are done display and debug trait, let's define ResponseError for AppError
```
impl ResponseError for AppError {

    fn status_code(&self) -> StatusCode {
        match self.error_type {
            AppErrorType::DbError => (StatusCode::INTERNAL_SERVER_ERROR),
            AppErrorType::NotFoundError => (StatusCode::NOT_FOUND),
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(AppErrorResponse {
            error: self.message(),
        })
    }
}
```

In the above code, we have used `status_code` to match different errors and provide status code according to it.

As soon the ResponseError is define, We use From trait for error type conversion
```
impl From<PoolError> for AppError {
    fn from(error: PoolError) -> AppError {
        AppError {
            message: None,
            cause: Some(error.to_string()),
            error_type: AppErrorType::DbError,
        }
    }
}
impl From<Error> for AppError {
    fn from(error: Error) -> AppError {
        AppError {
            message: None,
            cause: Some(error.to_string()),
            error_type: AppErrorType::DbError,
        }
    }
}
```

Let's implement default message for the error types.
```
impl AppError {
    // we are handling the none. function name should match field name
    fn message(&self) -> String {
        match &*self {
            // Error message is found then clone otherwise default message
            AppError {
                cause: _,
                message: Some(message),
                error_type: _,
            } => message.clone(),
            AppError {
                cause: _,
                message: None,
                error_type: AppErrorType::NotFoundError,
            } => "The requested item was not found".to_string(),
            _ => "An unexpected error has occured".to_string(),
        }
    }
}
```

We are done with all the necessary changes in error.rs file. Let's start with api handlers and db handlers

<a href="https://github.com/chaudharypraveen98/actix-question-bank-stackoverflow/blob/master/src/api_handlers.rs">'./src/api_handlers.rs'</a>

```
pub async fn get_tags(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let client: Client = state.pool.get().await?;
    let result = db::get_tags(&client).await;
    result.map(|tags| HttpResponse::Ok().json(tags))
}
```

Make sure to enable logger to get vivid description. You can use the **actix default logger** or the **slog logger**. You can read more about slog <a href="https://dev.to/chaudharypraveen98/adding-slog-logger-to-actix-web-2332" target="_blank">here.</a>

```
async fn configure_pool(pool: Pool, log: Logger) -> Result<Client, AppError> {
    pool.get().await.map_err(|err| {
        let sublog = log.new(o!("cause"=>err.to_string()));
        crit!(sublog, "Error creating client");
        AppError::from(err)
    })
}


pub async fn get_tags(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let sublog = state.log.new(o!("handler" => "get_tags"));
    let client: Client = configure_pool(state.pool.clone(), sublog.clone()).await?;
    let result = db::get_tags(&client).await;
    result.map(|tags| HttpResponse::Ok().json(tags))
}
```

Let's change the Db handler file.

<a href="https://github.com/chaudharypraveen98/actix-question-bank-stackoverflow/blob/master/src/db.rs">'./src/db.rs'</a>

```
pub async fn get_tags(client: &Client) -> Result<Vec<Tag>, AppError> {
    let statement = client.prepare("select * from tag limit 10;").await?;
    let tags = client
        .query(&statement, &[])
        .await
        .expect("Error getting tags")
        .iter()
        .map(|row| Tag::from_row_ref(row).unwrap())
        .collect::<Vec<Tag>>();

    Ok(tags)
}
```

Let's start our server and hit the api endpoint.

**Client side error**

<img src="./error%20client.png" alt="client side error">


**Server side error**

<img src="./error%20terminal.png" alt="terminal error">


Wow!! We have now awesome logs and error message for the client user.

#### Logging
We will learn how to use slog logger for logging in Actix web.
<a href="https://actix.rs/">Actix web</a> is a powerful, pragmatic, and extremely fast web framework for Rust and <a href="https://docs.rs/slog/2.7.0/slog/">Slog</a> is an ecosystem of reusable components for structured, extensible, composable logging for Rust. We will be using two crates of slog : `slog-async` and `slog-term` with the core Slog Core Package.

### Why Slog over default log crate?
* extensible
* composable
* flexible
* structured and both human and machine-readable
* contextual

### Crates used
* <a href="https://actix.rs/">Actix Web</a> : powerful web framework.
* <a href="https://crates.io/crates/slog">Slog Core Crate</a> : core package to the gateway of logging modules.
* <a href="https://crates.io/crates/slog-term">Slog Term</a> : Unix terminal drain and formatter for slog-rs.
* <a href="https://crates.io/crates/slog-term">Slog Term</a> : Asynchronous drain for slog-rs.

### Crated Version and Code
Simply paste the code in the `cargo.toml` file
```
slog = "2.7.0"
slog-term = "2.9.0"
slog-async = "2.7.0"
```

### Default template for Actix web
It is a default <a href="https://actix.rs/">hello world program</a> of Actix web 
```
use actix_web::{ web, App, HttpServer};

async fn index() -> &'static str {
    "Hello world!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting the server at 127.0.0.1:8080");

    HttpServer::new(|| {
        App::new()
            .service(web::resource("/index.html").to(|| async { "Hello world!" }))
            .service(web::resource("/").to(index))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

### Configure Logger

```
use slog;
use slog::{Logger,o,Drain,info};
use slog_term;
use slog_async;

fn configure_log()->Logger{
    let decorator = slog_term::TermDecorator::new().build();
    let console_drain = slog_term::FullFormat::new(decorator).build().fuse();
    
    // It is used for Synchronization
    let console_drain = slog_async::Async::new(console_drain).build().fuse();

    // Root logger
    slog::Logger::root(console_drain,o!("v"=>env!("CARGO_PKG_VERSION")))
}
```

Let's break the configuration function and understand what is happening behind the scene.

* **TermDecorator Decorator** : IT is used for formatting terminal output implemented using **term crate**. This decorator will add nice formatting to the logs it’s outputting. *Note* It does not deal with serialization so is !Sync. Run in a separate thread with slog_async::Async. We will be using the slog async with it. We can <a href="https://docs.rs/slog-term/2.9.0/slog_term/index.html#structs">other decorator</a> like CompactFormat, PlainRecordDecorator etc according to need.
* **FullFormat** : It is a Drain that will take *Decorator* as an argument, used for terminal output. Decorator is for formatting and Drain is for outputting.
* **Synchronization via Async Slog** : They are three ways slog to do synchronization out of which **PlainSyncDecorator** and **slog_async** are the efficient one depending on the need. Other than the two, the last **Synchronization via Mutex** is not efficient. You can read more <a href="https://docs.rs/slog-term/2.9.0/slog_term/index.html#structs">here</a>. We are using the synchronization with slog_async.
* **Logger::root** :<a href="https://docs.rs/slog/2.7.0/slog/struct.Logger.html#method.root">Logger</a> is used to execute logging statements. It takes two arguments 
    1. **drain** - destination where to forward logging Records for processing.
    2. **context** - list of key-value pairs associated with it.
* **o!** : Macro for building group of key-value pairs used as a content in Logger.

**fuse()** : It is used for panicking if something went wrong. It is necessary to call fuse as the root logger must take a Drain which is error free.

### Passing log instance to the handlers
Add the following line of code in main function
```
let log = configure_log();
```
It will configure the logger and ready to use now.


### Passing a log instance
```
HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(log.clone()))
            .service(web::resource("/index.html").to(|| async { "Hello world!" }))
            .service(web::resource("/").to(index))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
```

**web::Data::new(log.clone())** : It is an application data stored with App::app_data() method available through the HttpRequest::app_data method at runtime.

### Accessing the Log instance in function handlers
```
async fn index(log: web::Data<Logger>) -> &'static str {
    info!(log,
        "Inside Hello World"
    );
    "Hello world!"
}
```

**info!** : It is  a macro used for the building Info Level Record Or Context(key-value pair) used by Logger to output. They are a bunch of macros you can be used for <a href="https://docs.rs/slog/2.7.0/slog/index.html#macros">different level records</a>

**log: web::Data<Logger>** : -
Essentials helper functions and types for application registration.

Request Extractors
* Data: Application data item
* ReqData: Request-local data item
* Path: URL path parameters / dynamic segments
* Query: URL query parameters
* Header: Typed header
* Json: JSON payload
* Form: URL-encoded payload
* Bytes: Raw payload

We are using the `Data` method to access the application data initialised in server instance in main function.

### Complete Code
```
use actix_web::{web, App, HttpServer};
// IT is used as a logging middleware. We can even use the default logger with actix.
use slog;
use slog::{Logger,o,Drain,info};
use slog_term;
use slog_async;

fn configure_log()->Logger{
    // Formatting the output https://docs.rs/slog-term/2.9.0/slog_term/index.html#
    let decorator = slog_term::TermDecorator::new().build();

    // Drain for outputting https://docs.rs/slog-term/2.9.0/slog_term/index.html#structs
    // fuse is used for panicking if something went wrong. It is necessary to call fuse as the root logger must take a Drain which is error free.
    let console_drain = slog_term::FullFormat::new(decorator).build().fuse();

    // It is used for Synchronization https://docs.rs/slog-term/2.9.0/slog_term/index.html#structs
    let console_drain = slog_async::Async::new(console_drain).build().fuse();
    slog::Logger::root(console_drain,o!("v"=>env!("CARGO_PKG_VERSION")))
}
async fn index(log: web::Data<Logger>) -> &'static str {
    info!(log,
        "Inside Hello World"
    );
    "Hello world!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let log = configure_log();

    info!(log,
        "Starting the server at http://127.0.0.1:8080/"
    );

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(log.clone()))
            .service(web::resource("/index.html").to(|| async { "Hello world!" }))
            .service(web::resource("/").to(index))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```


#### Future Scopes 
To integrate the stackoverflow scraper to get the question requested by the user and saving it in the database for future use.

#### Api Response Test
Benchmark on my system Intel(R) Core(TM) i3-7020U CPU @ 2.30GHz

1. Simple deserialize - hello world get request
  * 80000 requests per second with 30 concurrent requests

2. DB Read  - get request
  * 8500 requests per second with 30 concurrent requests

3. Db Write from json - post request
  * 5700 requests per second with 30 concurrent requests