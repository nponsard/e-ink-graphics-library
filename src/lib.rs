#![no_std]

pub enum TransparencySetting {
    None,
    BlackTransparent,
    WhiteTransparent,
}

#[cfg(feature = "ssd1680")]
pub mod ssd1680;
pub trait ErrorType {
    /// Error type
    type Error: core::fmt::Debug;
}

/// White : true, Black : false
/// In the buffer, one byte corresponds to 8 pixels on the x axis.
pub trait BWDisplay: ErrorType {
    fn set_pixel(&mut self, x: u16, y: u16, color: bool) -> Result<(), Self::Error>;
    fn fill(&mut self, color: bool) -> Result<(), Self::Error>;
    fn set_buffer(&mut self, buffer: &[u8]) -> Result<(), Self::Error>;
    fn draw_buffer(
        &mut self,
        buffer: &[u8],
        x: u16,
        y: u16,
        w: u16,
        h: u16,
    ) -> Result<(), Self::Error>;
    fn draw_buffer_with_transparency(
        &mut self,
        buffer: &[u8],
        x: u16,
        y: u16,
        w: u16,
        h: u16,
        transparency: TransparencySetting,
    ) -> Result<(), Self::Error>;
    fn refresh(&mut self, force_full: bool) -> Result<(), Self::Error>;
}
