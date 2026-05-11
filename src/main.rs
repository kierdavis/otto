mod automaton;
mod datamodel;
mod midi;
mod music_theory;
mod realtime;
mod ui;
mod util;

fn main() {
  std::thread::spawn(realtime::main);
  ui::main()
}
