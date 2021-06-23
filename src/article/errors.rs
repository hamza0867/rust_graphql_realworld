use juniper::{graphql_value, FieldError, IntoFieldError};

pub enum ArticleError {
    NotFound
}

impl IntoFieldError for ArticleError {
    fn into_field_error(self) -> FieldError {
        match self {
            ArticleError::NotFound => FieldError::new("Not found", graphql_value!({
                "code": "article.not.found"
            }) )
        }
    }
}

