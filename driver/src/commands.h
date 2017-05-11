
#ifndef __COMMANDS_H__
#define __COMMANDS_H__

#include <stdint.h>

#define COMMAND_CONTROL_MOTOR 0x10
#define COMMAND_LIGHTS_ON 0x31
#define COMMAND_LIGHTS_OFF 0x30
#define COMMAND_MASTER_ON 0x40
#define COMMAND_MASTER_OFF 0x43
#define COMMAND_CONTROL_SERVO 0x66

enum Commands {
  ControlMotor = COMMAND_CONTROL_MOTOR,
  LightsOn = COMMAND_LIGHTS_ON,
  LightsOff = COMMAND_LIGHTS_OFF,
  MasterOn = COMMAND_MASTER_ON,
  MasterOff = COMMAND_MASTER_OFF,
  ControlServo = COMMAND_CONTROL_SERVO
};

uint8_t get_command_length(Commands command) {
  switch (command) {
    case ControlMotor: return 3;
    case LightsOn: return 0;
    case LightsOff: return 0;
    case MasterOn: return 0;
    case MasterOff: return 0;
    case ControlServo: return 3;
  }
  return 0;
}

bool is_valid_command(uint8_t id) {
  switch ((Commands)id) {
    case ControlMotor: return true;
    case LightsOn: return true;
    case LightsOff: return true;
    case MasterOn: return true;
    case MasterOff: return true;
    case ControlServo: return true;
  }
  return true;
}

#endif
