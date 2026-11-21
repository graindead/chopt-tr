//! Truss's themes (the same palettes), then iced's own.

use iced::theme::Palette;
use iced::{Theme, color};

/// The default theme: warm charcoal with Rust-orange accents.
pub fn rust_theme() -> Theme {
    Theme::custom(
        "Rust",
        Palette {
            background: color!(0x1b1715),
            text: color!(0xefe6de),
            primary: color!(0xe2622a),
            success: color!(0x7cc47f),
            warning: color!(0xf2b545),
            danger: color!(0xe5484d),
        },
    )
}

/// Deep navy with a clear blue accent.
fn kusha_theme() -> Theme {
    Theme::custom(
        "Kusha",
        Palette {
            background: color!(0x131924),
            text: color!(0xe2e9f4),
            primary: color!(0x3b8ef0),
            success: color!(0x5cc89c),
            warning: color!(0xf2c14e),
            danger: color!(0xeb5a6b),
        },
    )
}

/// Deep plum with a vivid purple accent.
fn nick_theme() -> Theme {
    Theme::custom(
        "Nick",
        Palette {
            background: color!(0x19141f),
            text: color!(0xece4f4),
            primary: color!(0x9d5cf0),
            success: color!(0x6cc79a),
            warning: color!(0xf2c14e),
            danger: color!(0xeb5a72),
        },
    )
}

fn bridge_dark() -> Theme {
    Theme::custom(
        "Bridge Dark",
        Palette {
            background: color!(0x15161c),
            text: color!(0xe4e4ea),
            primary: color!(0x7c5cff),
            success: color!(0x3ccf8e),
            warning: color!(0xf5b942),
            danger: color!(0xf0506e),
        },
    )
}

/// Every theme, in Truss's order (Rust first).
pub fn all() -> Vec<Theme> {
    [rust_theme(), kusha_theme(), nick_theme(), bridge_dark()].into_iter().chain(Theme::ALL.iter().cloned()).collect()
}

pub fn by_name(name: &str) -> Theme {
    all().into_iter().find(|t| t.to_string() == name).unwrap_or_else(rust_theme)
}
