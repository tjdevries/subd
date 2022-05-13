-- Table for maintaining a singular user.
CREATE TABLE users (
    -- TODO: This should probably be something like a UUID
    id integer PRIMARY KEY UNIQUE NOT NULL,

    -- I wonder if we should also have a UUID for this to track twitch users over time
    -- and through name changes? (same for github_user)
    twitch_user TEXT UNIQUE,
    github_user TEXT UNIQUE

    -- TODO: Add this back in for metadata things
    -- last_updated DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- These might be useless...?
-- CREATE INDEX user__twitch_user on USER (twitch_user);
-- CREATE INDEX user__github_user on USER (github_user);

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
