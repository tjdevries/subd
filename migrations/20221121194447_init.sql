-- Ayresia says I don't need this
-- CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE users (
    user_id UUID PRIMARY KEY DEFAULT gen_random_uuid()
);

CREATE TABLE user_theme_songs (
    user_id UUID UNIQUE NOT NULL references users,
    song BYTEA NOT NULL
);

CREATE TABLE user_theme_song_history (
  user_id   UUID NOT NULL references users(user_id),
  played_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TYPE user_platform AS ENUM (
  'TWITCH',
  'YOUTUBE',
  'GITHUB'
);

CREATE TABLE user_chat_history (
  user_chat_history_id
    INT GENERATED ALWAYS AS IDENTITY,

  user_id     UUID NOT NULL references users,
  platform    user_platform NOT NULL,
  msg         TEXT,
  created_at  TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE twitch_users (
  twitch_user_id  TEXT UNIQUE PRIMARY KEY,
  user_id         UUID UNIQUE NOT NULL references users,
  login           TEXT UNIQUE NOT NULL,
  display_name    TEXT NOT NULL
);
