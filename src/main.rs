#![no_std]
#![no_main]

mod as1130;
mod frame;
mod glasses;

use arduino_hal::{I2c, Peripherals};
use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    let mut _i2c = I2c::new(
        dp.TWI,
        // `A4`: I2C SDA signal.
        pins.a4.into_pull_up_input(),
        // `A5`: I2C SCL signal.
        pins.a5.into_pull_up_input(),
        // 400KHz bus speed.
        400_000,
    );

    loop {}
}
