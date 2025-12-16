use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Players to try connecting to (in order)
    pub players: Vec<String>,
    /// Keybindings
    pub keybinds: Keybinds,
    /// Layout configuration
    pub layout: Layout,
    /// Widget configurations
    pub widgets: HashMap<String, WidgetConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            players: vec!["spotify".into(), "vlc".into(), "mpd".into()],
            keybinds: Keybinds::default(),
            layout: Layout::default(),
            widgets: default_widgets(),
        }
    }
}

/// Keybinding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Keybinds {
    pub quit: Vec<String>,
    pub toggle: Vec<String>,
    pub next: Vec<String>,
    pub prev: Vec<String>,
    pub seek_forward: Vec<String>,
    pub seek_backward: Vec<String>,
    pub volume_up: Vec<String>,
    pub volume_down: Vec<String>,
}

impl Default for Keybinds {
    fn default() -> Self {
        Self {
            quit: vec!["q".into(), "Escape".into()],
            toggle: vec![" ".into()],
            next: vec!["n".into(), "Right".into()],
            prev: vec!["p".into(), "Left".into()],
            seek_forward: vec!["l".into(), "Shift+Right".into()],
            seek_backward: vec!["h".into(), "Shift+Left".into()],
            volume_up: vec!["k".into(), "Up".into()],
            volume_down: vec!["j".into(), "Down".into()],
        }
    }
}

/// Layout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Layout {
    pub direction: Direction,
    pub children: Vec<LayoutChild>,
}

impl Default for Layout {
    fn default() -> Self {
        // Winamp-style: cover art on left, info on right (vertically centered)
        Self {
            direction: Direction::Horizontal,
            children: vec![
                LayoutChild::Widget("cover".into()),
                LayoutChild::Container(Layout {
                    direction: Direction::Vertical,
                    children: vec![
                        LayoutChild::Widget("spacer_top".into()),
                        LayoutChild::Widget("title".into()),
                        LayoutChild::Widget("artists".into()),
                        LayoutChild::Widget("album".into()),
                        // Progress bar with horizontal padding
                        LayoutChild::Container(Layout {
                            direction: Direction::Horizontal,
                            children: vec![
                                LayoutChild::Widget("pad_left".into()),
                                LayoutChild::Widget("progress".into()),
                                LayoutChild::Widget("pad_right".into()),
                            ],
                        }),
                        // Status line: position | status icon | length
                        LayoutChild::Container(Layout {
                            direction: Direction::Horizontal,
                            children: vec![
                                LayoutChild::Widget("pad_left".into()),
                                LayoutChild::Widget("position".into()),
                                LayoutChild::Widget("controls".into()),
                                LayoutChild::Widget("length".into()),
                                LayoutChild::Widget("pad_right".into()),
                            ],
                        }),
                        LayoutChild::Widget("spacer_bottom".into()),
                    ],
                }),
            ],
        }
    }
}

/// Layout direction
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    #[default]
    Vertical,
    Horizontal,
}

/// Layout child - can be a widget or nested container
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LayoutChild {
    Widget(String),
    Container(Layout),
}

/// Widget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum WidgetConfig {
    Label(LabelConfig),
    Progress(ProgressConfig),
    Volume(VolumeConfig),
    Button(ButtonConfig),
    CoverArt(CoverArtConfig),
    Empty(EmptyConfig),
}

/// Label widget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LabelConfig {
    pub text: String,
    pub align: Alignment,
    pub style: StyleConfig,
}

impl Default for LabelConfig {
    fn default() -> Self {
        Self {
            text: String::new(),
            align: Alignment::Center,
            style: StyleConfig::default(),
        }
    }
}

/// Progress bar configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProgressConfig {
    pub show_time: bool,
    pub filled_char: char,
    pub empty_char: char,
    pub style: StyleConfig,
}

impl Default for ProgressConfig {
    fn default() -> Self {
        Self {
            show_time: true,
            filled_char: '█',
            empty_char: '░',
            style: StyleConfig::default(),
        }
    }
}

/// Volume widget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VolumeConfig {
    pub show_percentage: bool,
    pub filled_char: char,
    pub empty_char: char,
    pub style: StyleConfig,
}

impl Default for VolumeConfig {
    fn default() -> Self {
        Self {
            show_percentage: true,
            filled_char: '█',
            empty_char: '░',
            style: StyleConfig::default(),
        }
    }
}

/// Button widget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ButtonConfig {
    pub action: ButtonAction,
    pub text: String,
    pub style: StyleConfig,
}

impl Default for ButtonConfig {
    fn default() -> Self {
        Self {
            action: ButtonAction::Toggle,
            text: "$status-icon".into(),
            style: StyleConfig::default(),
        }
    }
}

/// Button actions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ButtonAction {
    #[default]
    Toggle,
    Next,
    Prev,
    VolumeUp,
    VolumeDown,
}

/// Cover art configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CoverArtConfig {
    pub width: Option<u16>,
    pub height: Option<u16>,
    pub use_ascii: bool,
    /// Use 24-bit true color for cover art. When false, uses 16 standard terminal colors.
    pub true_color: bool,
}

impl Default for CoverArtConfig {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            use_ascii: false,
            true_color: false,
        }
    }
}

/// Empty widget for spacing
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct EmptyConfig {
    pub height: Option<u16>,
    pub width: Option<u16>,
}

/// Text alignment
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Alignment {
    Left,
    #[default]
    Center,
    Right,
}

/// Style configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct StyleConfig {
    pub fg: Option<String>,
    pub bg: Option<String>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

/// Create default widget configurations
fn default_widgets() -> HashMap<String, WidgetConfig> {
    let mut widgets = HashMap::new();

    widgets.insert("title".into(), WidgetConfig::Label(LabelConfig {
        text: "$title".into(),
        align: Alignment::Center,
        style: StyleConfig { bold: true, ..Default::default() },
    }));

    widgets.insert("artists".into(), WidgetConfig::Label(LabelConfig {
        text: "$artists".into(),
        align: Alignment::Center,
        style: StyleConfig::default(),
    }));

    widgets.insert("album".into(), WidgetConfig::Label(LabelConfig {
        text: "$album".into(),
        align: Alignment::Center,
        style: StyleConfig { italic: true, ..Default::default() },
    }));

    widgets.insert("progress".into(), WidgetConfig::Progress(ProgressConfig {
        show_time: false,
        ..Default::default()
    }));

    widgets.insert("position".into(), WidgetConfig::Label(LabelConfig {
        text: "$position".into(),
        align: Alignment::Left,
        style: StyleConfig::default(),
    }));

    widgets.insert("length".into(), WidgetConfig::Label(LabelConfig {
        text: "$length".into(),
        align: Alignment::Right,
        style: StyleConfig::default(),
    }));

    widgets.insert("controls".into(), WidgetConfig::Label(LabelConfig {
        text: "$status-icon".into(),
        align: Alignment::Center,
        style: StyleConfig::default(),
    }));

    widgets.insert("volume".into(), WidgetConfig::Volume(VolumeConfig::default()));

    widgets.insert("cover".into(), WidgetConfig::CoverArt(CoverArtConfig::default()));

    // Flexible spacers for vertical centering
    widgets.insert("spacer_top".into(), WidgetConfig::Empty(EmptyConfig::default()));
    widgets.insert("spacer_bottom".into(), WidgetConfig::Empty(EmptyConfig::default()));

    // Fixed-width padding for horizontal spacing
    widgets.insert("pad_left".into(), WidgetConfig::Empty(EmptyConfig { width: Some(1), height: None }));
    widgets.insert("pad_right".into(), WidgetConfig::Empty(EmptyConfig { width: Some(1), height: None }));

    widgets
}

impl Config {
    /// Load configuration from file
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config from {:?}", config_path))?;

            // Parse JSONC (JSON with comments)
            let config: Config = json5::from_str(&content)
                .with_context(|| "Failed to parse config file")?;

            Ok(config)
        } else {
            // Create default config
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory {:?}", parent))?;
        }

        let content = serde_json::to_string_pretty(self)
            .with_context(|| "Failed to serialize config")?;

        fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config to {:?}", config_path))?;

        Ok(())
    }

    /// Get the config file path
    pub fn config_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("", "", "mplay")
            .context("Failed to determine config directory")?;

        Ok(proj_dirs.config_dir().join("config.json"))
    }
}
