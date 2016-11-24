
#ifndef __COMMANDS_H__
#define __COMMANDS_H__

#include <stdint.h>

#define COMMAND_CONTROL_MOTOR 0x10
#define COMMAND_COLLECT_SAMPLES 0x20
#define COMMAND_LIGHTS_ON 0x31
#define COMMAND_LIGHTS_OFF 0x30

enum Commands {
  ControlMotor = COMMAND_CONTROL_MOTOR,
  CollectSamples = COMMAND_COLLECT_SAMPLES,
  LightsOn = COMMAND_LIGHTS_ON,
  LightsOff = COMMAND_LIGHTS_OFF,
};

uint8_t get_command_length(Commands command) {
  switch (command) {
    case ControlMotor: return 3;
    case CollectSamples: return 1;
    case LightsOn: return 0;
    case LightsOff: return 0;
  }
  return 0;
}

bool is_valid_command(uint8_t id) {
  switch ((Commands)id) {
    case ControlMotor: return true;
    case CollectSamples: return true;
    case LightsOn: return true;
    case LightsOff: return true;
  }
  return true;
}

#endif