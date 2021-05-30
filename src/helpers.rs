use chrono::{Duration, Local};
use dbus::{
    arg,
    blocking::{stdintf::org_freedesktop_dbus::Properties, Connection, Proxy},
};
use std::{collections::HashMap, time::Duration as StdDuration};

const MPRIS_PREFIX: &str = "org.mpris.MediaPlayer2.";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn is_playing(player: &str, conn: &Connection) -> Result<bool> {
    let proxy = conn.with_proxy(
        format!("{}{}", MPRIS_PREFIX, player),
        "/org/mpris/MediaPlayer2",
        StdDuration::from_millis(5000),
    );

    let status: String = proxy.get("org.mpris.MediaPlayer2.Player", "PlaybackStatus")?;

    match status.as_str() {
        "Playing" => Ok(true),
        "Paused" => Ok(false),
        "Stopped" => Ok(false),
        _ => panic!("Somehow got a status other than \"Playing\", \"Paused\", or \"Stopped\""),
    }
}

pub fn get_player(conn: &Connection) -> Result<Option<String>> {
    let proxy = conn.with_proxy("org.freedesktop.DBus", "/", StdDuration::from_millis(5000));

    let (mut names,): (Vec<String>,) =
        proxy.method_call("org.freedesktop.DBus", "ListNames", ())?;

    let mut player = String::new();
    for name in names
        .iter_mut()
        .filter_map(|n| n.as_str().strip_prefix(MPRIS_PREFIX))
    {
        player = name.to_string();
        match is_playing(name, &conn)? {
            true => break,
            false => continue,
        };
    }

    if player.is_empty() {
        return Ok(None);
    }

    Ok(Some(player))
}

pub fn get_end_time(proxy: &Proxy<&Connection>) -> Result<i64> {
    let metadata: arg::PropMap = proxy.get("org.mpris.MediaPlayer2.Player", "Metadata")?;
    let position: i64 = proxy.get("org.mpris.MediaPlayer2.Player", "Position")?;

    let len_data: Option<&i64> = arg::prop_cast(&metadata, "mpris:length");
    let len: i64 = match len_data {
        Some(val) => *val,
        None => 1,
    };
    let remaining = Duration::microseconds(len - position);

    let end = Local::now() + remaining;
    Ok(end.timestamp())
}

pub fn get_data(proxy: &Proxy<&Connection>) -> Result<HashMap<String, String>> {
    let mut data: HashMap<String, String> = HashMap::new();

    let metadata: arg::PropMap = proxy.get("org.mpris.MediaPlayer2.Player", "Metadata")?;

    for key in ["xesam:title", "xesam:album", "xesam:url"].iter() {
        let value_data: Option<&String> = arg::prop_cast(&metadata, key);
        let value: String = match value_data {
            Some(val) => val.to_string(),
            None => format!("Unknown {}", key.strip_prefix("xesam:").unwrap()),
        };

        data.insert(key.strip_prefix("xesam:").unwrap().to_string(), value);
    }

    let artists: Option<&Vec<String>> = arg::prop_cast(&metadata, "xesam:artist");
    let artist: String = match artists {
        Some(val) => val.join(" "),
        None => "Unknown artist".to_string(),
    };
    data.insert(String::from("artist"), artist);

    Ok(data)
}
