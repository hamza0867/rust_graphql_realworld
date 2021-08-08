use diesel::result::QueryResult;
use diesel::prelude::*;
use crate::db_schema::articles ;
use crate::db_schema::tags;
use crate::db_schema::tag_article;
use crate::db_schema::user_favorites_article;
use crate::db::DbPool;
use diesel::pg::upsert::*;
use chrono::{DateTime, Utc};
use super::resolvers::NewArticle;
use slugify::slugify;

#[derive(Queryable, PartialEq, Debug )]
pub struct ArticleEntity {
    pub id: i32,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub author_id: i32
}

#[derive(Queryable, PartialEq, Insertable)]
#[table_name = "tags"]
pub struct TagEntity {
    tag: String
}

#[derive(Queryable, PartialEq, Insertable)]
#[table_name = "tag_article"]
pub struct TagArticleEntity {
    tag: String,
    article_id: i32
}

#[derive(Queryable, PartialEq, Insertable)]
#[table_name = "user_favorites_article"]
pub struct UserFavoritesArticle {
    article_id: i32,
    user_id: i32,
    active: bool
}


#[derive(Insertable)]
#[table_name = "articles"]
pub struct NewArticleDTO {
    pub title: String,
    pub description: Option<String>,
    pub body: String,
    pub slug: String,
    pub author_id: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}

fn new_article_dto_from_new_article(new_article: &NewArticle, author_id: i32) -> NewArticleDTO {
    NewArticleDTO {
        title: new_article.title.clone(),
        description: new_article.description.clone(),
        body: new_article.body.clone(),
        slug: slugify!(new_article.title.as_str()),
        author_id,
        created_at: Utc::now(),
        updated_at: Utc::now()
    }
}

pub fn create(pool: &DbPool, new_article: NewArticle, given_author_id: i32 ) -> QueryResult<ArticleEntity> {
    use diesel::insert_into;
    use crate::db_schema::articles::dsl::*;
    let conn = pool.get().unwrap();
    let created_article_entity = conn.transaction::<_,diesel::result::Error,_>(|| {
        let new_article_dto = new_article_dto_from_new_article(&new_article, given_author_id);
        let created_article_entity = insert_into(articles)
        .values(&new_article_dto)
        .get_result::<ArticleEntity>(&conn)?;
        let returned_article_id = created_article_entity.id;
        if let Some(tag_list) = &new_article.tag_list {
            use crate::db_schema::tags::dsl::*;
            let insertable_tags: Vec<TagEntity> = tag_list.iter().map(|given_tag| {
                TagEntity {
                    tag: given_tag.to_owned()
                }
            }).collect();
            insert_into(tags)
            .values(&insertable_tags)
            .on_conflict(on_constraint("tags_pkey"))
            .do_nothing()
            .execute(&conn)?;

            let insertable_article_tags: Vec<TagArticleEntity> = tag_list.iter().map(|given_tag| {
                TagArticleEntity{
                    tag: given_tag.to_owned(),
                    article_id: returned_article_id 
                }
            }).collect();
            use crate::db_schema::tag_article::dsl::*;
            let result  =  insert_into(tag_article)
            .values(&insertable_article_tags)
            .execute(&conn);
            if let Err(e) = result {
                return Err(e);
            } 
        }
        Ok(created_article_entity)
        })?;
    Ok(created_article_entity)
}

pub fn get_user_favorites_article(pool: &DbPool, given_user_id: i32, given_article_id: i32) -> QueryResult<bool> {
    let conn = pool.get().unwrap();
    use crate::db_schema::user_favorites_article::dsl::*;
    let favorites = user_favorites_article.filter(
            user_id.eq(given_user_id).and(article_id.eq(given_article_id))
          ).select(active).first::<bool>(&conn);
    match favorites {
        Err(diesel::result::Error::NotFound) => {
                use diesel::insert_into;
                let exec_result = insert_into(user_favorites_article)
                .values(&UserFavoritesArticle{
                    user_id: given_user_id,
                    article_id: given_article_id,
                    active: false
                }).execute(&conn);
                if let Err(e) = exec_result  {
                    return Err(e);
                }
                Ok(false)
        },
        Ok(x) => Ok(x),
        Err(y) => Err(y)
    }
}

pub fn get_by_slug(pool: &DbPool, given_slug: String) -> QueryResult<ArticleEntity> {
    use crate::db_schema::articles::dsl::*;
    let conn = pool.get().unwrap();
    let entity = articles.filter(slug.eq(given_slug)).first::<ArticleEntity>(&conn)?;
    Ok(entity)
}

use super::resolvers::{ArticlesPage, ArticlesOptions};
pub fn get_articles(pool: &DbPool, options: ArticlesOptions) -> QueryResult<ArticlesPage> {
    let conn = pool.get().unwrap();
    use diesel::pg::Pg;
    let mut query = crate::db_schema::articles::table.into_boxed::<Pg>();
    if let Some(given_tag) = options.tag {
        use crate::db_schema::tag_article::dsl::*;
        let article_ids = tag_article.filter(tag.eq(given_tag)).select(article_id).load::<i32>(&conn)?;
        use crate::db_schema::articles::dsl::*;
        query = query.filter(id.eq_any(article_ids));
    }
    if let Some(given_author) = options.author {
        use crate::db_schema::users::dsl::*;
        use crate::db_schema::users::dsl::id;
        let given_author_id = users.filter(username.eq(given_author)).select(id).first::<i32>(&conn)?;
        use crate::db_schema::articles::dsl::*;
        query = query.filter(author_id.eq(given_author_id));
    }
    if let Some(given_favorited_by) = options.favorited {
        use crate::db_schema::users::dsl::*;
        use crate::db_schema::user_favorites_article::dsl::*;
        let given_favorited_by_id = users.filter(username.eq(given_favorited_by)).select(id).first::<i32>(&conn)?;
        let article_ids = user_favorites_article.filter(active.eq(true).and(user_id.eq(given_favorited_by_id))).select(article_id).load::<i32>(&conn)?;
        query = query.filter(crate::db_schema::articles::dsl::id.eq_any(article_ids));
    }
    use crate::db_schema::articles::dsl::created_at;
    let articles = query.offset(options.offset.unwrap_or(0) as i64)
    .limit(options.limit.unwrap_or(20) as i64)
    .order_by(created_at.asc())
    .load::<ArticleEntity>(&conn)?;

    let count = articles.len() as i32;

    Ok(ArticlesPage {
        articles,
        articles_count: count
    })
}

use super::resolvers:: FeedOptions;

pub fn get_feed(pool: &DbPool, user_id: i32,options: FeedOptions) -> QueryResult<ArticlesPage> {
    let conn = pool.get().unwrap();

    use crate::db_schema::articles::dsl::*;
    use crate::db_schema::follows::dsl::*;

    let followed_authors_ids = follows.filter(follower_id.eq(user_id)).select(followed_id);
    let query = articles.filter(
        author_id.eq_any(followed_authors_ids)
    );
    use crate::db_schema::articles::dsl::created_at;
    let found_articles = query.offset(options.offset.unwrap_or(0) as i64)
    .limit(options.limit.unwrap_or(20) as i64)
    .order_by(created_at.asc())
    .load::<ArticleEntity>(&conn)?;

    let count = found_articles.len() as i32;

    Ok(ArticlesPage {
        articles: found_articles,
        articles_count: count
    })
}
