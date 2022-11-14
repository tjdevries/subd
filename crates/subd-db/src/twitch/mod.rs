use anyhow::Result;
use subd_types::UserID;

pub struct TwitchSubscriptionEvent {}

impl TwitchSubscriptionEvent {
    pub fn save(self) -> Result<()> {
        Ok(())
    }

    pub fn from_user(_user_id: &UserID) -> Option<Self> {
        None
    }
}
