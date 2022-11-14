use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
};

use serde::{Deserialize, Serialize};
use twitch_api2::pubsub::channel_points::Redemption;
pub use twitch_api2::pubsub::channel_subscriptions::ChannelSubscribeEventsV1Reply;
use twitch_irc::message::PrivmsgMessage;

pub mod consts;
pub mod twitch;

pub type UserID = i64;
pub type TwitchUserID = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    // Info
    TwitchChatMessage(PrivmsgMessage),
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
    Request { msg: PrivmsgMessage },
    Start { display_name: String },
    Finish { display_name: String, success: bool },
    Format { sender: String },
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

pub fn get_nyx_sub() -> TwitchSubscriptionEvent {
    let twitch_username = consts::get_twitch_broadcaster_username();
    let twitch_channel_id = consts::get_twitch_broadcaster_channel_id();
    let message = serde_json::json!(
    {
        "channel_name": twitch_username,
        "benefit_end_month": 11,
        "user_name": "nyxkrage",
        "display_name": "NyxKrage",
        "channel_id": twitch_channel_id,
        "user_id": "1234",
        "time": "2020-10-20T22:17:43.242793831Z",
        "sub_message": {
        "message": "You are my favorite streamer",
        "emotes": null,
        },
        "sub_plan": "1000",
        "sub_plan_name": "Channel Subscription (emilgardis)",
        "months": 0,
        "cumulative_months": 1,
        "context": "sub",
        "is_gift": false,
        "multi_month_duration": 0,
    });

    let subscription = serde_json::from_value(message).unwrap();
    TwitchSubscriptionEvent { subscription }
}

pub fn get_prime_sub() -> TwitchSubscriptionEvent {
    let twitch_username = consts::get_twitch_broadcaster_username();
    let twitch_channel_id = consts::get_twitch_broadcaster_channel_id();
    let message = serde_json::json!(
    {
        "benefit_end_month": 11,
        "user_name": "theprimeagen",
        "display_name": "ThePrimeagen",
        "channel_name": twitch_username,
        "channel_id": twitch_channel_id,
        "user_id": "1234",
        "time": "2020-10-20T22:17:43.242793831Z",
        "sub_message": {
            "message": "You are my favorite streamer",
            "emotes": null
        },
        "sub_plan": "1000",
        "sub_plan_name": "Channel Subscription (emilgardis)",
        "months": 0,
        "cumulative_months": 1,
        "context": "sub",
        "is_gift": false,
        "multi_month_duration": 0
    });

    let subscription = serde_json::from_value(message).unwrap();
    TwitchSubscriptionEvent { subscription }
}
