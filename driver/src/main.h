
#ifndef __MAIN_H__
#define __MAIN_H__

#include <stdint.h>
#include "commands.h"

void handle_command(Commands command, uint8_t *buffer);
void master_on();
void master_off();

enum ParserState {
  ReceivingCommand,
  ReceivingData,
  Validating,
};

#endif
