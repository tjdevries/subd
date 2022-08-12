use serde::{Deserialize, Serialize};
use twitch_api2::pubsub::channel_subscriptions::Emote;
use twitch_irc::message::RGBColor;

use crate::{TwitchUserID, UserID, UserRoles};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TwitchUser {
    // User ID for subd
    user_id: UserID,

    // ID associated on twitch with this user
    id: TwitchUserID,
    login: String,
    name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TwitchMessage {
    /// A string uniquely identifying this message. Can be used with `/delete <message_id>` to
    /// delete single messages (see also the `CLEARMSG` message type)
    pub message_id: String,

    /// ID of the channel that the message was sent to.
    pub channel_id: String,

    /// Login name of the channel that the message was sent to.
    pub channel_login: String,

    /// The message text that was sent.
    pub text: String,

    /// Whether this message was made using the `/me` command.
    ///
    /// These type of messages are typically fully colored with `name_color` and
    /// have no `:` separating the sending user and the message.
    ///
    /// The `message_text` does not contain the `/me` command or the control sequence
    /// (`\x01ACTION <msg>\x01`) that is used for these action messages.
    // pub is_action: bool,

    /// The user that sent this message.
    pub sender: TwitchUser,

    /// Metadata related to the chat badges in the `badges` tag.
    ///
    /// Currently this is used only for `subscriber`, to indicate the exact number of months
    /// the user has been a subscriber. This number is finer grained than the version number in
    /// badges. For example, a user who has been a subscriber for 45 months would have a
    /// `badge_info` value of 45 but might have a `badges` `version` number for only 3 years.
    // pub badge_info: Vec<Badge>,
    /// List of badges that should be displayed alongside the message.
    // pub badges: Vec<Badge>,
    roles: UserRoles,

    /// If present, specifies how many bits were cheered with this message.
    pub bits: Option<u64>,

    /// If present, specifies the color that the user's name should be displayed in. A value
    /// of `None` here signifies that the user has not picked any particular color.
    /// Implementations differ on how they handle this, on the Twitch website users are assigned
    /// a pseudorandom but consistent-per-user color if they have no color specified.
    pub name_color: Option<RGBColor>,

    /// A list of emotes in this message. Each emote replaces a part of the `message_text`.
    /// These emotes are sorted in the order that they appear in the message.
    pub emotes: Vec<Emote>,
}
