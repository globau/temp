use std::env;
use std::ffi::OsString;
use std::io::{self, Cursor, Write};
use std::process::exit;
use std::thread::sleep;
use std::time::Instant;

use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use kira::{AudioManager, AudioManagerSettings, DefaultBackend};
use nix::unistd::{fork, ForkResult};

static DING_SAMPLE: &'static [u8] = include_bytes!("../ogg/ding.ogg");
static ERROR_SAMPLE: &'static [u8] = include_bytes!("../ogg/error.ogg");

const USAGE: &str = r#"Usage: ding [command]

Without a command, `ding` plays a happy ding sound.

With a command, `ding` will execute that command and play a ding if the
command completed without error, otherwise an error sound will be played.

If the command took more than five seconds to execute, the elapsed time will be
shown."#;

fn play_audio_and_wait(return_code: i32) {
    let data = if return_code == 0 {
        DING_SAMPLE
    } else {
        ERROR_SAMPLE
    };
    if let Ok(mut manager) = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()) {
        if let Ok(sound) = StaticSoundData::from_cursor(Cursor::new(data)) {
            let sound_data = sound.with_settings(StaticSoundSettings::default());
            let duration = sound_data.duration();
            let _ = manager.play(sound_data);
            sleep(duration);
        }
    }
}

fn main() {
    let mut return_code = 0;
    let args: Vec<OsString> = env::args_os().collect();

    if args.len() > 1 {
        let first = args[1].to_string_lossy();
        if first == "-h" || first == "--help" {
            println!("{}", USAGE);
            exit(0);
        }

        let start = Instant::now();

        match std::process::Command::new(&args[1])
            .args(&args[2..])
            .status()
        {
            Ok(status) => return_code = status.code().unwrap_or(0),
            Err(e) => {
                writeln!(io::stderr(), "{}", e).unwrap();
                return_code = 1;
            }
        }

        let duration = start.elapsed().as_secs();
        if duration > 5 {
            let mut secs = duration;
            let hours = secs / 3600;
            secs %= 3600;
            let minutes = secs / 60;
            secs %= 60;

            let mut elapsed = String::new();
            if hours > 0 {
                elapsed.push_str(&format!("{}h", hours));
            }
            if minutes > 0 {
                elapsed.push_str(&format!("{}m", minutes));
            }
            if secs > 0 {
                elapsed.push_str(&format!("{}s", secs));
            }

            writeln!(io::stderr(), "Elapsed: {}", elapsed).unwrap();
        }
    }

    unsafe {
        match fork() {
            Ok(ForkResult::Child) => {
                play_audio_and_wait(return_code);
                exit(0);
            }
            Ok(ForkResult::Parent { .. }) => exit(return_code),
            Err(err) => {
                eprintln!("fork failed: {}", err);
                exit(1);
            }
        }
    }
}
