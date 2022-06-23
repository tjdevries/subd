use serde::{Deserialize, Serialize};
pub use twitch_api2::pubsub::channel_subscriptions::ChannelSubscribeEventsV1Reply;
use twitch_irc::message::PrivmsgMessage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    // Info
    TwitchChatMessage(PrivmsgMessage),
    TwitchSubscriptionCount(usize),
    TwitchSubscription(TwitchSubscriptionEvent),
    GithubSponsorshipEvent,

    // Requests
    RequestTwitchSubCount,

    // Control
    Shutdown,
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

// const MY_CHANNEL: UserId = 114257969;

pub fn get_nyx_sub() -> TwitchSubscriptionEvent {
    let message = r##"
{
    "benefit_end_month": 11,
    "user_name": "nyxkrage",
    "display_name": "NyxKrage",
    "channel_name": "teej_dv",
    "user_id": "1234",
    "channel_id": "27620241",
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
}
"##;

    let subscription = serde_json::from_str(message).unwrap();
    TwitchSubscriptionEvent { subscription }
}

pub fn get_prime_sub() -> TwitchSubscriptionEvent {
    let message = r##"
{
    "benefit_end_month": 11,
    "user_name": "theprimeagen",
    "display_name": "ThePrimeagen",
    "channel_name": "teej_dv",
    "user_id": "1234",
    "channel_id": "27620241",
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
}
"##;

    let subscription = serde_json::from_str(message).unwrap();
    TwitchSubscriptionEvent { subscription }
}
