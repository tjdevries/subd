-- Ayresia says I don't need this
-- CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE users (
    user_id UUID PRIMARY KEY DEFAULT gen_random_uuid()
);


CREATE TABLE user_stream_character_information(
  /* user_id UUID UNIQUE NOT NULL references users, */
  username TEXT NOT NULL UNIQUE,
  obs_character TEXT NOT NULL,
  voice TEXT NOT NULL,
  random boolean NOT NULL DEFAULT false
);

CREATE TABLE user_roles (
  user_id UUID UNIQUE NOT NULL references users,
  last_updated TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,

  -- Github
  is_github_sponsor boolean NOT NULL,

  -- Twitch
  is_twitch_mod     boolean NOT NULL,
  is_twitch_vip     boolean NOT NULL,
  is_twitch_founder boolean NOT NULL,
  is_twitch_staff   boolean NOT NULL,
  -- TODO: Twitch sub should probably be a number or an enum
  is_twitch_sub     boolean NOT NULL

  -- ...
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

CREATE TABLE user_messages (
  user_message_id INT GENERATED ALWAYS AS IDENTITY,

  user_id     UUID NOT NULL references users,
  platform    user_platform NOT NULL,
  contents    TEXT NOT NULL,
  -- TODO: Emotes
  created_at  TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- TODO: Could also save twitch chat history...?
--  This could be full of richer data?

CREATE TABLE twitch_users (
  twitch_user_id  TEXT UNIQUE PRIMARY KEY,
  user_id         UUID UNIQUE NOT NULL references users,
  login           TEXT UNIQUE NOT NULL,
  display_name    TEXT NOT NULL
);
