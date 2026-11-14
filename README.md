# CHOpt Truss (chopt-tr)

A Rust port of [CHOpt](https://github.com/GenericMadScientist/CHOpt), the Star Power path optimiser for Clone Hero, by Raymond Wright. This is the optimiser built into [Truss](https://github.com/graindead), published here on its own.

It finds the Star Power activations that score the most on a chart. It prints CHOpt's summary of the path and draws the path image in CHOpt's layout.

## What it covers

- **Engine:** Clone Hero, standard and precision mode.
- **Parts:** five-fret guitar, co-op guitar, bass, rhythm and keys.
- **Charts:** `.chart` and `.mid`, read with SightRead's rules.
- **Options:** squeeze, early whammy, lazy whammy, whammy delay, video calibration, song speed and lefty flip.

It does not cover drums, GHL, Rock Band or other game engines.

## Accuracy

Scores and paths were compared with the CHOpt 1.19 command line on a real song library. Every run matched exactly:

- 1,786 part/difficulty runs at default settings
- 97 runs with custom settings
- 20 runs with early whammy above squeeze

The note sprites match CHOpt's own. The image's text uses a different font from CHOpt's, so text pixels differ.

## Use

```bash
cargo run --release -- "path/to/song folder" -o path.png
```

Run `chopt-tr --help` to see all the options. To use it as a library:

```rust
let song = chopt::load(&notes_bytes, is_midi, &ini_bytes)?;
let out = chopt::run(song, &chopt::Options::default(), &std::sync::atomic::AtomicBool::new(false))?;
println!("{}", out.summary);
chopt::draw::save_png(&out.image, std::path::Path::new("path.png"))?;
```

The search runs on half the CPU cores, at below-normal priority on Windows.

## Licence

GPL-3.0-or-later, the same as CHOpt. CHOpt is copyright (C) 2020-2026 Raymond Wright.
