use juniper::{FieldResult, GraphQLInputObject, IntoFieldError};
use serde::Deserialize;

use super::auth;
use super::db::{NewUserDTO, UserEntity};
use super::model::{Profile, User};
use crate::schema::{Context, MutationRoot, QueryRoot};

#[derive(GraphQLInputObject)]
#[graphql(description = "Payload to register a user to the app")]
pub struct NewUser {
    email: String,
    password: String,
    username: String,
}

#[derive(GraphQLInputObject, Deserialize)]
#[graphql(description = "Payload to update a registered user to the app")]
pub struct UserUpdateDTO {
    email: Option<String>,
    password: Option<String> ,
    username: Option<String>,
    image: Option<String>,
    bio: Option<String>,
}

#[derive(GraphQLInputObject)]
#[graphql(description = "Payload to authenticate to the app")]
pub struct AuthPayload {
    username: String,
    password: String,
}

#[juniper::graphql_object(Context = Context)]
impl QueryRoot {
    fn profile(username: String) -> FieldResult<Profile> {
        Ok(Profile {
            username,
            bio: "bio".to_owned(),
            image: "image url".to_owned(),
            following: true,
        })
    }
}

impl From<NewUser> for NewUserDTO {
    fn from(new_user: NewUser) -> Self {
        let hashed_pwd = bcrypt::hash(new_user.password, bcrypt::DEFAULT_COST).unwrap();
        Self {
            email: new_user.email,
            username: new_user.username,
            password_hash: hashed_pwd,
        }
    }
}

impl From<UserEntity> for User {
    fn from(user_entity: UserEntity) -> Self {
        Self {
            username: user_entity.username,
            email: user_entity.email,
            bio: user_entity.bio,
            image: user_entity.image,
            token: auth::get_token(user_entity.id),
        }
    }
}

use super::errors::UserError;
#[juniper::graphql_object(Context = Context)]
impl MutationRoot {
    fn register_user(context: &Context, new_user: NewUser) -> FieldResult<User> {
        use super::db::create;
        let pool = &context.db_pool;
        let user = create(pool, NewUserDTO::from(new_user))?;
        Ok(User::from(user))
    }

    fn authenticate(context: &Context, auth_payload: AuthPayload) -> FieldResult<User> {
        let pool = &context.db_pool;
        use super::db::get_user_by_username;
        let user = get_user_by_username(pool, &auth_payload.username);
        if let Err(e) = user {
            return match e {
                diesel::result::Error::NotFound => {
                    Err(UserError::InvalidUsernameOrPassword.into_field_error())
                }
                _ => {
                    eprintln!("{}", e);
                    use juniper::{graphql_value, FieldError};
                    Err(FieldError::new(
                        "Internal Server Error",
                        graphql_value!({
                            "code": "internal.server.error"
                        }),
                    ))
                }
            };
        };
        let user = user.unwrap();
        let valid = bcrypt::verify(auth_payload.password, &user.password_hash).unwrap();
        if valid {
            Ok(User::from(user))
        } else {
            Err(UserError::InvalidUsernameOrPassword.into_field_error())
        }
    }

    fn update_user(context: &Context, user_update_dto: UserUpdateDTO) -> FieldResult<User> {
      use super::auth::decode_token;
      if context.token.is_none() {
        return Err(UserError::Unauthorized.into_field_error())
      }
      let token = context.token.as_ref().unwrap().as_str();
      let claims_result = decode_token(token);
      if claims_result.is_err() {
        return Err(UserError::Unauthorized.into_field_error())
      }
      let claims = claims_result.unwrap().claims;
      let id = claims.sub.parse::<i32>().unwrap();

      let pool = &context.db_pool;
      use super::db::get_user_by_id;
      let user = get_user_by_id(pool, &id).unwrap();
      Ok(User::from(user))
    }
}

