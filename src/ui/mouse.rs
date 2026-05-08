use super::util::{Rect, Xy};
use crate::util::Matrix;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Zone {
  ClockSrcBuiltin,
  ClockSrcMidi,
}

#[derive(Debug)]
pub struct ZoneMap(Matrix<Option<Zone>>);

impl ZoneMap {
  pub fn new(screen_size: Xy) -> Self {
    Self(Matrix::new(
      screen_size.x.into(),
      screen_size.y.into(),
      |_, _| None,
    ))
  }

  pub fn clear(&mut self) {
    self.0.fill(|_, _| None);
  }

  pub fn set(&mut self, rect: Rect, zone: Zone) {
    for y in rect.top()..rect.bottom() {
      for x in rect.left()..rect.right() {
        self.0[(x.into(), y.into())] = Some(zone);
      }
    }
  }

  pub fn get(&self, coords: Xy) -> Option<Zone> {
    self.0[(coords.x.into(), coords.y.into())]
  }
}
