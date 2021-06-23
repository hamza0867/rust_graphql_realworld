-- Your SQL goes here
create table articles (
  id serial primary key,
  slug varchar not null ,
  title varchar not null,
  description varchar,
  body varchar not null,
  created_at timestamptz not null,
  updated_at timestamptz not null,
  author_id integer not null references users (id) on delete cascade 
);

create table tags (
  tag varchar primary key
);

create table tag_article (
  tag varchar references tags,
  article_id integer references articles,
  primary key (tag, article_id)
);
