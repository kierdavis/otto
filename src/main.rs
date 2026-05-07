mod automaton;
mod datamodel;
mod realtime;
mod ui;
mod util;

fn main() {
  std::thread::spawn(realtime::main);
  ui::main()
}
