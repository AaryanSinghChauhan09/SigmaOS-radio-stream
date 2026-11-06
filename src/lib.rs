//! SigmaOS-radio-stream — internet radio player model
//! (Omarchy `radio.omarchy.org` / hypnotix parity).
//!
//! Station directory with genre search, favorites, now-playing ICY
//! metadata extraction, and playback state. Pure logic — audio output
//! is delegated to the media subsystem.

/// ICY/Shoutcast-style now-playing metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NowPlaying {
    pub title: String,
    pub artist: Option<String>,
}

impl NowPlaying {
    /// Parse `StreamTitle='Artist - Title';` from an ICY metadata block.
    pub fn parse_icy_meta(block: &str) -> Option<Self> {
        let start = block.find("StreamTitle='")? + "StreamTitle='".len();
        let end = block[start..].find('\'')? + start;
        let raw = block[start..end].trim();
        if raw.is_empty() || raw == "StreamTitle='';" {
            return None;
        }
        // common convention: "Artist - Title"
        if let Some((artist, title)) = raw.split_once(" - ") {
            Some(Self {
                title: title.trim().into(),
                artist: Some(artist.trim().into()),
            })
        } else {
            Some(Self {
                title: raw.into(),
                artist: None,
            })
        }
    }

    /// Display form: "Artist - Title" or just title.
    pub fn display(&self) -> String {
        match &self.artist {
            Some(a) => format!("{a} - {}", self.title),
            None => self.title.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RadioStation {
    pub title: String,
    pub stream_url: String,
    pub genre: String,
    pub country: String,
    pub bitrate_kbps: u16,
    pub favorite: bool,
}

#[derive(Debug, Default)]
pub struct RadioStreamer {
    stations: Vec<RadioStation>,
    current_stream: Option<String>,
    now_playing: Option<NowPlaying>,
    volume: u8, // 0-100
}

impl RadioStreamer {
    pub fn new() -> Self {
        Self { volume: 80, ..Default::default() }
    }

    /// Full registration; use `add_station` for shorthand.
    pub fn add(&mut self, station: RadioStation) {
        self.stations.push(station);
    }

    pub fn add_station(&mut self, title: &str, url: &str, genre: &str) {
        self.stations.push(RadioStation {
            title: title.into(),
            stream_url: url.into(),
            genre: genre.into(),
            country: String::new(),
            bitrate_kbps: 0,
            favorite: false,
        });
    }

    /// Case-insensitive substring search across title/genre/country.
    pub fn search(&self, query: &str) -> Vec<&RadioStation> {
        let q = query.to_lowercase();
        self.stations
            .iter()
            .filter(|s| {
                s.title.to_lowercase().contains(&q)
                    || s.genre.to_lowercase().contains(&q)
                    || s.country.to_lowercase().contains(&q)
            })
            .collect()
    }

    /// Stations in a genre (case-insensitive).
    pub fn by_genre(&self, genre: &str) -> Vec<&RadioStation> {
        let g = genre.to_lowercase();
        self.stations
            .iter()
            .filter(|s| s.genre.to_lowercase() == g)
            .collect()
    }

    pub fn toggle_favorite(&mut self, title: &str) -> bool {
        if let Some(s) = self.stations.iter_mut().find(|s| s.title == title) {
            s.favorite = !s.favorite;
            s.favorite
        } else {
            false
        }
    }

    pub fn favorites(&self) -> Vec<&RadioStation> {
        self.stations.iter().filter(|s| s.favorite).collect()
    }

    pub fn play(&mut self, title: &str) -> bool {
        if let Some(s) = self.stations.iter().find(|s| s.title == title) {
            self.current_stream = Some(s.stream_url.clone());
            self.now_playing = None;
            true
        } else {
            false
        }
    }

    pub fn stop(&mut self) {
        self.current_stream = None;
        self.now_playing = None;
    }

    pub fn is_playing(&self) -> bool {
        self.current_stream.is_some()
    }

    /// Feed an ICY metadata block from the stream.
    pub fn update_metadata(&mut self, block: &str) {
        self.now_playing = NowPlaying::parse_icy_meta(block);
    }

    pub fn now_playing(&self) -> Option<&NowPlaying> {
        self.now_playing.as_ref()
    }

    pub fn set_volume(&mut self, v: u8) {
        self.volume = v.min(100);
    }

    pub fn volume(&self) -> u8 {
        self.volume
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_radio_streamer_aaa() {
        let mut radio = RadioStreamer::new();
        radio.add_station("Synthwave FM", "https://stream.synth.org/live", "Electronic");
        assert!(radio.play("Synthwave FM"));
        assert!(radio.is_playing());
    }

    #[test]
    fn icy_metadata_parsing() {
        let np = NowPlaying::parse_icy_meta(
            "StreamTitle='Daft Punk - Get Lucky';StreamUrl='';",
        )
        .unwrap();
        assert_eq!(np.artist.as_deref(), Some("Daft Punk"));
        assert_eq!(np.title, "Get Lucky");
        assert_eq!(np.display(), "Daft Punk - Get Lucky");
        assert!(NowPlaying::parse_icy_meta("StreamTitle='';").is_none());
    }

    #[test]
    fn search_and_genre_filter() {
        let mut r = RadioStreamer::new();
        r.add_station("Jazz 88", "u1", "Jazz");
        r.add_station("Jazz Hop", "u2", "Jazz");
        r.add_station("Rock On", "u3", "Rock");
        assert_eq!(r.by_genre("jazz").len(), 2);
        assert_eq!(r.search("rock").len(), 1);
    }

    #[test]
    fn favorites_and_volume() {
        let mut r = RadioStreamer::new();
        r.add_station("A", "u", "g");
        assert!(r.toggle_favorite("A"));
        assert_eq!(r.favorites().len(), 1);
        r.set_volume(250);
        assert_eq!(r.volume(), 100);
        r.stop();
        assert!(!r.is_playing());
    }
}
