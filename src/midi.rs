use crate::music_theory::Pitch;
use alsa::{
  Direction::Playback,
  seq::{
    EvNote, Event,
    EventType::{Noteoff, Noteon},
    PortCap, PortType, Seq,
  },
};

pub struct Interface {
  seq: Seq,
  output_port_id: i32,
}

impl Interface {
  pub fn open() -> Self {
    let seq = Seq::open(
      None, // open the default sequencer
      Some(Playback),
      true, // non-blocking mode
    )
    .expect("failed to open ALSA sequencer");

    seq
      .set_client_name(c"otto")
      .expect("failed to set ALSA sequencer client name");

    let output_port_id = seq
      .create_simple_port(
        c"out",
        PortCap::READ | PortCap::SUBS_READ, // what other clients can do to this port
        PortType::APPLICATION,
      )
      .expect("failed to create ALSA sequencer output port");

    Self {
      seq,
      output_port_id,
    }
  }

  pub fn emit_note_on(&self, pitch: Pitch) {
    let mut ev = Event::new(
      Noteon,
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
      .expect("failed to emit ALSA sequencer event");
  }

  pub fn emit_note_off(&self, pitch: Pitch) {
    let mut ev = Event::new(
      Noteoff,
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
      .expect("failed to emit ALSA sequencer event");
  }

  pub fn flush(&self) {
    self
      .seq
      .drain_output()
      .expect("failed to flush ALSA sequencer events");
  }
}
