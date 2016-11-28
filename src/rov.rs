
use errors::*;
use std::io::Read;
use std::thread;
use std::time::Duration;
use serial::{self, SerialPort, SerialPortSettings};
use std::sync::mpsc::{self, Sender, Receiver};

const COMMAND_CONTROL_MOTOR: u8 = 0x10;
const COMMAND_COLLECT_SAMPLES: u8 = 0x20;
const COMMAND_LIGHTS_ON: u8 = 0x31;
const COMMAND_LIGHTS_OFF: u8 = 0x30;
const COMMAND_MASTER_ON: u8 = 0x40;
const COMMAND_MASTER_OFF: u8 = 0x43;

#[derive(Clone)]
pub enum RovCommand {
    ControlMotor { id: u8, throttle: i16 },
    CollectSamples { amount: u8 },
    LightsOn,
    LightsOff,
    MasterOn,
    MasterOff,
}

impl RovCommand {
    pub fn to_byte_slice(&self) -> Vec<u8> {
        match *self {
            RovCommand::ControlMotor { id, throttle } => {
                vec![COMMAND_CONTROL_MOTOR,
                     id,
                     ((throttle >> 8) & 0xFF) as u8,
                     (throttle & 0xFF) as u8]
            }
            RovCommand::CollectSamples { amount } => vec![COMMAND_COLLECT_SAMPLES, amount],
            RovCommand::LightsOn => vec![COMMAND_LIGHTS_ON],
            RovCommand::LightsOff => vec![COMMAND_LIGHTS_OFF],
            RovCommand::MasterOn => vec![COMMAND_MASTER_ON],
            RovCommand::MasterOff => vec![COMMAND_MASTER_OFF],
        }
    }
}

#[derive(Debug)]
pub enum RovResponse {
    Byte(u8),
}


pub struct Rov {
    command_sender: Sender<Option<RovCommand>>,
    response_receiver: Receiver<RovResponse>,
}

use std::path::PathBuf;

impl Rov {
    pub fn new(port_path: PathBuf) -> Rov {
        let (command_sender, command_receiver) = mpsc::channel();
        let (response_sender, response_receiver) = mpsc::channel();
        thread::spawn(|| Rov::start_device_thread(port_path, command_receiver, response_sender));
        Rov {
            command_sender: command_sender,
            response_receiver: response_receiver,
        }
    }

    pub fn send_command(&mut self, command: RovCommand) -> Result<()> {
        self.command_sender
            .send(Some(command))
            .chain_err(|| "Could not send command to device thread")
    }

    pub fn responses(&mut self) -> Vec<RovResponse> {
        self.response_receiver.try_iter().collect()
    }

    fn start_device_thread(port_path: PathBuf,
                           command_receiver: Receiver<Option<RovCommand>>,
                           response_sender: Sender<RovResponse>) {
        // Open port
        let mut port = serial::open(&port_path).expect("Couldn't open port");

        let mut settings = serial::PortSettings::default();
        settings.set_char_size(serial::CharSize::Bits8);
        settings.set_parity(serial::Parity::ParityNone);
        settings.set_stop_bits(serial::StopBits::Stop1);
        settings.set_baud_rate(serial::BaudRate::Baud115200).expect("Error setting baud rate");
        port.configure(&settings).expect("Error configuring port");

        port.set_timeout(Duration::from_millis(5)).expect("Error setting timeout");

        // Wait for a few milliseconds
        thread::sleep(Duration::from_millis(1000));

        'device: loop {
            // Check for commands to send
            for command_option in command_receiver.try_iter() {
                if let Some(command) = command_option {
                    Rov::write_message(&mut port, &command.to_byte_slice())
                        .expect("Could not write message.");
                } else {
                    break 'device;
                }
            }

            // Read 1 byte
            let mut buffer = [0u8; 1];
            use std::io;
            match port.read_exact(&mut buffer) {
                Ok(()) => {
                    response_sender.send(RovResponse::Byte(buffer[0]))
                    .expect("Could send response to receiver");
                }
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                    // Do nothing
                }
                Err(e) => panic!("Couldn't read from port: {:?}", e),
            }
        }
    }


    fn write_message<S>(port: &mut S, message: &[u8]) -> Result<()>
        where S: serial::SerialPort
    {
        let parity = message.iter().skip(1).fold(message[0], |acc, i| acc ^ i);
        port.write(message).chain_err(|| "Couldn't write message")?;
        port.write(&[parity]).chain_err(|| "Couldn't write parity")?;
        Ok(())
    }
}

impl Drop for Rov {
    fn drop(&mut self) {
        let _ = self.command_sender.send(None);
    }
}
