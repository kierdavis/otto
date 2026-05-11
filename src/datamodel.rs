use crate::automaton;
use enum_ext::enum_extend;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed};

#[enum_extend]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ClockSrc {
  Builtin,
  Midi,
}

static AUTOMATON_STATE: Mutex<Option<automaton::State>> = Mutex::new(None);
static CLOCK_INDICATOR_LIT: AtomicBool = AtomicBool::new(false);
static CLOCK_SRC: AtomicUsize = AtomicUsize::new(0);

fn default_automaton_state() -> automaton::State {
  automaton::State::new(
    9,
    9,
    [
      ((5, 0), automaton::Heading::NegY),
      ((1, 1), automaton::Heading::PosX),
      ((2, 2), automaton::Heading::PosX),
      ((0, 3), automaton::Heading::PosY),
      ((4, 3), automaton::Heading::NegX),
      ((2, 5), automaton::Heading::NegX),
      ((3, 5), automaton::Heading::NegY),
      ((4, 5), automaton::Heading::PosY),
      ((2, 6), automaton::Heading::PosY),
      ((7, 6), automaton::Heading::PosX),
      ((5, 7), automaton::Heading::NegX),
      ((7, 7), automaton::Heading::PosY),
      ((5, 8), automaton::Heading::PosX),
    ],
  )
}

#[derive(Clone, Debug)]
#[must_use]
pub enum Change {
  AdvanceAutomatonState,
  SetClockSrc(ClockSrc),
  ToggleClockIndicator,
}

impl Change {
  pub fn apply(self) {
    match &self {
      &Self::AdvanceAutomatonState => {
        let mut option = AUTOMATON_STATE.lock().expect("poisoned");
        let old = option.get_or_insert_with(default_automaton_state);
        let (new, _) = old.next();
        *option = Some(new);
      }
      &Self::SetClockSrc(new) => {
        CLOCK_SRC.store(new.ordinal(), Relaxed);
      }
      &Self::ToggleClockIndicator => {
        CLOCK_INDICATOR_LIT.fetch_not(Relaxed);
      }
    }
    crate::realtime::on_datamodel_change(self.clone());
    crate::ui::on_datamodel_change(self);
  }
}

pub fn automaton_state() -> automaton::State {
  AUTOMATON_STATE
    .lock()
    .expect("poisoned")
    .get_or_insert_with(default_automaton_state)
    .clone()
}

pub fn clock_indicator_lit() -> bool {
  CLOCK_INDICATOR_LIT.load(Relaxed)
}

pub fn clock_src() -> ClockSrc {
  ClockSrc::from_ordinal(CLOCK_SRC.load(Relaxed)).unwrap()
}
