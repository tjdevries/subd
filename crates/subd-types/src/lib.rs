use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
};

use serde::{Deserialize, Serialize};
use twitch_api2::pubsub::channel_points::Redemption;
pub use twitch_api2::pubsub::channel_subscriptions::ChannelSubscribeEventsV1Reply;

#[cfg(feature = "sql")]
pub mod consts;

pub mod twitch;

// TODO: How do we derive this better?
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct UserID(pub uuid::Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct TwitchUserID(pub String);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(
    feature = "sql",
    derive(sqlx::Type),
    sqlx(type_name = "user_platform", rename_all = "SCREAMING_SNAKE_CASE")
)]
pub enum UserPlatform {
    Twitch,
    Youtube,
    Github,
    Discord,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserMessage {
    pub user_id: UserID,
    pub user_name: String,
    pub roles: UserRoles,
    pub platform: UserPlatform,
    pub contents: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ElevenLabsRequest {
    pub voice_text: String,
    pub message: String,
    pub username: String,
    pub voice: Option<String>,

    pub reverb: bool,

    // I know it's not actually a string, but we aren't doing any math on it
    pub pitch: Option<String>,
    pub stretch: Option<String>,
    
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformOBSTextRequest {
    // pub voice: String,
    // pub voice_text: String,
    pub text_source: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerHotkeyRequest {
    pub hotkey: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamCharacterRequest {
    pub source: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceVisibilityRequest {
    pub scene: String,
    pub source: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkyboxRequest {
    pub msg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    /// The primary Message event. Should be used wherever possible.
    UserMessage(UserMessage),

    ElevenLabsRequest(ElevenLabsRequest),
    TransformOBSTextRequest(TransformOBSTextRequest),
    StreamCharacterRequest(StreamCharacterRequest),
    SourceVisibilityRequest(SourceVisibilityRequest),
    SkyboxRequest(SkyboxRequest),
    TriggerHotkeyRequest(TriggerHotkeyRequest),

    /// TwitchChatMessage is only used for messages
    /// that are explicitly for twitch related items. In general
    /// you should use UserMessage instead. This will handle messages
    /// from any service.
    TwitchChatMessage(twitch::TwitchMessage),

    TwitchSubscriptionCount(usize),
    TwitchSubscription(TwitchSubscriptionEvent),
    GithubSponsorshipEvent,

    // OBS
    ObsSetScene {
        scene: String,
    },

    // UserEvents
    ThemesongDownload(ThemesongDownload),
    ThemesongPlay(ThemesongPlay),

    // Requests
    RequestTwitchSubCount,
    RequestTwitchMessage(String),
    TwitchChannelPointsRedeem(Redemption),

    /// Backend Only
    LunchBytesVoting(LunchBytesCommand),

    /// Backend -> Front Status message
    LunchBytesStatus(LunchBytesStatus),

    /// Raffle Stuff
    RaffleStatus(RaffleStatus),

    // Control
    Shutdown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RaffleStatus {
    Disabled,
    Ongoing {
        title: String,
        entries: HashMap<String, usize>,
    },
    Winner {
        users: HashSet<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LunchBytesCommand {
    VoteUp { id: u32, weight: u32 },
    VoteDown { id: u32, weight: u32 },
    VoteDuplicate { id: u32 },
    Suggest { text: String },
    Show,
    Hide,
    MarkComplete,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LunchBytesStatus {
    pub enabled: bool,
    pub topics: Vec<LunchBytesTopic>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LunchBytesTopic {
    pub id: u32,
    pub text: String,
    pub votes: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ThemesongDownload {
    Request { msg: UserMessage },
    Start { display_name: String },
    Finish { display_name: String, success: bool },
    Format { sender: String },
}

impl ThemesongDownload {
    pub fn format<T>(sender: T) -> Event
    where
        T: ToString + 'static,
    {
        Event::ThemesongDownload(ThemesongDownload::Format {
            sender: sender.to_string(),
        })
    }

    pub fn finish<T>(display_name: T, success: bool) -> Event
    where
        T: ToString + 'static,
    {
        Event::ThemesongDownload(ThemesongDownload::Finish {
            display_name: display_name.to_string(),
            success,
        })
    }

    pub fn start<T>(display_name: T) -> Event
    where
        T: ToString + 'static,
    {
        Event::ThemesongDownload(ThemesongDownload::Start {
            display_name: display_name.to_string(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ThemesongPlay {
    Start {
        user_id: UserID,
        display_name: String,
    },

    Finish {
        user_id: UserID,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubUser {
    pub id: String,
    pub login: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TwitchSubLevel {
    Unknown,
    Tier1,
    Tier2,
    Tier3,
}

// cafce25: put PartialOrd into the list in derive
// cafce25: Ord is ordering
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    GithubSponsor { tier: String },
    TwitchMod,
    TwitchVIP,
    TwitchFounder,
    TwitchSub(TwitchSubLevel),
    TwitchStaff,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct UserRoles {
    pub roles: HashSet<Role>,
}

impl UserRoles {
    pub fn add_role(&mut self, role: Role) -> () {
        self.roles.insert(role);
    }

    pub fn is_github_sponsor(&self) -> bool {
        self.roles
            .iter()
            .find(|r| matches!(r, Role::GithubSponsor { tier: _ }))
            .is_some()
    }

    pub fn is_twitch_mod(&self) -> bool {
        self.roles.contains(&Role::TwitchMod)
    }

    pub fn is_twitch_vip(&self) -> bool {
        self.roles.contains(&Role::TwitchVIP)
    }

    pub fn is_twitch_founder(&self) -> bool {
        self.roles.contains(&Role::TwitchFounder)
    }

    pub fn is_twitch_staff(&self) -> bool {
        self.roles.contains(&Role::TwitchStaff)
    }

    pub fn is_twitch_sub(&self) -> bool {
        self.roles
            .iter()
            .find(|r| matches!(r, Role::TwitchSub(_)))
            .is_some()
    }
}

impl Display for UserRoles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut truths = vec![];

        if self.is_github_sponsor() {
            truths.push("github_sponsor");
        }
        if self.is_twitch_mod() {
            truths.push("twitch_mod");
        }
        if self.is_twitch_vip() {
            truths.push("twitch_vip");
        }
        if self.is_twitch_founder() {
            truths.push("twitch_founder");
        }
        if self.is_twitch_sub() {
            truths.push("twitch_sub");
        }
        if self.is_twitch_staff() {
            truths.push("twitch_staff");
        }

        write!(f, "{}", truths.join(","))
    }
}

impl UserRoles {
    pub fn is_moderator(&self) -> bool {
        self.is_twitch_mod()
    }

    pub fn support_amount(&self) -> f64 {
        let mut amount = 0.;

        // TODO: Should get sponsor tier
        if self.is_github_sponsor() {
            amount += 5.;
        }

        // TODO: Should get twitch sub tier
        if self.is_twitch_sub() {
            amount += 2.5;
        }

        amount
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TwitchSubscriptionEvent {
    subscription: ChannelSubscribeEventsV1Reply,
}

impl From<ChannelSubscribeEventsV1Reply> for TwitchSubscriptionEvent {
    fn from(subscription: ChannelSubscribeEventsV1Reply) -> Self {
        Self { subscription }
    }
}

impl TwitchSubscriptionEvent {
    pub fn display_name(&self) -> String {
        match &self.subscription {
            ChannelSubscribeEventsV1Reply::Sub(sub) => &sub.display_name,
            ChannelSubscribeEventsV1Reply::ReSub(sub) => &sub.display_name,
            ChannelSubscribeEventsV1Reply::SubGift(sub) => &sub.display_name,
            ChannelSubscribeEventsV1Reply::ResubGift(sub) => &sub.display_name,
            ChannelSubscribeEventsV1Reply::ExtendSub(sub) => &sub.display_name,
            _ => unimplemented!(),
        }
        .to_string()
    }
}
