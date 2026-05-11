use crate::music_theory::Pitch;
use alsa::seq::{self, EvNote, EventType, PortCap, PortType, Seq};

pub struct Interface {
  seq: Seq,
  sync_port_id: i32,
  output_port_id: i32,
}

impl Interface {
  pub fn open() -> Self {
    let seq = Seq::open(
      None, // open the default sequencer
      None, // duplex
      true, // non-blocking mode
    )
    .expect("failed to open ALSA sequencer");

    seq
      .set_client_name(c"otto")
      .expect("failed to set ALSA sequencer client name");

    let sync_port_id = seq
      .create_simple_port(
        c"sync",
        PortCap::WRITE | PortCap::SUBS_WRITE, // what other clients can do to this port
        PortType::APPLICATION,
      )
      .expect("failed to create ALSA sequencer sync port");

    let output_port_id = seq
      .create_simple_port(
        c"out",
        PortCap::READ | PortCap::SUBS_READ, // what other clients can do to this port
        PortType::APPLICATION,
      )
      .expect("failed to create ALSA sequencer output port");

    Self {
      seq,
      sync_port_id,
      output_port_id,
    }
  }

  pub fn try_recv(&self) -> Option<Event> {
    const EAGAIN: i32 = 11;
    let mut input = self.seq.input();
    loop {
      match input.event_input() {
        Ok(ev) => match ev.get_type() {
          EventType::Clock if ev.get_dest().port == self.sync_port_id => return Some(Event::Clock),
          _ => {}
        },
        Err(err) if err.errno() == EAGAIN => return None,
        Err(err) => panic!("failed to receive ALSA sequencer event: {err}"),
      }
    }
  }

  pub fn send_note_on(&self, pitch: Pitch) {
    let mut ev = seq::Event::new(
      EventType::Noteon,
      &EvNote {
        channel: 0,
        note: pitch.to_midi(),
        velocity: 32,
        off_velocity: 32,
        duration: 0,
      },
    );
    ev.set_source(self.output_port_id);
    ev.set_subs();
    ev.set_direct();
    self
      .seq
      .event_output(&mut ev)
      .expect("failed to send ALSA sequencer event");
  }

  pub fn send_note_off(&self, pitch: Pitch) {
    let mut ev = seq::Event::new(
      EventType::Noteoff,
      &EvNote {
        channel: 0,
        note: pitch.to_midi(),
        velocity: 0,
        off_velocity: 0,
        duration: 0,
      },
    );
    ev.set_source(self.output_port_id);
    ev.set_subs();
    ev.set_direct();
    self
      .seq
      .event_output(&mut ev)
      .expect("failed to send ALSA sequencer event");
  }

  pub fn flush(&self) {
    self
      .seq
      .drain_output()
      .expect("failed to flush ALSA sequencer events");
  }
}

pub enum Event {
  Clock,
}
