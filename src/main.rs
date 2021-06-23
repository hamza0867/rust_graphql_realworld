#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate slugify;
extern crate juniper;

use actix_web::{ middleware, web, App, Error, HttpResponse, HttpServer};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use db::DbPool;
use juniper::http::playground::playground_source;
use juniper::http::GraphQLRequest;
use schema::Context;

mod db;
mod db_schema;
mod schema;
mod user;
mod article;

use crate::schema::{create_schema, Schema};

pub async fn graphql(
    pool: web::Data<DbPool>,
    schema: web::Data<Schema>,
    data: web::Json<GraphQLRequest>,
    credentials: Option<BearerAuth>,
) -> Result<HttpResponse, Error> {
    let res = web::block(move || {
        let ctx = Context {
            db_pool: pool.get_ref().to_owned(),
            token: credentials.map(|auth| {
                auth.token().to_string()
            }) ,
        };
        let res = data.execute_sync(&schema, &ctx);
        Ok::<_, serde_json::error::Error>(serde_json::to_string(&res)?)
    })
    .await
    .map_err(Error::from)?;

    Ok(HttpResponse::Ok()
    .content_type("application/json")
    .body(res))
}

pub async fn graphql_playground() -> HttpResponse {
    HttpResponse::Ok()
    .content_type("text/html; charset=utf-8")
    .body(playground_source("/graphql", None))
}

pub fn register(config: &mut web::ServiceConfig) {
    config
    .data(create_schema())
    .route("/graphql", web::post().to(graphql))
    .route("/graphiql", web::get().to(graphql_playground));
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db_pool = db::get_db_pool();
    HttpServer::new(move || {
        App::new()
        .data(db_pool.clone())
        .wrap(middleware::Logger::default())
        .configure(register)
        .default_service(web::to(|| async { "404" }))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

