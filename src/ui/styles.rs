use crossterm::style::{Attributes, Color::*, ContentStyle};

const DEFAULT: ContentStyle = ContentStyle {
  foreground_color: None,
  background_color: None,
  underline_color: None,
  attributes: Attributes::none(),
};

pub const SELECTED: ContentStyle = ContentStyle {
  foreground_color: Some(Yellow),
  ..DEFAULT
};
