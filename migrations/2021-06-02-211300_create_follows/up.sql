-- Your SQL goes here
CREATE TABLE follows (
  follower_id integer references users ,
  followed_id integer references users ,
  active boolean not null,
  primary key (follower_id, followed_id)
) 
