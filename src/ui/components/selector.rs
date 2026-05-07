use crate::datamodel::View;
use crate::ui::canvas::Canvas;
use crate::ui::components::Component;
use crate::ui::mouse;
use crate::ui::styles;
use crate::ui::util::{Rect, WStr, Xy};
use std::cell::Cell;

pub const SEP: WStr = WStr {
  val: " · ",
  width: 3,
};

pub trait Enum: std::fmt::Debug + Eq + Sized + 'static {
  fn all() -> &'static [Self];
  fn label(&self) -> WStr;
  fn mouse_zone(&self) -> mouse::Zone;
}

pub struct Selector<T: Enum> {
  selected: View<T>,
  width: u16,
  origin: Cell<Option<Xy>>,
}

pub fn new<T: Enum>(selected: View<T>) -> Selector<T> {
  Selector {
    selected,
    width: T::all()
      .iter()
      .map(|variant| variant.label().width + SEP.width)
      .sum::<u16>()
      - SEP.width,
    origin: Cell::new(None),
  }
}

impl<T: Enum> Component for Selector<T> {
  fn place(&self, available: Rect, mouse_map: &mut mouse::ZoneMap) -> Option<Xy> {
    let fixed_size = Xy {
      x: self.width,
      y: 1,
    };
    if available.size.x >= fixed_size.x && available.size.y >= fixed_size.y {
      self.origin.set(Some(available.top_left));
      {
        let mut x = 0;
        for (i, variant) in T::all().iter().enumerate() {
          if i != 0 {
            x += SEP.width;
          }
          mouse_map.set(
            Rect {
              top_left: available.top_left.add_x(x),
              size: Xy {
                x: variant.label().width,
                y: 1,
              },
            },
            variant.mouse_zone(),
          );
          x += variant.label().width;
        }
      }
      Some(fixed_size)
    } else {
      self.origin.set(None);
      None
    }
  }

  fn paint(&self, canvas: &mut Canvas) {
    if let Some(origin) = self.origin.get() {
      canvas.move_to(origin);
      let selected = self.selected.get();
      for (i, variant) in T::all().iter().enumerate() {
        if i != 0 {
          canvas.write(SEP.val);
        }
        if *variant == selected {
          canvas.set_style(styles::SELECTED);
        }
        canvas.write(variant.label().val);
        canvas.reset_style();
      }
    }
  }
}
