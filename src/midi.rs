use crate::music_theory::Pitch;
use alsa::poll::{Descriptors, poll, pollfd};
use alsa::seq::{self, EvNote, EventType, PortCap, PortType, Seq};
use std::time::{Duration, Instant};

fn poll_descriptors<'a>(seq: &'a Seq) -> impl Descriptors + 'a {
  (seq, Some(alsa::Direction::Capture))
}

pub struct Interface {
  seq: Seq,
  sync_port_id: i32,
  output_port_id: i32,
  pollfds: Vec<pollfd>,
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

    let pollfds = poll_descriptors(&seq)
      .get()
      .expect("failed to get pollfds for ALSA sequence client");

    Self {
      seq,
      sync_port_id,
      output_port_id,
      pollfds,
    }
  }

  pub fn recv_timeout(&mut self, timeout: Duration) -> Option<Event> {
    self.recv_deadline(Instant::now() + timeout)
  }

  pub fn recv_deadline(&mut self, deadline: Instant) -> Option<Event> {
    let mut input = self.seq.input();

    loop {
      let now = Instant::now();
      if now >= deadline {
        return None;
      }

      let userspace_buffer_is_empty = input
        .event_input_pending(false)
        .expect("failed to query ALSA sequencer client for buffered events")
        == 0;
      if userspace_buffer_is_empty {
        let timeout_ms = deadline.duration_since(now).as_millis().try_into().unwrap();
        match poll(&mut self.pollfds, timeout_ms) {
          Ok(0) => return None, // Timed out.
          Ok(_) => {}           // Maybe there's an event to read, or the fd is closed.
          Err(err) => panic!("failed to poll ALSA sequencer for events: {err}"),
        }

        use alsa::poll::Flags;
        let revents = poll_descriptors(&self.seq)
          .revents(&self.pollfds)
          .expect("failed to extract revents from ALSA sequencer poll response");
        if !revents.contains(Flags::IN) {
          panic!("ALSA sequencer poll returned {revents:?}");
        }
      }

      // If we get here, there's an event ready to read - either from the userspace buffer or from the kernel.
      let ev = input
        .event_input()
        .expect("failed to read event from ALSA sequencer");
      match ev.get_type() {
        EventType::Clock if ev.get_dest().port == self.sync_port_id => return Some(Event::Clock),
        _ => {} // Event ignored.
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
