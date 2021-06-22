use dbus::{
    blocking::Connection,
    message::MatchRule,
    strings::{Member, Path},
};

pub fn add_conn_match(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let mut props_rule = MatchRule::new();
    props_rule.member = Some(Member::new("PropertiesChanged")?);
    props_rule.path = Some(Path::new("/org/mpris/MediaPlayer2")?);

    conn.add_match_no_cb(&props_rule.match_str())?;

    Ok(())
}
