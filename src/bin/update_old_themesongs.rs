use anyhow::Result;

// -- new columsn: url, start, end
// --
// -- THIS ISN"T RIGHT
// SELECT DISTINCT TWITCH_CHAT_HISTORY.user_id, msg, timestamp
//   FROM TWITCH_CHAT_HISTORY
//     INNER JOIN USER_THEME_SONGS on USER_THEME_SONGS.user_id = TWITCH_CHAT_HISTORY.user_id
//   WHERE msg LIKE '!themesong https%'
//     GROUP BY TWITCH_CHAT_HISTORY.user_id
//     ORDER BY timestamp DESC, TWITCH_CHAT_HISTORY.user_id DESC;
// 
// -- select * from USER_THEME_SONGS;
// 
// -- SELECT COUNT(*) FROM TWITCH_CHAT_HISTORY WHERE user_id = 138;
#[tokio::main]
async fn main() -> Result<()> {
    // Pre: Update all the themesong code to save the url, start and end times
    //
    // Get all the themesong requests
    // Insert info into DB
    // update db to have these be NOT NULL because NOT NULL is good for living
    Ok(())
}
