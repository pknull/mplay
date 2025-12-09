use image::DynamicImage;
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;

/// Cover art cache and loader
pub struct CoverArtLoader {
    cache: HashMap<String, Option<CoverArtImage>>,
    pending: Option<String>,
    receiver: Receiver<(String, Option<DynamicImage>)>,
    sender: Sender<(String, Option<DynamicImage>)>,
    picker: Option<Picker>,
}

/// Loaded cover art image
pub struct CoverArtImage {
    pub image: DynamicImage,
    pub protocol: Option<StatefulProtocol>,
}

impl CoverArtLoader {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        // Try to create a picker for the terminal's image protocol
        let picker = Picker::from_query_stdio().ok();

        Self {
            cache: HashMap::new(),
            pending: None,
            receiver: rx,
            sender: tx,
            picker,
        }
    }

    /// Request cover art for a URL (non-blocking)
    pub fn request(&mut self, url: &str) {
        // Check cache first
        if self.cache.contains_key(url) {
            return;
        }

        // Check if already loading (compare without allocation)
        if self.pending.as_deref() == Some(url) {
            return;
        }

        // Start loading in background (single allocation)
        let url_owned = url.to_string();
        self.pending = Some(url_owned.clone());
        let tx = self.sender.clone();

        thread::spawn(move || {
            let result = load_image(&url_owned);
            let _ = tx.send((url_owned, result));
        });
    }

    /// Get cover art if available, creating protocol image if needed
    pub fn get(&mut self, url: &str) -> Option<&mut CoverArtImage> {
        // Check for completed loads
        loop {
            match self.receiver.try_recv() {
                Ok((loaded_url, img_opt)) => {
                    if self.pending.as_ref() == Some(&loaded_url) {
                        self.pending = None;
                    }
                    let cover = img_opt.map(|image| CoverArtImage {
                        image,
                        protocol: None,
                    });
                    self.cache.insert(loaded_url, cover);
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }

        // Get from cache and create protocol if needed
        let entry = self.cache.get_mut(url)?;
        let cover = entry.as_mut()?;

        // Create protocol image if we have a picker and don't have one yet
        if cover.protocol.is_none() {
            if let Some(ref mut picker) = self.picker {
                let protocol = picker.new_resize_protocol(cover.image.clone());
                cover.protocol = Some(protocol);
            }
        }

        Some(cover)
    }
}

fn load_image(url: &str) -> Option<DynamicImage> {
    if url.starts_with("file://") {
        let path = url.strip_prefix("file://")?;
        let path = urlencoding::decode(path).ok()?;
        // Read file bytes and detect format from content (not extension)
        let bytes = std::fs::read(Path::new(path.as_ref())).ok()?;
        image::load_from_memory(&bytes).ok()
    } else if url.starts_with("http://") || url.starts_with("https://") {
        let response = ureq::get(url)
            .timeout(std::time::Duration::from_secs(10))
            .call()
            .ok()?;

        let mut bytes = Vec::new();
        response.into_reader().read_to_end(&mut bytes).ok()?;
        image::load_from_memory(&bytes).ok()
    } else {
        // Try as direct file path - read bytes to detect format from content
        let bytes = std::fs::read(Path::new(url)).ok()?;
        image::load_from_memory(&bytes).ok()
    }
}
