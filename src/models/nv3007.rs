use embedded_graphics_core::pixelcolor::Rgb565;
use embedded_hal::delay::DelayNs;

use crate::{
    ConfigurationError, dcs::{
        BitsPerPixel, ExitSleepMode, InterfaceExt, PixelFormat, SetAddressMode, SetDisplayOn, SetPixelFormat
    }, interface::{Interface, InterfaceKind}, models::{Model, ModelInitError}, options::ModelOptions
};

/// ST7796 display in Rgb565 color mode.
pub struct NV3007;

impl Model for NV3007 {
    type ColorFormat = Rgb565;
    const FRAMEBUFFER_SIZE: (u16, u16) = (168, 428);

    fn init<DELAY, DI>(
        &mut self,
        di: &mut DI,
        delay: &mut DELAY,
        options: &ModelOptions,
    ) -> Result<SetAddressMode, ModelInitError<DI::Error>>
    where
        DELAY: DelayNs,
        DI: Interface,
    {
        if !matches!(
            DI::KIND,
            InterfaceKind::Serial4Line | InterfaceKind::Parallel8Bit | InterfaceKind::Parallel16Bit
        ) {
            return Err(ModelInitError::InvalidConfiguration(
                ConfigurationError::UnsupportedInterface,
            ));
        }

        let madctl = SetAddressMode::from(options);

        delay.delay_ms(200);

        di.write_raw(0xFF, &[0xA5])?; // Enable private registers
        di.write_command(ExitSleepMode)?;
        delay.delay_ms(200);

        //
        //// Block below based on Arduino_GFX and other sources
        //

        // Power and timing settings
        di.write_raw(0x9A, &[0x08])?;
        di.write_raw(0x9B, &[0x08])?;
        di.write_raw(0x9C, &[0xB0])?;
        di.write_raw(0x9D, &[0x17])?;
        di.write_raw(0x9E, &[0xC2])?;
        di.write_raw(0x8F, &[0x22, 0x04])?;
        di.write_raw(0x84, &[0x90])?;
        di.write_raw(0x83, &[0x7B])?;
        di.write_raw(0x85, &[0x4F])?;

        // Gamma settings
        di.write_raw(0x6E, &[0x0F])?;
        di.write_raw(0x7E, &[0x0F])?;
        di.write_raw(0x60, &[0x00])?;
        di.write_raw(0x70, &[0x00])?;
        di.write_raw(0x6D, &[0x39])?;
        di.write_raw(0x7D, &[0x31])?;
        di.write_raw(0x61, &[0x0A])?;
        di.write_raw(0x71, &[0x0A])?;
        di.write_raw(0x6C, &[0x35])?;
        di.write_raw(0x7C, &[0x29])?;
        di.write_raw(0x62, &[0x0F])?;
        di.write_raw(0x72, &[0x0F])?;
        di.write_raw(0x68, &[0x4F])?;
        di.write_raw(0x78, &[0x45])?;
        di.write_raw(0x66, &[0x33])?;
        di.write_raw(0x76, &[0x33])?;
        di.write_raw(0x6B, &[0x14])?;
        di.write_raw(0x7B, &[0x14])?;
        di.write_raw(0x63, &[0x09])?;
        di.write_raw(0x73, &[0x09])?;
        di.write_raw(0x6A, &[0x13])?;
        di.write_raw(0x7A, &[0x16])?;
        di.write_raw(0x64, &[0x08])?;
        di.write_raw(0x74, &[0x08])?;
        di.write_raw(0x69, &[0x07])?;
        di.write_raw(0x79, &[0x0D])?;
        di.write_raw(0x65, &[0x05])?;
        di.write_raw(0x75, &[0x05])?;
        di.write_raw(0x67, &[0x33])?;
        di.write_raw(0x77, &[0x33])?;
        di.write_raw(0x6F, &[0x00])?;
        di.write_raw(0x7F, &[0x00])?;

        // Additional configuration
        di.write_raw(0x50, &[0x00])?;
        di.write_raw(0x52, &[0xD6])?;
        di.write_raw(0x53, &[0x04])?;
        di.write_raw(0x54, &[0x04])?;
        di.write_raw(0x55, &[0x1B])?;
        di.write_raw(0x56, &[0x1B])?;

        // More configuration registers
        di.write_raw(0xA0, &[0x2A, 0x24, 0x00])?;
        di.write_raw(0xA1, &[0x84])?;
        di.write_raw(0xA2, &[0x85])?;
        di.write_raw(0xA8, &[0x34])?;
        di.write_raw(0xA9, &[0x80])?;
        di.write_raw(0xAA, &[0x73])?;

        // Vendor specific display control
        di.write_raw(0xAB, &[0x03, 0x61])?;
        di.write_raw(0xAC, &[0x03, 0x65])?;
        di.write_raw(0xAD, &[0x03, 0x60])?;
        di.write_raw(0xAE, &[0x03, 0x64])?;

        // Vendor specific display control K-Q
        di.write_raw(0xB9, &[0x82])?;
        di.write_raw(0xBA, &[0x83])?;
        di.write_raw(0xBB, &[0x80])?;
        di.write_raw(0xBC, &[0x81])?;
        di.write_raw(0xBD, &[0x02])?;
        di.write_raw(0xBE, &[0x01])?;
        di.write_raw(0xBF, &[0x04])?;

        // Power control
        di.write_raw(0xC0, &[0x03])?;
        di.write_raw(0xC4, &[0x33])?;
        di.write_raw(0xC5, &[0x80])?;
        di.write_raw(0xC6, &[0x73])?;
        di.write_raw(0xC7, &[0x00])?;
        di.write_raw(0xC8, &[0x33, 0x33])?;
        di.write_raw(0xC9, &[0x5B])?;
        di.write_raw(0xCA, &[0x5A])?;
        di.write_raw(0xCB, &[0x5D])?;
        di.write_raw(0xCC, &[0x5C])?;
        di.write_raw(0xCD, &[0x33, 0x33])?;
        di.write_raw(0xCE, &[0x5F])?;
        di.write_raw(0xCF, &[0x5E])?;
        di.write_raw(0xD0, &[0x61])?;
        di.write_raw(0xD1, &[0x60])?;

        // Pixel format / display control
        di.write_raw(0xB0, &[0x3A, 0x3A, 0x00, 0x00])?;
        di.write_raw(0xB6, &[0x32])?;
        di.write_raw(0xB7, &[0x80])?;
        di.write_raw(0xB8, &[0x73])?;

        // Gamma / color correction
        di.write_raw(0xE0, &[0x00])?;
        di.write_raw(0xE1, &[0x03, 0x0F])?;
        di.write_raw(0xE2, &[0x04])?;
        di.write_raw(0xE3, &[0x01])?;
        di.write_raw(0xE4, &[0x0E])?;
        di.write_raw(0xE5, &[0x01])?;
        di.write_raw(0xE6, &[0x19])?;
        di.write_raw(0xE7, &[0x10])?;
        di.write_raw(0xE8, &[0x10])?;
        di.write_raw(0xE9, &[0x21])?;
        di.write_raw(0xEA, &[0x12])?;
        di.write_raw(0xEB, &[0xD0])?;
        di.write_raw(0xEC, &[0x04])?;
        di.write_raw(0xED, &[0x07])?;
        di.write_raw(0xEE, &[0x07])?;
        di.write_raw(0xEF, &[0x09])?;
        di.write_raw(0xF0, &[0xD0])?;
        di.write_raw(0xF1, &[0x0E])?;
        di.write_raw(0xF9, &[0x56])?;
        di.write_raw(0xF2, &[0x26, 0x1B, 0x0B, 0x20])?;
        di.write_raw(0xEC, &[0x04])?;  // repeated per original
        

        di.write_raw(0xFF, &[0x00])?; // Disable private registers

        // Tearing effect and other settings
        di.write_raw(0x35, &[0x00])?;
        di.write_raw(0x44, &[0x00, 0x10])?;
        di.write_raw(0x46, &[0x10])?;

        //
        //// END INITIALIZATION BLOCK
        // 

        // Set orientation, etc.
        di.write_command(madctl)?;

        // Pixel format
        let pf = PixelFormat::with_all(BitsPerPixel::from_rgb_color::<Self::ColorFormat>());
        di.write_command(SetPixelFormat::new(pf))?;

        di.write_command(ExitSleepMode)?;
        delay.delay_ms(200);

        di.write_command(SetDisplayOn)?;

        Ok(madctl)
    }
}
