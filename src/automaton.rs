use crate::util::Matrix;
use enum_map::{Enum, EnumMap, enum_map};
use std::sync::{Arc, OnceLock};

#[derive(Clone, Copy, Debug, Enum, Eq, Hash, PartialEq)]
pub enum Heading {
  PosX,
  PosY,
  NegX,
  NegY,
}

use Heading::*;

impl Heading {
  pub const fn rotated_ccw(self) -> Self {
    match self {
      PosX => NegY,
      PosY => PosX,
      NegX => PosY,
      NegY => NegX,
    }
  }
  pub const fn flipped(self) -> Self {
    match self {
      PosX => NegX,
      PosY => NegY,
      NegX => PosX,
      NegY => PosY,
    }
  }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Bounce {
  pub wall: Heading,
  pub coord_along_wall: usize,
}

#[derive(Clone)]
pub struct State(Arc<StateImpl>);

struct StateImpl {
  matrix: Matrix<EnumMap<Heading, bool>>,
  next: OnceLock<(State, Vec<Bounce>)>,
}

impl State {
  pub fn new(
    width: usize,
    height: usize,
    gliders: impl IntoIterator<Item = ((usize, usize), Heading)>,
  ) -> Self {
    let mut matrix = Matrix::new(width, height, |_, _| enum_map! { _ => false });
    for (xy, heading) in gliders {
      matrix[xy][heading] = true;
    }
    Self(Arc::new(StateImpl {
      matrix,
      next: OnceLock::new(),
    }))
  }

  /*
  pub fn empty(width: usize, height: usize) -> Self {
    Self::new(width, height, std::iter::empty())
  }
  */

  pub fn width(&self) -> usize {
    self.0.matrix.width()
  }

  pub fn height(&self) -> usize {
    self.0.matrix.height()
  }

  #[cfg(test)]
  pub fn gliders(&self) -> impl Iterator<Item = ((usize, usize), Heading)> {
    self.0.matrix.enumerate().flat_map(|(xy, &cell)| {
      cell
        .into_iter()
        .filter(|&(_, has)| has)
        .map(move |(hdg, _)| (xy, hdg))
    })
  }

  pub fn gliders_at(&self, x: usize, y: usize) -> impl Iterator<Item = Heading> {
    self.0.matrix[(x, y)]
      .into_iter()
      .filter(|&(_, has)| has)
      .map(|(hdg, _)| hdg)
  }

  pub fn next(&self) -> (State, &[Bounce]) {
    let &(ref next_state, ref bounces) = self.0.next.get_or_init(|| {
      let old_matrix = &self.0.matrix;
      let width = old_matrix.width();
      let height = old_matrix.height();
      let mut new_matrix = Matrix::new(width, height, |_, _| enum_map! { _ => false });
      let mut bounces = Vec::new();
      for ((x, y), &cell) in old_matrix.enumerate() {
        // Check for glider collisions in this cell and rotate them accordingly.
        let cell: EnumMap<Heading, bool> = match cell.values().filter(|&&b| b).count() {
          // 0 or 1 glider(s) => no change.
          0 | 1 => cell,
          // 2 gliders => they both rotate 90° clockwise.
          2 => enum_map! { hdg => cell[hdg.rotated_ccw()] },
          // 3 or more gliders => they all rotate 180°.
          _ => enum_map! { hdg => cell[hdg.flipped()] },
        };
        // Move each glider in this cell forwards, adding its resulting location into new_matrix.
        for (heading, has_glider) in cell.into_iter() {
          if !has_glider {
            continue;
          }
          let new_heading = match heading {
            PosX if x == width - 1 => {
              bounces.push(Bounce {
                wall: PosX,
                coord_along_wall: y,
              });
              NegX
            }
            PosY if y == height - 1 => {
              bounces.push(Bounce {
                wall: PosY,
                coord_along_wall: x,
              });
              NegY
            }
            NegX if x == 0 => {
              bounces.push(Bounce {
                wall: NegX,
                coord_along_wall: y,
              });
              PosX
            }
            NegY if y == 0 => {
              bounces.push(Bounce {
                wall: NegY,
                coord_along_wall: x,
              });
              PosY
            }
            heading => heading,
          };
          let new_xy = match new_heading {
            PosX => (x + 1, y),
            PosY => (x, y + 1),
            NegX => (x - 1, y),
            NegY => (x, y - 1),
          };
          new_matrix[new_xy][new_heading] = true;
        }
      }
      let new_state = State(Arc::new(StateImpl {
        matrix: new_matrix,
        next: OnceLock::new(),
      }));
      (new_state, bounces)
    });
    (next_state.clone(), bounces)
  }
}

#[cfg(test)]
mod tests {
  pub use super::{
    Bounce,
    Heading::{NegX, NegY, PosX, PosY},
    State,
  };

  // Pattern designed by this deleted Reddit user:
  // https://www.reddit.com/r/otomata/comments/lrd7n4/comment/gon0dfu/
  #[test]
  fn test_pattern() {
    let st = State::new(
      9,
      9,
      [
        ((5, 0), NegY),
        ((1, 1), PosX),
        ((2, 2), PosX),
        ((0, 3), PosY),
        ((4, 3), NegX),
        ((2, 5), NegX),
        ((3, 5), NegY),
        ((4, 5), PosY),
        ((2, 6), PosY),
        ((7, 6), PosX),
        ((5, 7), NegX),
        ((7, 7), PosY),
        ((5, 8), PosX),
      ],
    );
    let (st, b) = st.next();
    assert_eq!(
      b,
      &[Bounce {
        wall: NegY,
        coord_along_wall: 5
      }]
    );
    let (st, b) = st.next();
    assert_eq!(
      b,
      &[
        Bounce {
          wall: PosX,
          coord_along_wall: 6
        },
        Bounce {
          wall: PosY,
          coord_along_wall: 7
        }
      ]
    );
    let (st, b) = st.next();
    assert_eq!(
      b,
      &[
        Bounce {
          wall: NegX,
          coord_along_wall: 5
        },
        Bounce {
          wall: PosY,
          coord_along_wall: 2
        }
      ]
    );
    let (st, b) = st.next();
    assert_eq!(
      b,
      &[
        Bounce {
          wall: PosY,
          coord_along_wall: 4
        },
        Bounce {
          wall: PosX,
          coord_along_wall: 8
        }
      ]
    );
    assert_eq!(
      &st.gliders().collect::<Vec<_>>(),
      &[
        ((3, 1), NegY),
        ((5, 1), PosX),
        ((6, 2), PosX),
        ((0, 3), NegX),
        ((0, 3), NegY),
        ((5, 4), PosY),
        ((2, 5), PosX),
        ((7, 5), NegY),
        ((2, 6), NegY),
        ((5, 6), NegX),
        ((3, 7), PosX),
        ((4, 7), NegY),
        ((7, 8), NegX),
      ]
    );
  }
}
