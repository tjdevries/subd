use std::{collections::HashSet, sync::Mutex};

use anyhow::Result;
use once_cell::sync::OnceCell;
use subd_types::{Event, UserID, UserRoles};
use tokio::sync::broadcast;

#[derive(Debug, Default)]
pub struct RaffleStatus {
    users: HashSet<UserID>,
    entries: Vec<UserID>,
}

impl RaffleStatus {
    fn enter_user(&mut self, user_id: &UserID, user_roles: &UserRoles) -> Result<bool> {
        if self.users.contains(&user_id) {
            return Ok(false);
        }

        self.users.insert(*user_id);

        let votes = 1 + user_roles.support_amount().ceil() as u32;
        (1..votes).for_each(|_| {
            self.entries.push(*user_id);
        });

        Ok(true)
    }
}

fn raffle_status() -> &'static Mutex<RaffleStatus> {
    static INSTANCE: OnceCell<Mutex<RaffleStatus>> = OnceCell::new();
    INSTANCE.get_or_init(|| {
        // Not sure what I changed for this to happen
        Mutex::new(RaffleStatus::default())
    })
}

pub async fn handle(_tx: &broadcast::Sender<Event>, user_id: &UserID) -> Result<()> {
    let status = raffle_status();

    let mut conn = subd_db::get_handle().await;
    let user_roles = subd_db::get_user_roles(&mut conn, &user_id).await?;
    status.lock().unwrap().enter_user(user_id, &user_roles)?;
    dbg!(status);

    Ok(())
}
