//! System-related registers
//!
//! This module contains registers for system-level configuration including:
//! - Real-time clock (RTC) control
//! - Crystal oscillator trimming
//! - Event masking
//!
//! These registers control fundamental timing and system behavior of the device.
//! Proper configuration is essential for reliable operation.

use core::convert::Infallible;

use regiface::{register, FromByteArray, ReadableRegister, ToByteArray, WritableRegister};

/// RTC control register (address: 0x0902)
///
/// Controls the 64kHz real-time clock used for:
/// - Sleep mode wake-up timing
/// - RX duty cycling
/// - TX/RX timeout timing
///
/// # Important Notes
/// - RTC must be enabled for timed operations
/// - RTC is automatically enabled when needed by commands
/// - RTC should be stopped after implicit header timeout
/// - RTC uses the RC64k oscillator as time base
#[register(0x0902u16)]
#[derive(Debug, Clone, Copy, ReadableRegister, WritableRegister, Default)]
pub struct RtcControl {
    /// Enable RTC operation
    /// - true = RTC enabled
    /// - false = RTC disabled (default)
    pub enabled: bool,
}

/// XTA trim register (address: 0x0911)
///
/// Controls the crystal load capacitance on XTA pin.
/// The internal trimming capacitors eliminate the need for
/// external load capacitors on the crystal.
///
/// # Important Notes
/// - Default value is 0x05 (13.6pF) at POR/cold start
/// - Auto-set to 0x12 (19.7pF) when entering STDBY_XOSC
/// - Set to 0x2F (33.4pF) when using TCXO
/// - Must be in STDBY_XOSC to change value
/// - Changes before STDBY_XOSC will be overwritten
#[register(0x0911u16)]
#[derive(Debug, Clone, Copy, ReadableRegister, WritableRegister)]
pub struct XtaTrim {
    /// XTA pin capacitance trimming
    /// - Steps of 0.47pF
    /// - 0x00 = 11.3pF (minimum)
    /// - 0x2F = 33.4pF (maximum)
    /// - Default: 0x05 (13.6pF)
    /// - STDBY_XOSC: 0x12 (19.7pF)
    /// - TCXO mode: 0x2F (33.4pF)
    pub value: u8,
}

impl Default for XtaTrim {
    fn default() -> Self {
        Self { value: 0x05 }
    }
}

/// XTB trim register (address: 0x0912)
///
/// Controls the crystal load capacitance on XTB pin.
/// The internal trimming capacitors eliminate the need for
/// external load capacitors on the crystal.
///
/// # Important Notes
/// - Default value is 0x05 (13.6pF) at POR/cold start
/// - Auto-set to 0x12 (19.7pF) when entering STDBY_XOSC
/// - Must be in STDBY_XOSC to change value
/// - Changes before STDBY_XOSC will be overwritten
/// - When using TCXO, XTB should be left unconnected
#[register(0x0912u16)]
#[derive(Debug, Clone, Copy, ReadableRegister, WritableRegister)]
pub struct XtbTrim {
    /// XTB pin capacitance trimming
    /// - Steps of 0.47pF
    /// - 0x00 = 11.3pF (minimum)
    /// - 0x2F = 33.4pF (maximum)
    /// - Default: 0x05 (13.6pF)
    /// - STDBY_XOSC: 0x12 (19.7pF)
    pub value: u8,
}

impl Default for XtbTrim {
    fn default() -> Self {
        Self { value: 0x05 }
    }
}

/// Event mask register (address: 0x0944)
///
/// Controls which events generate interrupts.
/// Used in conjunction with the IRQ system to control
/// which events can trigger interrupts on DIO pins.
///
/// # Important Notes
/// - Used to clear timeout events in implicit header mode
/// - Part of workaround for implicit header timeout behavior
/// - Should be used carefully as it affects system responsiveness
#[register(0x0944u16)]
#[derive(Debug, Clone, Copy, ReadableRegister, WritableRegister, Default)]
pub struct EventMask {
    /// Event mask bits
    /// Each bit masks a specific event type:
    /// - 0 = Event masked (no interrupt)
    /// - 1 = Event enabled (generates interrupt)
    pub mask: u8,
}

impl FromByteArray for RtcControl {
    type Error = Infallible;
    type Array = [u8; 1];

    fn from_bytes(bytes: Self::Array) -> Result<Self, Self::Error> {
        Ok(Self {
            enabled: bytes[0] & 0x01 != 0,
        })
    }
}

impl ToByteArray for RtcControl {
    type Error = Infallible;
    type Array = [u8; 1];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok([self.enabled as u8])
    }
}

impl FromByteArray for XtaTrim {
    type Error = Infallible;
    type Array = [u8; 1];

    fn from_bytes(bytes: Self::Array) -> Result<Self, Self::Error> {
        Ok(Self {
            value: bytes[0] & 0x2F,
        })
    }
}

impl ToByteArray for XtaTrim {
    type Error = Infallible;
    type Array = [u8; 1];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok([self.value & 0x2F])
    }
}

impl FromByteArray for XtbTrim {
    type Error = Infallible;
    type Array = [u8; 1];

    fn from_bytes(bytes: Self::Array) -> Result<Self, Self::Error> {
        Ok(Self {
            value: bytes[0] & 0x2F,
        })
    }
}

impl ToByteArray for XtbTrim {
    type Error = Infallible;
    type Array = [u8; 1];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok([self.value & 0x2F])
    }
}

impl FromByteArray for EventMask {
    type Error = Infallible;
    type Array = [u8; 1];

    fn from_bytes(bytes: Self::Array) -> Result<Self, Self::Error> {
        Ok(Self { mask: bytes[0] })
    }
}

impl ToByteArray for EventMask {
    type Error = Infallible;
    type Array = [u8; 1];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok([self.mask])
    }
}
