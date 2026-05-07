use crate::datamodel;
use crate::ui::canvas::Canvas;
use crate::ui::components::Component;
use crate::ui::mouse;
use crate::ui::util::{Rect, Xy};
use std::cell::Cell;

pub struct Automaton {
  origin: Cell<Option<Xy>>,
}

pub fn new() -> Automaton {
  Automaton {
    origin: Cell::new(None),
  }
}

impl Component for Automaton {
  fn place(&self, available: Rect, mouse_map: &mut mouse::ZoneMap) -> Option<Xy> {
    let state = datamodel::automaton_state().get();
    let fixed_size = Xy {
      x: (state.width() * 4 + 1).try_into().unwrap(),
      y: (state.height() * 2 + 1).try_into().unwrap(),
    };
    if available.size.x >= fixed_size.x && available.size.y >= fixed_size.y {
      self.origin.set(Some(available.top_left));
      Some(fixed_size)
    } else {
      self.origin.set(None);
      None
    }
  }

  fn paint(&self, canvas: &mut Canvas) {
    if let Some(origin) = self.origin.get() {
      let state = datamodel::automaton_state().get();
      assert!(state.width() > 0);
      assert!(state.height() > 0);

      for state_y in 0..state.height() {
        canvas.move_to(origin.add_y(u16::try_from(state_y).unwrap() * 2));
        canvas.write(if state_y == 0 {
          "┏━━━"
        } else {
          "┠───"
        });
        for _ in 1..state.width() {
          canvas.write(if state_y == 0 {
            "┯━━━"
          } else {
            "┼───"
          });
        }
        canvas.write(if state_y == 0 { "┓" } else { "┨" });

        canvas.move_to(origin.add_y(u16::try_from(state_y).unwrap() * 2 + 1));
        for state_x in 0..state.width() {
          canvas.write(if state_x == 0 { "┃ " } else { "│ " });
          canvas.write({
            let mut iter = state.gliders_at(state_x, state_y);
            let single_glider_heading = iter.next();
            let multiple_gliders = iter.next().is_some();
            if multiple_gliders {
              "o "
            } else if let Some(heading) = single_glider_heading {
              use crate::automaton::Heading::*;
              match heading {
                PosX => "⮞ ",
                PosY => "⮟ ",
                NegX => "⮜ ",
                NegY => "⮝ ",
              }
            } else {
              "  "
            }
          });
        }
        canvas.write("┃");
      }

      canvas.move_to(origin.add_y(u16::try_from(state.height()).unwrap() * 2));
      canvas.write("┗━━━");
      for _ in 1..state.width() {
        canvas.write("┷━━━");
      }
      canvas.write("┛");
    }
  }
}
