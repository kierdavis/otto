use crate::datamodel::{self, Change};
use crate::midi;
use crossbeam::channel::{self, Receiver, Sender, TryRecvError};
use std::sync::OnceLock;
use std::thread::sleep;
use std::time::{Duration, Instant};

pub fn main() {
  let (datamodel_changes_sender, datamodel_changes) = channel::bounded(32);
  DATAMODEL_CHANGES_SENDER
    .set(datamodel_changes_sender)
    .expect("realtime::main called more than once");

  (Realtime {
    midi_iface: midi::Interface::open(),
    datamodel_changes,
    pending_precompute_next_automaton_state: true,
  })
  .main()
}

const MAX_BASE_TICK_INTERVAL: Duration = Duration::from_millis(50);

struct Realtime {
  midi_iface: midi::Interface,
  datamodel_changes: Receiver<Change>,
  pending_precompute_next_automaton_state: bool,
}

impl Realtime {
  fn main(mut self) {
    loop {
      match datamodel::clock_src() {
        datamodel::ClockSrc::Builtin => self.main_loop_builtin_clock(),
        datamodel::ClockSrc::Midi => self.main_loop_midi_clock(),
      }
    }
  }

  fn main_loop_builtin_clock(&mut self) {
    let bpm = 140;
    let beat_interval = Duration::from_micros(60_000_000 / bpm);

    // Ticks need to be as frequent as double the BPM, since that's how often we toggle the clock indicator.
    // Ticks also need to be sufficiently frequent that we service idle interactions often enough.
    let mut ticks_per_beat = 2;
    while beat_interval / ticks_per_beat > MAX_BASE_TICK_INTERVAL {
      ticks_per_beat *= 2;
    }

    let tick_interval = beat_interval / ticks_per_beat;
    let mut beat_divider = ClockDivider::new(ticks_per_beat);
    let mut toggle_indicator_divider = ClockDivider::new(ticks_per_beat / 2);

    let mut next_tick = Instant::now() + tick_interval;

    loop {
      let now = Instant::now();
      if next_tick > now {
        match self.service_one_idle_task() {
          NothingToService => {
            // Just sleep until the next tick.
            sleep(next_tick - now);
          }
          ConsumedTime => {
            // Check the wallclock again.
            continue;
          }
          RestartMainLoop => return,
        }
      }

      // Tick occurs now.
      debug_assert!(next_tick <= Instant::now());
      if beat_divider.tick() {
        self.beat();
      }
      if toggle_indicator_divider.tick() {
        Change::ToggleClockIndicator.apply();
      }
      next_tick += tick_interval;
    }
  }

  fn main_loop_midi_clock(&mut self) {
    let ticks_per_beat = 24; // standard
    let mut beat_divider = ClockDivider::new(ticks_per_beat);
    let mut toggle_indicator_divider = ClockDivider::new(ticks_per_beat / 2);

    loop {
      match self.midi_iface.try_recv() {
        Some(midi::Event::Clock) => {
          // Tick occurs now.
          if beat_divider.tick() {
            self.beat();
          }
          if toggle_indicator_divider.tick() {
            Change::ToggleClockIndicator.apply();
          }
        }
        None => match self.service_one_idle_task() {
          NothingToService => {
            sleep(MAX_BASE_TICK_INTERVAL);
          }
          ConsumedTime => {}
          RestartMainLoop => return,
        },
      }
    }
  }

  fn service_one_idle_task(&mut self) -> ServiceIdleTaskResult {
    if self.pending_precompute_next_automaton_state {
      datamodel::automaton_state().next();
      self.pending_precompute_next_automaton_state = false;
      return ConsumedTime;
    }
    loop {
      match self.datamodel_changes.try_recv() {
        Ok(Change::AdvanceAutomatonState) => {}
        Ok(Change::SetClockSrc(_)) => return RestartMainLoop,
        Ok(Change::ToggleClockIndicator) => {}
        Err(TryRecvError::Empty) => break,
        Err(TryRecvError::Disconnected) => unreachable!("DATAMODEL_CHANGES_SENDER dropped"),
      }
    }
    NothingToService
  }

  fn beat(&self) {
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

    let old_state = datamodel::automaton_state();
    let (_, bounces) = old_state.next();
    for &bounce in bounces {
      use crate::automaton::Heading::*;
      let scale = match bounce.wall {
        PosX | NegX => &x_scale,
        PosY | NegY => &y_scale,
      };
      let pitch = scale.at(bounce.coord_along_wall);
      self.midi_iface.send_note_off(pitch);
      self.midi_iface.send_note_on(pitch);
    }
    self.midi_iface.flush();
    Change::AdvanceAutomatonState.apply();
  }
}

enum ServiceIdleTaskResult {
  NothingToService,
  ConsumedTime,
  RestartMainLoop,
}

use ServiceIdleTaskResult::*;

static DATAMODEL_CHANGES_SENDER: OnceLock<Sender<Change>> = OnceLock::new();

pub fn on_datamodel_change(change: Change) {
  DATAMODEL_CHANGES_SENDER
    .get()
    .expect("realtime::main hasn't been called yet")
    .try_send(change)
    .expect("realtime thread crashed");
}

struct ClockDivider {
  divisor: u32,
  pos: u32,
}

impl ClockDivider {
  fn new(divisor: u32) -> Self {
    Self { divisor, pos: 0 }
  }
  fn tick(&mut self) -> bool {
    let result = self.pos == 0;
    self.pos = (self.pos + 1) % self.divisor;
    result
  }
}
