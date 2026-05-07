use super::util::Xy;
use crossterm::{
  cursor::MoveTo,
  queue,
  style::{ContentStyle, ResetColor, SetStyle},
  terminal::{Clear, ClearType},
};
use std::io::{StdoutLock, Write};

pub struct Canvas<'a> {
  sink: StdoutLock<'a>,
}

impl<'a> Canvas<'a> {
  pub fn new(sink: StdoutLock<'a>) -> Self {
    Self { sink }
  }

  pub fn into_inner(self) -> StdoutLock<'a> {
    self.sink
  }

  pub fn flush(&mut self) {
    self.sink.flush().expect("failed to write to stdout")
  }

  pub fn write(&mut self, string: &'static str) {
    self
      .sink
      .write_all(string.as_bytes())
      .expect("failed to write to stdout")
  }

  pub fn write_repeat(&mut self, string: &'static str, count: usize) {
    for _ in 0..count {
      self.write(string);
    }
  }

  pub fn clear(&mut self) {
    queue!(self.sink, Clear(ClearType::All)).expect("failed to write to stdout")
  }

  pub fn move_to(&mut self, coords: Xy) {
    queue!(self.sink, MoveTo(coords.x, coords.y)).expect("failed to write to stdout")
  }

  pub fn set_style(&mut self, style: ContentStyle) {
    queue!(self.sink, SetStyle(style)).expect("failed to write to stdout")
  }

  pub fn reset_style(&mut self) {
    queue!(self.sink, ResetColor).expect("failed to write to stdout")
  }
}
