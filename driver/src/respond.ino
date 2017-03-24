
#define RESPONSE_MOTOR 0x10
#define RESPONSE_COLLECTING_SAMPLES 0x20
#define RESPONSE_COLLECTING_SAMPLES_NOT 0x21
#define RESPONSE_LIGHTS_ON 0x31
#define RESPONSE_LIGHTS_OFF 0x30
#define RESPONSE_MASTER_ON 0x40
#define RESPONSE_MASTER_OFF 0x43
#define RESPONSE_SERVO 0x66

void say_motor(uint8_t id, int16_t thrust) {
    uint8_t buf[] = {
        RESPONSE_MOTOR,
        id,
        (uint8_t)((thrust>>8)&0xff),
        (uint8_t)((thrust)&0xff)
        };
    Serial.write(buf, 4);
}

void say_collecting_samples(uint16_t milliseconds_left) {
    uint8_t buf[] = {
        RESPONSE_COLLECTING_SAMPLES,
        (uint8_t)((milliseconds_left>>8)&0xff),
        (uint8_t)((milliseconds_left)&0xff)
        };
    Serial.write(buf, 3);
}

void say_collecting_samples_not() {
    Serial.write(RESPONSE_COLLECTING_SAMPLES_NOT);
}

void say_lights_on() {
    Serial.write(RESPONSE_LIGHTS_ON);
}

void say_lights_off() {
    Serial.write(RESPONSE_LIGHTS_OFF);
}

void say_master_on() {
    Serial.write(RESPONSE_MASTER_ON);
}

void say_master_off() {
    Serial.write(RESPONSE_MASTER_OFF);
}

void say_servo(uint8_t id, int16_t microseconds) {
    uint8_t buf[] = {
        RESPONSE_SERVO,
        id,
        (uint8_t)((microseconds>>8)&0xff),
        (uint8_t)((microseconds)&0xff)
        };
    Serial.write(buf, 4);
}

