-- Add migration script here

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4()
);

CREATE TABLE user_theme_songs (
    user_id UUID UNIQUE NOT NULL references users(id),
    song BYTEA NOT NULL
);

CREATE TABLE USER_THEME_SONG_HISTORY (
  user_id   UUID references users(id),
  played_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
