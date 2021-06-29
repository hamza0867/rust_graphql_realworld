use super::db::ArticleEntity;
use crate::schema::Context;
use crate::user::model::Profile;
use chrono::{Utc, DateTime};
use juniper::FieldResult;
use crate::user::auth;
use diesel::prelude::*;

#[juniper::graphql_object(Context = Context, name = "Article")]
impl ArticleEntity {

    fn slug(&self) -> &str {
        self.slug.as_str()
    }

    fn title(&self) -> &str {
        self.title.as_str()
    }

    fn body(&self) -> &str {
        self.body.as_str()
    }

    fn description(&self) -> &Option<String> {
        &self.description
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    fn author(&self, context: &Context) -> FieldResult<Profile > {
        let pool = &context.db_pool;
        let author = crate::user::db::get_user_by_id(pool, &self.author_id)?;
        let follower_id = context
        .token
        .clone()
        .map(|token| auth::get_id_from_token(&Some(token)).unwrap());
        let following;
        if let Some(found_id) = follower_id {
            following = crate::user::db::get_follows(pool, &found_id, &author.username)?;
        } else {
            following = false;
        }
        Ok(Profile {
            username: author.username,
            bio: author.bio,
            image: author.image,
            following,
        })
    }

    fn favorited(&self, context: &Context) -> FieldResult<bool>{

        let pool = &context.db_pool;
        let follower_id = context
        .token
        .clone()
        .map(|token| auth::get_id_from_token(&Some(token)).unwrap());
        if let Some(found_id) = follower_id {
            let result = super::db::get_user_favorites_article(pool, found_id, self.id);
            match result {
                Ok(favorited) => Ok(favorited),
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
        } else {
            Ok(false)
        }
    }

    fn favorites_count(&self, context: &Context) -> FieldResult<i32>{
        let pool = &context.db_pool;
        let conn = pool.get().unwrap();
        use crate::db_schema::user_favorites_article::dsl::*;
        use diesel::dsl::count_star;
        let result = user_favorites_article.filter(
            article_id.eq(self.id).and(active.eq(true))
        ).select(count_star()).first::<i64>(&conn);
        match result {
            Ok(favorites_count) => Ok(favorites_count as i32),
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

    fn tag_list(&self, context: &Context) -> FieldResult<Vec<String>> {
        use crate::db_schema::tag_article::dsl::*;
        let pool = &context.db_pool;
        let conn = pool.get().unwrap();
        let result = tag_article.filter(article_id.eq(self.id)).select(tag).load::<String>(&conn);
        match result {
            Ok(tag_list) => Ok(tag_list),
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

