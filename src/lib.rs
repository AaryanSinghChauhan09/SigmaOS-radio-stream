#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RadioStation {
    pub title: String,
    pub stream_url: String,
    pub genre: String,
}

#[derive(Debug, Default)]
pub struct RadioStreamer {
    stations: Vec<RadioStation>,
    current_stream: Option<String>,
}

impl RadioStreamer {
    pub fn new() -> Self { Self { stations: Vec::new(), current_stream: None } }
    pub fn add_station(&mut self, title: &str, url: &str, genre: &str) {
        self.stations.push(RadioStation {
            title: title.into(),
            stream_url: url.into(),
            genre: genre.into(),
        });
    }
    pub fn play(&mut self, title: &str) -> bool {
        if let Some(s) = self.stations.iter().find(|s| s.title == title) {
            self.current_stream = Some(s.stream_url.clone());
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_radio_streamer_aaa() {
        // Arrange
        let mut radio = RadioStreamer::new();
        radio.add_station("Synthwave FM", "https://stream.synth.org/live", "Electronic");
        // Act
        let res = radio.play("Synthwave FM");
        // Assert
        assert!(res);
    }
}
