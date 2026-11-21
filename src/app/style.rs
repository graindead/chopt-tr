//! Truss's look: quiet styling with hairline dividers, muted secondary text and the accent
//! colour kept for what you act on. Colours come from the current theme.

use iced::widget::{button, container, pick_list, text_input};
use iced::{Background, Border, Color, Theme};

fn mix(a: Color, b: Color, t: f32) -> Color {
    Color { r: a.r + (b.r - a.r) * t, g: a.g + (b.g - a.g) * t, b: a.b + (b.b - a.b) * t, a: 1.0 }
}

pub fn bg(t: &Theme) -> Color {
    t.extended_palette().background.base.color
}

pub fn fg(t: &Theme) -> Color {
    t.extended_palette().background.base.text
}

/// Secondary text: labels, hints.
pub fn muted(t: &Theme) -> Color {
    mix(bg(t), fg(t), 0.55)
}

/// Dividers and outlines.
pub fn hairline(t: &Theme) -> Color {
    mix(bg(t), fg(t), 0.11)
}

/// A slightly raised surface (inputs).
pub fn raised(t: &Theme) -> Color {
    mix(bg(t), fg(t), 0.035)
}

pub fn accent(t: &Theme) -> Color {
    t.extended_palette().primary.base.color
}

/// A one-pixel divider.
pub fn divider<'a, M: 'a>(vertical: bool) -> iced::Element<'a, M> {
    let c = container(iced::widget::space()).style(|t: &Theme| container::Style {
        background: Some(hairline(t).into()),
        ..Default::default()
    });
    if vertical { c.width(1).height(iced::Length::Fill).into() } else { c.height(1).width(iced::Length::Fill).into() }
}

/// Top navigation: muted text, the current page in full colour.
pub fn nav_button(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |t, s| button::Style {
        background: None,
        text_color: if active || matches!(s, button::Status::Hovered) { fg(t) } else { muted(t) },
        ..Default::default()
    }
}

/// A quiet outlined button.
pub fn secondary(t: &Theme, s: button::Status) -> button::Style {
    let hover = matches!(s, button::Status::Hovered | button::Status::Pressed);
    button::Style {
        background: Some(Background::Color(if hover { raised(t) } else { bg(t) })),
        text_color: if s == button::Status::Disabled { muted(t) } else { fg(t) },
        border: Border { color: hairline(t), width: 1.0, radius: 6.0.into() },
        ..Default::default()
    }
}

/// The main action, in the accent colour.
pub fn primary(t: &Theme, s: button::Status) -> button::Style {
    let mut st = button::primary(t, s);
    st.border.radius = 6.0.into();
    st
}

pub fn input(t: &Theme, s: text_input::Status) -> text_input::Style {
    let mut st = text_input::default(t, s);
    st.background = Background::Color(raised(t));
    st.border = Border {
        color: if matches!(s, text_input::Status::Focused { .. }) { accent(t) } else { hairline(t) },
        width: 1.0,
        radius: 6.0.into(),
    };
    st.placeholder = muted(t);
    st
}

pub fn quiet_pick_list(t: &Theme, s: pick_list::Status) -> pick_list::Style {
    let mut st = pick_list::default(t, s);
    st.background = Background::Color(bg(t));
    st.border = Border {
        color: if matches!(s, pick_list::Status::Hovered | pick_list::Status::Opened { .. }) { muted(t) } else { hairline(t) },
        width: 1.0,
        radius: 6.0.into(),
    };
    st.handle_color = muted(t);
    st
}

/// A hairline-outlined panel on the page background.
pub fn panel(t: &Theme) -> container::Style {
    container::Style {
        background: None,
        border: Border { color: hairline(t), width: 1.0, radius: 6.0.into() },
        ..Default::default()
    }
}

/// Muted text.
pub fn muted_text(t: &Theme) -> iced::widget::text::Style {
    iced::widget::text::Style { color: Some(muted(t)) }
}
