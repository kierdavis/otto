use crate::datamodel::{self, Change};
use crate::midi;
use crossbeam::channel::{self, Receiver, Sender, TryRecvError};
use std::sync::OnceLock;
use std::thread::sleep;
use std::time::Instant;

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
    let beat_rate = ClockRate::per_minute(140);
    let note_rate = beat_rate * 2;
    let toggle_indicator_rate = beat_rate * 2;

    let tick_rate = ClockRate::divisible_into(note_rate, toggle_indicator_rate);
    let tick_rate = tick_rate.as_fast_as(min_poll_idle_tasks_rate());

    let mut note_divider = note_rate.new_divider_relative_to(tick_rate);
    let mut toggle_indicator_divider = toggle_indicator_rate.new_divider_relative_to(tick_rate);

    let tick_interval = tick_rate.interval();
    let mut next_tick = Instant::now() + tick_interval;

    loop {
      let now = Instant::now();
      if next_tick > now {
        match self.service_idle_tasks() {
          NothingToService => sleep(next_tick - now),
          TimeElapsed => continue, // Next time might already be overdue.
          RestartMainLoop => return,
        }
      }

      // Tick occurs now.
      debug_assert!(next_tick <= Instant::now());
      if note_divider.tick() {
        self.advance_automaton();
      }
      if toggle_indicator_divider.tick() {
        Change::ToggleClockIndicator.apply();
      }
      next_tick += tick_interval;
    }
  }

  fn main_loop_midi_clock(&mut self) {
    let ticks_per_beat = 24; // standard
    let notes_per_beat = 2;
    let ticks_per_note = ticks_per_beat / notes_per_beat;
    let mut note_divider = ClockDivider::new(ticks_per_note);
    let mut toggle_indicator_divider = ClockDivider::new(ticks_per_beat / 2);

    let recv_timeout = min_poll_idle_tasks_rate().interval();

    loop {
      match self.midi_iface.recv_timeout(recv_timeout) {
        Some(midi::Event::Clock) => {
          // Tick occurs now.
          if note_divider.tick() {
            self.advance_automaton();
          }
          if toggle_indicator_divider.tick() {
            Change::ToggleClockIndicator.apply();
          }
        }
        None => {}
      }
      match self.service_idle_tasks() {
        NothingToService | TimeElapsed => {}
        RestartMainLoop => return,
      }
    }
  }

  fn service_idle_tasks(&mut self) -> ServiceIdleTasksResult {
    if self.pending_precompute_next_automaton_state {
      datamodel::automaton_state().next();
      self.pending_precompute_next_automaton_state = false;
      return TimeElapsed;
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

  fn advance_automaton(&self) {
    use crate::music_theory::{Pitch, Scale};
    let scale = Scale([
      Pitch::from_midi(38),
      Pitch::from_midi(41),
      Pitch::from_midi(43),
      Pitch::from_midi(45),
      Pitch::from_midi(48),
      Pitch::from_midi(50),
      Pitch::from_midi(53),
      Pitch::from_midi(55),
      Pitch::from_midi(57),
    ]);

    let old_state = datamodel::automaton_state();
    let (_, bounces) = old_state.next();
    for &bounce in bounces {
      let pitch = scale.at(bounce.coord_along_wall);
      self.midi_iface.send_note_off(pitch);
      self.midi_iface.send_note_on(pitch);
    }
    self.midi_iface.flush();
    Change::AdvanceAutomatonState.apply();
  }
}

enum ServiceIdleTasksResult {
  NothingToService,
  TimeElapsed,
  RestartMainLoop,
}

use ServiceIdleTasksResult::*;

static DATAMODEL_CHANGES_SENDER: OnceLock<Sender<Change>> = OnceLock::new();

pub fn on_datamodel_change(change: Change) {
  DATAMODEL_CHANGES_SENDER
    .get()
    .expect("realtime::main hasn't been called yet")
    .try_send(change)
    .expect("realtime thread crashed");
}

fn min_poll_idle_tasks_rate() -> ClockRate {
  ClockRate::per_second(10)
}

struct ClockDivider {
  divisor: u64,
  pos: u64,
}

impl ClockDivider {
  fn new(divisor: u64) -> Self {
    Self { divisor, pos: 0 }
  }
  fn tick(&mut self) -> bool {
    let result = self.pos == 0;
    self.pos = (self.pos + 1) % self.divisor;
    result
  }
}

use clock_rate::ClockRate;
mod clock_rate {
  use super::ClockDivider;
  use num::{
    integer::{gcd, lcm},
    rational::Ratio,
  };
  use std::ops::Mul;
  use std::time::Duration;

  #[derive(Clone, Copy, Debug)]
  pub struct ClockRate(Ratio<u64>);

  impl ClockRate {
    pub fn per_second(x: u64) -> Self {
      Self(Ratio::new(x, 1))
    }

    pub fn per_minute(x: u64) -> Self {
      Self(Ratio::new(x, 60))
    }

    pub fn divisible_into(r1: Self, r2: Self) -> Self {
      Self(Ratio::new(
        lcm(*r1.0.numer(), *r2.0.numer()),
        gcd(*r1.0.denom(), *r2.0.denom()),
      ))
    }

    pub fn as_fast_as(self, minimum: Self) -> Self {
      Self(self.0 * rational_to_integer_ceil(minimum.0 / self.0))
    }

    pub fn interval(self) -> Duration {
      Duration::from_micros((1_000_000 * *self.0.denom()) / *self.0.numer())
    }

    pub fn new_divider_relative_to(self, base: Self) -> ClockDivider {
      let divisor = base.0 / self.0;
      assert_eq!(*divisor.denom(), 1);
      ClockDivider::new(*divisor.numer())
    }
  }

  impl Mul<u64> for ClockRate {
    type Output = Self;
    fn mul(self, multiplier: u64) -> Self {
      Self(self.0 * multiplier)
    }
  }

  fn rational_to_integer_ceil(r: Ratio<u64>) -> u64 {
    (*r.numer() + *r.denom() - 1) / *r.denom()
  }
}
