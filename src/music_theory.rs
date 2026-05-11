// Internal representation is the same as MIDI and ALSA sequencer.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Pitch(u8);

impl Pitch {
  pub const fn from_midi(val: u8) -> Self {
    Self(val)
  }
  pub const fn to_midi(self) -> u8 {
    self.0
  }
}

pub struct Scale(pub [Pitch; 9]);

impl Scale {
  pub fn at(&self, offset: usize) -> Pitch {
    self.0[offset]
  }
}
