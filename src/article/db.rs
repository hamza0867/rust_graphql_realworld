use diesel::result::QueryResult;
use diesel::prelude::*;
use crate::db_schema::articles ;
use crate::db_schema::tags;
use crate::db_schema::tag_article;
use crate::db::DbPool;
use diesel::pg::upsert::*;
use chrono::{DateTime, Utc};
use super::model::Article;
use super::resolvers::NewArticle;
use slugify::slugify;

#[derive(Queryable, PartialEq)]
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

pub fn create(pool: &DbPool, new_article: NewArticle, given_author_id: i32 ) -> QueryResult<Article> {
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
        let author = crate::user::db::get_user_by_id(pool, &given_author_id)?;
        let following = crate::user::db::get_follows(pool, &given_author_id, &author.username)?;
    Ok(Article {
        slug: created_article_entity.slug,
        title: created_article_entity.title,
        description: created_article_entity.description,
        body: created_article_entity.body,
        created_at: created_article_entity.created_at,
        updated_at: created_article_entity.updated_at,
        tag_list: new_article.tag_list.unwrap_or(vec![]) ,
        favorited: false,
        favorites_count: 0,
        author: crate::user::model::Profile {
            username: author.username,
            bio: author.bio,
            image: author.image,
            following,
        }
    })
}
