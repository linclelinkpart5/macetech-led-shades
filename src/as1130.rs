use arduino_hal::i2c::Error as I2cError;
use arduino_hal::I2c;
use bitflags::bitflags;
use embedded_hal::blocking::i2c::Write;

use crate::frame::{BitFrame, FrameHelpers, PwmFrame};

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

    fn write(i2c: &mut I2c, bytes: &[u8]) -> Result<(), I2cError> {
        i2c.write(Self::ADDR, bytes)
    }

    /// Send a value to a device's register.
    fn write_register(i2c: &mut I2c, target_register: u8, value: u8) -> Result<(), I2cError> {
        Self::write(i2c, &[target_register, value])
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

    /// Set configuration options.
    // #define AS1130_low_vdd_rst 7
    // #define AS1130_low_vdd_stat 6
    // #define AS1130_led_error_correction 5
    // #define AS1130_dot_corr 4
    // #define AS1130_common_addr 3
    fn set_configs(i2c: &mut I2c, options: u8, mem_config: u8) -> Result<(), I2cError> {
        Self::select_control_memory(i2c)?;

        let value = options | (mem_config & 0b111);
        Self::write_register(i2c, CONTROL_AS1130_CONFIG, value)?;

        Ok(())
    }

    /// Configure interrupt mask.
    // #define AS1130_selected_pic 7
    // #define AS1130_watchdog 6
    // #define AS1130_por 5
    // #define AS1130_overtemp 4
    // #define AS1130_low_vdd 3
    // #define AS1130_open_err 2
    // #define AS1130_short_err 1
    // #define AS1130_movie_fin 0
    fn set_interrupt_mask(i2c: &mut I2c, options: u8) -> Result<(), I2cError> {
        Self::select_control_memory(i2c)?;
        Self::write_register(i2c, CONTROL_INTERRUPT_MASK, options)?;

        Ok(())
    }

    /// Select movie frame to generate interrupt.
    fn set_interrupt_frame(i2c: &mut I2c, frame: u8) -> Result<(), I2cError> {
        Self::select_control_memory(i2c)?;

        let value = frame & 0b11111;
        Self::write_register(i2c, CONTROL_INTERRUPT_FRAME_DEF, value)?;

        Ok(())
    }

    /// Configure test/shutdown register.
    // #define AS1130_test_all 4
    // #define AS1130_auto_test 3
    // #define AS1130_manual_test 2
    // #define AS1130_init 1
    // #define AS1130_shdn 0
    fn set_shutdown_test(i2c: &mut I2c, options: u8) -> Result<(), I2cError> {
        Self::select_control_memory(i2c)?;

        let value = options & 0b11111;
        Self::write_register(i2c, CONTROL_SHUTDOWN_OPEN_SHORT, value)?;

        Ok(())
    }

    /// Configure I2C watchdog.
    fn set_i2c_watchdog(i2c: &mut I2c, timeout: u8, enable: bool) -> Result<(), I2cError> {
        Self::select_control_memory(i2c)?;

        let value = ((timeout & 0b11111) << 1) | (enable as u8);
        Self::write_register(i2c, CONTROL_I2C_INTERFACE_MON, value)?;

        Ok(())
    }

    /// Configure clock sync.
    // #define AS1130_clock_speed_1MHz 0b00
    // #define AS1130_clock_speed_500kHz 0b01
    // #define AS1130_clock_speed_125kHz 0b10
    // #define AS1130_clock_speed_32kHz 0b11
    // #define AS1130_sync_OUT 0b10
    // #define AS1130_sync_IN 0b01
    fn set_clock_sync(i2c: &mut I2c, clock_speed: u8, sync_dir: u8) -> Result<(), I2cError> {
        Self::select_control_memory(i2c)?;

        let value = ((clock_speed & 0b11) << 2) | (sync_dir & 0b11);
        Self::write_register(i2c, CONTROL_CLK_SYNC, value)?;

        Ok(())
    }

    /// Send contents of bit frame to correct AS1130 registers.
    fn write_bit_frame(
        i2c: &mut I2c,
        frame_index: u8,
        bit_frame: &BitFrame,
    ) -> Result<(), I2cError> {
        Self::write_register(i2c, REGISTER_SELECT, frame_index + MEMORY_ON_OFF_START)?;

        let buffer = FrameHelpers::create_bit_buffer(bit_frame);

        Self::write(i2c, &buffer)?;

        Ok(())
    }

    /// Send contents of PWM frame to correct AS1130 registers.
    fn write_pwm_frame(
        i2c: &mut I2c,
        frame_index: u8,
        pwm_frame: &PwmFrame,
    ) -> Result<(), I2cError> {
        Self::write_register(i2c, REGISTER_SELECT, frame_index + MEMORY_BLINK_PWM_START)?;

        let buffer = FrameHelpers::create_pwm_buffer(pwm_frame);

        Self::write(i2c, &buffer)?;

        Ok(())
    }

    /// Send contents of bit frame to blink AS1130 registers.
    /// Mostly used to clear blink frames, since blink is useless.
    fn write_blink_frame(
        i2c: &mut I2c,
        frame_index: u8,
        bit_frame: &BitFrame,
    ) -> Result<(), I2cError> {
        Self::write_register(i2c, REGISTER_SELECT, frame_index + MEMORY_BLINK_PWM_START)?;

        let buffer = FrameHelpers::create_bit_buffer(bit_frame);

        Self::write(i2c, &buffer)?;

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

bitflags! {
    // #define AS1130_low_vdd_rst 7
    // #define AS1130_low_vdd_stat 6
    // #define AS1130_led_error_correction 5
    // #define AS1130_dot_corr 4
    // #define AS1130_common_addr 3
    struct ConfigFlags: u8 {
        const LOW_VDD_RST = 0b1 << 7;
        const LOW_VDD_STAT = 0b1 << 6;
        const LED_ERROR_CORR = 0b1 << 5;
        const DOT_CORR = 0b1 << 4;
        const COMMON_ADDR = 0b1 << 3;
        // const MEM_CONFIG_2 = 0b1 << 2;
        // const MEM_CONFIG_1 = 0b1 << 1;
        // const MEM_CONFIG_0 = 0b1 << 0;
    }
}

impl ConfigFlags {
    pub(crate) fn with_mem_config(self, mem_config: u8) -> Self {
        const MASK: u8 = 0b111;

        let bits =
            // Reset memory config bits to 0.
            (self.bits & !MASK)
            // Set memory config bits to desired values.
            | (mem_config & MASK);

        unsafe { Self::from_bits_unchecked(bits) }
    }
}

bitflags! {
    // #define AS1130_selected_pic 7
    // #define AS1130_watchdog 6
    // #define AS1130_por 5
    // #define AS1130_overtemp 4
    // #define AS1130_low_vdd 3
    // #define AS1130_open_err 2
    // #define AS1130_short_err 1
    // #define AS1130_movie_fin 0
    struct InterruptMaskFlags: u8 {
        const SELECTED_PIC = 0b1 << 7;
        const WATCHDOG = 0b1 << 6;
        const POR = 0b1 << 5;
        const OVERTEMP = 0b1 << 4;
        const LOW_VDD = 0b1 << 3;
        const OPEN_ERR = 0b1 << 2;
        const SHORT_ERR = 0b1 << 1;
        const MOVIE_FIN = 0b1 << 0;
    }
}

bitflags! {
    // #define AS1130_test_all 4
    // #define AS1130_auto_test 3
    // #define AS1130_manual_test 2
    // #define AS1130_init 1
    // #define AS1130_shdn 0
    struct ShutdownTestFlags: u8 {
        const TEST_ALL = 0b1 << 4;
        const AUTO_TEST = 0b1 << 3;
        const MANUAL_TEST = 0b1 << 2;
        const INIT = 0b1 << 1;
        const SHDN = 0b1 << 0;
    }
}

#[repr(u8)]
#[derive(Copy, Clone)]
enum ClockSpeed {
    Mhz1 = 0b00,
    Khz500 = 0b01,
    Khz125 = 0b10,
    Khz32 = 0b11,
}

#[repr(u8)]
#[derive(Copy, Clone)]
enum SyncDir {
    In = 0b01,
    Out = 0b10,
}
