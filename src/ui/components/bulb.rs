use crate::ui::canvas::Canvas;
use crate::ui::components::Component;
use crate::ui::mouse;
use crate::ui::util::{Rect, Xy};
use std::cell::Cell;

pub const WIDTH: u16 = 2;

pub struct Bulb {
  is_lit: &'static dyn Fn() -> bool,
  origin: Cell<Option<Xy>>,
}

pub fn new(is_lit: &'static dyn Fn() -> bool) -> Bulb {
  Bulb {
    is_lit,
    origin: Cell::new(None),
  }
}

impl Component for Bulb {
  fn place(&self, available: Rect, _mouse_map: &mut mouse::ZoneMap) -> Option<Xy> {
    let fixed_size = Xy { x: WIDTH, y: 1 };
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
      canvas.move_to(origin);
      canvas.write(if (self.is_lit)() { "🔴" } else { "⚫" });
    }
  }
}
