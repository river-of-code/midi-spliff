use bitmatch::bitmatch;
use midir::{Ignore, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use std::collections::HashMap;
use std::error::Error;
use std::io::stdin;

const MIDI_SPLIFF: &str = "midi-spliff";

fn main() {
    match run() {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err),
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut midi_in = MidiInput::new(MIDI_SPLIFF)?;
    midi_in.ignore(Ignore::None);
    let mut inputs: HashMap<String, MidiInputConnection<()>> = HashMap::new();

    let in_ports = midi_in.ports();
    for port in in_ports.iter() {
        let mi = MidiInput::new(MIDI_SPLIFF)?;
        let cxn = mi
            .connect(
                port,
                MIDI_SPLIFF,
                move |stamp, message, _| {
                    println!(
                        "[{}]: {:?} @ {} {:?}",
                        MIDI_SPLIFF,
                        Message::parse(message),
                        stamp,
                        message,
                    );
                },
                (),
            )
            .unwrap();
        let port_name = midi_in.port_name(port).unwrap();
        println!("[input] connecting to {}...", port_name);
        inputs.insert(port_name.to_string(), cxn);
    }

    let midi_out = MidiOutput::new(MIDI_SPLIFF)?;
    let out_ports = midi_out.ports();
    let mut outputs: HashMap<String, MidiOutputConnection> = HashMap::new();

    for (_i, port) in out_ports.iter().enumerate() {
        let port_name = midi_out.port_name(port).unwrap();
        println!("[output] connecting to {}...", port_name);
        let mo = MidiOutput::new(MIDI_SPLIFF)?;
        let cxn = mo.connect(port, &port_name)?;
        outputs.insert(port_name.to_string(), cxn);
    }

    println!("Connections open (press enter to exit) ...");

    let mut input = String::new();
    input.clear();
    stdin().read_line(&mut input)?; // wait for next enter key press

    println!("Closing connection");
    Ok(())
}

#[derive(Debug)]
struct Message {
    status: MidiStatus,
    channel: u8,
    byte_one: u8,
    byte_two: Option<u8>,
}

#[derive(Debug)]
enum MidiStatus {
    NoteOn,
    NoteOff,
    AfterTouch,
    ControllerValue,
    ProgramChange,
    ChannelPressure,
    PitchBend,
    Unknown,
}

impl MidiStatus {
    #[bitmatch]
    pub fn get_status(a: &u8) -> (MidiStatus, u8) {
        #[bitmatch]
        match a {
            "1000_nnnn" => (MidiStatus::NoteOff, n),
            "1001_nnnn" => (MidiStatus::NoteOn, n),
            "1010_nnnn" => (MidiStatus::AfterTouch, n),
            "1011_nnnn" => (MidiStatus::ControllerValue, n),
            "1100_nnnn" => (MidiStatus::ProgramChange, n),
            "1101_nnnn" => (MidiStatus::ChannelPressure, n),
            "1110_nnnn" => (MidiStatus::PitchBend, n),
            _ => (MidiStatus::Unknown, 0),
        }
    }
}
impl Message {
    /*
    ----------------------------------------------------------------------------------
    Status    Byte 1    Byte 2    Message           Legend
    ----------------------------------------------------------------------------------
    1000nnnn  0kkkkkkk  0vvvvvvv  Note Off          n=channel k=key v=velocity
    1001nnnn  0kkkkkkk  0vvvvvvv  Note On           n=channel k=key v=velocity
    1010nnnn  0kkkkkkk  0ppppppp  AfterTouch        n=channel k=key p=pressure
    1011nnnn  0ccccccc  0vvvvvvv  Controller Value  n=channel c=controller v=value
    1100nnnn  0ppppppp  [none]    Program Change    n=channel p=preset
    1101nnnn  0ppppppp  [none]    Channel Pressure  n=channel p=pressure
    1110nnnn  0fffffff  0ccccccc  Pitch Bend        n=channel c=coarse f=fine (14 bit)
    ----------------------------------------------------------------------------------
    */
    fn parse(message: &[u8]) -> Message {
        let status = MidiStatus::get_status(&message[0]);

        match status {
            (MidiStatus::NoteOff, ch) => Message {
                status: MidiStatus::NoteOff,
                channel: ch,
                byte_one: message[1],       // key
                byte_two: Some(message[2]), // velocity
            },
            (MidiStatus::NoteOn, ch) => Message {
                status: MidiStatus::NoteOn,
                channel: ch,
                byte_one: message[1],       // key
                byte_two: Some(message[2]), // velocity
            },
            (MidiStatus::AfterTouch, ch) => Message {
                status: MidiStatus::AfterTouch,
                channel: ch,
                byte_one: message[1],       // key
                byte_two: Some(message[2]), // pressure
            },
            (MidiStatus::ControllerValue, ch) => Message {
                status: MidiStatus::ControllerValue,
                channel: ch,
                byte_one: message[1],       // key
                byte_two: Some(message[2]), // pressure
            },
            (MidiStatus::ProgramChange, ch) => Message {
                status: MidiStatus::ProgramChange,
                channel: ch,
                byte_one: message[1], // key
                byte_two: None,       // pressure
            },
            (MidiStatus::ChannelPressure, ch) => Message {
                status: MidiStatus::ChannelPressure,
                channel: ch,
                byte_one: message[1], // key
                byte_two: None,       // pressure
            },
            (MidiStatus::PitchBend, ch) => Message {
                status: MidiStatus::PitchBend,
                channel: ch,
                byte_one: message[1],       // key
                byte_two: Some(message[2]), // pressure
            },
            _ => Message {
                status: MidiStatus::Unknown,
                channel: 0,
                byte_one: 0,
                byte_two: None,
            },
        }
    }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
