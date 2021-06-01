use dbus::blocking::Connection;
use discord_rich_presence::{new_client, DiscordIpc};
use serde_json::json;
use std::{error::Error, thread::sleep, time::Duration};
use urlencoding::decode;

mod helpers;
mod monitor;

const DISCORD_APP_ID: &str = "831641858643460106";

fn main() -> Result<(), Box<dyn Error>> {
    let conn = Connection::new_session()?;
    let mut ipc = new_client(&DISCORD_APP_ID)?;
    monitor::add_conn_match(&conn)?;

    loop {
        if ipc.connect().is_ok() {
            break;
        } else {
            sleep(Duration::from_secs(5));
            continue;
        }
    }

    let mut has_closed = false;
    loop {
        if conn.process(Duration::from_millis(5000))? {
            // Defer presence update so timestamps on seek are accurate
            sleep(Duration::from_millis(500));
            update_presence(&conn, &mut ipc)?;
            has_closed = false;
        } else if helpers::get_player(&conn)?.is_none() && !has_closed && ipc.reconnect().is_ok() {
            has_closed = true;
        } else {
            sleep(Duration::from_secs(5));
        }
    }
}

fn update_presence(
    conn: &Connection,
    ipc: &mut impl DiscordIpc,
) -> Result<(), Box<dyn std::error::Error>> {
    let player = match helpers::get_player(&conn)? {
        Some(val) => val,
        None => {
            sleep(Duration::from_secs(5));
            return Ok(());
        }
    };

    let proxy = conn.with_proxy(
        format!("org.mpris.MediaPlayer2.{}", &player),
        "/org/mpris/MediaPlayer2",
        Duration::from_millis(5000),
    );
    let data = helpers::get_data(&proxy)?;

    let uri = decode(&data["url"])?;
    match uri.strip_prefix("file://") {
        Some(_) => (),
        None => return Ok(()),
    };

    let state = format!("{} - {}", data["artist"], data["album"]);
    let mut payload = json!({
        "state": state.chars().take(128).collect::<String>(),
        "details": data["title"].chars().take(128).collect::<String>(),
        "timestamps": {
            "end": helpers::get_end_time(&proxy)?
        },
        "assets": {
            "large_text": format!("Listening with {}", &player),
            "large_image": "logo",
        }
    });

    if !helpers::is_playing(&player, &conn)? {
        let data = payload.as_object_mut().unwrap();
        data.remove("timestamps");
        payload = data.clone().into();
    }

    if ipc.set_activity(payload).is_err() {
        ipc.reconnect().ok();
    }

    Ok(())
}
