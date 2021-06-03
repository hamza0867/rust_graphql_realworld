use diesel::result::QueryResult;
use diesel::prelude::*;
use crate::db_schema::users;
use crate::db_schema::users::dsl::*;
use crate::db_schema::follows;
use crate::db_schema::follows::dsl::*;
use crate::db::DbPool;
use diesel::pg::upsert::*;

#[derive(Queryable, PartialEq)]
pub struct UserEntity {
    pub id: i32,
    pub email: String,
    pub username: String,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub password_hash: String,
}


#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUserDTO {
    pub email: String,
    pub password_hash: String,
    pub username: String,
}

#[derive(AsChangeset)]
#[table_name = "users"]
pub struct UserUpdateDTO {
    pub email: String,
    pub username: String,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub password_hash: String,
}

#[derive(Insertable)]
#[table_name = "follows"]
pub struct NewFollowsDTO {
    pub follower_id: i32,
    pub followed_id: i32,
    pub active: bool,
}



pub fn create(pool: &DbPool, new_user: NewUserDTO) -> QueryResult<UserEntity> {
    use diesel::insert_into;
    let conn = pool.get().unwrap();
    insert_into(users)
        .values(&new_user)
        .get_result::<UserEntity>(&conn)
}

pub fn get_user_by_username(pool: &DbPool, given_username: &String) -> QueryResult<UserEntity> {
    let conn = pool.get().unwrap();
    users
    .filter(username.eq(given_username))
    .first::<UserEntity>(&conn)
}

pub fn get_user_by_id(pool: &DbPool, given_id: &i32) -> QueryResult<UserEntity> {
    let conn = pool.get().unwrap();
    users
    .filter(id.eq(given_id))
    .first::<UserEntity>(&conn)
}

pub fn update_user(pool: &DbPool, user_update_dto: UserUpdateDTO, given_id: &i32) -> QueryResult<UserEntity> {             
    let conn = pool.get().unwrap();
    diesel::update(users.filter(id.eq(given_id)))
    .set(user_update_dto).get_result::<UserEntity>(&conn)
}

pub fn get_follows(pool: &DbPool, given_follower_id: &i32, given_followed_username: &String) -> QueryResult<bool> {
    let conn = pool.get().unwrap();
    let given_followed_id = users
    .filter(username.eq(given_followed_username))
    .select(id)
    .first::<i32>(&conn)?;
    let follows_active = follows.filter(
        follower_id.eq(given_follower_id)
        .and(followed_id.eq(given_followed_id))
    ).select(active)
    .first::<bool>(&conn); 
    match follows_active {
        Err(diesel::result::Error::NotFound) => {
                use diesel::insert_into;
                let exec_result = insert_into(follows)
                .values(&NewFollowsDTO{
                    followed_id: given_followed_id,
                    follower_id: given_follower_id.to_owned(),
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

pub fn follow(pool: &DbPool, given_follower_id: &i32, given_followed_username: &String) -> QueryResult<()> {
    let conn = pool.get().unwrap();
    let given_followed_id = users
    .filter(username.eq(given_followed_username))
    .select(id)
    .first::<i32>(&conn)?;
                use diesel::insert_into;
    insert_into(follows)
      .values(&NewFollowsDTO {
        followed_id:given_followed_id,
        follower_id: given_follower_id.to_owned(),
        active: true
      }).on_conflict(
        on_constraint("follows_pkey")
      ).do_update()
      .set(active.eq(true))
      .execute(&conn)
      .map(|_| ())
}

