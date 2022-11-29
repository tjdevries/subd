use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use twitch_irc::message::PrivmsgMessage;

use crate::{Role, TwitchSubLevel, TwitchUserID, UserRoles};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TwitchUser {
    // ID associated on twitch with this user
    pub id: TwitchUserID,
    pub login: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TwitchChannel {
    id: String,
    login: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Emote {
    // FIXME: Mention how to get the emote picture
    /// ID of emote
    pub id: String,
    // Start index of emote in message
    // pub start: i64,
    // End index of emote in message
    // pub end: i64,
}

impl From<twitch_irc::message::Emote> for Emote {
    fn from(emote: twitch_irc::message::Emote) -> Self {
        Self { id: emote.id }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Color {
    /// Red component
    pub r: u8,
    /// Green component
    pub g: u8,
    /// Blue component
    pub b: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TwitchMessage {
    /// A string uniquely identifying this message. Can be used with `/delete <message_id>` to
    /// delete single messages (see also the `CLEARMSG` message type)
    pub message_id: String,

    /// ID of the channel that the message was sent to.
    pub channel: TwitchChannel,

    /// Whether this message was made using the `/me` command.
    ///
    /// These type of messages are typically fully colored with `name_color` and
    /// have no `:` separating the sending user and the message.
    ///
    /// The `message_text` does not contain the `/me` command or the control sequence
    /// (`\x01ACTION <msg>\x01`) that is used for these action messages.
    pub is_action: bool,

    /// The user that sent this message.
    pub sender: TwitchUser,

    /// The message text that was sent.
    pub text: String,

    /// Metadata related to the chat badges in the `badges` tag.
    ///
    /// NOTE: This is ONLY for roles related to twitch... perhaps we should change this
    /// to TwitchRoles or similar, but I'm not really ready for that yet
    pub roles: UserRoles,

    /// If present, specifies how many bits were cheered with this message.
    pub bits: Option<u64>,

    /// If present, specifies the color that the user's name should be displayed in. A value
    /// of `None` here signifies that the user has not picked any particular color.
    /// Implementations differ on how they handle this, on the Twitch website users are assigned
    /// a pseudorandom but consistent-per-user color if they have no color specified.
    pub name_color: Option<Color>,

    /// A list of emotes in this message. Each emote replaces a part of the `message_text`.
    /// These emotes are sorted in the order that they appear in the message.
    pub emotes: Vec<Emote>,
}

fn get_twitch_roles_from_msg(msg: &PrivmsgMessage) -> UserRoles {
    let mut roles = HashSet::new();
    if msg
        .badges
        .iter()
        .any(|b| b.name == "moderator" || b.name == "broadcaster")
    {
        roles.insert(Role::TwitchMod);
    }

    if msg.badges.iter().any(|b| b.name == "vip") {
        roles.insert(Role::TwitchVIP);
    }
    if msg.badges.iter().any(|b| b.name == "founder") {
        roles.insert(Role::TwitchFounder);
    }
    if msg.badges.iter().any(|b| b.name == "subscriber") {
        roles.insert(Role::TwitchSub(TwitchSubLevel::Unknown));
    }
    if msg.badges.iter().any(|b| b.name == "staff") {
        roles.insert(Role::TwitchStaff);
    }

    UserRoles {
        roles,
        ..Default::default()
    }
}

impl TwitchMessage {
    pub fn from_msg(msg: PrivmsgMessage) -> Self {
        let roles = get_twitch_roles_from_msg(&msg);

        Self {
            message_id: msg.message_id,
            channel: TwitchChannel {
                id: msg.channel_id,
                login: msg.channel_login,
            },
            is_action: msg.is_action,
            sender: TwitchUser {
                id: TwitchUserID(msg.sender.id),
                login: msg.sender.login,
                name: msg.sender.name,
            },
            text: msg.message_text,
            roles,
            bits: msg.bits,
            name_color: msg.name_color.map(|c| Color {
                r: c.r,
                g: c.g,
                b: c.b,
            }),
            emotes: msg.emotes.into_iter().map(|e| e.into()).collect(),
        }
    }
}
