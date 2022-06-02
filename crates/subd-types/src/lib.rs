use serde::{Deserialize, Serialize};
use twitch_irc::message::PrivmsgMessage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    // Info
    TwitchChatMessage(PrivmsgMessage),
    TwitchSubscriptionCount(usize),
    TwitchSubscription,
    GithubSponsorshipEvent,

    // Requests
    RequestTwitchSubCount,

    // Control
    Shutdown,
}
