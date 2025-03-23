use core::fmt::Debug;

use embedded_hal::{
    delay::DelayNs,
    digital::{InputPin, OutputPin},
    spi::SpiDevice,
};
use ssd1680_rs::{self, SSD1680, error::Error};

use super::{BWDisplay, ErrorType, TransparencySetting};

const FRAME_BUFFER_SIZE: usize = 176 * 296;

pub struct Ssd1680Display<
    RST: OutputPin,
    DC: OutputPin,
    BUSY: InputPin,
    DELAY: DelayNs,
    SPI: SpiDevice,
> {
    driver: SSD1680<RST, DC, BUSY, DELAY, SPI>,
    // maximum possible frame buffer size for SSD1680
    frame_buffer: [u8; FRAME_BUFFER_SIZE],
    width: u16,
    height: u16,
    refresh_count: u8,
}

impl<RST: OutputPin, DC: OutputPin, BUSY: InputPin, DELAY: DelayNs, SPI: SpiDevice>
    Ssd1680Display<RST, DC, BUSY, DELAY, SPI>
{
    pub fn new(
        rst: RST,
        dc: DC,
        busy: BUSY,
        delay: DELAY,
        spi: SPI,
        config: ssd1680_rs::config::DisplayConfig,
    ) -> Self {
        let driver = SSD1680::new(rst, dc, busy, delay, spi, config);
        Ssd1680Display {
            driver,
            frame_buffer: [0; 176 * 296],
            width: config.width,
            height: config.height,
            refresh_count: 10,
        }
    }
}
impl<RST: OutputPin, DC: OutputPin, BUSY: InputPin, DELAY: DelayNs, SPI: SpiDevice> ErrorType
    for Ssd1680Display<RST, DC, BUSY, DELAY, SPI>
{
    type Error = ssd1680_rs::error::Error<SPI::Error, RST::Error, DC::Error, BUSY::Error>;
}

impl<
    RST: OutputPin,
    DC: OutputPin,
    BUSY: InputPin,
    DELAY: DelayNs,
    SPI: SpiDevice,
    S: Debug,
    R: Debug,
    D: Debug,
    B: Debug,
> BWDisplay for Ssd1680Display<RST, DC, BUSY, DELAY, SPI>
where
    SPI: SpiDevice<Error = S>,
    RST: OutputPin<Error = R>,
    DC: OutputPin<Error = D>,
    BUSY: InputPin<Error = B>,
{
    fn set_pixel(&mut self, x: u16, y: u16, color: bool) -> Result<(), Error<S, R, D, B>> {
        let address = get_address(x, y, self.width);
        self.frame_buffer[address.buffer_position] = (self.frame_buffer[address.buffer_position]
            & !(1 << address.byte_offset))
            | ((color as u8) << address.byte_offset);
        Ok(())
    }

    fn fill(&mut self, color: bool) -> Result<(), Error<S, R, D, B>> {
        self.frame_buffer = [(color as u8) * 255; FRAME_BUFFER_SIZE];
        Ok(())
    }

    fn set_buffer(&mut self, buffer: &[u8]) -> Result<(), Error<S, R, D, B>> {
        self.frame_buffer.copy_from_slice(buffer);
        Ok(())
    }

    fn draw_buffer(
        &mut self,
        buffer: &[u8],
        x: u16,
        y: u16,
        w: u16,
        h: u16,
    ) -> Result<(), Error<S, R, D, B>> {
        self.draw_buffer_with_transparency(buffer, x, y, w, h, TransparencySetting::None)
    }

    fn draw_buffer_with_transparency(
        &mut self,
        buffer: &[u8],
        x: u16,
        y: u16,
        w: u16,
        h: u16,
        transparency: TransparencySetting,
    ) -> Result<(), Error<S, R, D, B>> {
        for j in 0..h {
            for i in 0..(w / 8) + 1 {
                let address_frambuffer_byte = get_address(x + i * 8, y + j, self.width);
                let address_buffer_byte = get_address(i * 8, j, w);

                let mut framebuffer_byte =
                    self.frame_buffer[address_frambuffer_byte.buffer_position];
                let offset = address_frambuffer_byte.byte_offset;
                if i != 0 && offset != 0 {
                    let previous_byte = buffer[address_buffer_byte.buffer_position - 1];

                    match transparency {
                        TransparencySetting::None => {
                            framebuffer_byte &= 0xff_u8.checked_shr(offset.into()).unwrap_or(0);
                            framebuffer_byte |= previous_byte << (8 - offset);
                        }
                        _ => {
                            unimplemented!()
                        }
                    }
                }
                if i < (w / 8) {
                    let current_byte = buffer[address_buffer_byte.buffer_position];

                    match transparency {
                        TransparencySetting::None => {
                            framebuffer_byte &=
                                0xff_u8.checked_shl((8 - offset).into()).unwrap_or(0);
                            framebuffer_byte |= current_byte >> offset;
                        }
                        _ => {
                            unimplemented!()
                        }
                    }
                }

                self.frame_buffer[address_frambuffer_byte.buffer_position] = framebuffer_byte;
            }
        }

        Ok(())
    }

    fn refresh(&mut self, force_full: bool) -> Result<(), Error<S, R, D, B>> {
        self.driver.hw_init()?;
        self.driver
            .write_bw_bytes(&self.frame_buffer[0..(self.height * self.width / 8) as usize])?;
        if self.refresh_count >= 5 || force_full {
            self.driver.full_refresh()?;
            self.refresh_count = 0;
        } else {
            self.driver.partial_refresh()?;
            self.refresh_count += 1;
        }
        self.driver.enter_deep_sleep()
    }
}

struct Address {
    pub buffer_position: usize,
    pub byte_offset: u8,
}

fn get_address(x: u16, y: u16, width: u16) -> Address {
    let frambuffer_position = (x / 8 + y * width / 8) as usize;
    let byte_offset = (x % 8) as u8;
    Address {
        buffer_position: frambuffer_position,
        byte_offset,
    }
}
