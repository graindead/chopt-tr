# CHOpt Truss (chopt-tr)

A Rust port of **[CHOpt](https://github.com/GenericMadScientist/CHOpt)**, the Star Power path optimiser for Clone Hero, created by **Raymond Wright ([GenericMadScientist](https://github.com/GenericMadScientist))**.

## Credit

All credit for CHOpt goes to Raymond Wright, who created it. That covers:

- the pathing algorithm and its optimisations;
- the scoring rules for Clone Hero's engine;
- the path summary;
- the path image's design, layout and note sprites;
- SightRead, the chart reader whose rules this port follows.

This repository only translates that work into Rust. If CHOpt has been useful to you, please support and star [the original](https://github.com/GenericMadScientist/CHOpt).

## Why this exists

[Truss](https://github.com/graindead) is a single-file Rust app. I wanted to offer CHOpt's pathing inside it, as a tab that can path any chart in your library. CHOpt is a C++/Qt program, so including it as it is would have meant shipping it and its Qt libraries alongside Truss and running it as a separate process.

Porting the Clone Hero parts to Rust let them fit into Truss directly. Truss can call the pathing itself, cancel a run part-way, and show the path image while it's still being drawn.

This repository publishes that port on its own, so the code is public under CHOpt's licence.

## Performance

The port keeps CHOpt's results and changes how the work is done:

- **Activation search:** the hash sets and repeated scans of the search's inner loop are replaced by reused, stamped scratch arrays and a skip list. Most of the time used to go there.
- **Star Power totals:** the totals between any two points come from prefix sums instead of being re-added each time.
- **Path graph:** it is built in waves, and each wave's points are worked out in parallel. Edges are kept in exactly the order CHOpt keeps them, so ties resolve the same way.
- **CPU use:** the search uses half the CPU cores, at below-normal priority on Windows, so a game or OBS running at the same time keeps its share.
- **Image:** it is drawn in strips, so Truss can show it as it goes.

Times on a 16-thread PC against the CHOpt 1.19 command line, including program start-up:

| Chart | CHOpt | chopt-tr | Speed-up |
|---|---|---|---|
| Normal-length song (TesseracT – Crucible), path only | 0.09 s | 0.03 s | ~3× |
| Full-album chart (Melanie Martinez – Cry Baby), path only | 2.27 s | 0.36 s | ~6× |
| Full-album chart, path and image | 3.49 s | 0.57 s | ~6× |

It covers CHOpt's Clone Hero engine for five-fret parts, from `.chart` and `.mid` files. CHOpt itself also handles drums, GHL, other games' engines and more.

## The app

`chopt-tr` is a window styled like Truss, with Truss's themes and its frosted glass on Windows. It has CHOpt's settings, in CHOpt's order:

- Instrument and difficulty
- Squeeze, early whammy, lazy whammy and whammy delay
- Song speed, video lag and activation opacity
- Draw BPMs, Draw Time Signatures and Draw solo sections
- Blank chart, Precision Mode and Lefty flip

To load a `.chart` or `.mid`, use Load chart, drop the file on the window, or open it with the app. The song.ini is read from the same folder. The path summary and image appear in the window, and Save image writes the image as a PNG or BMP. Your settings are remembered between runs, as in CHOpt.

```bash
cargo run --release --bin chopt-tr
```

## Command line

```bash
cargo run --release --bin chopt-tr-cli -- "path/to/song folder" -o path.png
```

`chopt-tr-cli --help` lists the options, which match CHOpt's command-line flags. To use the code as a library, without the app's dependencies, depend on the crate with `default-features = false`:

```rust
let song = chopt::load(&notes_bytes, is_midi, &ini_bytes)?;
let out = chopt::run(song, &chopt::Options::default(), &std::sync::atomic::AtomicBool::new(false))?;
println!("{}", out.summary);
chopt::draw::save_png(&out.image, std::path::Path::new("path.png"))?;
```

## Licence

GPL-3.0-or-later, the same as CHOpt. CHOpt is copyright (C) 2020-2026 Raymond Wright, and this port is a derivative work of it.
