use std::{thread::sleep, time::Duration};
use std::sync::Arc;

pub fn play_sound(sound_data: &'static Vec<u8>) {

  std::thread::spawn(move || {
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&stream_handle).unwrap();

    let cursor = std::io::Cursor::new(sound_data);
    let source = rodio::Decoder::new(cursor).unwrap();

    sink.append(source);
    sink.sleep_until_end();
  });
}