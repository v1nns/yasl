//! Yet another spotifyd listener
//! Reads song metadata using DBus interface, buffers it and
//! print to stdout using Polybar formatting style.

use dbus::{arg, blocking::Connection};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

// The time between polling cycles, in other words, how long to wait
// before polling DBus for spotifyd's metadata again.
const POLLING_DELAY: Duration = Duration::from_secs(4);

// String formatting based on Polybar-style. For more info, check:
// https://github.com/polybar/polybar/wiki/Formatting
fn format_output(status: &str, artist: &str, title: &str) -> String {
    let tmp;
    let output: String = match &status as &str {
        "Playing" => {
            tmp = format!(" {} - {}", artist, title);
            tmp
        }
        "Paused" => {
            // TODO: transform into a parameter
            tmp = format!("%{{F#8A8B9B}} {} - {}%{{F-}}", artist, title);
            tmp
        }
        _ => {
            // Occurred some error
            tmp = String::new();
            tmp
        }
    };

    String::from_utf8_lossy(&output.into_bytes()).into()
}

// Retrieve information from spotifyd using DBus interface
fn get_current_status() -> String {
    // Connect to server and create a proxy object. A proxy implements several
    // interfaces, in this case we'll use OrgFreedesktopDBusProperties, which
    // allows us to call "get".
    let conn = Connection::new_session().expect("Could not create a new session to DBus");
    let proxy = conn.with_proxy(
        "org.mpris.MediaPlayer2.spotifyd",
        "/org/mpris/MediaPlayer2",
        Duration::from_millis(3000),
    );
    use dbus::blocking::stdintf::org_freedesktop_dbus::Properties;

    // Get playback status
    let status: String = proxy
        .get("org.mpris.MediaPlayer2.Player", "PlaybackStatus")
        .expect("Could not retrieve playback status");

    // In case it is not playing, just return that is not playing, right?
    if status.contains("Stopped") {
        return String::new();
    }

    // Get music metadata
    let metadata: arg::PropMap = proxy
        .get("org.mpris.MediaPlayer2.Player", "Metadata")
        .expect("Could not retrieve music metadata");

    // Get artists and title from metadata
    let artists: &Vec<String> =
        arg::cast(&metadata["xesam:artist"].0).expect("Could not extract artists info");
    let title: &String =
        arg::cast(&metadata["xesam:title"].0).expect("Could not extract title info");

    // For a bigger purpose, gonna use only the first artist name, sorry guys
    let artist: &String = artists.first().expect("Could not extract first artist");

    // Returns a formatted string taking into account the playback status
    format_output(&status, &artist, &title)
}

fn main() {
    let mut time: Duration = Duration::from_secs(2);
    let mut buffer: String = String::new();
    let (tx, rx) = mpsc::channel();

    // Update the track metadata in a separate thread, in order to mitigate
    // DBus errors from attempts to extract info
    thread::spawn(move || loop {
        tx.send(get_current_status()).unwrap();
        thread::sleep(POLLING_DELAY);
    });

    // Endless loop through the eternity (or at the least, until this process dies)
    loop {
        // Process output only if got DBus current status successfully.
        // Otherwise, previously received output is reused.
        if let Ok(output) = rx.try_recv() {
            if output != buffer {
                // Read changed output to buffer
                buffer = output;

                // Reset default sleep time
                time = Duration::from_secs(2)
            }
        }
        // Still gotta figure out the best way to increase sleep time
        // else {
        //     // In case we could not get info from the other thread,
        //     // keep increasing time to avoid any slow down
        //     time += Duration::from_secs(2);
        // }

        // Echo buffer to the outside world
        println!("{}", &buffer);

        // Alright, it is time to sleep!
        thread::sleep(time);
    }
}
