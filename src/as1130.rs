use arduino_hal::i2c::Error as I2cError;
use arduino_hal::I2c;
use embedded_hal::blocking::i2c::Write;

// Entry code for changing memory registers.
const REGISTER_SELECT: u8 = 0xFD;

// Memory register addresses.
const MEMORY_ON_OFF_START: u8 = 0x01;
const MEMORY_BLINK_PWM_START: u8 = 0x40;
const MEMORY_DOT_CORRECTION: u8 = 0x80;
const MEMORY_CONTROL_REGISTERS: u8 = 0xC0;

// Control register addresses.
const CONTROL_PICTURE: u8 = 0x00;
const CONTROL_MOVIE: u8 = 0x01;
const CONTROL_MOVIE_MODE: u8 = 0x02;
const CONTROL_FRAME_TIME: u8 = 0x03;
const CONTROL_DISPLAY_OPTION: u8 = 0x04;
const CONTROL_CURRENT_SOURCE: u8 = 0x05;
const CONTROL_AS1130_CONFIG: u8 = 0x06;
const CONTROL_INTERRUPT_MASK: u8 = 0x07;
const CONTROL_INTERRUPT_FRAME_DEF: u8 = 0x08;
const CONTROL_SHUTDOWN_OPEN_SHORT: u8 = 0x09;
const CONTROL_I2C_INTERFACE_MON: u8 = 0x0A;
const CONTROL_CLK_SYNC: u8 = 0x0B;
const CONTROL_INTERRUPT_STATUS: u8 = 0x0E;
const CONTROL_AS1130_STATUS: u8 = 0x0F;
const CONTROL_OPEN_LED_BEGIN: u8 = 0x20;

trait AS1130 {
    const ADDR: u8;

    /// Send a value to a device's register.
    fn write_register(i2c: &mut I2c, target_register: u8, value: u8) -> Result<(), I2cError> {
        i2c.write(Self::ADDR, &[target_register, value])
    }

    /// Select control memory area for subsequent writes.
    fn select_control_memory(i2c: &mut I2c) -> Result<(), I2cError> {
        Self::write_register(i2c, REGISTER_SELECT, MEMORY_CONTROL_REGISTERS)
    }

    /// Select memory frame to display.
    fn set_frame(i2c: &mut I2c, frame_index: u8, enable: bool) -> Result<(), I2cError> {
        Self::select_control_memory(i2c)?;

        Self::write_register(
            i2c,
            CONTROL_PICTURE,
            ((enable as u8) << 6) | (frame_index & 0b11111),
        )?;

        Ok(())
    }

    /// Set movie display options.
    fn set_movie(
        i2c: &mut I2c,
        start_frame_index: u8,
        frame_count: u8,
        loop_frame: bool,
        enable: bool,
    ) -> Result<(), I2cError> {
        Self::select_control_memory(i2c)?;

        let value = ((enable as u8) << 6) | (start_frame_index & 0b11111);
        Self::write_register(i2c, CONTROL_MOVIE, value)?;

        let value = (1 << 7) | ((loop_frame as u8) << 6) | (frame_count & 0b11111);
        Self::write_register(i2c, CONTROL_MOVIE_MODE, value)?;

        Ok(())
    }

    /// Set movie frame speed and scrolling options.
    fn set_movie_options(
        i2c: &mut I2c,
        fading: bool,
        scroll_dir: bool,
        enable_scroll: bool,
        frame_delay: u8,
    ) -> Result<(), I2cError> {
        Self::select_control_memory(i2c)?;

        let value = ((fading as u8) << 7)
            | ((scroll_dir as u8) << 6)
            | ((enable_scroll as u8) << 4)
            | (frame_delay & 0b1111);
        Self::write_register(i2c, CONTROL_FRAME_TIME, value)?;

        Ok(())
    }

    /// Configure movie looping options.
    fn set_movie_looping(i2c: &mut I2c, num_loops: u8) -> Result<(), I2cError> {
        Self::select_control_memory(i2c)?;

        let value = ((num_loops & 0b111) << 5) | 0b1011;
        Self::write_register(i2c, CONTROL_DISPLAY_OPTION, value)?;

        Ok(())
    }

    /// Set brightness level for device.
    fn set_brightness(i2c: &mut I2c, brightness: u8) -> Result<(), I2cError> {
        Self::select_control_memory(i2c)?;
        Self::write_register(i2c, CONTROL_CURRENT_SOURCE, brightness)?;

        Ok(())
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
