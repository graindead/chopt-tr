// CHOpt Truss: a Rust port of CHOpt (https://github.com/GenericMadScientist/CHOpt).
// Copyright (C) 2020-2026 Raymond Wright. Rust port by graindead.
// Licensed under the GNU General Public License v3.0 or later.

//! The app: CHOpt's settings in a window styled like Truss (its themes and frosted glass). Load
//! a chart, choose the settings and find the path; the summary and image show in the window.

#![windows_subsystem = "windows"]

mod backdrop;
mod settings;
mod style;
mod themes;

use chopt::{Difficulty, Instrument, Options, Output, Song};
use iced::widget::{button, column, container, image, mouse_area, pick_list, row, scrollable, slider, space, text, text_input, toggler};
use iced::{Alignment, Color, Element, Font, Length, Subscription, Task, Theme};
use settings::Settings;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

const BOLD: Font = Font { weight: iced::font::Weight::Bold, ..Font::DEFAULT };
/// Rows of measures per image strip (each strip is one texture).
const ROWS_PER_STRIP: u32 = 20;
/// Height of the image's header (title, artist, charter) above the first row.
const HEADER_HEIGHT: u32 = 217;

fn main() -> iced::Result {
    let settings = Settings::load();
    let frosted = cfg!(windows) && settings.frosted;
    // SAFETY: no other threads exist yet.
    unsafe {
        #[cfg(windows)]
        if std::env::var_os("WGPU_BACKEND").is_none() {
            std::env::set_var("WGPU_BACKEND", "dx12");
        }
        // A transparent window needs DX12 to present through a composition visual.
        if frosted && std::env::var_os("WGPU_DX12_PRESENTATION_SYSTEM").is_none() {
            std::env::set_var("WGPU_DX12_PRESENTATION_SYSTEM", "Visual");
        }
    }
    let (size, position) = initial_window();
    iced::application(move || App::boot(frosted), App::update, App::view)
        .title("CHOpt Truss")
        .theme(App::theme)
        .style(App::app_style)
        .subscription(App::subscription)
        .window(iced::window::Settings {
            size,
            position,
            min_size: Some(iced::Size::new(640.0, 480.0)),
            transparent: frosted,
            ..Default::default()
        })
        .antialiasing(false)
        .run()
}

/// The window's first size and place (as Truss): 1240x820 at most, shrunk to fit the screen's
/// usable area (not under the taskbar) and centred in it.
fn initial_window() -> (iced::Size, iced::window::Position) {
    let want: iced::Size<f32> = iced::Size::new(1240.0, 820.0);
    #[cfg(windows)]
    {
        use windows::Win32::Foundation::RECT;
        use windows::Win32::UI::HiDpi::GetDpiForSystem;
        use windows::Win32::UI::WindowsAndMessaging::{SPI_GETWORKAREA, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SystemParametersInfoW};
        let mut wa = RECT::default();
        // SAFETY: SPI_GETWORKAREA writes one RECT to the pointer given.
        let ok = unsafe {
            SystemParametersInfoW(SPI_GETWORKAREA, 0, Some(&mut wa as *mut RECT as *mut _), SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0)).is_ok()
        };
        if ok && wa.right > wa.left && wa.bottom > wa.top {
            // The work area is in this process's pixels: physical if it's DPI aware yet, else
            // already scaled to 96 DPI. Either way this gives logical units.
            let scale = (unsafe { GetDpiForSystem() }.max(96)) as f32 / 96.0;
            let (left, top) = (wa.left as f32 / scale, wa.top as f32 / scale);
            let (aw, ah) = ((wa.right - wa.left) as f32 / scale, (wa.bottom - wa.top) as f32 / scale);
            // Room for the title bar and borders, plus a small margin.
            const FRAME_W: f32 = 16.0;
            const FRAME_H: f32 = 40.0;
            const MARGIN: f32 = 16.0;
            let size = iced::Size::new(
                want.width.min(aw - FRAME_W - 2.0 * MARGIN).max(640.0),
                want.height.min(ah - FRAME_H - 2.0 * MARGIN).max(480.0),
            );
            let x = left + ((aw - size.width - FRAME_W) / 2.0).max(0.0);
            let y = top + ((ah - size.height - FRAME_H) / 2.0).max(0.0);
            return (size, iced::window::Position::Specific(iced::Point::new(x, y)));
        }
    }
    (want, iced::window::Position::Centered)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Page {
    Path,
    Settings,
}

/// A chart that's been read.
struct Loaded {
    file: PathBuf,
    song: Arc<Song>,
}

struct Found {
    output: Output,
    strips: Vec<image::Handle>,
    seconds: f64,
    blank: bool,
}

impl std::fmt::Debug for Found {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Found")
    }
}

struct App {
    settings: Settings,
    theme: Theme,
    /// Frosted glass in effect this run (changes apply on restart).
    frosted: bool,
    page: Page,
    loading: Option<PathBuf>,
    chart: Option<Result<Loaded, String>>,
    instrument: Instrument,
    difficulty: Difficulty,
    lazy: String,
    delay: String,
    speed: String,
    running: Option<Arc<AtomicBool>>,
    run_id: u64,
    result: Option<Result<Arc<Found>, String>>,
    /// A message under the result: where the image was saved, or what went wrong.
    note: Option<Result<String, String>>,
    /// The image made for the result showing, once opened: opening it again reuses it.
    opened: Option<PathBuf>,
}

#[derive(Debug, Clone)]
enum Message {
    Page(Page),
    PickFile,
    Picked(Option<PathBuf>),
    Dropped(PathBuf),
    Loaded(PathBuf, Result<Arc<Song>, String>),
    Instrument(Instrument),
    Difficulty(Difficulty),
    Squeeze(u32),
    EarlyWhammy(u32),
    Lazy(String),
    Delay(String),
    Speed(String),
    VideoLag(i32),
    Opacity(u32),
    DrawBpms(bool),
    DrawTimeSigs(bool),
    DrawSolos(bool),
    Blank(bool),
    Precision(bool),
    Lefty(bool),
    Run,
    Stop,
    Done(u64, Result<Arc<Found>, String>),
    Save,
    Saved(Option<Result<PathBuf, String>>),
    Open,
    Opened(Result<PathBuf, String>),
    Theme(String),
    Frosted(bool),
    Transparency(f32),
    ResetSettings,
    Link(&'static str),
    Nothing,
}

/// Reads a .chart or .mid and the song.ini beside it. Read only.
fn load_chart(file: &Path) -> Result<Song, String> {
    let name = file.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    let lower = name.to_ascii_lowercase();
    let is_midi = lower.ends_with(".mid") || lower.ends_with(".midi");
    if !is_midi && !lower.ends_with(".chart") {
        return Err(format!("{name} isn't a .chart or .mid file."));
    }
    let notes = std::fs::read(file).map_err(|e| format!("Couldn't read {name}: {e}"))?;
    let ini = file
        .parent()
        .and_then(|dir| std::fs::read_dir(dir).ok())
        .and_then(|rd| rd.flatten().find(|e| e.file_name().to_string_lossy().eq_ignore_ascii_case("song.ini")))
        .and_then(|e| std::fs::read(e.path()).ok())
        .unwrap_or_default();
    chopt::load(&notes, is_midi, &ini).map_err(|e| format!("{name} invalid: {e}"))
}

fn render_strips(out: &Output) -> Vec<image::Handle> {
    let total = chopt::draw::image_height(&out.image);
    let step = ROWS_PER_STRIP * chopt::draw::ROW_HEIGHT;
    let mut strips = Vec::new();
    let mut y = 0;
    while y < total {
        // The first strip also holds the header.
        let end = if y == 0 { (step + HEADER_HEIGHT).min(total) } else { (y + step).min(total) };
        if let Some((w, h, data)) = chopt::draw::render(&out.image, y, end) {
            strips.push(image::Handle::from_rgba(w, h, data));
        }
        y = end;
    }
    strips
}

/// An int the way Qt's QString::toInt reads one (CHOpt's text boxes): optional sign, digits,
/// nothing else.
fn to_int(s: &str) -> Option<i32> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    s.parse().ok()
}

fn sanitize(s: &str) -> String {
    s.chars().map(|c| if r#"\/:*?"<>|"#.contains(c) { '_' } else { c }).collect::<String>().trim().to_string()
}

/// Opens a file with the app Windows uses for its type (as double-clicking it would).
fn open_file(path: &Path) -> Result<(), String> {
    #[cfg(windows)]
    {
        std::process::Command::new("explorer").arg(path).spawn().map(|_| ()).map_err(|e| e.to_string())
    }
    #[cfg(not(windows))]
    {
        std::process::Command::new("xdg-open").arg(path).spawn().map(|_| ()).map_err(|e| e.to_string())
    }
}

fn open_url(url: &str) {
    #[cfg(windows)]
    let _ = std::process::Command::new("explorer").arg(url).spawn();
    #[cfg(not(windows))]
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();
}

impl App {
    fn boot(frosted: bool) -> (Self, Task<Message>) {
        let settings = Settings::load();
        let mut app = App {
            theme: themes::by_name(&settings.theme),
            lazy: settings.lazy_whammy_ms.to_string(),
            delay: settings.whammy_delay_ms.to_string(),
            speed: settings.speed.to_string(),
            settings,
            frosted,
            page: Page::Path,
            loading: None,
            chart: None,
            instrument: Instrument::Guitar,
            difficulty: Difficulty::Expert,
            running: None,
            run_id: 0,
            result: None,
            note: None,
            opened: None,
        };
        let mut task = app.backdrop_task();
        // A chart given on the command line (or through "Open with").
        if let Some(file) = std::env::args_os().nth(1).map(PathBuf::from).filter(|p| p.is_file()) {
            task = Task::batch([task, app.load(file)]);
        }
        (app, task)
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    /// The window background: see-through over the frosted glass when it's on.
    fn app_style(&self, theme: &Theme) -> iced::theme::Style {
        let p = theme.extended_palette();
        let bg = p.background.base.color;
        let a = if self.frosted { 1.0 - self.settings.transparency.clamp(0.1, 1.0) } else { 1.0 };
        // The window presents pre-multiplied alpha, so the colour is scaled by it too.
        iced::theme::Style { background_color: Color { r: bg.r * a, g: bg.g * a, b: bg.b * a, a }, text_color: p.background.base.text }
    }

    /// Asks Windows to blur what's behind the window, tinted with the theme's background.
    fn backdrop_task(&self) -> Task<Message> {
        if !self.frosted {
            return Task::none();
        }
        let c = self.theme.palette().background;
        let tint = [c.r, c.g, c.b].map(|v| (v * 255.0).round() as u8);
        // Applied again shortly after, in case the window wasn't fully set up the first time.
        let at = |delay_ms: u64| {
            Task::perform(tokio::time::sleep(std::time::Duration::from_millis(delay_ms)), |_| ())
                .then(|_| iced::window::oldest())
                .and_then(iced::window::raw_id::<Message>)
                .map(move |hwnd| {
                    backdrop::apply(hwnd, tint);
                    Message::Nothing
                })
        };
        Task::batch([at(0), at(500)])
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::event::listen_with(|e, _, _| match e {
            iced::Event::Window(iced::window::Event::FileDropped(p)) => Some(Message::Dropped(p)),
            _ => None,
        })
    }

    /// The options to run with, from the settings and text boxes (as CHOpt's get_settings).
    fn options(&self) -> Options {
        let s = &self.settings;
        Options {
            instrument: self.instrument,
            difficulty: self.difficulty,
            precision: s.precision,
            squeeze: s.squeeze,
            early_whammy: s.early_whammy,
            lazy_whammy_ms: to_int(&self.lazy).unwrap_or(0).max(0) as u32,
            whammy_delay_ms: to_int(&self.delay).unwrap_or(0).max(0) as u32,
            video_lag_ms: s.video_lag_ms,
            speed: to_int(&self.speed).unwrap_or(100).max(0) as u32,
            lefty: s.lefty,
            draw_bpms: s.draw_bpms,
            draw_solos: s.draw_solos,
            draw_time_sigs: s.draw_time_sigs,
            blank: s.blank,
            activation_opacity: s.activation_opacity as f32 / 100.0,
        }
    }

    fn stop(&mut self) {
        if let Some(c) = self.running.take() {
            c.store(true, Ordering::Relaxed);
        }
    }

    fn load(&mut self, file: PathBuf) -> Task<Message> {
        self.stop();
        self.loading = Some(file.clone());
        if let Some(dir) = file.parent() {
            self.settings.last_folder = Some(dir.to_path_buf());
            self.settings.save();
        }
        let f = file.clone();
        Task::perform(
            async move { tokio::task::spawn_blocking(move || load_chart(&f).map(Arc::new)).await.map_err(|e| e.to_string()).and_then(|r| r) },
            move |r| Message::Loaded(file.clone(), r),
        )
    }

    fn update(&mut self, m: Message) -> Task<Message> {
        // Settings that are remembered are saved as they change.
        let mut save = true;
        match m {
            Message::Page(p) => {
                self.page = p;
                save = false;
            }
            Message::PickFile => {
                let dir = self.settings.last_folder.clone();
                return Task::perform(
                    async move {
                        let mut d = rfd::AsyncFileDialog::new().set_title("Load chart").add_filter("Charts", &["chart", "mid", "midi"]);
                        if let Some(dir) = dir {
                            d = d.set_directory(dir);
                        }
                        d.pick_file().await.map(|f| f.path().to_path_buf())
                    },
                    Message::Picked,
                );
            }
            Message::Picked(Some(p)) | Message::Dropped(p) => {
                self.page = Page::Path;
                return self.load(p);
            }
            Message::Picked(None) => save = false,
            Message::Loaded(file, r) => {
                save = false;
                // Only the file asked for last counts.
                if self.loading.as_ref() != Some(&file) {
                    return Task::none();
                }
                self.loading = None;
                self.result = None;
                self.note = None;
                self.opened = None;
                self.chart = Some(r.map(|song| {
                    // Keep the chosen part when the chart has it, else its first part.
                    let insts = song.instruments();
                    if !insts.contains(&self.instrument)
                        && let Some(i) = insts.first()
                    {
                        self.instrument = *i;
                    }
                    let diffs = song.difficulties(self.instrument);
                    if !diffs.contains(&self.difficulty)
                        && let Some(d) = diffs.first()
                    {
                        self.difficulty = *d;
                    }
                    Loaded { file, song }
                }));
            }
            Message::Instrument(i) => {
                save = false;
                self.instrument = i;
                if let Some(Ok(c)) = &self.chart {
                    let diffs = c.song.difficulties(i);
                    if !diffs.contains(&self.difficulty)
                        && let Some(d) = diffs.first()
                    {
                        self.difficulty = *d;
                    }
                }
            }
            Message::Difficulty(d) => {
                save = false;
                self.difficulty = d;
            }
            Message::Squeeze(v) => self.settings.squeeze = v,
            Message::EarlyWhammy(v) => self.settings.early_whammy = v,
            Message::Lazy(s) => {
                if let Some(v) = to_int(&s) {
                    self.settings.lazy_whammy_ms = v;
                }
                self.lazy = s;
            }
            Message::Delay(s) => {
                if let Some(v) = to_int(&s) {
                    self.settings.whammy_delay_ms = v;
                }
                self.delay = s;
            }
            Message::Speed(s) => {
                if let Some(v) = to_int(&s) {
                    self.settings.speed = v;
                }
                self.speed = s;
            }
            Message::VideoLag(v) => self.settings.video_lag_ms = v,
            Message::Opacity(v) => self.settings.activation_opacity = v,
            Message::DrawBpms(b) => self.settings.draw_bpms = b,
            Message::DrawTimeSigs(b) => self.settings.draw_time_sigs = b,
            Message::DrawSolos(b) => self.settings.draw_solos = b,
            Message::Blank(b) => self.settings.blank = b,
            Message::Precision(b) => self.settings.precision = b,
            Message::Lefty(b) => self.settings.lefty = b,
            Message::Run => {
                let Some(Ok(chart)) = &self.chart else { return Task::none() };
                let song = (*chart.song).clone();
                // CHOpt's check: Clone Hero plays 5%-5000% in steps of 5%.
                let speed_ok = to_int(&self.speed).is_some_and(|s| (5..=5000).contains(&s) && s % 5 == 0);
                if !speed_ok {
                    self.result = Some(Err("Speed not supported by Clone Hero".into()));
                    return Task::none();
                }
                self.stop();
                let cancel = Arc::new(AtomicBool::new(false));
                self.running = Some(cancel.clone());
                self.result = None;
                self.note = None;
                self.opened = None;
                self.run_id += 1;
                let id = self.run_id;
                let opts = self.options();
                return Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            let t = std::time::Instant::now();
                            let output = chopt::run(song, &opts, &cancel)?;
                            let seconds = t.elapsed().as_secs_f64();
                            let strips = render_strips(&output);
                            Ok(Arc::new(Found { output, strips, seconds, blank: opts.blank }))
                        })
                        .await
                        .map_err(|e| e.to_string())
                        .and_then(|r| r)
                    },
                    move |r| Message::Done(id, r),
                );
            }
            Message::Stop => {
                save = false;
                self.stop();
            }
            Message::Done(id, r) => {
                save = false;
                // Only the latest run counts; a stopped one reports "Cancelled".
                if id != self.run_id || matches!(&r, Err(e) if e == "Cancelled") {
                    return Task::none();
                }
                self.running = None;
                self.result = Some(r);
            }
            Message::Save => {
                let Some(Ok(res)) = &self.result else { return Task::none() };
                let res = res.clone();
                let name = format!("{} - {} path.png", sanitize(&res.output.image.artist), sanitize(&res.output.image.song_name));
                let dir = self.settings.last_folder.clone();
                return Task::perform(
                    async move {
                        let mut d = rfd::AsyncFileDialog::new()
                            .set_title("Save image")
                            .set_file_name(name)
                            .add_filter("PNG image", &["png"])
                            .add_filter("BMP image", &["bmp"]);
                        if let Some(dir) = dir {
                            d = d.set_directory(dir);
                        }
                        let path = d.save_file().await?.path().to_path_buf();
                        let ext = path.extension().map(|e| e.to_string_lossy().to_ascii_lowercase());
                        if !matches!(ext.as_deref(), Some("png" | "bmp")) {
                            return Some(Err("Not a valid image file".to_string()));
                        }
                        let p2 = path.clone();
                        Some(
                            tokio::task::spawn_blocking(move || chopt::draw::save_png(&res.output.image, &p2))
                                .await
                                .map_err(|e| e.to_string())
                                .and_then(|r| r)
                                .map(|_| path),
                        )
                    },
                    Message::Saved,
                );
            }
            Message::Saved(r) => {
                save = false;
                match r {
                    Some(Ok(p)) => self.note = Some(Ok(format!("Saved to {}", p.display()))),
                    Some(Err(e)) => self.note = Some(Err(format!("Couldn't save: {e}"))),
                    None => {}
                }
            }
            Message::Open => {
                let Some(Ok(res)) = &self.result else { return Task::none() };
                if let Some(p) = self.opened.clone().filter(|p| p.exists()) {
                    return Task::perform(async move { open_file(&p).map(|_| p) }, Message::Opened);
                }
                let res = res.clone();
                let base = format!("{} - {} path", sanitize(&res.output.image.artist), sanitize(&res.output.image.song_name));
                return Task::perform(
                    async move {
                        // In Pictures: the Photos app can't open images from hidden folders such
                        // as AppData.
                        let dir = directories::UserDirs::new()
                            .and_then(|u| u.picture_dir().map(|d| d.join("CHOpt Truss")))
                            .ok_or("No folder to put the image in")?;
                        tokio::task::spawn_blocking(move || {
                            std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
                            // A free name ("… (2).png" if taken), so an image a viewer has open
                            // is never replaced under it.
                            let path = (1..)
                                .map(|n| dir.join(if n == 1 { format!("{base}.png") } else { format!("{base} ({n}).png") }))
                                .find(|p| !p.exists())
                                .ok_or("No free file name")?;
                            chopt::draw::save_png(&res.output.image, &path)?;
                            open_file(&path).map(|_| path)
                        })
                        .await
                        .map_err(|e| e.to_string())?
                    },
                    Message::Opened,
                );
            }
            Message::Opened(r) => {
                save = false;
                match r {
                    Ok(p) => self.opened = Some(p),
                    Err(e) => self.note = Some(Err(format!("Couldn't open the image: {e}"))),
                }
            }
            Message::Theme(name) => {
                self.theme = themes::by_name(&name);
                self.settings.theme = name;
                self.settings.save();
                return self.backdrop_task();
            }
            Message::Frosted(b) => self.settings.frosted = b,
            Message::Transparency(v) => self.settings.transparency = v,
            Message::ResetSettings => {
                let keep = (self.settings.theme.clone(), self.settings.frosted, self.settings.transparency, self.settings.last_folder.clone());
                self.settings = Settings::default();
                (self.settings.theme, self.settings.frosted, self.settings.transparency, self.settings.last_folder) = keep;
                self.lazy = self.settings.lazy_whammy_ms.to_string();
                self.delay = self.settings.whammy_delay_ms.to_string();
                self.speed = self.settings.speed.to_string();
            }
            Message::Link(url) => {
                save = false;
                open_url(url);
            }
            Message::Nothing => save = false,
        }
        if save {
            self.settings.save();
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let nav = |label: &'static str, page: Page| -> Element<'_, Message> {
            let active = self.page == page;
            column![
                button(text(label).size(14)).padding([6, 2]).style(style::nav_button(active)).on_press(Message::Page(page)),
                container(space()).height(2).width(Length::Fill).style(move |t: &Theme| container::Style {
                    background: active.then(|| style::accent(t).into()),
                    ..Default::default()
                }),
            ]
            .width(Length::Shrink)
            .into()
        };
        let bar = row![
            text("CHOpt Truss").size(15).font(BOLD),
            space().width(10),
            nav("Path", Page::Path),
            nav("Settings", Page::Settings),
            space().width(Length::Fill),
            button(text("Load chart…").size(13)).padding([6, 12]).style(style::secondary).on_press(Message::PickFile),
        ]
        .spacing(14)
        .align_y(Alignment::Center);
        let body = match self.page {
            Page::Path => self.view_path(),
            Page::Settings => self.view_settings(),
        };
        column![container(bar).padding([8, 18]).width(Length::Fill), style::divider(false), body].into()
    }

    fn view_path(&self) -> Element<'_, Message> {
        let mut left = column![].spacing(12).width(340);
        left = left.push(self.view_chart_info());
        left = left.push(self.view_options());

        let right: Element<'_, Message> = match &self.result {
            None if self.running.is_some() => container(text("Optimising, please wait…").size(13)).padding(20).into(),
            None if matches!(self.chart, Some(Ok(_))) => container(text("Choose the settings and press Find path.").size(13).style(style::muted_text)).padding(20).into(),
            None => container(
                column![
                    text("Load a .chart or .mid to find its best Star Power path.").size(13),
                    text("Use Load chart, or drop a file on the window. Charts are only read.").size(12).style(style::muted_text),
                ]
                .spacing(6),
            )
            .padding(20)
            .into(),
            Some(Err(e)) => container(text(e.clone()).size(13).style(text::danger)).padding(20).into(),
            Some(Ok(res)) => {
                let what = if res.blank { "Drawn" } else { "Found" };
                let mut top = column![
                    row![
                        text(format!("{what} in {:.2} s", res.seconds)).size(12).style(style::muted_text).width(Length::Fill),
                        button(text("Open image").size(13)).padding([6, 12]).style(style::secondary).on_press(Message::Open),
                        button(text("Save image…").size(13)).padding([6, 12]).style(style::primary).on_press(Message::Save),
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center),
                ]
                .spacing(10);
                if !res.output.summary.is_empty() {
                    top = top.push(container(text(res.output.summary.clone()).size(13).font(Font::MONOSPACE)).padding(10).width(Length::Fill).style(style::panel));
                }
                match &self.note {
                    Some(Ok(s)) => top = top.push(text(s.clone()).size(12).style(text::success)),
                    Some(Err(e)) => top = top.push(text(e.clone()).size(12).style(text::danger)),
                    None => {}
                }
                let strips = res.strips.iter().fold(column![], |col, h| col.push(image(h.clone()).width(Length::Fill).filter_method(image::FilterMethod::Linear)));
                // Clicking the path opens it full size, as Open image does.
                let strips = mouse_area(container(strips).max_width(1024)).on_press(Message::Open).interaction(iced::mouse::Interaction::Pointer);
                scrollable(column![top, strips].spacing(12).padding(16)).height(Length::Fill).into()
            }
        };

        row![
            scrollable(container(left).padding(16)).height(Length::Fill),
            style::divider(true),
            container(right).width(Length::Fill).height(Length::Fill)
        ]
        .height(Length::Fill)
        .into()
    }

    fn view_chart_info(&self) -> Element<'_, Message> {
        if let Some(f) = &self.loading {
            let name = f.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            return text(format!("Reading {name}…")).size(12).into();
        }
        match &self.chart {
            None => column![
                text("No chart loaded").size(14).font(BOLD),
                text("Load a .chart or .mid (its song.ini is read from the same folder).").size(12).style(style::muted_text),
            ]
            .spacing(4)
            .into(),
            Some(Err(e)) => text(e.clone()).size(12).style(text::danger).into(),
            Some(Ok(c)) => {
                let file = c.file.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                let folder = c.file.parent().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                column![
                    text(c.song.name.clone()).size(15).font(BOLD),
                    text(format!("{} · {}", c.song.artist, c.song.charter)).size(12),
                    text(format!("{folder} / {file}")).size(11).style(style::muted_text),
                ]
                .spacing(2)
                .into()
            }
        }
    }

    /// CHOpt's settings, in CHOpt's order.
    fn view_options(&self) -> Element<'_, Message> {
        let s = &self.settings;
        let label = |l: &'static str| text(l).size(13).width(130);
        let line = |l: &'static str, w: Element<'static, Message>| -> Element<'static, Message> { row![label(l), w].spacing(8).align_y(Alignment::Center).into() };
        let value = |v: String| text(v).size(12).width(44).align_x(iced::alignment::Horizontal::Right);
        let num = |v: &str, msg: fn(String) -> Message| text_input("0", v).on_input(msg).on_submit(Message::Run).padding(5).width(90).size(13).style(style::input);
        let toggle = |on: bool, l: &'static str, msg: fn(bool) -> Message| toggler(on).label(l).text_size(13).size(18.0).on_toggle(msg);

        let song = match &self.chart {
            Some(Ok(c)) => Some(&c.song),
            _ => None,
        };
        let insts = song.map(|s| s.instruments()).unwrap_or_default();
        let diffs = song.map(|s| s.difficulties(self.instrument)).unwrap_or_default();
        let inst_pick = pick_list(insts, song.map(|_| self.instrument), Message::Instrument).width(Length::Fill).text_size(13).style(style::quiet_pick_list);
        let diff_pick = pick_list(diffs, song.map(|_| self.difficulty), Message::Difficulty).width(Length::Fill).text_size(13).style(style::quiet_pick_list);

        let go: Element<'_, Message> = if self.running.is_some() {
            row![
                button(text("Optimising…").size(14)).padding([8, 16]).style(style::primary),
                button(text("Stop").size(13)).padding([6, 12]).style(style::secondary).on_press(Message::Stop),
            ]
            .spacing(8)
            .align_y(Alignment::Center)
            .into()
        } else {
            let label = if s.blank { "Draw image" } else { "Find path" };
            button(text(label).size(14)).padding([8, 16]).style(style::primary).on_press_maybe(song.map(|_| Message::Run)).into()
        };

        column![
            line("Engine", text("Clone Hero").size(13).into()),
            row![label("Instrument"), inst_pick].spacing(8).align_y(Alignment::Center),
            row![label("Difficulty"), diff_pick].spacing(8).align_y(Alignment::Center),
            style::divider(false),
            row![label("Squeeze %"), slider(0..=100u32, s.squeeze, Message::Squeeze), value(s.squeeze.to_string())].spacing(8).align_y(Alignment::Center),
            row![label("Early Whammy %"), slider(0..=100u32, s.early_whammy, Message::EarlyWhammy), value(s.early_whammy.to_string())]
                .spacing(8)
                .align_y(Alignment::Center),
            row![label("Lazy Whammy (ms)"), num(&self.lazy, Message::Lazy)].spacing(8).align_y(Alignment::Center),
            row![label("Whammy Delay (ms)"), num(&self.delay, Message::Delay)].spacing(8).align_y(Alignment::Center),
            row![label("Speed (%)"), num(&self.speed, Message::Speed)].spacing(8).align_y(Alignment::Center),
            row![label("Video Lag"), slider(-200..=200i32, s.video_lag_ms, Message::VideoLag), value(s.video_lag_ms.to_string())]
                .spacing(8)
                .align_y(Alignment::Center),
            row![label("Activation Opacity"), slider(0..=100u32, s.activation_opacity, Message::Opacity), value(format!("{:.2}", s.activation_opacity as f32 / 100.0))]
                .spacing(8)
                .align_y(Alignment::Center),
            style::divider(false),
            toggle(s.draw_bpms, "Draw BPMs", Message::DrawBpms),
            toggle(s.draw_time_sigs, "Draw Time Signatures", Message::DrawTimeSigs),
            toggle(s.draw_solos, "Draw solo sections", Message::DrawSolos),
            toggle(s.blank, "Blank chart", Message::Blank),
            toggle(s.precision, "Precision Mode", Message::Precision),
            toggle(s.lefty, "Lefty flip", Message::Lefty),
            space().height(4),
            go,
        ]
        .spacing(10)
        .into()
    }

    fn view_settings(&self) -> Element<'_, Message> {
        let names: Vec<String> = themes::all().iter().map(|t| t.to_string()).collect();
        let section = |title: &'static str, content: Element<'static, Message>| -> Element<'static, Message> {
            container(column![text(title).font(BOLD).size(15), content].spacing(10)).padding(16).width(Length::Fill).style(style::panel).into()
        };
        let pct = (self.settings.transparency * 100.0).round() as u32;
        let mut glass = column![
            toggler(self.settings.frosted).label("Frosted glass").text_size(13).size(18.0).on_toggle(Message::Frosted),
            row![
                text("See-through").size(13).width(110),
                slider(0.1f32..=1.0, self.settings.transparency, Message::Transparency).step(0.05f32),
                text(format!("{pct}%")).size(12).width(44)
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(10);
        if self.settings.frosted != self.frosted {
            glass = glass.push(text("Turning frosted glass on or off takes effect when the app restarts.").size(12).style(style::muted_text));
        }
        if !cfg!(windows) {
            glass = glass.push(text("Frosted glass needs Windows.").size(12).style(style::muted_text));
        }
        let link = |label: &'static str, url: &'static str| button(text(label).size(12)).padding([4, 10]).style(style::secondary).on_press(Message::Link(url));
        let page = column![
            text("Settings").font(BOLD).size(24),
            section(
                "Theme",
                pick_list(names, Some(self.settings.theme.clone()), Message::Theme).width(260).text_size(13).style(style::quiet_pick_list).into()
            ),
            section("Window", glass.into()),
            section(
                "CHOpt settings",
                column![
                    text("Your CHOpt settings are remembered between runs. This puts them back to CHOpt's defaults.").size(12).style(style::muted_text),
                    button(text("Reset to CHOpt's defaults").size(13)).padding([6, 12]).style(style::secondary).on_press(Message::ResetSettings),
                ]
                .spacing(8)
                .into()
            ),
            section(
                "About",
                column![
                    text("CHOpt Truss is a Rust port of CHOpt, the Star Power path optimiser for Clone Hero, created by Raymond Wright (GenericMadScientist). All credit for CHOpt's pathing, scoring rules and path images goes to him.").size(12),
                    text("Licensed under the GPL-3.0, as CHOpt is. The look comes from Truss.").size(12).style(style::muted_text),
                    row![
                        link("CHOpt on GitHub", "https://github.com/GenericMadScientist/CHOpt"),
                        link("CHOpt Truss on GitHub", "https://github.com/graindead/chopt-tr"),
                    ]
                    .spacing(8),
                ]
                .spacing(8)
                .into()
            ),
        ]
        .spacing(14)
        .max_width(720);
        scrollable(container(page).padding(24).center_x(Length::Fill)).height(Length::Fill).into()
    }
}
