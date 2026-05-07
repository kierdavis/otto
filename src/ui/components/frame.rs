use crate::ui::canvas::Canvas;
use crate::ui::components::{
  Component,
  bulb::{self, Bulb},
};
use crate::ui::mouse;
use crate::ui::util::{Rect, WStr, Xy};
use std::cell::Cell;
use std::rc::Rc;

pub struct Frame<C: Component> {
  title: WStr,
  bulb: Option<Rc<Bulb>>,
  child: C,
  rect: Cell<Option<Rect>>,
}

pub struct Builder {
  title: WStr,
  bulb: Option<Rc<Bulb>>,
}

pub fn new(title: WStr) -> Builder {
  Builder { title, bulb: None }
}

impl Builder {
  pub fn with_bulb(self, bulb: Rc<Bulb>) -> Self {
    Self {
      bulb: Some(bulb),
      ..self
    }
  }

  pub fn containing<C: Component>(self, child: C) -> Frame<C> {
    let Self { title, bulb } = self;
    Frame {
      title,
      bulb,
      child,
      rect: Cell::new(None),
    }
  }
}

impl<C: Component> Component for Frame<C> {
  fn place(&self, available: Rect, mouse_map: &mut mouse::ZoneMap) -> Option<Xy> {
    let min_size = Xy {
      x: 4
        + self.title.width
        + (if self.bulb.is_some() {
          bulb::WIDTH + 2
        } else {
          0
        }),
      y: 2,
    };
    if available.size.x < min_size.x || available.size.y < min_size.y {
      self.rect.set(None);
      return None;
    }
    let child_available = Rect {
      top_left: available.top_left + Xy { x: 2, y: 1 },
      size: available.size - Xy { x: 4, y: 2 },
    };
    if let Some(child_used) = self.child.place(child_available, mouse_map) {
      let rect = Rect {
        top_left: available.top_left,
        size: Xy {
          x: min_size.x.max(child_used.x + 4),
          y: min_size.y.max(child_used.y + 2),
        },
      };
      self.rect.set(Some(rect));
      if let Some(bulb) = self.bulb.as_ref() {
        let bulb_rect = Rect {
          top_left: rect.top_right().sub_x(4),
          size: Xy { x: 2, y: 1 },
        };
        let bulb_placed = bulb.place(bulb_rect, mouse_map).is_some();
        debug_assert!(bulb_placed);
      }
      Some(rect.size)
    } else {
      self.rect.set(None);
      None
    }
  }

  fn paint(&self, canvas: &mut Canvas) {
    if let Some(rect) = self.rect.get() {
      canvas.move_to(rect.top_left());
      canvas.write("┌╴");
      canvas.write(self.title.val);
      canvas.write("╶");
      if let Some(bulb) = self.bulb.as_ref() {
        canvas.write_repeat(
          "─",
          (rect.width() - self.title.width - bulb::WIDTH - 6).into(),
        );
        canvas.write("┤");
        bulb.paint(canvas);
        canvas.write("├");
      } else {
        canvas.write_repeat("─", (rect.width() - self.title.width - 4).into());
      }
      canvas.write("┐");

      for dy in 1..rect.height() - 1 {
        canvas.move_to(rect.top_left().add_y(dy));
        canvas.write("│");
        canvas.move_to(rect.top_right().add_y(dy).sub_x(1));
        canvas.write("│");
      }

      canvas.move_to(rect.bottom_left().sub_y(1));
      canvas.write("└");
      canvas.write_repeat("─", (rect.width() - 2).into());
      canvas.write("┘");
    }

    self.child.paint(canvas);
  }
}
