use dbus::blocking::Connection;
use discord_rich_presence::{new_client, DiscordIpc};
use serde_json::json;
use std::{convert::From, error::Error, thread::sleep, time::Duration};
use urlencoding::decode;

mod helpers;
mod monitor;

const DISCORD_APP_ID: &str = "831641858643460106";
// Players that have an icon supported
const KNOWN_PLAYERS: [&str; 3] = ["vlc", "strawberry", "audacious"];

fn main() -> Result<(), Box<dyn Error>> {
    let conn = Connection::new_session()?;
    let mut ipc = new_client(&DISCORD_APP_ID)?;
    monitor::add_conn_match(&conn)?;

    loop {
        match ipc.get_valid_path()? {
            Some(_) => {
                ipc.connect()?;
                break;
            }
            None => continue,
        };
    }

    let mut has_closed = false;
    loop {
        if conn.process(Duration::from_millis(5000))? {
            // Defer presence update so timestamps on seek are accurate
            sleep(Duration::from_millis(500));
            update_presence(&conn, &mut ipc)?;
            has_closed = false;
        } else if helpers::get_player(&conn)?.is_none()
            && !has_closed
            && ipc.get_valid_path()?.is_some()
        {
            ipc.reconnect()?;
            has_closed = true;
        }
    }
}

fn update_presence(
    conn: &Connection,
    ipc: &mut impl DiscordIpc,
) -> Result<(), Box<dyn std::error::Error>> {
    let player = match helpers::get_player(&conn)? {
        Some(val) => val,
        None => return Ok(()),
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
    let mut large_image: String = String::from("large-icon");
    if KNOWN_PLAYERS.contains(&player.as_str()) {
        large_image += &format!("-{}", player).as_str();
    } else {
        large_image += "-unknown";
    }

    let mut payload = json!({
        "state": state.chars().take(128).collect::<String>(),
        "details": data["title"].chars().take(128).collect::<String>(),
        "timestamps": {
            "end": helpers::get_end_time(&proxy)?
        },
        "assets": {
            "large_text": format!("Listening with {}", &player),
            "large_image": large_image,
            "small_text": "Playing",
            "small_image": "status-playing"
        }
    });

    if !helpers::is_playing(&player, &conn)? {
        let data = payload.as_object_mut().unwrap();
        data.remove("timestamps");
        data["assets"]
            .as_object_mut()
            .unwrap()
            .insert(String::from("small_text"), "Paused".into());
        data["assets"]
            .as_object_mut()
            .unwrap()
            .insert(String::from("small_image"), "status-paused".into());
        payload = data.clone().into();
    }

    if ipc.set_activity(payload).is_err() && ipc.get_valid_path()?.is_some() {
        ipc.reconnect()?
    }

    Ok(())
}
