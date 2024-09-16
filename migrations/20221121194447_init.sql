-- Ayresia says I don't need this
-- CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE users (
    user_id UUID PRIMARY KEY DEFAULT gen_random_uuid()
);

CREATE TYPE ai_song_status AS ENUM (
  'STREAMING',
  'COMPLETED',
);



  -- TODO: Figure out status
CREATE TABLE ai_song_playlist(
  song_id UUID NOT NULL,
  title TEXT NOT NULL,
  tags TEXT NOT NULL,
  prompt TEXT NOT NULL,
  username TEXT NOT NULL,
  audio_url TEXT NOT NULL,
  lyric TEXT NOT NULL,
  gpt_description_prompt TEXT NOT NULL,
  last_updated TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
  created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- ALTER TABLE twitch_stream_state ADD COLUMN current_song_id UUID UNIQUE references ai_song_playlist;
-- ALTER TABLE twitch_stream_state ADD COLUMN ai_background_theme TEXT;
CREATE TABLE twitch_stream_state(
  sub_only_tts boolean NOT NULL DEFAULT false,
  explicit_soundeffects boolean NOT NULL DEFAULT false,
  implicit_soundeffects boolean NOT NULL DEFAULT false,
  global_voice boolean NOT NULL DEFAULT false,
  
  dalle_model TEXT NOT NULL DEFAULT "dalle-3",
  dalle_mode boolean NOT NULL DEFAULT true,
  enable_stable_diffusion boolean NOT NULL DEFAULT false,
  ai_background_theme TEXT,
  
  current_song_id UUID UNIQUE references ai_song_playlist
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

CREATE TABLE suno_request ()
  user_id UUID UNIQUE NOT NULL references users,
  request TEXT NOT NULL,
  created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
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

CREATE TABLE redemptions (
title      TEXT NOT NULL ,

  twitch_id  UUID NOT NULL,
  
  -- TODO: Add NOT NULL, after we populate/delete redemptions
  redemptions ADD COLUMN twitch_id UUID;
  reward_id  UUID NOT NULL references twitch_rewards (twitch_id),
  user_name  TEXT NOT NULL,
  cost       INT NOT NULL,
  user_input TEXT
  
  created_at  TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE skybox_requests (
  blockade_id INT NOT NULL,
  prompt TEXT NOT NULL,
  skybox_style_id INT NOT NULL,
  file_url TEXT, 
  username  TEXT NOT NULL,
  
  created_at  TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
  completed_at  TIMESTAMPTZ
);


CREATE TABLE obs_sources (
  source TEXT UNIQUE NOT NULL,
  scene TEXT NOT NULL,
  position_x DECIMAL NOT NULL,
  position_y DECIMAL NOT NULL,
  scale      DECIMAL NOT NULL
);

