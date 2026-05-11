use crate::datamodel::{self, Change, ClockSrc};
use crate::midi;
use crossbeam::{
  channel::{self, Receiver, Sender},
  select,
};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

pub fn main() {
  let (datamodel_changes_sender, datamodel_changes) = channel::bounded(32);
  DATAMODEL_CHANGES_SENDER
    .set(datamodel_changes_sender)
    .expect("ui::main called more than once");

  let midi_iface = midi::Interface::open();
  use crate::music_theory::{Pitch, Scale};
  let x_scale = Scale([
    Pitch::from_midi(38),
    Pitch::from_midi(40),
    Pitch::from_midi(41),
    Pitch::from_midi(43),
    Pitch::from_midi(45),
    Pitch::from_midi(46),
    Pitch::from_midi(48),
    Pitch::from_midi(50),
    Pitch::from_midi(52),
  ]);
  let y_scale = Scale([
    Pitch::from_midi(62),
    Pitch::from_midi(64),
    Pitch::from_midi(65),
    Pitch::from_midi(67),
    Pitch::from_midi(69),
    Pitch::from_midi(70),
    Pitch::from_midi(72),
    Pitch::from_midi(74),
    Pitch::from_midi(76),
  ]);

  let mut clock = Clock::builtin(140);

  loop {
    select! {
      recv(clock.tick_channel().unwrap_or(&channel::never())) -> _ => {
        let actions = clock.tick();
        if actions.beat {
          let old_state = datamodel::automaton_state();
          let (_, bounces) = old_state.next();
          for &bounce in bounces {
            use crate::automaton::Heading::*;
            let scale = match bounce.wall { PosX | NegX => &x_scale, PosY | NegY => &y_scale };
            midi_iface.emit_note_off(scale.at(bounce.coord_along_wall));
            midi_iface.emit_note_on(scale.at(bounce.coord_along_wall));
          }
          midi_iface.flush();
          Change::AdvanceAutomatonState.apply();
          // Precompute state for next clock.
          datamodel::automaton_state().next();
        }
        if actions.toggle_indicator {
          Change::ToggleClockIndicator.apply();
        }
      }
      recv(datamodel_changes) -> change_result => {
        match change_result.expect("DATAMODEL_CHANGES_SENDER dropped") {
          Change::AdvanceAutomatonState => {},
          Change::SetClockSrc(_) => {
            clock = match datamodel::clock_src() {
              ClockSrc::Builtin => Clock::builtin(140),
              ClockSrc::Midi => Clock::midi(),
            };
          },
          Change::ToggleClockIndicator => {},
        }
      }
    }
  }
}

static DATAMODEL_CHANGES_SENDER: OnceLock<Sender<Change>> = OnceLock::new();

pub fn on_datamodel_change(change: Change) {
  DATAMODEL_CHANGES_SENDER
    .get()
    .expect("realtime::main hasn't been called yet")
    .try_send(change)
    .expect("realtime thread crashed");
}

enum Clock {
  Builtin {
    ticker: Receiver<Instant>,
    beat_divider: ClockDivider,
    toggle_indicator_divider: ClockDivider,
  },
  Midi {},
}

struct ClockTick {
  beat: bool,
  toggle_indicator: bool,
}

impl Clock {
  fn builtin(bpm: u64) -> Self {
    // Tick rate is 2x the bpm since that's how fast we need to toggle the UI clock indicator.
    let tick_interval_us = (1_000_000 * 60 / 2) / bpm;
    Self::Builtin {
      ticker: channel::tick(Duration::from_micros(tick_interval_us)),
      beat_divider: ClockDivider::new(2),
      toggle_indicator_divider: ClockDivider::new(1),
    }
  }
  fn midi() -> Self {
    Self::Midi {}
  }
  fn tick_channel(&self) -> Option<&Receiver<Instant>> {
    match *self {
      Self::Builtin { ref ticker, .. } => Some(ticker),
      Self::Midi { .. } => None,
    }
  }
  fn tick(&mut self) -> ClockTick {
    match *self {
      Self::Builtin {
        ticker: _,
        ref mut beat_divider,
        ref mut toggle_indicator_divider,
      } => ClockTick {
        beat: beat_divider.tick(),
        toggle_indicator: toggle_indicator_divider.tick(),
      },
      Self::Midi {} => ClockTick {
        beat: false,
        toggle_indicator: false,
      },
    }
  }
}

struct ClockDivider {
  divisor: usize,
  pos: usize,
}

impl ClockDivider {
  fn new(divisor: usize) -> Self {
    Self { divisor, pos: 0 }
  }
  fn tick(&mut self) -> bool {
    let result = self.pos == 0;
    self.pos = (self.pos + 1) % self.divisor;
    result
  }
}
