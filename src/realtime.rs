use crate::datamodel::{self, Change, ClockSrc};
use crate::midi;
use crossbeam::{
  channel::{self, Sender},
  select,
};
use std::sync::OnceLock;
use std::time::Duration;

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

  let builtin_clock_bpm = 140 * 2;
  let mut builtin_clock = match datamodel::clock_src() {
    ClockSrc::Builtin => channel::tick(Duration::from_micros(60_000_000 / builtin_clock_bpm)),
    ClockSrc::Midi => channel::never(),
  };

  loop {
    select! {
      recv(builtin_clock) -> _ => {
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
        Change::ToggleClockIndicator.apply();
        // Precompute state for next clock.
        datamodel::automaton_state().next();
      }
      recv(datamodel_changes) -> change_result => {
        match change_result.expect("DATAMODEL_CHANGES_SENDER dropped") {
          Change::AdvanceAutomatonState => {},
          Change::SetClockSrc(_) => {
            builtin_clock = match datamodel::clock_src() {
              ClockSrc::Builtin => channel::tick(Duration::from_micros(60_000_000 / builtin_clock_bpm)),
              ClockSrc::Midi => channel::never(),
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
