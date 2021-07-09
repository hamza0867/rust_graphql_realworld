use juniper::{FieldResult, GraphQLInputObject, IntoFieldError,
GraphQLObject};
use crate::schema::Context;
use crate::user::auth;
use super::db::ArticleEntity;



#[derive(GraphQLInputObject)]
#[graphql(description = "Payload to create an article")]
pub struct NewArticle {
    pub title: String,
    pub description: Option<String>,
    pub body: String,
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
}

#[derive(GraphQLInputObject)]
pub struct ArticlesOptions {
   pub tag: Option<String>,
   pub author: Option<String>,
   pub favorited: Option<String>,
   pub limit: Option<i32>,
   pub offset: Option<i32>
}

#[derive(GraphQLObject)]
#[graphql(context = Context)]
pub struct ArticlesPage {
   pub articles: Vec<ArticleEntity>,
   pub articles_count: i32
}
pub struct ArticleQuery;

#[juniper::graphql_object(Context = Context)]
impl ArticleQuery {
    fn get_article(context: &Context, slug: String) -> FieldResult<ArticleEntity> {
        let pool = &context.db_pool;
        use super::db::get_by_slug;
        let article_result = get_by_slug(pool, slug);
        match article_result {
            Err(diesel::result::Error::NotFound) => Err(super::errors::ArticleError::NotFound.into_field_error()),
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
}

