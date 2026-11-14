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

[Truss](https://github.com/graindead) is a single-file Rust app. It needed CHOpt's pathing built in, as a tab that can path any chart in your library. CHOpt is a C++/Qt program, so using it directly would have meant shipping it and its Qt libraries alongside Truss and running it as a separate process.

Porting the Clone Hero parts to Rust let Truss do three things that CHOpt's design didn't allow:

- call the pathing directly;
- cancel a run part-way;
- show the path image while it's still being drawn.

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

## Use

```bash
cargo run --release -- "path/to/song folder" -o path.png
```

To see all the options, run `chopt-tr --help`. To use it as a library:

```rust
let song = chopt::load(&notes_bytes, is_midi, &ini_bytes)?;
let out = chopt::run(song, &chopt::Options::default(), &std::sync::atomic::AtomicBool::new(false))?;
println!("{}", out.summary);
chopt::draw::save_png(&out.image, std::path::Path::new("path.png"))?;
```

## Licence

GPL-3.0-or-later, the same as CHOpt. CHOpt is copyright (C) 2020-2026 Raymond Wright, and this port is a derivative work of it.
