use dbus::blocking::Connection;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use std::{
    error::Error,
    thread::sleep,
    time::{Duration, Instant},
};

mod helpers;
mod monitor;

const DISCORD_APP_ID: &str = "831641858643460106";

fn main() -> Result<(), Box<dyn Error>> {
    let conn = Connection::new_session()?;
    let mut ipc = DiscordIpcClient::new(DISCORD_APP_ID)?;
    monitor::add_conn_match(&conn)?;

    loop {
        if ipc.connect().is_ok() {
            break;
        }
        sleep(Duration::from_secs(1));
        continue;
    }

    let mut has_closed = false;
    let mut last_updated = Instant::now();
    loop {
        if conn.process(Duration::from_secs(5))? {
            // Defer presence update so timestamps on seek are accurate
            sleep(Duration::from_millis(1000));
            update_presence(&conn, &mut ipc, &mut last_updated)?;
            has_closed = false;
            continue;
        } else if Instant::now().duration_since(last_updated).as_secs() >= 30 {
            if update_presence(&conn, &mut ipc, &mut last_updated).is_ok() {
                has_closed = false;
                continue;
            }
        } else if helpers::get_player(&conn)?.is_none() && !has_closed && ipc.reconnect().is_ok() {
            has_closed = true;
            continue;
        }
        sleep(Duration::from_secs(1));
    }
}

fn update_presence(
    conn: &Connection,
    ipc: &mut impl DiscordIpc,
    last_updated: &mut Instant,
) -> Result<(), Box<dyn std::error::Error>> {
    let player = match helpers::get_player(conn)? {
        Some(val) => val,
        None => return Ok(()),
    };

    let proxy = conn.with_proxy(
        format!("org.mpris.MediaPlayer2.{}", &player),
        "/org/mpris/MediaPlayer2",
        Duration::from_millis(5000),
    );
    let data = helpers::get_data(&proxy)?;

    let state: String = format!("{} - {}", data["artist"], data["album"])
        .chars()
        .take(128)
        .collect();
    let details: String = data["title"].chars().take(128).collect();
    let large_text: String = format!("Listening with {}", &player);
    let assets = activity::Assets::new()
        .large_text(large_text.as_str())
        .large_image("logo")
        .small_text("Playing")
        .small_image("playing");
    let mut payload = activity::Activity::new()
        .state(&state)
        .details(&details)
        .assets(assets.clone());

    if helpers::is_playing(&player, conn)? {
        let end_time = helpers::get_end_time(&proxy)?;
        if let Some(end_time) = end_time {
            payload = payload.timestamps(activity::Timestamps::new().end(end_time as i64));
        };
    } else {
        let assets = assets.clone().small_text("Paused").small_image("paused");
        payload = payload.assets(assets);
    }

    if Instant::now().duration_since(*last_updated).as_secs() >= 5 {
        if ipc.set_activity(payload).is_err() {
            ipc.reconnect().ok();
        } else {
            *last_updated = Instant::now();
        }
    }

    Ok(())
}
