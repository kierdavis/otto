mod canvas;
mod components;
mod mouse;
mod styles;
mod util;

use self::canvas::Canvas;
use self::components::{Component, Components};
use self::util::{Rect, Xy};
use crate::datamodel::{self, Change};
use crossbeam::{
  channel::{self, Receiver, Sender},
  select,
};
use crossterm::event::Event;
use std::ops::ControlFlow::{self, Break, Continue};
use std::sync::OnceLock;
use std::thread;
use std::time::{Duration, Instant};

pub fn main() {
  use crossterm::{execute, terminal};
  use std::panic;

  fn reset(dest: &mut impl std::io::Write) {
    let _ = terminal::disable_raw_mode();
    let _ = execute!(
      dest,
      crossterm::cursor::Show,
      crossterm::event::DisableMouseCapture,
      crossterm::terminal::LeaveAlternateScreen,
      crossterm::style::ResetColor,
    );
  }

  // If the thread panics, reset the terminal before displaying the panic message.
  let old_panic_hook: &_ = Box::leak(panic::take_hook());
  panic::set_hook(Box::new(|arg| {
    // Can't use stdout since the lock is already held.
    reset(&mut std::io::stderr().lock());
    old_panic_hook(arg)
  }));

  let mut stdout = std::io::stdout().lock();
  terminal::enable_raw_mode().expect("failed to enable raw mode");
  execute!(
    stdout,
    crossterm::cursor::Hide,
    crossterm::event::EnableMouseCapture,
    crossterm::terminal::EnterAlternateScreen,
  )
  .expect("failed to write to stdout");

  let mut canvas = Canvas::new(stdout);
  main2(&mut canvas);
  let mut stdout = canvas.into_inner();

  reset(&mut stdout);
  panic::set_hook(Box::new(old_panic_hook));
}

fn main2<'a, 'b>(canvas: &'a mut Canvas<'b>) {
  let (datamodel_changes_sender, datamodel_changes) = channel::bounded(32);
  DATAMODEL_CHANGES_SENDER
    .set(datamodel_changes_sender)
    .expect("ui::main called more than once");
  let (pending_flush_sender, pending_flush_receiver) = channel::bounded(1);
  (UI {
    canvas,
    components: Components::build(),
    screen_size: crossterm::terminal::size()
      .expect("failed to read screen size")
      .into(),
    mouse_map: mouse::ZoneMap::new(Xy::ZERO),
    place_ok: false,
    terminal_events: receive_events(),
    datamodel_changes,
    clock_indicator_timeout: channel::never(),
    pending_flush_sender,
    pending_flush_receiver,
  })
  .run()
}

struct UI<'a, 'b> {
  canvas: &'a mut Canvas<'b>,
  components: Components,
  screen_size: Xy,
  mouse_map: mouse::ZoneMap,
  place_ok: bool,
  terminal_events: Receiver<Event>,
  datamodel_changes: Receiver<Change>,
  clock_indicator_timeout: Receiver<Instant>,
  pending_flush_sender: Sender<()>,
  pending_flush_receiver: Receiver<()>,
}

impl<'a, 'b> UI<'a, 'b> {
  fn run(mut self) {
    self.place_and_paint_all();
    loop {
      select! {
        recv(self.terminal_events) -> result => {
          let flow = self.on_terminal_event(result.expect("receive_events thread crashed"));
          if flow.is_break() {
            break
          }
        }
        recv(self.datamodel_changes) -> result => {
          self.on_datamodel_change(result.expect("DATAMODEL_CHANGES_SENDER dropped"));
        }
        recv(self.clock_indicator_timeout) -> result => {
          result.expect("clock_indicator_timeout channel closed");
          Change::SetClockIndicatorLit(false).apply();
        }
        recv(self.pending_flush_receiver) -> result => {
          result.expect("pending_flush_sender dropped");
          self.canvas.flush();
        }
      }
    }
  }

  fn on_terminal_event(&mut self, event: Event) -> ControlFlow<()> {
    use crossterm::event::{
      Event::{Key, Mouse, Resize},
      KeyCode::{Char, Esc},
      KeyEvent,
      KeyEventKind::Press,
      KeyModifiers,
      MouseButton::Left,
      MouseEvent,
      MouseEventKind::Down,
    };

    match event {
      Resize(x, y) => {
        self.screen_size = Xy { x, y };
        self.place_and_paint_all();
        Continue(())
      }

      Key(KeyEvent {
        kind: Press,
        code: Esc,
        modifiers: KeyModifiers::NONE,
        ..
      })
      | Key(KeyEvent {
        kind: Press,
        code: Char('c'),
        modifiers: KeyModifiers::CONTROL,
        ..
      }) => Break(()),

      Key(KeyEvent {
        kind: Press,
        code: Char('l'),
        modifiers: KeyModifiers::CONTROL,
        ..
      }) => {
        self.place_and_paint_all();
        Continue(())
      }

      Mouse(MouseEvent {
        kind: Down(Left),
        column: x,
        row: y,
        modifiers: KeyModifiers::NONE,
      }) => match self.mouse_map.get(Xy { x, y }) {
        None => Continue(()),
        Some(mouse::Zone::ClockSrcBuiltin) => {
          Change::SetClockSrc(datamodel::ClockSrc::Builtin).apply();
          Continue(())
        }
        Some(mouse::Zone::ClockSrcMidi) => {
          Change::SetClockSrc(datamodel::ClockSrc::Midi).apply();
          Continue(())
        }
      },

      _ => Continue(()),
    }
  }

  fn on_datamodel_change(&mut self, change: Change) {
    match change {
      Change::SetClockIndicatorLit(true) => {
        self.clock_indicator_timeout = channel::after(Duration::from_millis(150));
      }
      _ => {}
    }

    if self.place_ok {
      match change {
        Change::AdvanceAutomatonState => {
          self.components.automaton.paint(self.canvas);
        }
        Change::SetClockIndicatorLit(_) => {
          self.components.clock_indicator.paint(self.canvas);
        }
        Change::SetClockSrc(_) => {
          self.components.clock_src_selector.paint(self.canvas);
        }
      }
      let _ = self.pending_flush_sender.try_send(());
    }
  }

  fn place_and_paint_all(&mut self) {
    let screen_rect = Rect {
      top_left: Xy::ZERO,
      size: self.screen_size,
    };
    self.mouse_map = mouse::ZoneMap::new(self.screen_size);
    self.place_ok = self
      .components
      .root
      .place(screen_rect, &mut self.mouse_map)
      .is_some();
    if !self.place_ok {
      self.mouse_map.clear();
    }

    self.canvas.clear();
    if self.place_ok {
      self.components.root.paint(self.canvas);
    } else {
      self.canvas.move_to(Xy::ZERO);
      self.canvas.write("window too small");
    }
    let _ = self.pending_flush_sender.try_send(());
  }
}

fn receive_events() -> Receiver<Event> {
  let (sender, receiver) = channel::bounded(8);
  thread::spawn(move || {
    loop {
      let ev = crossterm::event::read().expect("failed to read event");
      sender.send(ev).expect("ui thread crashed");
    }
  });
  receiver
}

static DATAMODEL_CHANGES_SENDER: OnceLock<Sender<Change>> = OnceLock::new();

pub fn on_datamodel_change(change: Change) {
  DATAMODEL_CHANGES_SENDER
    .get()
    .expect("ui::main hasn't been called yet")
    .try_send(change)
    .expect("ui thread crashed");
}
