
#include <Arduino.h>
#include "commands.h"

#define LIGHTS_RELAY_PIN 13

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

#define T100_POLECOUNT 6
#define T200_POLECOUNT 7

// TODO: Replace the (fake) addresses here with the actual addresses.
Arduino_I2C_ESC motors[] = {
  // Forward and sideways vectored motors.
  Arduino_I2C_ESC(0x29, T100_POLECOUNT), // 0
  Arduino_I2C_ESC(0x2A, T100_POLECOUNT), // 1
  Arduino_I2C_ESC(0x2B, T100_POLECOUNT), // 2
  Arduino_I2C_ESC(0x2C, T100_POLECOUNT), // 3
  // Top motors
  Arduino_I2C_ESC(0x2D, T200_POLECOUNT), // 4
  Arduino_I2C_ESC(0x2E, T200_POLECOUNT)  // 5
};

void setup()
{
  Serial.begin(115200);
  parser_state = ParserState::ReceivingCommand;
  pinMode(LIGHTS_RELAY_PIN, OUTPUT);
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
}

void handle_command(Commands command, uint8_t *buffer)
{
  switch (command)
  {
    case ControlMotor: {
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
