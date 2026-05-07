use crate::ui::canvas::Canvas;
use crate::ui::components::Component;
use crate::ui::mouse;
use crate::ui::util::{Rect, Xy};
use std::marker::PhantomData;

pub trait Dir {
  fn available_for_remaining_children(available: Rect, used_so_far: Xy) -> Rect;
  fn update_used_so_far(used_so_far: &mut Xy, child_used: Xy);
}

pub struct Hoz;
impl Dir for Hoz {
  fn available_for_remaining_children(available: Rect, used_so_far: Xy) -> Rect {
    available.hsplit_at(used_so_far.x).1
  }
  fn update_used_so_far(used_so_far: &mut Xy, child_used: Xy) {
    *used_so_far = Xy {
      x: used_so_far.x + child_used.x,
      y: used_so_far.y.max(child_used.y),
    };
  }
}

pub struct Vert;
impl Dir for Vert {
  fn available_for_remaining_children(available: Rect, used_so_far: Xy) -> Rect {
    available.vsplit_at(used_so_far.y).1
  }
  fn update_used_so_far(used_so_far: &mut Xy, child_used: Xy) {
    *used_so_far = Xy {
      x: used_so_far.x.max(child_used.x),
      y: used_so_far.y + child_used.y,
    };
  }
}

pub struct Stack<D: Dir, C: Component> {
  children: Vec<C>,
  dir: PhantomData<D>,
}

pub fn new_hoz<C: Component>(children: Vec<C>) -> Stack<Hoz, C> {
  new(children)
}

pub fn new_vert<C: Component>(children: Vec<C>) -> Stack<Vert, C> {
  new(children)
}

fn new<D: Dir, C: Component>(children: Vec<C>) -> Stack<D, C> {
  Stack {
    children,
    dir: PhantomData,
  }
}

impl<D: Dir, C: Component> Component for Stack<D, C> {
  fn place(&self, available: Rect, mouse_map: &mut mouse::ZoneMap) -> Option<Xy> {
    let mut used_so_far = Xy::ZERO;
    for child in &self.children {
      if let Some(child_used) = child.place(
        D::available_for_remaining_children(available, used_so_far),
        mouse_map,
      ) {
        D::update_used_so_far(&mut used_so_far, child_used);
      } else {
        for child in &self.children {
          child.place(Rect::ZERO, mouse_map);
        }
        return None;
      }
    }
    Some(used_so_far)
  }

  fn paint(&self, canvas: &mut Canvas) {
    for child in &self.children {
      child.paint(canvas);
    }
  }
}
