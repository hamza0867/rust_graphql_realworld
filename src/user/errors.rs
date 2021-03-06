use juniper::{graphql_value, FieldError, IntoFieldError};

pub enum UserError {
    InvalidUsernameOrPassword,
    Unauthorized,
    NotFound
}

impl IntoFieldError for UserError {
    fn into_field_error(self) -> FieldError {
        match self {
            UserError::InvalidUsernameOrPassword => FieldError::new(
                "Invalid username or password",
                graphql_value!({
                    "code": "invalid.username.or.password"
                }),
            ),
            UserError::Unauthorized => FieldError::new("Unauthorized", graphql_value!({
                "code": "unauthorized"
            }) ),
            UserError::NotFound => FieldError::new("Not found", graphql_value!({
                "code": "user.not.found"
            }) )
        }
    }
}

