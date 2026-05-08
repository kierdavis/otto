use std::ops::{Index, IndexMut};

#[derive(Clone, Debug)]
pub struct Matrix<T> {
  data: Vec<T>,
  width: usize,
}

impl<T> Matrix<T> {
  pub fn new(width: usize, height: usize, mut fill: impl FnMut(usize, usize) -> T) -> Self {
    Self {
      data: (0..height)
        .flat_map(|y| (0..width).map(move |x| (x, y)))
        .map(|(x, y)| fill(x, y))
        .collect(),
      width,
    }
  }

  pub fn width(&self) -> usize {
    self.width
  }

  pub fn height(&self) -> usize {
    debug_assert_eq!(self.data.len() % self.width, 0);
    self.data.len() / self.width
  }

  pub fn enumerate(&self) -> impl Iterator<Item = ((usize, usize), &T)> + '_ {
    self
      .data
      .iter()
      .enumerate()
      .map(|(idx, val)| ((idx % self.width, idx / self.width), val))
  }

  pub fn fill(&mut self, mut f: impl FnMut(usize, usize) -> T) {
    for (i, cell) in self.data.iter_mut().enumerate() {
      *cell = f(i % self.width, i / self.width);
    }
  }
}

impl<T> Index<(usize, usize)> for Matrix<T> {
  type Output = T;
  fn index(&self, coords: (usize, usize)) -> &T {
    &self.data[coords.0 + coords.1 * self.width]
  }
}

impl<T> IndexMut<(usize, usize)> for Matrix<T> {
  fn index_mut(&mut self, coords: (usize, usize)) -> &mut T {
    &mut self.data[coords.0 + coords.1 * self.width]
  }
}
