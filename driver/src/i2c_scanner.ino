
#include <Wire.h>
#include "respond.h"

void i2c_scan() {
    uint8_t error, address, n_devices;

    n_devices = 0;

    for (address = 1; address < 127; address++) {
        Wire.beginTransmission(address);
        error = Wire.endTransmission();

        if (error == 0) {
            say_i2c_found(address, 0);
            n_devices++;
        } else if (error == 4) {
            say_i2c_found(address, 4);
        }
    }

    if (n_devices == 0) {
        say_no_i2c();
    }
}

