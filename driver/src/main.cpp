
#include <Arduino.h>
#include <Servo.h>
#include "commands.h"

#define LIGHTS_RELAY_PIN 13
#define MAX_CONTROL_SIGNAL 1100
#define MIN_CONTROL_SIGNAL 1900

void handle_command(Commands command, uint8_t *buffer);

enum ParserState {
  ReceivingCommand,
  ReceivingData,
  Validating,
};

Commands command_received;
uint8_t buffer[4];
uint8_t buffer_idx;
uint8_t bytes_to_read;
uint8_t command_crc;
ParserState parser_state;



Servo motors[6];

void setup()
{
  Serial.begin(115200);
  parser_state = ParserState::ReceivingCommand;
  pinMode(LIGHTS_RELAY_PIN, OUTPUT);

  // TODO: Replace the (fake) pins numbers here with the actual pins.
  motors[0].attach(2);
  motors[1].attach(3);
  motors[2].attach(4);
  motors[3].attach(5);
  motors[4].attach(6);
  motors[5].attach(7);

  for (uint8_t i = 0; i < 6; i++) {
    // Write the stop signal, which is exactly in the middle of the control
    // signal range
    motors[i].writeMicroseconds(MIN_CONTROL_SIGNAL + MAX_CONTROL_SIGNAL / 2);
  }
  // Delay to allow the ESC to recognize the stopped signal
  delay(1000);
}

void loop()
{
  switch (parser_state) {
    case ReceivingCommand:
    {
      if (Serial.available() <= 0)
      {
        break;
      }
      uint8_t in = Serial.read();
      if (is_valid_command(in)) {
        command_received = (Commands) in;
        buffer_idx = 0;
        bytes_to_read = get_command_length(command_received);
        parser_state = ReceivingData;
      } else {
        // Invalid data. :( See if the next packet is correct.
        break;
      }
    }
    case ReceivingData:
    {
      if (buffer_idx == bytes_to_read)
      {
        // We have read all the bytes
        parser_state = Validating;
        break;
      }
      if (Serial.available() <= 0)
      {
        break;
      }
      buffer[buffer_idx++] = Serial.read();
      break;
    }
    case Validating:
    {
      if (Serial.available() <= 0)
      {
        break;
      }
      command_crc = Serial.read();
      uint8_t crc = command_received;
      for (uint8_t i = 0; i < bytes_to_read; i++)
      {
        crc ^= buffer[i];
      }
      if (command_crc == crc)
      {
        handle_command(command_received, buffer);
      }
      parser_state = ReceivingCommand;
      break;
    }
  }

  // TODO: Add call to motors[*].update() to prevent the int used for tracking
  // RPM from overflowing. It is recommended to call it 4 to 10 times a second.

  // TODO: Add code to check if the motors are alive. If any of them aren't
  // connected, send a message to the surface.
}

void handle_command(Commands command, uint8_t *buffer)
{
  switch (command)
  {
    case ControlMotor: {
      uint8_t motor_id = buffer[0];
      if (motor_id < 6)
      {
        int16_t throttle = (buffer[1] << 8) | buffer[0];
        int16_t control_signal = map(throttle, INT16_MIN, INT16_MAX, MIN_CONTROL_SIGNAL, MAX_CONTROL_SIGNAL);
        motors[motor_id].writeMicroseconds(control_signal);
        break;
      }
      // TODO: Send back error message when motor_id is greater then 6
      break;
    }
    case CollectSamples: break;
    case LightsOn: {
      digitalWrite(LIGHTS_RELAY_PIN, HIGH);
      break;
    }
    case LightsOff: {
      digitalWrite(LIGHTS_RELAY_PIN, LOW);
      break;
    }
  }
}
