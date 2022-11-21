use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
};

use anyhow::Result;
use once_cell::sync::OnceCell;
use rand::Rng;
use subd_types::{Event, RaffleStatus, UserID, UserRoles};
use tokio::sync::broadcast;
use tracing::info;

#[derive(Debug, Default, PartialEq, Eq)]
enum State {
    #[default]
    Closed,
    Open,
    Complete,
}

#[derive(Debug, Default)]
pub struct RaffleState {
    state: State,
    title: String,
    users: HashMap<UserID, String>,
    entries: Vec<UserID>,
}

impl RaffleState {
    fn enter_user(
        &mut self,
        user_id: &UserID,
        user_name: &str,
        user_roles: &UserRoles,
    ) -> Result<bool> {
        if self.state != State::Open {
            return Ok(false);
        }

        if self.users.contains_key(&user_id) {
            return Ok(false);
        }

        self.users.insert(user_id.clone(), user_name.to_string());

        let votes = 1 + user_roles.support_amount().ceil() as u32;
        (0..votes).for_each(|_| {
            self.entries.push(user_id.clone());
        });

        Ok(true)
    }

    fn resume(&mut self) {
        self.state = State::Open;
    }

    fn reset(&mut self) {
        self.title = String::default();
        self.users = HashMap::default();
        self.entries = Vec::default();
    }

    fn open(&mut self, title: String) {
        self.reset();
        self.state = State::Open;
        self.title = title;
    }

    fn close(&mut self) {
        self.state = State::Closed;
    }

    fn start(&mut self, max_winners: usize) -> HashSet<UserID> {
        self.state = State::Complete;

        let max_winners = std::cmp::min(self.users.len(), max_winners);
        let mut rng = rand::thread_rng();

        let mut result = HashSet::default();
        while result.len() < max_winners {
            let num = rng.gen_range(0..self.entries.len());
            result.insert(self.entries[num].clone());
        }

        result
    }

    fn to_status(&self) -> Option<RaffleStatus> {
        if self.state == State::Closed {
            return Some(RaffleStatus::Disabled);
        }

        if self.state == State::Complete {
            return None;
        }

        let mut entries: HashMap<String, usize> = HashMap::new();
        for user_id in self.entries.iter() {
            let user_name = self.users.get(user_id).unwrap();
            entries
                .entry(user_name.clone())
                .and_modify(|counter| *counter += 1)
                .or_insert(1);
        }

        return Some(RaffleStatus::Ongoing {
            title: self.title.clone(),
            entries,
        });
    }
}

fn raffle_status() -> &'static Mutex<RaffleState> {
    static INSTANCE: OnceCell<Mutex<RaffleState>> = OnceCell::new();
    INSTANCE.get_or_init(|| {
        // Not sure what I changed for this to happen
        Mutex::new(RaffleState::default())
    })
}

#[tracing::instrument(skip(tx, user_id, contents))]
pub async fn handle(
    tx: &broadcast::Sender<Event>,
    user_id: &UserID,
    user_name: &str,
    contents: &str,
) -> Result<()> {
    todo!("raffle::handle");

    // TODO: Figure out how to let people enter with emotes in the raffle message
    // cause that's just a twitch necessity. it'd also be funny if we could show the
    // emote that they entered with when they win

    // let mut conn = subd_db::get_handle().await;
    // let status = raffle_status();
    // let user_roles = subd_db::get_user_roles(&mut conn, &user_id).await?;
    //
    // if contents == "!raffle" {
    //     if !status.lock().unwrap().enter_user(
    //         user_id,
    //         user_name,
    //         &user_roles,
    //     )? {
    //         return Ok(());
    //     }
    // } else {
    //     let splitmsg = contents
    //         .split(" ")
    //         .map(|s| s.to_string())
    //         .collect::<Vec<String>>();
    //
    //     anyhow::ensure!(splitmsg.len() >= 2, "not a valid raffle command");
    //
    //     let is_mod = user_roles.is_moderator();
    //     match (is_mod, splitmsg[1].as_str()) {
    //         (true, "start") => {
    //             let count =
    //                 splitmsg.get(2).unwrap_or(&"1".to_string()).parse()?;
    //             let winners = status.lock().unwrap().start(count);
    //             println!("=======================================");
    //             println!("WINNERS: {:?}", winners);
    //             println!("=======================================");
    //
    //             let mut users = HashSet::new();
    //             for winner in winners.iter() {
    //                 let user = subd_db::get_twitch_user_from_user_id(
    //                     &mut conn, *winner,
    //                 )
    //                 .await?;
    //                 users.insert(user.display_name);
    //             }
    //             tx.send(Event::RaffleStatus(RaffleStatus::Winner { users }))?;
    //
    //             return Ok(());
    //         }
    //
    //         (true, "resume") => {
    //             status.lock().unwrap().resume();
    //         }
    //         (true, "reset") => {
    //             status.lock().unwrap().reset();
    //         }
    //         (true, "open") => {
    //             anyhow::ensure!(splitmsg.len() >= 3, "open requires a title");
    //             status.lock().unwrap().open(splitmsg[2..].join(" "));
    //         }
    //         (true, "close") => {
    //             status.lock().unwrap().close();
    //         }
    //         (_, "enter") => {
    //             if !status.lock().unwrap().enter_user(
    //                 user_id,
    //                 user_name,
    //                 &user_roles,
    //             )? {
    //                 return Ok(());
    //             }
    //             // println!("User Entered: {:?}", entered);
    //             // TODO: Could probably send some feedback about whether you have
    //             // entered the raffle or not
    //         }
    //         _ => {
    //             // TODO: might want to error here and say that this wasn't a real command
    //             info!("Invalid raffle command");
    //             return Ok(());
    //         }
    //     };
    // }
    //
    // let status = status.lock().unwrap();
    // if let Some(raffle_status) = status.to_status() {
    //     tx.send(Event::RaffleStatus(raffle_status))?;
    // }
    //
    // let user_count = status.users.len();
    // let entry_count = status.entries.len();
    //
    // info!(
    //     title = %status.title,
    //     stat = ?status.state,
    //     user_count = user_count,
    //     entry_count = entry_count,
    //     "raffle status:"
    // );
    //
    // Ok(())
}
