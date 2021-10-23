use juniper::{FieldResult, GraphQLInputObject, GraphQLObject, IntoFieldError};

use super::db::ArticleEntity;
use crate::schema::Context;
use crate::user::auth;

#[derive(GraphQLInputObject)]
#[graphql(description = "Payload to create an article")]
pub struct NewArticle {
    pub title: String,
    pub description: Option<String>,
    pub body: String,
    pub tag_list: Option<Vec<String>>,
}

pub struct UpdateArticle {
    pub title: Option<String>,
    pub description: Option<String>,
    pub body: Option<String>,
    pub tag_list: Option<Vec<String>>,
}

pub struct ArticleMutation;

#[juniper::graphql_object(Context = Context)]
impl ArticleMutation {
    fn create_article(context: &Context, new_article: NewArticle) -> FieldResult<ArticleEntity> {
        use super::db::create;
        let pool = &context.db_pool;
        let id = auth::get_id_from_token(&context.token);
        if let Err(e) = id {
            return Err(e);
        };
        let author_id = id.unwrap();
        let article = create(pool, new_article, author_id)?;
        Ok(article)
    }

    //fn update_article(
    //context: &Context,
    //update_article: UpdateArticle,
    //) -> FieldResult<ArticleEntity> {
    //use super::db::create;
    //let pool = &context.db_pool;
    //let id = auth::get_id_from_token(&context.token);
    //if let Err(e) = id {
    //return Err(e);
    //};
    //let author_id = id.unwrap();
    //let article = create(pool, update_article, author_id)?;
    //Ok(article)
    //}

    fn delete_article(context: &Context, article_slug: String) -> FieldResult<String> {
        use super::db::delete;
        let pool = &context.db_pool;
        let id = auth::get_id_from_token(&context.token);
        if let Err(e) = id {
            return Err(e);
        };
        let author_id = id.unwrap();
        use super::db::get_by_slug;
        let article = get_by_slug(pool, article_slug)?;
        if article.author_id != author_id {
            return Err(crate::user::errors::UserError::Unauthorized.into_field_error());
        }
        delete(pool, article.id)?;
        Ok(article.slug)
    }
}

#[derive(GraphQLInputObject)]
pub struct ArticlesOptions {
    pub tag: Option<String>,
    pub author: Option<String>,
    pub favorited: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(GraphQLObject)]
#[graphql(context = Context)]
pub struct ArticlesPage {
    pub articles: Vec<ArticleEntity>,
    pub articles_count: i32,
}

#[derive(GraphQLInputObject)]
pub struct FeedOptions {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

pub struct ArticleQuery;

#[juniper::graphql_object(Context = Context)]
impl ArticleQuery {
    fn get_article(context: &Context, slug: String) -> FieldResult<ArticleEntity> {
        let pool = &context.db_pool;
        use super::db::get_by_slug;
        let article_result = get_by_slug(pool, slug);
        match article_result {
            Err(diesel::result::Error::NotFound) => {
                Err(super::errors::ArticleError::NotFound.into_field_error())
            }
            Ok(article) => Ok(article),
            Err(e) => {
                eprintln!("{}", e);
                use juniper::{graphql_value, FieldError};
                Err(FieldError::new(
                    "Internal Server Error",
                    graphql_value!({
                        "code": "internal.server.error"
                    }),
                ))
            }
        }
    }

    fn get_articles(context: &Context, options: ArticlesOptions) -> FieldResult<ArticlesPage> {
        let pool = &context.db_pool;
        use super::db::get_articles;
        let articles_result = get_articles(pool, options);
        match articles_result {
            Ok(articles) => Ok(articles),
            Err(e) => {
                eprintln!("{}", e);
                use juniper::{graphql_value, FieldError};
                Err(FieldError::new(
                    "Internal Server Error",
                    graphql_value!({
                        "code": "internal.server.error"
                    }),
                ))
            }
        }
    }

    fn feed(context: &Context, options: Option<FeedOptions>) -> FieldResult<ArticlesPage> {
        let id = auth::get_id_from_token(&context.token);
        if let Err(e) = id {
            return Err(e);
        };
        let user_id = id.unwrap();

        let pool = &context.db_pool;

        let feed_options = options.unwrap_or(FeedOptions {
            limit: None,
            offset: None,
        });

        use super::db::get_feed;
        let articles_result = get_feed(pool, user_id, feed_options);
        match articles_result {
            Ok(articles) => Ok(articles),
            Err(e) => {
                eprintln!("{}", e);
                use juniper::{graphql_value, FieldError};
                Err(FieldError::new(
                    "Internal Server Error",
                    graphql_value!({
                        "code": "internal.server.error"
                    }),
                ))
            }
        }
    }
}
