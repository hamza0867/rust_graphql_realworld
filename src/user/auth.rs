use serde::{Deserialize, Serialize};
use chrono::{Duration, prelude::*};
use jsonwebtoken as jwt;
use jwt::{DecodingKey, EncodingKey, decode, Validation, TokenData};
use super::errors::UserError;
use juniper::{FieldError, IntoFieldError};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    exp: usize, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    iat: usize, // Optional. Issued at (as UTC timestamp)
    iss: String, // Optional. Issuer
    pub sub: String, // Optional. Subject (whom token refers to)
}

pub fn get_token(id: i32) -> String {
    let sub = id.to_string();
    let now = Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + Duration::minutes(60)).timestamp() as usize;
    let iss = "real_world_rust_graphql".to_string();
    let claims = Claims { exp, iss, iat, sub };
    jwt::encode(&jwt::Header::default() , &claims, &EncodingKey::from_secret("real_world_rust_graphql".as_ref()))
    .expect("jwt creation failed!")
}

pub fn decode_token(token: &str) -> jwt::errors::Result<TokenData<Claims>> {
    decode::<Claims>(token, &DecodingKey::from_secret("real_world_rust_graphql".as_ref()), &Validation::default())
}

pub fn get_id_from_token(token: &Option<String>) -> Result<i32, FieldError>{

    if token.is_none() {
        return Err(UserError::Unauthorized.into_field_error())
    }
    let token = token.as_ref().unwrap().as_str();
    let claims_result = decode_token(token);
    if claims_result.is_err() {
        return Err(UserError::Unauthorized.into_field_error())
    }
    let claims = claims_result.unwrap().claims;
    let id = claims.sub.parse::<i32>().unwrap();
    Ok(id)
}

