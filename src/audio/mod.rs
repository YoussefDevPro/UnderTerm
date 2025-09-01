use rodio::{Decoder, OutputStream, OutputStreamHandle, Source};
use std::fs::File;
use std::io::BufReader;

pub struct Audio {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
}

impl Audio {
    pub fn new() -> Result<Self, rodio::StreamError> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        Ok(Audio {
            _stream: stream,
            stream_handle,
        })
    }

    pub fn play_open_settings_sound(&self) {
        let file = File::open("assets/sound/open_settings.mp3").unwrap();
        let decoder = Decoder::new(BufReader::new(file)).unwrap();
        self.stream_handle
            .play_raw(decoder.convert_samples())
            .unwrap();
    }

    pub fn play_text_sound(&self) {
        let file = File::open("assets/sound/A.mp3").unwrap();
        let decoder = Decoder::new(BufReader::new(file)).unwrap();
        self.stream_handle
            .play_raw(decoder.convert_samples())
            .unwrap();
    }

    pub fn play_enemy_encounter_sound(&self) {
        let file = File::open("assets/sound/enemy_encounter.mp3").unwrap();
        let decoder = Decoder::new(BufReader::new(file)).unwrap();
        self.stream_handle
            .play_raw(decoder.convert_samples())
            .unwrap();
    }
}
