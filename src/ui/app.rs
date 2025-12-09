use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseButton, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    widgets::Paragraph,
    Frame, Terminal,
};
use std::io;
use std::time::{Duration, Instant};

use crate::config::Config;
use crate::cover::CoverArtLoader;
use crate::mpris_client::{MprisClient, PlayerState};
use super::widgets::{render_layout, WidgetAreas};

/// Main application
pub struct App {
    config: Config,
    mpris: MprisClient,
    state: PlayerState,
    cover_loader: CoverArtLoader,
    running: bool,
    widget_areas: WidgetAreas,
}

impl App {
    /// Create a new App
    pub fn new(config: Config) -> Self {
        let mpris = MprisClient::new(config.players.clone());

        Self {
            config,
            mpris,
            state: PlayerState::default(),
            cover_loader: CoverArtLoader::new(),
            running: true,
            widget_areas: WidgetAreas::default(),
        }
    }

    /// Run the application
    pub fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Initial connection
        self.mpris.connect().ok();
        self.state = self.mpris.get_state();

        // Main loop
        let tick_rate = Duration::from_millis(100);
        let state_update_rate = Duration::from_millis(500);
        let mut last_tick = Instant::now();
        let mut last_state_update = Instant::now();

        while self.running {
            // Draw UI
            terminal.draw(|f| self.ui(f))?;

            // Handle events
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or(Duration::ZERO);

            if event::poll(timeout)? {
                match event::read()? {
                    Event::Key(key) => {
                        self.handle_key(key.code, key.modifiers)?;
                    }
                    Event::Mouse(mouse) => {
                        self.handle_mouse(mouse.kind, mouse.column, mouse.row)?;
                    }
                    _ => {}
                }
            }

            // Update player state less frequently
            if last_state_update.elapsed() >= state_update_rate {
                self.state = self.mpris.get_state();
                last_state_update = Instant::now();
            }

            last_tick = Instant::now();
        }

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    /// Render the UI
    fn ui(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let inner_area = area;

        if !self.state.connected {
            let msg = Paragraph::new("No MPRIS-compatible player found.\nStart a media player and press 'r' to reconnect.");
            frame.render_widget(msg, inner_area);
            return;
        }

        // Render the configured layout and track widget areas
        self.widget_areas = render_layout(
            frame,
            inner_area,
            &self.config.layout,
            &self.config.widgets,
            &self.state,
            &mut self.cover_loader,
        );
    }

    /// Handle mouse events
    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> Result<()> {
        if let MouseEventKind::Down(MouseButton::Left) = kind {
            // Check if click is in the controls area - toggle play/pause
            if let Some(controls_area) = self.widget_areas.controls {
                if col >= controls_area.x
                    && col < controls_area.x + controls_area.width
                    && row >= controls_area.y
                    && row < controls_area.y + controls_area.height
                {
                    self.mpris.toggle()?;
                    self.state = self.mpris.get_state();
                }
            }

            // Check if click is on progress bar for seeking
            if let Some(progress_area) = self.widget_areas.progress {
                if progress_area.width > 0
                    && col >= progress_area.x
                    && col < progress_area.x + progress_area.width
                    && row >= progress_area.y
                    && row < progress_area.y + progress_area.height
                {
                    let rel_col = col.saturating_sub(progress_area.x);
                    let ratio = rel_col as f64 / progress_area.width as f64;
                    let new_pos = Duration::from_secs_f64(self.state.length.as_secs_f64() * ratio);
                    self.mpris.set_position(new_pos)?;
                    self.state = self.mpris.get_state();
                }
            }
        }
        Ok(())
    }

    /// Handle key press
    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> Result<()> {
        let key_str = key_to_string(code, modifiers);

        // Check keybindings
        let keybinds = &self.config.keybinds;

        if keybinds.quit.iter().any(|k| k == &key_str) {
            self.running = false;
        } else if keybinds.toggle.iter().any(|k| k == &key_str) {
            self.mpris.toggle()?;
        } else if keybinds.next.iter().any(|k| k == &key_str) {
            self.mpris.next()?;
        } else if keybinds.prev.iter().any(|k| k == &key_str) {
            self.mpris.prev()?;
        } else if keybinds.seek_forward.iter().any(|k| k == &key_str) {
            self.mpris.seek_forward(Duration::from_secs(5))?;
        } else if keybinds.seek_backward.iter().any(|k| k == &key_str) {
            self.mpris.seek_backward(Duration::from_secs(5))?;
        } else if keybinds.volume_up.iter().any(|k| k == &key_str) {
            self.mpris.adjust_volume(0.05)?;
        } else if keybinds.volume_down.iter().any(|k| k == &key_str) {
            self.mpris.adjust_volume(-0.05)?;
        } else if let KeyCode::Char('r') = code {
            self.mpris.connect()?;
        }

        // Update state after action
        self.state = self.mpris.get_state();

        Ok(())
    }
}

/// Convert key event to string representation
fn key_to_string(code: KeyCode, modifiers: KeyModifiers) -> std::borrow::Cow<'static, str> {
    use std::borrow::Cow;

    // Get base key name (static str when possible, owned for dynamic keys)
    let key_name: Cow<'static, str> = match code {
        KeyCode::Char(' ') => Cow::Borrowed(" "),
        KeyCode::Char(c) => Cow::Owned(c.to_string()),
        KeyCode::Enter => Cow::Borrowed("Enter"),
        KeyCode::Esc => Cow::Borrowed("Escape"),
        KeyCode::Tab => Cow::Borrowed("Tab"),
        KeyCode::Backspace => Cow::Borrowed("Backspace"),
        KeyCode::Delete => Cow::Borrowed("Delete"),
        KeyCode::Left => Cow::Borrowed("Left"),
        KeyCode::Right => Cow::Borrowed("Right"),
        KeyCode::Up => Cow::Borrowed("Up"),
        KeyCode::Down => Cow::Borrowed("Down"),
        KeyCode::Home => Cow::Borrowed("Home"),
        KeyCode::End => Cow::Borrowed("End"),
        KeyCode::PageUp => Cow::Borrowed("PageUp"),
        KeyCode::PageDown => Cow::Borrowed("PageDown"),
        KeyCode::F(n) => Cow::Owned(format!("F{}", n)),
        _ => Cow::Borrowed("Unknown"),
    };

    // Check if we need modifiers
    let has_ctrl = modifiers.contains(KeyModifiers::CONTROL);
    let has_alt = modifiers.contains(KeyModifiers::ALT);
    let has_shift = modifiers.contains(KeyModifiers::SHIFT) && match code {
        KeyCode::Char(c) => !c.is_alphabetic(),
        _ => true,
    };

    // Fast path: no modifiers - return key name without allocation
    if !has_ctrl && !has_alt && !has_shift {
        return key_name;
    }

    // Build modifier string only when needed
    let mut result = String::with_capacity(16);
    if has_ctrl {
        result.push_str("Ctrl+");
    }
    if has_alt {
        result.push_str("Alt+");
    }
    if has_shift {
        result.push_str("Shift+");
    }
    result.push_str(&key_name);
    Cow::Owned(result)
}
