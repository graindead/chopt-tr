// CHOpt Truss: a Rust port of CHOpt (https://github.com/GenericMadScientist/CHOpt).
// Copyright (C) 2020-2026 Raymond Wright. Rust port by graindead.
// Licensed under the GNU General Public License v3.0 or later.

//! Command-line front end: finds the best Star Power path for a chart folder, prints CHOpt's
//! summary and optionally saves the path image.

use chopt::{Difficulty, Instrument, Options};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

const USAGE: &str = "\
Usage: chopt-tr <chart folder> [options]

  -i, --instrument <guitar|coop|bass|rhythm|keys>   (default guitar)
  -d, --diff <expert|hard|medium|easy>              (default expert)
  -o, --output <image.png>    save the path image
  -p, --precision             Clone Hero precision mode
  -s, --squeeze <0-100>       (default 100)
  -e, --early-whammy <0-100>  (default 100)
  -z, --lazy-whammy <ms>      (default 0)
  -w, --whammy-delay <ms>     (default 0)
  -v, --video-lag <ms>        -200 to 200 (default 0)
  -f, --speed <percent>       multiple of 5 (default 100)
  -l, --lefty                 lefty-flip the image
";

fn main() {
    if let Err(e) = real_main() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn real_main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let mut opts = Options::default();
    let mut folder: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    while let Some(a) = args.next() {
        let mut value = |name: &str| args.next().ok_or_else(|| format!("{name} needs a value\n\n{USAGE}"));
        let num = |s: String| s.parse::<i64>().map_err(|_| format!("Not a number: {s}"));
        match a.as_str() {
            "-h" | "--help" => {
                print!("{USAGE}");
                return Ok(());
            }
            "-i" | "--instrument" => {
                opts.instrument = match value(&a)?.to_lowercase().as_str() {
                    "guitar" => Instrument::Guitar,
                    "coop" | "guitarcoop" => Instrument::GuitarCoop,
                    "bass" => Instrument::Bass,
                    "rhythm" => Instrument::Rhythm,
                    "keys" => Instrument::Keys,
                    other => return Err(format!("Unknown instrument: {other}")),
                }
            }
            "-d" | "--diff" => {
                opts.difficulty = match value(&a)?.to_lowercase().as_str() {
                    "expert" => Difficulty::Expert,
                    "hard" => Difficulty::Hard,
                    "medium" => Difficulty::Medium,
                    "easy" => Difficulty::Easy,
                    other => return Err(format!("Unknown difficulty: {other}")),
                }
            }
            "-o" | "--output" => output = Some(value(&a)?.into()),
            "-p" | "--precision" => opts.precision = true,
            "-l" | "--lefty" => opts.lefty = true,
            "-s" | "--squeeze" => opts.squeeze = num(value(&a)?)?.clamp(0, 100) as u32,
            "-e" | "--early-whammy" => opts.early_whammy = num(value(&a)?)?.clamp(0, 100) as u32,
            "-z" | "--lazy-whammy" => opts.lazy_whammy_ms = num(value(&a)?)?.max(0) as u32,
            "-w" | "--whammy-delay" => opts.whammy_delay_ms = num(value(&a)?)?.max(0) as u32,
            "-v" | "--video-lag" => opts.video_lag_ms = num(value(&a)?)?.clamp(-200, 200) as i32,
            "-f" | "--speed" => opts.speed = num(value(&a)?)?.clamp(5, 5000) as u32,
            _ if a.starts_with('-') => return Err(format!("Unknown option: {a}\n\n{USAGE}")),
            _ => folder = Some(a.into()),
        }
    }
    let Some(folder) = folder else { return Err(USAGE.to_string()) };
    let (notes, is_midi) = find_notes(&folder)?;
    let data = std::fs::read(&notes).map_err(|e| format!("{}: {e}", notes.display()))?;
    let ini = std::fs::read(folder.join("song.ini")).unwrap_or_default();
    let song = chopt::load(&data, is_midi, &ini)?;
    let out = chopt::run(song, &opts, &AtomicBool::new(false))?;
    println!("{}", out.summary);
    if let Some(path) = output {
        chopt::draw::save_png(&out.image, &path)?;
        println!("Image saved to {}", path.display());
    }
    Ok(())
}

/// The folder's notes.mid, or else its notes.chart.
fn find_notes(folder: &Path) -> Result<(PathBuf, bool), String> {
    for (name, midi) in [("notes.mid", true), ("notes.chart", false)] {
        let p = folder.join(name);
        if p.is_file() {
            return Ok((p, midi));
        }
    }
    Err(format!("No notes.mid or notes.chart in {}", folder.display()))
}
