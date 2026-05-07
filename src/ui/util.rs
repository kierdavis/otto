#[derive(Clone, Copy, Debug)]
pub struct WStr {
  pub val: &'static str,
  pub width: u16,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Xy {
  pub x: u16,
  pub y: u16,
}

impl Xy {
  pub const ZERO: Self = Self { x: 0, y: 0 };

  #[allow(dead_code)]
  pub fn add_x(self, offset: u16) -> Xy {
    Xy {
      x: self.x + offset,
      y: self.y,
    }
  }

  #[allow(dead_code)]
  pub fn add_y(self, offset: u16) -> Xy {
    Xy {
      x: self.x,
      y: self.y + offset,
    }
  }

  #[allow(dead_code)]
  pub fn sub_x(self, offset: u16) -> Xy {
    Xy {
      x: self.x - offset,
      y: self.y,
    }
  }

  #[allow(dead_code)]
  pub fn sub_y(self, offset: u16) -> Xy {
    Xy {
      x: self.x,
      y: self.y - offset,
    }
  }
}

impl std::ops::Add for Xy {
  type Output = Self;
  fn add(self, other: Self) -> Self {
    Self {
      x: self.x + other.x,
      y: self.y + other.y,
    }
  }
}

impl std::ops::Sub for Xy {
  type Output = Self;
  fn sub(self, other: Self) -> Self {
    Self {
      x: self.x - other.x,
      y: self.y - other.y,
    }
  }
}

impl From<(u16, u16)> for Xy {
  fn from(pair: (u16, u16)) -> Xy {
    Xy {
      x: pair.0,
      y: pair.1,
    }
  }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Rect {
  pub top_left: Xy,
  pub size: Xy,
}

impl Rect {
  pub const ZERO: Self = Self {
    top_left: Xy::ZERO,
    size: Xy::ZERO,
  };

  #[allow(dead_code)]
  pub fn width(self) -> u16 {
    self.size.x
  }

  #[allow(dead_code)]
  pub fn height(self) -> u16 {
    self.size.y
  }

  #[allow(dead_code)]
  pub fn left(self) -> u16 {
    self.top_left.x
  }

  #[allow(dead_code)]
  pub fn top(self) -> u16 {
    self.top_left.y
  }

  #[allow(dead_code)]
  pub fn right(self) -> u16 {
    self.top_left.x + self.size.x
  }

  #[allow(dead_code)]
  pub fn bottom(self) -> u16 {
    self.top_left.y + self.size.y
  }

  #[allow(dead_code)]
  pub fn top_left(self) -> Xy {
    self.top_left
  }

  #[allow(dead_code)]
  pub fn top_right(self) -> Xy {
    self.top_left.add_x(self.size.x)
  }

  #[allow(dead_code)]
  pub fn bottom_left(self) -> Xy {
    self.top_left.add_y(self.size.y)
  }

  #[allow(dead_code)]
  pub fn bottom_right(self) -> Xy {
    self.top_left + self.size
  }

  pub fn hsplit_at(self, at: u16) -> (Rect, Rect) {
    if at > self.size.x {
      panic!("argument out of range");
    }
    (
      Rect {
        top_left: self.top_left,
        size: Xy {
          x: at,
          y: self.size.y,
        },
      },
      Rect {
        top_left: Xy {
          x: self.top_left.x + at,
          y: self.top_left.y,
        },
        size: Xy {
          x: self.size.x - at,
          y: self.size.y,
        },
      },
    )
  }

  pub fn vsplit_at(self, at: u16) -> (Rect, Rect) {
    if at > self.size.y {
      panic!("argument out of range");
    }
    (
      Rect {
        top_left: self.top_left,
        size: Xy {
          x: self.size.x,
          y: at,
        },
      },
      Rect {
        top_left: Xy {
          x: self.top_left.x,
          y: self.top_left.y + at,
        },
        size: Xy {
          x: self.size.x,
          y: self.size.y - at,
        },
      },
    )
  }
}
