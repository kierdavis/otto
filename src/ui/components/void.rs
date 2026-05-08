use crate::ui::canvas::Canvas;
use crate::ui::components::Component;
use crate::ui::mouse;
use crate::ui::util::{Rect, Xy};

pub struct Void(pub Xy);

pub fn new(size: Xy) -> Void {
  Void(size)
}

pub fn hoz(width: u16) -> Void {
  new(Xy { x: width, y: 1 })
}

pub fn vert(height: u16) -> Void {
  new(Xy { x: 1, y: height })
}

impl Component for Void {
  fn place(&self, available: Rect, _mouse_map: &mut mouse::ZoneMap) -> Option<Xy> {
    let fixed_size = self.0;
    if available.size.x >= fixed_size.x && available.size.y >= fixed_size.y {
      Some(fixed_size)
    } else {
      None
    }
  }

  fn paint(&self, _canvas: &mut Canvas) {}
}
