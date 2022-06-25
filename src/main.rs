#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate juniper;
extern crate slugify;

use actix_web::{
    middleware,
    web::{self, Data},
    App, Error, HttpResponse, HttpServer,
};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use db::DbPool;
use juniper_actix::{graphiql_handler, graphql_handler, playground_handler};
use schema::Context;

mod article;
mod db;
mod db_schema;
mod schema;
mod user;

use crate::schema::{create_schema, Schema};

pub async fn graphql(
    req: actix_web::HttpRequest,
    payload: actix_web::web::Payload,
    pool: web::Data<DbPool>,
    schema: web::Data<Schema>,
    credentials: Option<BearerAuth>,
) -> Result<HttpResponse, Error> {
    let ctx = Context {
        db_pool: pool.get_ref().to_owned(),
        token: credentials.map(|auth| auth.token().to_string()),
    };
    graphql_handler(&schema, &ctx, req, payload).await
}

async fn graphiql_route() -> Result<HttpResponse, Error> {
    graphiql_handler("/graphql", None).await
}
async fn playground_route() -> Result<HttpResponse, Error> {
    playground_handler("/graphql", None).await
}

pub fn register(config: &mut web::ServiceConfig) {
    config
        .app_data(Data::new(create_schema()))
        .service(
            web::resource("/graphql")
                .route(web::post().to(graphql))
                .route(web::get().to(graphql)),
        )
        .service(web::resource("/playground").route(web::get().to(playground_route)))
        .service(web::resource("/graphiql").route(web::get().to(graphiql_route)));
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db_pool = db::get_db_pool();
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(db_pool.clone()))
            .wrap(middleware::Logger::default())
            .configure(register)
            .default_service(web::to(|| async { "404" }))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
