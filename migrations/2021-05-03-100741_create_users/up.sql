-- Your SQL goes here
CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  email VARCHAR NOT NULL UNIQUE,
  username VARCHAR NOT NULL UNIQUE,
  bio TEXT,
  image VARCHAR,
  password_hash VARCHAR NOT NULL
)
