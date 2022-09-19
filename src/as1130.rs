use arduino_hal::i2c::Error as I2cError;
use arduino_hal::I2c;
use embedded_hal::blocking::i2c::Write;

trait AS1130 {
    const ADDR: u8;

    fn write_register(i2c: &mut I2c, target_register: u8, value: u8) -> Result<(), I2cError> {
        i2c.write(Self::ADDR, &[target_register, value])
    }
}

#[allow(non_camel_case_types)]
pub(crate) struct AS1130_L;
#[allow(non_camel_case_types)]
pub(crate) struct AS1130_R;

impl AS1130 for AS1130_L {
    const ADDR: u8 = 0x30;
}

impl AS1130 for AS1130_R {
    const ADDR: u8 = 0x37;
}
