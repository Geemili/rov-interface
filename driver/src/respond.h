#ifndef respond_h_INCLUDED
#define respond_h_INCLUDED

void say_motor(uint8_t id, int16_t thrust); 
void say_compass_orientation(int16_t x, int16_t y, int16_t z);
void say_compass_disabled(); 
void say_lights_on();
void say_lights_off();
void say_master_on();
void say_master_off();
void say_servo(uint8_t id, int16_t microseconds);
void say_no_i2c();
void say_i2c_found(uint8_t id, uint8_t error_code);
void say_version();

#endif // respond_h_INCLUDED

