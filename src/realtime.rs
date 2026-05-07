use crate::datamodel::{self, Change, ClockSrc};
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

  let builtin_clock_bpm = 160;
  let mut builtin_clock = match datamodel::clock_src().get() {
    ClockSrc::Builtin => channel::tick(Duration::from_micros(60_000_000 / builtin_clock_bpm)),
    ClockSrc::Midi => channel::never(),
  };
  loop {
    select! {
      recv(builtin_clock) -> _ => {
        Change::AdvanceAutomatonState.apply();
        Change::SetClockIndicatorLit(true).apply();
      }
      recv(datamodel_changes) -> change_result => {
        match change_result.expect("DATAMODEL_CHANGES_SENDER dropped") {
          Change::AdvanceAutomatonState => {},
          Change::SetClockIndicatorLit(_) => {},
          Change::SetClockSrc(_) => {
            builtin_clock = match datamodel::clock_src().get() {
              ClockSrc::Builtin => channel::tick(Duration::from_micros(60_000_000 / builtin_clock_bpm)),
              ClockSrc::Midi => channel::never(),
            };
          },
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
