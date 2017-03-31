
use errors::*;
use std::io::Read;
use std::thread;
use std::time::Duration;
use serialport::{self, SerialPort};
use std::sync::mpsc::{self, Sender, Receiver};

const COMMAND_CONTROL_MOTOR: u8 = 0x10;
const COMMAND_LIGHTS_ON: u8 = 0x31;
const COMMAND_LIGHTS_OFF: u8 = 0x30;
const COMMAND_MASTER_ON: u8 = 0x40;
const COMMAND_MASTER_OFF: u8 = 0x43;
const COMMAND_CONTROL_SERVO: u8 = 0x66;

#[derive(Clone, Debug)]
pub enum RovCommand {
    ControlMotor { id: u8, throttle: i16 },
    LightsOn,
    LightsOff,
    MasterOn,
    MasterOff,
    ControlServo { id: u8, microseconds: i16 },
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
            RovCommand::LightsOn => vec![COMMAND_LIGHTS_ON],
            RovCommand::LightsOff => vec![COMMAND_LIGHTS_OFF],
            RovCommand::MasterOn => vec![COMMAND_MASTER_ON],
            RovCommand::MasterOff => vec![COMMAND_MASTER_OFF],
            RovCommand::ControlServo { id, microseconds } => vec![COMMAND_CONTROL_SERVO, id, ((microseconds >> 8) & 0xFF) as u8, (microseconds & 0xFF) as u8,],
        }
    }
}

const RESPONSE_MOTOR: u8 = 0x10;
const RESPONSE_COMPASS_ORIENTATION: u8 = 0x20;
const RESPONSE_COMPASS_DISABLED: u8 = 0x21;
const RESPONSE_LIGHTS_ON: u8 = 0x31;
const RESPONSE_LIGHTS_OFF: u8 = 0x30;
const RESPONSE_MASTER_ON: u8 = 0x40;
const RESPONSE_MASTER_OFF: u8 = 0x43;
const RESPONSE_SERVO: u8 = 0x66;

#[derive(Debug)]
pub enum RovResponse {
    Motor { id: u8, throttle: i16 },
    CompassOrientation { x: i16, y: i16, z: i16 },
    CompassDisabled,
    LightsOn,
    LightsOff,
    MasterOn,
    MasterOff,
    Servo { id: u8, microseconds: i16 },
}

pub enum ParseStatus {
    Ok(RovResponse, usize), // bytes read
    TooShort,
    Invalid,
}

use std::collections::VecDeque;
impl RovResponse {
    /// The length of the response, not including the id
    pub fn response_length(command_byte: u8) -> Option<usize> {
        match command_byte {
            RESPONSE_MOTOR => Some(3),
            RESPONSE_COMPASS_ORIENTATION => Some(6),
            RESPONSE_COMPASS_DISABLED  => Some(0),
            RESPONSE_LIGHTS_ON => Some(0),
            RESPONSE_LIGHTS_OFF => Some(0),
            RESPONSE_MASTER_ON => Some(0),
            RESPONSE_MASTER_OFF => Some(0),
            RESPONSE_SERVO => Some(3),
            _ => None,
        }
    }

    // Make it not VecDeque. Change parsing method completely
    fn parse(buffer: &VecDeque<u8>) -> ParseStatus {
        let length = match Self::response_length(buffer[0]) {
            Some(len) => len,
            None => return ParseStatus::Invalid,
        };
        // The `+1` is because the length doesn't include the command id
        if buffer.len() < (length + 1) {
            return ParseStatus::TooShort;
        }

        let i16_from_bytes = |left: u8, right: u8| ((left as i16) << 8) | (right as i16);
        let command = match buffer[0] {
            RESPONSE_MOTOR => RovResponse::Motor {
                id: buffer[1],
                throttle: i16_from_bytes(buffer[2], buffer[3]),
            },

            RESPONSE_COMPASS_ORIENTATION => RovResponse::CompassOrientation {
                x: ((buffer[1] as i16) << 8) | (buffer[2] as i16),
                y: ((buffer[3] as i16) << 8) | (buffer[4] as i16),
                z: ((buffer[5] as i16) << 8) | (buffer[6] as i16),
            },

            RESPONSE_COMPASS_DISABLED => RovResponse::CompassDisabled,
            RESPONSE_LIGHTS_ON => RovResponse::LightsOn,
            RESPONSE_LIGHTS_OFF => RovResponse::LightsOff,
            RESPONSE_MASTER_ON => RovResponse::MasterOn,
            RESPONSE_MASTER_OFF => RovResponse::MasterOff,
            RESPONSE_SERVO => RovResponse::Servo {
                id: buffer[1],
                microseconds: ((buffer[2] as i16) << 8) | (buffer[3] as i16),
            },

            _ => return ParseStatus::Invalid,
        };
        ParseStatus::Ok(command, (length+1))
    }
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
        let mut port = serialport::open(&port_path).expect("Couldn't open port");

        let mut settings = serialport::SerialPortSettings::default();
        settings.data_bits = serialport::DataBits::Eight;
        settings.parity = serialport::Parity::None;
        settings.stop_bits = serialport::StopBits::One;
        settings.baud_rate = serialport::BaudRate::Baud115200;
        port.set_all(&settings).expect("Error configuring port");

        port.set_timeout(Duration::from_millis(5)).expect("Error setting timeout");

        // Wait for a few milliseconds
        thread::sleep(Duration::from_millis(1000));

        use std;
        let mut response_buffer = std::collections::VecDeque::new();

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
                    response_buffer.push_back(buffer[0]);
                    match RovResponse::parse(&response_buffer) {
                        ParseStatus::Ok(response, bytes_read) => {
                            response_sender.send(response).expect("Could send response to receiver");
                            for _ in 0..bytes_read {
                                response_buffer.pop_front();
                            }
                        }
                        ParseStatus::TooShort => {}
                        ParseStatus::Invalid => {response_buffer.pop_front();}
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                    // Do nothing
                }
                Err(e) => panic!("Couldn't read from port: {:?}", e),
            }

        }
    }


    fn write_message(port: &mut Box<SerialPort>, message: &[u8]) -> Result<()>
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
