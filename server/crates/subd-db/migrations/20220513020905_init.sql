-- Table for maintaining a singular user.
CREATE TABLE users (
    -- TODO: This should probably be something like a UUID
    id integer PRIMARY KEY UNIQUE NOT NULL,

    -- I wonder if we should also have a UUID for this to track twitch users over time
    -- and through name changes? (same for github_user)
    -- TODO: Should this be user_id?
    twitch_id TEXT UNIQUE,

    -- TODO: Switch to github_id if it exists
    github_user TEXT UNIQUE,

    -- TODO: Add this back in for metadata things
    -- last_updated DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL

    FOREIGN KEY(twitch_id) REFERENCES twitch_users(id)
);

CREATE TABLE twitch_users (
  id                INTEGER PRIMARY KEY NOT NULL,
  login             TEXT UNIQUE NOT NULL,
  display_name      TEXT NOT NULL,
  offline_image_url TEXT,
  profile_image_url TEXT,
  account_created_at TEXT,

  -- TODO: Should we periodically update people's user stuff? Once a day? Cron job? etc?
  -- User’s broadcaster type: "partner", "affiliate", or "".
  broadcaster_type TEXT NOT NULL,

  -- User’s account type: "staff", "admin", "global_mod", or "".
  account_type TEXT NOT NULL

  -- UNUSED:
  -- description	string	User’s channel description.
  -- email	string	User’s verified email address. Returned if the request includes the user:read:email scope.
  -- view_count	integer	Total number of views of the user’s channel.

);


CREATE TABLE twitch_moderators (
  broadcaster_id    INTEGER NOT NULL,
  user_id           INTEGER NOT NULL,
  last_updated      DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,

  FOREIGN KEY(broadcaster_id) REFERENCES twitch_users(id),
  FOREIGN KEY(user_id)      REFERENCES twitch_users(id)
);

CREATE TABLE twitch_gifted_subscriptions (
  id integer PRIMARY KEY AUTOINCREMENT,
  broadcaster_id  INTEGER NOT NULL,
  user_id         INTEGER NOT NULL,
  gifter_id       INTEGER NOT NULL,

  FOREIGN KEY(broadcaster_id)   REFERENCES twitch_users(id),
  FOREIGN KEY(user_id)        REFERENCES twitch_users(id),
  FOREIGN KEY(gifter_id)      REFERENCES twitch_users(id)
);

CREATE TABLE twitch_subscriptions (
  user_id     INTEGER NOT NULL,

  -- 0 to 3, Twitch Sub Tiers (0 represents 
  tier        INTEGER NOT NULL CHECK(tier >= 0 AND tier <= 3),

  -- if gift_id is NOT NULL, then the sub is a gift
  gift_id     INTEGER,

  -- Start Date can be NULL, because we may not have been running when the 
  start_date  DATETIME,
  -- noted date is when we mark down that this subscription is active
  noted_date  DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,

  PRIMARY KEY (user_id, tier, start_date),
  FOREIGN KEY(user_id)      REFERENCES twitch_users(id),
  FOREIGN KEY(gift_id)      REFERENCES twitch_gifted_subscriptions(id)
);

CREATE TABLE TWITCH_CHAT_HISTORY (
    id integer PRIMARY KEY AUTOINCREMENT,
    user_id BLOB,
    msg TEXT,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY(user_id) REFERENCES USERS(id)
);

-- quickly look up user information in table.
CREATE INDEX twitch_chat_history__user_id on TWITCH_CHAT_HISTORY (user_id);
-- TODO: Should I use another index here to sort by datetime? maybe it will do automatically.
