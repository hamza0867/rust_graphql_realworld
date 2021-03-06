use juniper::{FieldResult, GraphQLInputObject, IntoFieldError};
use serde::Deserialize;

use super::auth;
use super::db::{NewUserDTO, UserEntity, UserUpdateDTO};
use super::model::{Profile, User};
use crate::schema::Context;

#[derive(GraphQLInputObject)]
#[graphql(description = "Payload to register a user to the app")]
pub struct NewUser {
    email: String,
    password: String,
    username: String,
}

#[derive(GraphQLInputObject, Deserialize)]
#[graphql(description = "Payload to update a registered user to the app")]
pub struct UserUpdate {
    email: Option<String>,
    password: Option<String>,
    username: Option<String>,
    image: Option<String>,
    bio: Option<String>,
}

impl UserUpdate {
    fn to_entity(self, user_entity: UserEntity) -> UserUpdateDTO {
        UserUpdateDTO {
            email: self.email.unwrap_or(user_entity.email),
            password_hash: self
                .password
                .map(|s| bcrypt::hash(s, bcrypt::DEFAULT_COST).unwrap())
                .unwrap_or(user_entity.password_hash),
            username: self.username.unwrap_or(user_entity.username),
            image: self.image.or(user_entity.image),
            bio: self.bio.or(user_entity.bio),
        }
    }
}

#[derive(GraphQLInputObject)]
#[graphql(description = "Payload to authenticate to the app")]
pub struct AuthPayload {
    username: String,
    password: String,
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

pub struct UsersQuery;

#[juniper::graphql_object(Context = Context)]
impl UsersQuery {
    fn profile(context: &Context, username: String) -> FieldResult<Profile> {
        use super::auth::decode_token;
        if context.token.is_none() {
            return Err(UserError::Unauthorized.into_field_error());
        }
        let token = context.token.as_ref().unwrap().as_str();
        let claims_result = decode_token(token);
        if claims_result.is_err() {
            return Err(UserError::Unauthorized.into_field_error());
        }
        let claims = claims_result.unwrap().claims;
        let id = claims.sub.parse::<i32>().unwrap();

        let pool = &context.db_pool;
        use super::db::get_user_by_username;
        let user = get_user_by_username(pool, &username);
        if let Err(e) = user {
            return match e {
                diesel::result::Error::NotFound => Err(UserError::NotFound.into_field_error()),
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
        use super::db::get_follows;
        let following = get_follows(pool, &id, &username);
        if let Err(e) = following {
            eprintln!("{}", e);
            use juniper::{graphql_value, FieldError};
            return Err(FieldError::new(
                "Internal Server Error",
                graphql_value!({
                    "code": "internal.server.error"
                }),
            ));
        };
        let following = following.unwrap();
        Ok(Profile {
            username: user.username,
            bio: user.bio,
            image: user.image,
            following,
        })
    }
}

pub struct UsersMutation;

use super::errors::UserError;
#[juniper::graphql_object(Context = Context)]
impl UsersMutation {
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

    fn update_user(context: &Context, user_update: UserUpdate) -> FieldResult<User> {
        let pool = &context.db_pool;
        let id = auth::get_id_from_token(&context.token);
        if let Err(e) = id {
            return Err(e);
        };
        let id = id.unwrap();
        use super::db::get_user_by_id;
        let user = get_user_by_id(pool, &id).unwrap();
        let update_user_dto = user_update.to_entity(user);
        let updated_user = super::db::update_user(pool, update_user_dto, &id).unwrap();
        Ok(User::from(updated_user))
    }

    fn follow(context: &Context, username: String) -> FieldResult<Profile> {
        let pool = &context.db_pool;
        let id = auth::get_id_from_token(&context.token);
        if let Err(e) = id {
            return Err(e);
        };
        let id = id.unwrap();

        use super::db::get_user_by_username;
        let user = get_user_by_username(pool, &username);
        if let Err(e) = user {
            return match e {
                diesel::result::Error::NotFound => Err(UserError::NotFound.into_field_error()),
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
        use super::db::follow;
        let exec_result = follow(pool, &id, &username);
        if let Err(e) = exec_result {
            eprintln!("{}", e);
            use juniper::{graphql_value, FieldError};
            return Err(FieldError::new(
                "Internal Server Error",
                graphql_value!({
                    "code": "internal.server.error"
                }),
            ));
        };
        Ok(Profile {
            username,
            bio: user.bio,
            image: user.image,
            following: true,
        })
    }

    fn unfollow(context: &Context, username: String) -> FieldResult<Profile> {
        let pool = &context.db_pool;
        let id = auth::get_id_from_token(&context.token);
        if let Err(e) = id {
            return Err(e);
        };
        let id = id.unwrap();

        use super::db::get_user_by_username;
        let user = get_user_by_username(pool, &username);
        if let Err(e) = user {
            return match e {
                diesel::result::Error::NotFound => Err(UserError::NotFound.into_field_error()),
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
        use super::db::unfollow;
        let exec_result = unfollow(pool, &id, &username);
        if let Err(e) = exec_result {
            eprintln!("{}", e);
            use juniper::{graphql_value, FieldError};
            return Err(FieldError::new(
                "Internal Server Error",
                graphql_value!({
                    "code": "internal.server.error"
                }),
            ));
        };
        Ok(Profile {
            username,
            bio: user.bio,
            image: user.image,
            following: false,
        })
    }
}

