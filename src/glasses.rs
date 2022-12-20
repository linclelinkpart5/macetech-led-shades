use arduino_hal::i2c::Error as I2cError;
use arduino_hal::I2c;

use crate::as1130::{
    ClockSpeed, ConfigFlags, InterruptMaskFlags, ShutdownTestFlags, SyncDir, AS1130, AS1130_L,
    AS1130_R,
};
use crate::frame::{
    BitFrame, FrameHelpers, PwmFrame, EMPTY_BIT_FRAME, EMPTY_PWM_FRAME, SOLID_BIT_FRAME,
    SOLID_PWM_FRAME,
};

pub(crate) type Result = core::result::Result<(), I2cError>;

const STARTING_BRIGHTNESS: u8 = 16u8;

pub(crate) struct Glasses {
    bit_frame_l: BitFrame,
    bit_frame_r: BitFrame,
    pwm_frame_l: PwmFrame,
    pwm_frame_r: PwmFrame,
    i2c: I2c,
    brightness: u8,
}

impl Glasses {
    pub(crate) fn new(i2c: I2c) -> Self {
        Self {
            bit_frame_l: EMPTY_BIT_FRAME,
            bit_frame_r: EMPTY_BIT_FRAME,
            pwm_frame_l: EMPTY_PWM_FRAME,
            pwm_frame_r: EMPTY_PWM_FRAME,
            i2c,
            brightness: STARTING_BRIGHTNESS,
        }
    }

    pub(crate) fn init(&mut self) -> Result {
        AS1130_L::set_shutdown_test(&mut self.i2c, ShutdownTestFlags::empty())?;
        AS1130_R::set_shutdown_test(&mut self.i2c, ShutdownTestFlags::empty())?;

        arduino_hal::delay_ms(5);

        let flags = ShutdownTestFlags::TEST_ALL
            | ShutdownTestFlags::AUTO_TEST
            | ShutdownTestFlags::INIT
            | ShutdownTestFlags::SHDN;
        AS1130_L::set_shutdown_test(&mut self.i2c, flags)?;
        AS1130_R::set_shutdown_test(&mut self.i2c, flags)?;

        arduino_hal::delay_ms(5);

        let flags = ConfigFlags::LED_ERROR_CORR;
        AS1130_L::set_configs(&mut self.i2c, flags, 0b1)?;
        AS1130_R::set_configs(&mut self.i2c, flags, 0b1)?;

        AS1130_L::set_movie(&mut self.i2c, 0, 0, false, false)?;
        AS1130_R::set_movie(&mut self.i2c, 0, 0, false, false)?;

        AS1130_L::set_movie_options(&mut self.i2c, false, false, false, 0)?;
        AS1130_R::set_movie_options(&mut self.i2c, false, false, false, 0)?;

        AS1130_L::set_movie_looping(&mut self.i2c, 1)?;
        AS1130_R::set_movie_looping(&mut self.i2c, 1)?;

        AS1130_L::set_brightness(&mut self.i2c, STARTING_BRIGHTNESS)?;
        AS1130_R::set_brightness(&mut self.i2c, STARTING_BRIGHTNESS)?;

        AS1130_L::set_interrupt_mask(&mut self.i2c, InterruptMaskFlags::empty())?;
        AS1130_R::set_interrupt_mask(&mut self.i2c, InterruptMaskFlags::empty())?;

        AS1130_L::set_interrupt_frame(&mut self.i2c, 0)?;
        AS1130_R::set_interrupt_frame(&mut self.i2c, 0)?;

        AS1130_L::set_i2c_watchdog(&mut self.i2c, 64, true)?;
        AS1130_R::set_i2c_watchdog(&mut self.i2c, 64, true)?;

        // NOTE: Different sync directions are intentional!
        AS1130_L::set_clock_sync(&mut self.i2c, ClockSpeed::Mhz1, SyncDir::Out)?;
        AS1130_R::set_clock_sync(&mut self.i2c, ClockSpeed::Mhz1, SyncDir::In)?;

        self.pwm_frame_l = SOLID_PWM_FRAME;
        self.pwm_frame_r = SOLID_PWM_FRAME;

        AS1130_L::write_pwm_frame(&mut self.i2c, 0, &self.pwm_frame_l)?;
        AS1130_R::write_pwm_frame(&mut self.i2c, 0, &self.pwm_frame_r)?;

        self.bit_frame_l = EMPTY_BIT_FRAME;
        self.bit_frame_r = EMPTY_BIT_FRAME;

        AS1130_L::write_bit_frame(&mut self.i2c, 0, &self.bit_frame_l)?;
        AS1130_R::write_bit_frame(&mut self.i2c, 0, &self.bit_frame_r)?;

        AS1130_L::write_blink_frame(&mut self.i2c, 0, &self.bit_frame_l)?;
        AS1130_R::write_blink_frame(&mut self.i2c, 0, &self.bit_frame_r)?;

        AS1130_L::set_frame(&mut self.i2c, 0, true)?;
        AS1130_R::set_frame(&mut self.i2c, 0, true)?;

        Ok(())
    }

    pub(crate) fn switch_draw_type(&mut self, frame_index: u8, draw_type: DrawType) -> Result {
        match draw_type {
            DrawType::Bit => {
                self.bit_frame_l = EMPTY_BIT_FRAME;
                self.bit_frame_r = EMPTY_BIT_FRAME;

                AS1130_L::write_bit_frame(&mut self.i2c, frame_index, &self.bit_frame_l)?;
                AS1130_R::write_bit_frame(&mut self.i2c, frame_index, &self.bit_frame_r)?;

                self.pwm_frame_l = SOLID_PWM_FRAME;
                self.pwm_frame_r = SOLID_PWM_FRAME;

                AS1130_L::write_pwm_frame(&mut self.i2c, 0, &self.pwm_frame_l)?;
                AS1130_R::write_pwm_frame(&mut self.i2c, 0, &self.pwm_frame_r)?;
            }
            DrawType::Pwm => {
                self.pwm_frame_l = EMPTY_PWM_FRAME;
                self.pwm_frame_r = EMPTY_PWM_FRAME;

                AS1130_L::write_pwm_frame(&mut self.i2c, 0, &self.pwm_frame_l)?;
                AS1130_R::write_pwm_frame(&mut self.i2c, 0, &self.pwm_frame_r)?;

                self.bit_frame_l = SOLID_BIT_FRAME;
                self.bit_frame_r = SOLID_BIT_FRAME;

                AS1130_L::write_bit_frame(&mut self.i2c, frame_index, &self.bit_frame_l)?;
                AS1130_R::write_bit_frame(&mut self.i2c, frame_index, &self.bit_frame_r)?;
            }
        };

        Ok(())
    }
}

pub(crate) enum DrawType {
    Pwm,
    Bit,
}
