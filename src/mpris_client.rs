use anyhow::{Context, Result};
use mpris::{Metadata, PlaybackStatus, Player, PlayerFinder};
use std::time::Duration;

/// Current player state
#[derive(Debug, Clone, Default)]
pub struct PlayerState {
    pub connected: bool,
    pub player_name: String,
    pub title: String,
    pub artists: String,
    pub album: String,
    pub art_url: Option<String>,
    pub status: Status,
    pub position: Duration,
    pub length: Duration,
    pub volume: f64,
}

/// Playback status
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Status {
    Playing,
    Paused,
    #[default]
    Stopped,
}

impl From<PlaybackStatus> for Status {
    fn from(s: PlaybackStatus) -> Self {
        match s {
            PlaybackStatus::Playing => Status::Playing,
            PlaybackStatus::Paused => Status::Paused,
            PlaybackStatus::Stopped => Status::Stopped,
        }
    }
}

impl Status {
    pub fn icon(&self) -> &'static str {
        match self {
            Status::Playing => "⏸",
            Status::Paused => "▶",
            Status::Stopped => "⏹",
        }
    }
}

/// MPRIS client for controlling media players
pub struct MprisClient {
    player: Option<Player>,
    preferred_players: Vec<String>,
}

impl MprisClient {
    /// Create a new MPRIS client
    pub fn new(preferred_players: Vec<String>) -> Self {
        Self {
            player: None,
            preferred_players,
        }
    }

    /// Try to connect to a media player
    pub fn connect(&mut self) -> Result<bool> {
        let finder = PlayerFinder::new()
            .context("Failed to create player finder")?;

        // Try preferred players first
        for preferred in &self.preferred_players {
            let preferred_lower = preferred.to_lowercase();
            if let Ok(players) = finder.find_all() {
                for player in players {
                    let identity = player.identity().to_lowercase();
                    if identity.contains(&preferred_lower) {
                        self.player = Some(player);
                        return Ok(true);
                    }
                }
            }
        }

        // Fall back to any active player
        if let Ok(player) = finder.find_active() {
            self.player = Some(player);
            return Ok(true);
        }

        // Try first available player
        if let Ok(players) = finder.find_all() {
            if let Some(player) = players.into_iter().next() {
                self.player = Some(player);
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Check if still connected and reconnect if needed
    pub fn ensure_connected(&mut self) -> bool {
        if let Some(ref player) = self.player {
            // Check if player is still valid by trying to get identity
            if player.is_running() {
                return true;
            }
        }

        // Try to reconnect
        self.player = None;
        self.connect().unwrap_or(false)
    }

    /// Get current player state
    pub fn get_state(&mut self) -> PlayerState {
        if !self.ensure_connected() {
            return PlayerState::default();
        }

        let player = match &self.player {
            Some(p) => p,
            None => return PlayerState::default(),
        };

        let metadata = player.get_metadata().ok();
        let status = player
            .get_playback_status()
            .map(Status::from)
            .unwrap_or_default();

        let position = player
            .get_position()
            .unwrap_or(Duration::ZERO);

        let length = metadata
            .as_ref()
            .and_then(|m| m.length())
            .unwrap_or(Duration::ZERO);

        let volume = player
            .get_volume()
            .unwrap_or(1.0)
            .clamp(0.0, 1.0);

        PlayerState {
            connected: true,
            player_name: player.identity().to_string(),
            title: extract_title(&metadata),
            artists: extract_artists(&metadata),
            album: extract_album(&metadata),
            art_url: metadata.as_ref().and_then(|m| m.art_url().map(String::from)),
            status,
            position,
            length,
            volume,
        }
    }

    /// Toggle play/pause
    pub fn toggle(&mut self) -> Result<()> {
        if let Some(ref player) = self.player {
            player.play_pause()
                .context("Failed to toggle playback")?;
        }
        Ok(())
    }

    /// Next track
    pub fn next(&mut self) -> Result<()> {
        if let Some(ref player) = self.player {
            player.next()
                .context("Failed to go to next track")?;
        }
        Ok(())
    }

    /// Previous track
    pub fn prev(&mut self) -> Result<()> {
        if let Some(ref player) = self.player {
            player.previous()
                .context("Failed to go to previous track")?;
        }
        Ok(())
    }

    /// Seek forward by duration
    pub fn seek_forward(&mut self, duration: Duration) -> Result<()> {
        if let Some(ref player) = self.player {
            let offset = duration.as_micros() as i64;
            player.seek(offset)
                .context("Failed to seek forward")?;
        }
        Ok(())
    }

    /// Seek backward by duration
    pub fn seek_backward(&mut self, duration: Duration) -> Result<()> {
        if let Some(ref player) = self.player {
            let offset = -(duration.as_micros() as i64);
            player.seek(offset)
                .context("Failed to seek backward")?;
        }
        Ok(())
    }

    /// Set position
    pub fn set_position(&mut self, position: Duration) -> Result<()> {
        if let Some(ref player) = self.player {
            if let Ok(metadata) = player.get_metadata() {
                if let Some(track_id) = metadata.track_id() {
                    player.set_position(track_id, &position)
                        .context("Failed to set position")?;
                }
            }
        }
        Ok(())
    }

    /// Adjust volume by delta
    pub fn adjust_volume(&mut self, delta: f64) -> Result<()> {
        if let Some(ref player) = self.player {
            let current = player.get_volume().unwrap_or(1.0);
            let new_volume = (current + delta).clamp(0.0, 1.0);
            player.set_volume(new_volume)
                .context("Failed to adjust volume")?;
        }
        Ok(())
    }
}

fn extract_title(metadata: &Option<Metadata>) -> String {
    metadata
        .as_ref()
        .and_then(|m| m.title().map(String::from))
        .unwrap_or_else(|| "Unknown".into())
}

fn extract_artists(metadata: &Option<Metadata>) -> String {
    metadata
        .as_ref()
        .and_then(|m| m.artists())
        .map(|a| a.join(", "))
        .unwrap_or_else(|| "Unknown Artist".into())
}

fn extract_album(metadata: &Option<Metadata>) -> String {
    metadata
        .as_ref()
        .and_then(|m| m.album_name().map(String::from))
        .unwrap_or_else(|| "Unknown Album".into())
}

/// Format duration as MM:SS
pub fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    let mins = secs / 60;
    let secs = secs % 60;
    format!("{:02}:{:02}", mins, secs)
}
