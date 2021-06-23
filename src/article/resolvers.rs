use juniper::{FieldResult, GraphQLInputObject};

use super::model::Article;
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

pub struct ArticleMutation;

#[juniper::graphql_object(Context = Context)]
impl ArticleMutation {
    fn create_article(context: &Context, new_article: NewArticle) -> FieldResult<Article> {
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

