use rodio::{Decoder, OutputStream, OutputStreamHandle, Source};
use std::fs::File;
use std::io::BufReader;

pub struct Audio {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    _text_sound_sink: Option<rodio::Sink>,
}

impl Audio {
    pub fn new() -> Result<Self, rodio::StreamError> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        Ok(Audio {
            _stream: stream,
            stream_handle,
            _text_sound_sink: None,
        })
    }

    pub fn play_open_settings_sound(&self) {
        let file = File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/sound/open_settings.mp3")).unwrap();
        let decoder = Decoder::new(BufReader::new(file)).unwrap();
        self.stream_handle
            .play_raw(decoder.convert_samples())
            .unwrap();
    }

    pub fn play_text_sound(&mut self) {
        let file = File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/sound/A.mp3")).unwrap();
        let decoder = Decoder::new(BufReader::new(file)).unwrap();
        let sink = rodio::Sink::try_new(&self.stream_handle).unwrap();
        sink.append(decoder.convert_samples::<f32>());
        self._text_sound_sink = Some(sink);
    }

    pub fn play_enemy_encounter_sound(&self) {
        let file = File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/sound/enemy_encounter.mp3")).unwrap();
        let decoder = Decoder::new(BufReader::new(file)).unwrap();
        self.stream_handle
            .play_raw(decoder.convert_samples())
            .unwrap();
    }
}
