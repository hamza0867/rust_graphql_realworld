use diesel::result::QueryResult;
use diesel::prelude::*;
use crate::db_schema::users;
use crate::db_schema::users::dsl::*;

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

use crate::db::DbPool;
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


