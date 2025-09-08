use rodio::{Decoder, OutputStream, OutputStreamHandle, Source};
use std::io::Cursor;

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
        let bytes = include_bytes!("../../assets/sound/open_settings.mp3");
        let decoder = Decoder::new(Cursor::new(&bytes[..])).unwrap();
        self.stream_handle
            .play_raw(decoder.convert_samples())
            .unwrap();
    }

    pub fn play_text_sound(&mut self) {
        let bytes = include_bytes!("../../assets/sound/TEXT.mp3");
        let decoder = Decoder::new(Cursor::new(&bytes[..])).unwrap();
        let sink = rodio::Sink::try_new(&self.stream_handle).unwrap();
        sink.set_volume(4.0);
        sink.append(decoder.convert_samples::<f32>());
        self._text_sound_sink = Some(sink);
    }

    pub fn play_enemy_encounter_sound(&self) {
        let bytes = include_bytes!("../../assets/sound/enemy_encounter.mp3");
        let decoder = Decoder::new(Cursor::new(&bytes[..])).unwrap();
        self.stream_handle
            .play_raw(decoder.convert_samples())
            .unwrap();
    }
}

