-- Your SQL goes here
CREATE TABLE user_favorites_article (
  user_id integer not null references users (id) on delete cascade,
  article_id integer not null references articles (id) on delete cascade,
  active boolean not null,
  primary key(user_id, article_id)
);
