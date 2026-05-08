mod automaton;
mod bulb;
mod frame;
mod selector;
mod stack;
mod text;
mod void;

use super::canvas::Canvas;
use super::mouse;
use super::util::{Rect, WStr, Xy};
use crate::datamodel::{self, ClockSrc};
use std::rc::Rc;

pub trait Component {
  fn place(&self, available: Rect, mouse_map: &mut mouse::ZoneMap) -> Option<Xy>;
  fn paint(&self, canvas: &mut Canvas);
}

impl<C: Component + ?Sized> Component for Box<C> {
  fn place(&self, available: Rect, mouse_map: &mut mouse::ZoneMap) -> Option<Xy> {
    (**self).place(available, mouse_map)
  }
  fn paint(&self, canvas: &mut Canvas) {
    (**self).paint(canvas)
  }
}

impl<C: Component + ?Sized> Component for Rc<C> {
  fn place(&self, available: Rect, mouse_map: &mut mouse::ZoneMap) -> Option<Xy> {
    (**self).place(available, mouse_map)
  }
  fn paint(&self, canvas: &mut Canvas) {
    (**self).paint(canvas)
  }
}

impl selector::Enum for ClockSrc {
  fn all() -> &'static [Self] {
    &[Self::Builtin, Self::Midi]
  }
  fn label(&self) -> WStr {
    match *self {
      Self::Builtin => WStr {
        val: "builtin",
        width: 7,
      },
      Self::Midi => WStr {
        val: "midi",
        width: 4,
      },
    }
  }
  fn mouse_zone(&self) -> mouse::Zone {
    match *self {
      Self::Builtin => mouse::Zone::ClockSrcBuiltin,
      Self::Midi => mouse::Zone::ClockSrcMidi,
    }
  }
}

macro_rules! child_vec {
  [$($child:expr),* $(,)?] => {
    vec![$(Box::new($child) as Box<dyn Component>),*]
  };
}

pub struct Components {
  pub root: Box<dyn Component>,
  pub automaton: Rc<automaton::Automaton>,
  pub clock_indicator: Rc<bulb::Bulb>,
  pub clock_src_selector: Rc<selector::Selector<ClockSrc>>,
}

impl Components {
  pub fn build() -> Self {
    let automaton = Rc::new(automaton::new());
    let clock_indicator = Rc::new(bulb::new(&datamodel::clock_indicator_lit));
    let clock_src_selector = Rc::new(selector::new(&datamodel::clock_src));
    let root = Box::new(stack::new_vert(child_vec![
      void::vert(1),
      stack::new_hoz(child_vec![
        void::hoz(3),
        automaton.clone(),
        void::hoz(4),
        stack::new_vert(child_vec![
          frame::new(WStr {
            val: "CLOCK",
            width: 5
          })
          .with_bulb(clock_indicator.clone())
          .containing(stack::new_vert(child_vec![stack::new_hoz(child_vec![
            text::new(WStr {
              val: "src: ",
              width: 5
            }),
            clock_src_selector.clone(),
          ]),]),),
        ]),
      ]),
    ]));
    Self {
      root,
      automaton,
      clock_indicator,
      clock_src_selector,
    }
  }
}
