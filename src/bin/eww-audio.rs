use alsa::{
    mixer::{SelemChannelId, SelemId},
    Mixer,
};
use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};

fn main() {
    let out = Command::new("alsactl")
        .arg("monitor")
        .arg("default")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap()
        .stdout
        .unwrap();

    let buffer = BufReader::new(out);

    for _ in buffer.lines() {
        if let Some((vol, playback_mute, capture_mute)) = status() {
            let level = match vol {
                n if n > 50 => "high",
                n if n > 25 => "medium",
                _ => "low",
            };
            println!(
                r#"{{ "vol": {}, "playback_mute": {}, "capture_mute": {}, "level": "{}" }}"#,
                vol, playback_mute, capture_mute, level
            );
        }
    }
}

fn status() -> Option<(i64, bool, bool)> {
    let mixer = Mixer::new("default", true).ok()?;

    let selem_playback = mixer.find_selem(&SelemId::new("Master", 0))?;
    let (_, max) = selem_playback.get_playback_volume_range();
    let vol = selem_playback
        .get_playback_volume(SelemChannelId::mono())
        .ok()?
        * 100
        / max;

    let playback_mute = selem_playback
        .get_playback_switch(SelemChannelId::mono())
        .ok()?
        == 0;

    let selem_capture = mixer.find_selem(&SelemId::new("Capture", 0))?;
    let capture_mute = selem_capture
        .get_capture_switch(SelemChannelId::mono())
        .ok()?
        == 0;

    Some((vol, playback_mute, capture_mute))
}
