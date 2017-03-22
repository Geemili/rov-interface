
#include <Arduino.h>
#include <Servo.h>
#include "commands.h"
#include "main.h"

#define LIGHTS_RELAY_PIN 13

#define SAMPLER_RELAY_PIN 4
#define SAMPLER_SINGLE_SHOT_MS 2000

#define MAX_CONTROL_SIGNAL 1100
#define MIN_CONTROL_SIGNAL 1900

#ifndef INT16_MIN
#define INT16_MIN -32768
#endif
#ifndef INT16_MAX
#define INT16_MAX 32767
#endif

#define NUM_MOTORS 4
#define NUM_SERVOS 2

Commands command_received;
uint8_t buffer[4];
uint8_t buffer_idx;
uint8_t bytes_to_read;
uint8_t command_crc;
ParserState parser_state;

Servo motors[NUM_MOTORS];
Servo servos[NUM_SERVOS];
bool turn_off_motor;
uint32_t turn_off_motor_time;
bool robot_is_on;

void setup()
{
  Serial.begin(115200);
  parser_state = ReceivingCommand;
  master_on();
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

  if (turn_off_motor && millis() >= turn_off_motor_time) {
    digitalWrite(SAMPLER_RELAY_PIN, LOW);
    turn_off_motor = false;
  }
}

void handle_command(Commands command, uint8_t *buffer)
{
  if (command == MasterOn) {
    master_on();
    return;
  }
  if (!robot_is_on) {
    return;
  }
  switch (command)
  {
    case ControlMotor: {
      uint8_t motor_id = buffer[0];
      if (motor_id < NUM_MOTORS)
      {
        int16_t throttle = (buffer[1] << 8) | buffer[0];
        int16_t control_signal = map(throttle, INT16_MIN, INT16_MAX, MIN_CONTROL_SIGNAL, MAX_CONTROL_SIGNAL);
        motors[motor_id].writeMicroseconds(control_signal);
        break;
      }
      // TODO: Send back error message when motor_id is greater then 6
      break;
    }
    case CollectSamples: {
      uint32_t amount = buffer[0] * SAMPLER_SINGLE_SHOT_MS;
      if (turn_off_motor) {
        turn_off_motor_time += amount;
      } else {
        digitalWrite(SAMPLER_RELAY_PIN, HIGH);
        turn_off_motor = true;
        turn_off_motor_time = millis() + amount;
      }
      break;
    }
    case LightsOn: {
      digitalWrite(LIGHTS_RELAY_PIN, HIGH);
      break;
    }
    case LightsOff: {
      digitalWrite(LIGHTS_RELAY_PIN, LOW);
      break;
    }
    case MasterOn: {
      // We should never reach here
      break;
    }
    case MasterOff: {
      master_off();
      break;
    }
    case ControlServo: {
      uint8_t servo_id = buffer[0];
      if (servo_id < NUM_MOTORS)
      {
        int16_t microseconds = (buffer[1] << 8) | buffer[0];
        servos[servo_id].writeMicroseconds(microseconds);
        break;
      }
      // TODO: Send back error message when motor_id is greater then 6
      break;
    }
  }
}

void motors_stop() {
  for (uint8_t i = 0; i < NUM_MOTORS; i++) {
    // Write the stop signal, which is exactly in the middle of the control
    // signal range
    motors[i].writeMicroseconds(MIN_CONTROL_SIGNAL + MAX_CONTROL_SIGNAL / 2);
  }
}

void servos_reset() {
  for (uint8_t i = 0; i < NUM_SERVOS; i++) {
    // Write the stop signal, which is exactly in the middle of the control
    // signal range
    servos[i].writeMicroseconds(MIN_CONTROL_SIGNAL + MAX_CONTROL_SIGNAL / 2);
  }
}

void master_on() {
  robot_is_on = true;

  pinMode(LIGHTS_RELAY_PIN, OUTPUT);
  // TODO: Ask if the lights should default to on
  digitalWrite(LIGHTS_RELAY_PIN, LOW);

  pinMode(SAMPLER_RELAY_PIN, OUTPUT);
  digitalWrite(SAMPLER_RELAY_PIN, LOW);
  turn_off_motor = false;
  turn_off_motor_time = 0;

  /* ## Turn motors on ## */
  motors[0].attach(5);
  motors[1].attach(6);
  motors[2].attach(7);
  motors[3].attach(8);
  motors_stop();

  servos[0].attach(9);
  servos[1].attach(10);
  servos_reset();
  // Delay to allow the ESC to recognize the stopped signal
  delay(1000);
}

void master_off() {
  robot_is_on = false;

  digitalWrite(LIGHTS_RELAY_PIN, LOW);

  digitalWrite(SAMPLER_RELAY_PIN, LOW);

  motors_stop();
  servos_reset();
}

