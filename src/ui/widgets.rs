use ratatui::{
    layout::{Alignment as RatatuiAlignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::Paragraph,
    Frame,
};

use crate::config::{
    Alignment, Direction as LayoutDirection, Layout as LayoutConfig, LayoutChild,
    LabelConfig, ProgressConfig, VolumeConfig, WidgetConfig,
};
use crate::cover::CoverArtLoader;
use crate::mpris_client::{format_duration, PlayerState};
use std::collections::HashMap;

impl From<LayoutDirection> for Direction {
    fn from(d: LayoutDirection) -> Self {
        match d {
            LayoutDirection::Vertical => Direction::Vertical,
            LayoutDirection::Horizontal => Direction::Horizontal,
        }
    }
}

/// Track areas where interactive widgets are rendered
#[derive(Default, Clone, Copy)]
pub struct WidgetAreas {
    pub controls: Option<Rect>,
    pub progress: Option<Rect>,
}

/// Render the layout to the frame
pub fn render_layout(
    frame: &mut Frame,
    area: Rect,
    layout: &LayoutConfig,
    widgets: &HashMap<String, WidgetConfig>,
    state: &PlayerState,
    cover_loader: &mut CoverArtLoader,
) -> WidgetAreas {
    let mut widget_areas = WidgetAreas::default();

    if layout.children.is_empty() {
        return widget_areas;
    }

    let direction: Direction = layout.direction.into();

    // Calculate constraints based on widget types
    let constraints: Vec<Constraint> = layout
        .children
        .iter()
        .map(|child| match child {
            LayoutChild::Widget(name) => {
                if let Some(widget) = widgets.get(name) {
                    match widget {
                        WidgetConfig::Progress(_) | WidgetConfig::Volume(_) => {
                            if direction == Direction::Horizontal {
                                Constraint::Min(10)
                            } else {
                                Constraint::Length(1)
                            }
                        }
                        WidgetConfig::Label(_) => {
                            if direction == Direction::Horizontal {
                                // Fixed width for time labels, flexible for others
                                match name.as_str() {
                                    "position" | "length" => Constraint::Length(6),
                                    _ => Constraint::Min(1),
                                }
                            } else {
                                Constraint::Length(1)
                            }
                        }
                        WidgetConfig::Button(_) => Constraint::Length(1),
                        WidgetConfig::CoverArt(_) => {
                            // Square proportions: width = height * 2 (terminal chars are ~2:1)
                            // Cap at reasonable size to not crush other elements
                            if direction == Direction::Horizontal {
                                let width = area.height.saturating_mul(2).min(area.width / 2);
                                Constraint::Length(width)
                            } else {
                                let height = (area.width / 2).min(area.height / 2);
                                Constraint::Length(height)
                            }
                        }
                        WidgetConfig::Empty(c) => {
                            if direction == Direction::Vertical {
                                // If no height specified, use flexible space (for centering)
                                match c.height {
                                    Some(h) => Constraint::Length(h),
                                    None => Constraint::Min(0),
                                }
                            } else {
                                match c.width {
                                    Some(w) => Constraint::Length(w),
                                    None => Constraint::Min(0),
                                }
                            }
                        }
                    }
                } else {
                    Constraint::Length(1)
                }
            }
            LayoutChild::Container(nested) => {
                // Horizontal containers in vertical layout = 1 row
                // Vertical containers in horizontal layout = flexible
                let nested_dir: Direction = nested.direction.into();
                if direction == Direction::Vertical && nested_dir == Direction::Horizontal {
                    Constraint::Length(1)
                } else {
                    Constraint::Min(1)
                }
            }
        })
        .collect();

    let chunks = Layout::default()
        .direction(direction)
        .constraints(constraints)
        .split(area);

    for (i, child) in layout.children.iter().enumerate() {
        if i >= chunks.len() {
            break;
        }
        match child {
            LayoutChild::Widget(name) => {
                if let Some(widget_config) = widgets.get(name) {
                    render_widget(frame, chunks[i], widget_config, state, cover_loader);

                    // Track interactive widget areas
                    if name == "controls" {
                        widget_areas.controls = Some(chunks[i]);
                    } else if name == "progress" {
                        widget_areas.progress = Some(chunks[i]);
                    }
                }
            }
            LayoutChild::Container(nested) => {
                let nested_areas = render_layout(frame, chunks[i], nested, widgets, state, cover_loader);
                // Merge nested areas
                if nested_areas.controls.is_some() {
                    widget_areas.controls = nested_areas.controls;
                }
                if nested_areas.progress.is_some() {
                    widget_areas.progress = nested_areas.progress;
                }
            }
        }
    }

    widget_areas
}

/// Render a single widget
fn render_widget(
    frame: &mut Frame,
    area: Rect,
    config: &WidgetConfig,
    state: &PlayerState,
    cover_loader: &mut CoverArtLoader,
) {
    match config {
        WidgetConfig::Label(cfg) => render_label(frame, area, cfg, state),
        WidgetConfig::Progress(cfg) => render_progress(frame, area, cfg, state),
        WidgetConfig::Volume(cfg) => render_volume(frame, area, cfg, state),
        WidgetConfig::Button(cfg) => render_button(frame, area, cfg, state),
        WidgetConfig::CoverArt(cfg) => render_cover_art(frame, area, cfg, state, cover_loader),
        WidgetConfig::Empty(_) => {}
    }
}

/// Substitute variables in text
fn substitute_vars(text: &str, state: &PlayerState) -> String {
    text.replace("$title", &state.title)
        .replace("$artists", &state.artists)
        .replace("$artist", &state.artists)
        .replace("$album", &state.album)
        .replace("$status-icon", state.status.icon())
        .replace("$status", &format!("{:?}", state.status))
        .replace("$position", &format_duration(state.position))
        .replace("$length", &format_duration(state.length))
        .replace("$volume", &format!("{}%", (state.volume * 100.0) as u8))
        .replace("$player", &state.player_name)
}

/// Render a label widget
fn render_label(frame: &mut Frame, area: Rect, config: &LabelConfig, state: &PlayerState) {
    let text = substitute_vars(&config.text, state);
    let alignment = match config.align {
        Alignment::Left => RatatuiAlignment::Left,
        Alignment::Center => RatatuiAlignment::Center,
        Alignment::Right => RatatuiAlignment::Right,
    };

    let style = build_style(&config.style);

    let paragraph = Paragraph::new(text)
        .alignment(alignment)
        .style(style);

    frame.render_widget(paragraph, area);
}

/// Render a progress bar widget
fn render_progress(frame: &mut Frame, area: Rect, config: &ProgressConfig, state: &PlayerState) {
    let progress = if state.length.as_secs() > 0 {
        (state.position.as_secs_f64() / state.length.as_secs_f64()).clamp(0.0, 1.0)
    } else {
        0.0
    };

    if config.show_time {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(6),
                Constraint::Min(10),
                Constraint::Length(6),
            ])
            .split(area);

        let pos_text = format_duration(state.position);
        frame.render_widget(
            Paragraph::new(pos_text).alignment(RatatuiAlignment::Right),
            chunks[0],
        );

        render_progress_bar(frame, chunks[1], progress, config);

        let len_text = format_duration(state.length);
        frame.render_widget(
            Paragraph::new(len_text).alignment(RatatuiAlignment::Left),
            chunks[2],
        );
    } else {
        render_progress_bar(frame, area, progress, config);
    }
}

fn render_progress_bar(frame: &mut Frame, area: Rect, progress: f64, config: &ProgressConfig) {
    let width = area.width as usize;
    let filled = (progress * width as f64) as usize;
    let empty = width.saturating_sub(filled);

    let mut bar = String::with_capacity(width * 4); // UTF-8 chars can be up to 4 bytes
    for _ in 0..filled {
        bar.push(config.filled_char);
    }
    for _ in 0..empty {
        bar.push(config.empty_char);
    }

    let style = build_style(&config.style);
    frame.render_widget(Paragraph::new(bar).style(style), area);
}

fn render_volume(frame: &mut Frame, area: Rect, config: &VolumeConfig, state: &PlayerState) {
    let volume = state.volume.clamp(0.0, 1.0);

    if config.show_percentage {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(5),
                Constraint::Min(10),
            ])
            .split(area);

        let vol_text = format!("{:3}%", (volume * 100.0) as u8);
        frame.render_widget(
            Paragraph::new(vol_text).alignment(RatatuiAlignment::Right),
            chunks[0],
        );

        render_volume_bar(frame, chunks[1], volume, config);
    } else {
        render_volume_bar(frame, area, volume, config);
    }
}

fn render_volume_bar(frame: &mut Frame, area: Rect, volume: f64, config: &VolumeConfig) {
    let width = area.width as usize;
    let filled = (volume * width as f64) as usize;
    let empty = width.saturating_sub(filled);

    let mut bar = String::with_capacity(width * 4); // UTF-8 chars can be up to 4 bytes
    for _ in 0..filled {
        bar.push(config.filled_char);
    }
    for _ in 0..empty {
        bar.push(config.empty_char);
    }

    let style = build_style(&config.style);
    frame.render_widget(Paragraph::new(bar).style(style), area);
}

fn render_button(
    frame: &mut Frame,
    area: Rect,
    config: &crate::config::ButtonConfig,
    state: &PlayerState,
) {
    let text = substitute_vars(&config.text, state);
    let style = build_style(&config.style);

    let paragraph = Paragraph::new(text)
        .alignment(RatatuiAlignment::Center)
        .style(style);

    frame.render_widget(paragraph, area);
}

fn render_cover_art(frame: &mut Frame, area: Rect, config: &crate::config::CoverArtConfig, state: &PlayerState, cover_loader: &mut CoverArtLoader) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    // Request cover art if we have a URL
    if let Some(ref url) = state.art_url {
        cover_loader.request(url);

        // Render with colored half-blocks (like fum)
        if let Some(cover) = cover_loader.get(url) {
            let lines = render_image_halfblocks(&cover.image, area.width as usize, area.height as usize, config.true_color);
            if !lines.is_empty() {
                let paragraph = Paragraph::new(lines);
                frame.render_widget(paragraph, area);
                return;
            }
        }
    }

    // Fallback: show placeholder
    let text = if state.art_url.is_some() {
        "Loading..."
    } else {
        "[No Cover]"
    };

    let v_pad = area.height.saturating_sub(1) / 2;
    let mut lines: Vec<Line> = (0..v_pad).map(|_| Line::from("")).collect();
    lines.push(Line::from(text));

    let paragraph = Paragraph::new(lines).alignment(RatatuiAlignment::Center);
    frame.render_widget(paragraph, area);
}

/// Map RGB to the nearest of the 16 standard terminal colors.
fn rgb_to_ansi16(r: u8, g: u8, b: u8) -> Color {
    // Standard 16-color ANSI palette (approximate RGB values)
    const PALETTE: [(u8, u8, u8, Color); 16] = [
        (0, 0, 0, Color::Black),
        (128, 0, 0, Color::Red),
        (0, 128, 0, Color::Green),
        (128, 128, 0, Color::Yellow),
        (0, 0, 128, Color::Blue),
        (128, 0, 128, Color::Magenta),
        (0, 128, 128, Color::Cyan),
        (192, 192, 192, Color::Gray),
        (128, 128, 128, Color::DarkGray),
        (255, 0, 0, Color::LightRed),
        (0, 255, 0, Color::LightGreen),
        (255, 255, 0, Color::LightYellow),
        (0, 0, 255, Color::LightBlue),
        (255, 0, 255, Color::LightMagenta),
        (0, 255, 255, Color::LightCyan),
        (255, 255, 255, Color::White),
    ];

    let mut best_color = Color::Black;
    let mut best_dist = u32::MAX;

    for (pr, pg, pb, color) in PALETTE {
        // Euclidean distance in RGB space
        let dr = (r as i32 - pr as i32).pow(2) as u32;
        let dg = (g as i32 - pg as i32).pow(2) as u32;
        let db = (b as i32 - pb as i32).pow(2) as u32;
        let dist = dr + dg + db;

        if dist < best_dist {
            best_dist = dist;
            best_color = color;
        }
    }

    best_color
}

/// Render image using colored half-block characters (▄) like fum
fn render_image_halfblocks(img: &image::DynamicImage, target_width: usize, target_height: usize, true_color: bool) -> Vec<Line<'static>> {
    use ratatui::text::Span;
    use image::GenericImageView;

    if target_width == 0 || target_height == 0 {
        return vec![];
    }

    // Resize image to fit (height * 2 because each char represents 2 vertical pixels)
    let resized = img.thumbnail(target_width as u32, target_height.saturating_mul(2) as u32);
    let (img_w, img_h) = resized.dimensions();

    let mut lines = Vec::with_capacity(target_height);

    // Process 2 pixel rows at a time
    for term_y in 0..target_height {
        let img_y_top = term_y.saturating_mul(2) as u32;
        let img_y_bot = term_y.saturating_mul(2).saturating_add(1) as u32;

        let mut spans = Vec::with_capacity(target_width);

        for term_x in 0..target_width {
            let img_x = term_x as u32;

            if img_x >= img_w {
                spans.push(Span::raw(" "));
                continue;
            }

            let top_pixel = if img_y_top < img_h {
                let p = resized.get_pixel(img_x, img_y_top);
                (p[0], p[1], p[2])
            } else {
                (0, 0, 0)
            };

            let bot_pixel = if img_y_bot < img_h {
                let p = resized.get_pixel(img_x, img_y_bot);
                (p[0], p[1], p[2])
            } else {
                (0, 0, 0)
            };

            // ▄ = lower half block: foreground = bottom pixel, background = top pixel
            let (fg, bg) = if true_color {
                (
                    Color::Rgb(bot_pixel.0, bot_pixel.1, bot_pixel.2),
                    Color::Rgb(top_pixel.0, top_pixel.1, top_pixel.2),
                )
            } else {
                (
                    rgb_to_ansi16(bot_pixel.0, bot_pixel.1, bot_pixel.2),
                    rgb_to_ansi16(top_pixel.0, top_pixel.1, top_pixel.2),
                )
            };

            spans.push(Span::styled(
                "▄",
                Style::default().fg(fg).bg(bg),
            ));
        }

        lines.push(Line::from(spans));
    }

    lines
}

fn build_style(config: &crate::config::StyleConfig) -> Style {
    let mut style = Style::default();

    if let Some(ref fg) = config.fg {
        if let Some(color) = parse_color(fg) {
            style = style.fg(color);
        }
    }

    if let Some(ref bg) = config.bg {
        if let Some(color) = parse_color(bg) {
            style = style.bg(color);
        }
    }

    let mut modifier = Modifier::empty();
    if config.bold {
        modifier |= Modifier::BOLD;
    }
    if config.italic {
        modifier |= Modifier::ITALIC;
    }
    if config.underline {
        modifier |= Modifier::UNDERLINED;
    }

    style.add_modifier(modifier)
}

fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim().to_lowercase();

    match s.as_str() {
        "black" => return Some(Color::Black),
        "red" => return Some(Color::Red),
        "green" => return Some(Color::Green),
        "yellow" => return Some(Color::Yellow),
        "blue" => return Some(Color::Blue),
        "magenta" => return Some(Color::Magenta),
        "cyan" => return Some(Color::Cyan),
        "gray" | "grey" => return Some(Color::Gray),
        "white" => return Some(Color::White),
        "darkgray" | "darkgrey" => return Some(Color::DarkGray),
        "lightred" => return Some(Color::LightRed),
        "lightgreen" => return Some(Color::LightGreen),
        "lightyellow" => return Some(Color::LightYellow),
        "lightblue" => return Some(Color::LightBlue),
        "lightmagenta" => return Some(Color::LightMagenta),
        "lightcyan" => return Some(Color::LightCyan),
        _ => {}
    }

    if let Some(hex) = s.strip_prefix('#') {
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            return Some(Color::Rgb(r, g, b));
        } else if hex.len() == 3 {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
            return Some(Color::Rgb(r, g, b));
        }
    }

    None
}
