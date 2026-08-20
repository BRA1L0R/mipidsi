use embedded_graphics_core::pixelcolor::Rgb565;
use embedded_hal::delay::DelayNs;

use crate::{
    dcs::{ExitSleepMode, InterfaceExt, SetAddressMode, SetDisplayOn, SetInvertMode},
    interface::{Interface, InterfaceKind},
    models::{Model, ModelInitError},
    options::ModelOptions,
    ConfigurationError,
};

#[derive(Clone, Copy)]
struct InitCommand {
    instruction: u8,
    parameters: &'static [u8],
}

impl InitCommand {
    const fn new(instruction: u8, parameters: &'static [u8]) -> Self {
        Self {
            instruction,
            parameters,
        }
    }
}

// Panel-specific power, source/gate timing and gamma settings for the 2.79 inch
// N279 family. These are intentionally separate from the standard NV3007
// sequence: the two panels use the same controller and resolution but require
// different private-register values.
const N279_INIT: &[InitCommand] = &[
    InitCommand::new(0xFF, &[0xA5]),
    InitCommand::new(0x9A, &[0x08]),
    InitCommand::new(0x9B, &[0x08]),
    InitCommand::new(0x9C, &[0xB0]),
    InitCommand::new(0x9D, &[0x16]),
    InitCommand::new(0x9E, &[0xC4]),
    InitCommand::new(0x8F, &[0x55, 0x04]),
    InitCommand::new(0x84, &[0x90]),
    InitCommand::new(0x83, &[0x7B]),
    InitCommand::new(0x85, &[0x33]),
    InitCommand::new(0x60, &[0x00]),
    InitCommand::new(0x70, &[0x00]),
    InitCommand::new(0x61, &[0x02]),
    InitCommand::new(0x71, &[0x02]),
    InitCommand::new(0x62, &[0x04]),
    InitCommand::new(0x72, &[0x04]),
    InitCommand::new(0x6C, &[0x29]),
    InitCommand::new(0x7C, &[0x29]),
    InitCommand::new(0x6D, &[0x31]),
    InitCommand::new(0x7D, &[0x31]),
    InitCommand::new(0x6E, &[0x0F]),
    InitCommand::new(0x7E, &[0x0F]),
    InitCommand::new(0x66, &[0x21]),
    InitCommand::new(0x76, &[0x21]),
    InitCommand::new(0x68, &[0x3A]),
    InitCommand::new(0x78, &[0x3A]),
    InitCommand::new(0x63, &[0x07]),
    InitCommand::new(0x73, &[0x07]),
    InitCommand::new(0x64, &[0x05]),
    InitCommand::new(0x74, &[0x05]),
    InitCommand::new(0x65, &[0x02]),
    InitCommand::new(0x75, &[0x02]),
    InitCommand::new(0x67, &[0x23]),
    InitCommand::new(0x77, &[0x23]),
    InitCommand::new(0x69, &[0x08]),
    InitCommand::new(0x79, &[0x08]),
    InitCommand::new(0x6A, &[0x13]),
    InitCommand::new(0x7A, &[0x13]),
    InitCommand::new(0x6B, &[0x13]),
    InitCommand::new(0x7B, &[0x13]),
    InitCommand::new(0x6F, &[0x00]),
    InitCommand::new(0x7F, &[0x00]),
    InitCommand::new(0x50, &[0x00]),
    InitCommand::new(0x52, &[0xD6]),
    InitCommand::new(0x53, &[0x08]),
    InitCommand::new(0x54, &[0x08]),
    InitCommand::new(0x55, &[0x1E]),
    InitCommand::new(0x56, &[0x1C]),
    InitCommand::new(0xA0, &[0x2B, 0x24, 0x00]),
    InitCommand::new(0xA1, &[0x87]),
    InitCommand::new(0xA2, &[0x86]),
    InitCommand::new(0xA5, &[0x00]),
    InitCommand::new(0xA6, &[0x00]),
    InitCommand::new(0xA7, &[0x00]),
    InitCommand::new(0xA8, &[0x36]),
    InitCommand::new(0xA9, &[0x7E]),
    InitCommand::new(0xAA, &[0x7E]),
    InitCommand::new(0xB9, &[0x85]),
    InitCommand::new(0xBA, &[0x84]),
    InitCommand::new(0xBB, &[0x83]),
    InitCommand::new(0xBC, &[0x82]),
    InitCommand::new(0xBD, &[0x81]),
    InitCommand::new(0xBE, &[0x80]),
    InitCommand::new(0xBF, &[0x01]),
    InitCommand::new(0xC0, &[0x02]),
    InitCommand::new(0xC1, &[0x00]),
    InitCommand::new(0xC2, &[0x00]),
    InitCommand::new(0xC3, &[0x00]),
    InitCommand::new(0xC4, &[0x33]),
    InitCommand::new(0xC5, &[0x7E]),
    InitCommand::new(0xC6, &[0x7E]),
    InitCommand::new(0xC8, &[0x33, 0x33]),
    InitCommand::new(0xC9, &[0x68]),
    InitCommand::new(0xCA, &[0x69]),
    InitCommand::new(0xCB, &[0x6A]),
    InitCommand::new(0xCC, &[0x6B]),
    InitCommand::new(0xCD, &[0x33, 0x33]),
    InitCommand::new(0xCE, &[0x6C]),
    InitCommand::new(0xCF, &[0x6D]),
    InitCommand::new(0xD0, &[0x6E]),
    InitCommand::new(0xD1, &[0x6F]),
    InitCommand::new(0xAB, &[0x03, 0x67]),
    InitCommand::new(0xAC, &[0x03, 0x6B]),
    InitCommand::new(0xAD, &[0x03, 0x68]),
    InitCommand::new(0xAE, &[0x03, 0x6C]),
    InitCommand::new(0xB3, &[0x00]),
    InitCommand::new(0xB4, &[0x00]),
    InitCommand::new(0xB5, &[0x00]),
    InitCommand::new(0xB6, &[0x32]),
    InitCommand::new(0xB7, &[0x7E]),
    InitCommand::new(0xB8, &[0x7E]),
    InitCommand::new(0xE0, &[0x00]),
    InitCommand::new(0xE1, &[0x03, 0x0F]),
    InitCommand::new(0xE2, &[0x04]),
    InitCommand::new(0xE3, &[0x01]),
    InitCommand::new(0xE4, &[0x0E]),
    InitCommand::new(0xE5, &[0x01]),
    InitCommand::new(0xE6, &[0x19]),
    InitCommand::new(0xE7, &[0x10]),
    InitCommand::new(0xE8, &[0x10]),
    InitCommand::new(0xEA, &[0x12]),
    InitCommand::new(0xEB, &[0xD0]),
    InitCommand::new(0xEC, &[0x04]),
    InitCommand::new(0xED, &[0x07]),
    InitCommand::new(0xEE, &[0x07]),
    InitCommand::new(0xEF, &[0x09]),
    InitCommand::new(0xF0, &[0xD0]),
    InitCommand::new(0xF1, &[0x0E]),
    InitCommand::new(0xF9, &[0x17]),
    InitCommand::new(0xF2, &[0x2C, 0x1B, 0x0B, 0x20]),
    InitCommand::new(0xE9, &[0x29]),
    InitCommand::new(0xEC, &[0x04]),
    InitCommand::new(0x35, &[0x00]),
    InitCommand::new(0x44, &[0x00, 0x10]),
    InitCommand::new(0x46, &[0x10]),
    InitCommand::new(0xFF, &[0x00]),
];

/// NV3007 N279 2.79 inch display in Rgb565 color mode.
///
/// The common N279 panels have a 142x428 active area inside the controller's
/// 168x428 framebuffer. Configure the builder with a 142x428 display size and
/// the panel-specific horizontal offset (commonly 12 pixels).
pub struct NV3007N279;

impl Model for NV3007N279 {
    type ColorFormat = Rgb565;

    const FRAMEBUFFER_SIZE: (u16, u16) = (168, 428);
    const RESET_DURATION: u32 = 120_000;

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
        if !matches!(DI::KIND, InterfaceKind::Serial4Line) {
            return Err(ModelInitError::InvalidConfiguration(
                ConfigurationError::UnsupportedInterface,
            ));
        }

        // Allow the panel to settle after reset (or the software reset issued
        // by Builder when no reset pin is configured).
        delay.delay_ms(120);

        for command in N279_INIT {
            di.write_raw(command.instruction, command.parameters)?;
        }

        let madctl = SetAddressMode::from(options);
        di.write_command(madctl)?;
        di.write_command(SetInvertMode::new(options.invert_colors))?;

        // PFSET on the NV3007 only defines DBI[2:0]. 0b101 selects the
        // two-byte RGB565 transfer format used by this model.
        di.write_raw(0x3A, &[0x05])?;

        di.write_command(ExitSleepMode)?;
        delay.delay_ms(120);
        di.write_command(SetDisplayOn)?;

        Ok(madctl)
    }
}
