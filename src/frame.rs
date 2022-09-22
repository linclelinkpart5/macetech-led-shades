// Frame dimensions, note that these are for a single AS1130 chip!
pub(crate) const FRAME_ROWS: usize = 8;
pub(crate) const FRAME_COLS: usize = 12;

pub(crate) type BitFrame = [u8; FRAME_COLS];
pub(crate) type BitBuffer = [u8; FRAME_COLS * 2 + 1];

pub(crate) type PwmFrame = [[u8; FRAME_ROWS]; FRAME_COLS];
pub(crate) type PwmBuffer = [u8; FRAME_COLS * (FRAME_ROWS + 1)];

pub(crate) struct FrameHelpers;

impl FrameHelpers {
    pub(crate) fn create_bit_buffer(bit_frame: &BitFrame) -> BitBuffer {
        let mut buffer = [0u8; FRAME_COLS * 2 + 1];

        let mut i = 0;

        buffer[i] = 0;
        i += 1;

        for bits in bit_frame.iter() {
            buffer[i] = *bits << 2;
            i += 1;
            buffer[i] = *bits >> 6;
            i += 1;
        }

        buffer
    }

    pub(crate) fn create_pwm_buffer(pwm_frame: &PwmFrame) -> PwmBuffer {
        let mut buffer = [0u8; FRAME_COLS * (FRAME_ROWS + 1)];

        let mut i = 0;

        for x in 0..FRAME_COLS {
            buffer[i] = 26 + (x as u8 * 11);
            i += 1;

            for y in 0..FRAME_ROWS {
                buffer[i] = pwm_frame[x][y];
                i += 1;
            }
        }

        buffer
    }
}
