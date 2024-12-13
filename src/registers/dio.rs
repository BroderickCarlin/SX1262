//! Digital IO related registers
//!
//! This module contains registers for configuring the digital I/O pins (DIOs).
//! The SX126x has 3 configurable DIO pins that can be used for:
//! - Generic interrupt signaling
//! - RF switch control (DIO2)
//! - TCXO power control (DIO3)
//!
//! Each DIO can be configured as:
//! - Input/Output
//! - With pull-up/pull-down resistors
//! - Mapped to different interrupt sources
//!
//! # Pin States in Different Modes
//! - Reset: All DIOs pulled down
//! - Sleep: DIOs high impedance with weak pull-down
//! - Active modes: DIOs configured as per registers

use core::convert::Infallible;

use regiface::{register, FromByteArray, ReadableRegister, ToByteArray, WritableRegister};

/// DIO output enable register (address: 0x0580)
///
/// Controls which DIOs are configured as outputs. When configured as
/// outputs, DIOs can signal interrupts or control external components.
///
/// # Important Notes
/// - DIO2 output configuration is ignored when used for RF switch control
/// - DIO3 output configuration is ignored when used for TCXO control
/// - DIOs are automatically configured when mapped to interrupts
#[register(0x0580u16)]
#[derive(Debug, Clone, Copy, ReadableRegister, WritableRegister)]
pub struct DioOutputEnable {
    /// Enable DIO1 as output
    /// DIO1 is typically used as the primary interrupt line
    pub dio1: bool,

    /// Enable DIO2 as output
    /// DIO2 can be used for RF switch control or interrupts
    pub dio2: bool,

    /// Enable DIO3 as output
    /// DIO3 can be used for TCXO control or interrupts
    pub dio3: bool,
}

/// DIO input enable register (address: 0x0583)
///
/// Controls which DIOs are configured as inputs. Input configuration
/// is typically not needed when DIOs are used for interrupts or
/// control functions.
///
/// # Important Notes
/// - Input configuration is overridden when DIO is used for RF switch or TCXO
/// - Input state can be read even when configured as output
#[register(0x0583u16)]
#[derive(Debug, Clone, Copy, ReadableRegister, WritableRegister)]
pub struct DioInputEnable {
    /// Enable DIO1 as input
    pub dio1: bool,

    /// Enable DIO2 as input
    pub dio2: bool,

    /// Enable DIO3 as input
    pub dio3: bool,
}

/// DIO pull-up control register (address: 0x0584)
///
/// Controls internal pull-up resistors on DIOs. Pull-ups are typically
/// used when DIO is configured as input to provide a defined logic level
/// when external circuit is high impedance.
///
/// # Important Notes
/// - Pull-up and pull-down should not be enabled simultaneously
/// - Pull-up is ~50kΩ at typical conditions
/// - Pull-up configuration ignored when pin used for RF switch/TCXO
#[register(0x0584u16)]
#[derive(Debug, Clone, Copy, ReadableRegister, WritableRegister)]
pub struct DioPullUpControl {
    /// Enable pull-up on DIO1
    pub dio1: bool,

    /// Enable pull-up on DIO2
    pub dio2: bool,

    /// Enable pull-up on DIO3
    pub dio3: bool,
}

/// DIO pull-down control register (address: 0x0585)
///
/// Controls internal pull-down resistors on DIOs. Pull-downs are typically
/// used when DIO is configured as input to provide a defined logic level
/// when external circuit is high impedance.
///
/// # Important Notes
/// - Pull-up and pull-down should not be enabled simultaneously
/// - Pull-down is ~50kΩ at typical conditions
/// - Pull-down configuration ignored when pin used for RF switch/TCXO
#[register(0x0585u16)]
#[derive(Debug, Clone, Copy, ReadableRegister, WritableRegister)]
pub struct DioPullDownControl {
    /// Enable pull-down on DIO1
    pub dio1: bool,

    /// Enable pull-down on DIO2
    pub dio2: bool,

    /// Enable pull-down on DIO3
    pub dio3: bool,
}

/// DIO3 output voltage control register (address: 0x0920)
///
/// Controls the regulated voltage output on DIO3 when used for TCXO control.
/// The voltage regulator can supply up to 4mA for powering an external TCXO.
///
/// # Important Notes
/// - VBAT must be at least 200mV higher than selected voltage
/// - Voltage regulator has typical 70μA quiescent current
/// - Takes up to 100μs to reach regulated voltage
/// - Used in conjunction with SetDIO3AsTCXOCtrl command
#[register(0x0920u16)]
#[derive(Debug, Clone, Copy, ReadableRegister, WritableRegister)]
pub struct Dio3OutputVoltage {
    /// TCXO supply voltage selection
    /// - 0x00 = 1.6V (min VBAT = 1.8V)
    /// - 0x01 = 1.7V (min VBAT = 1.9V)
    /// - 0x02 = 1.8V (min VBAT = 2.0V)
    /// - 0x03 = 2.2V (min VBAT = 2.4V)
    /// - 0x04 = 2.4V (min VBAT = 2.6V)
    /// - 0x05 = 2.7V (min VBAT = 2.9V)
    /// - 0x06 = 3.0V (min VBAT = 3.2V)
    /// - 0x07 = 3.3V (min VBAT = 3.5V)
    pub voltage: u8,
}

impl FromByteArray for DioOutputEnable {
    type Error = Infallible;
    type Array = [u8; 1];

    fn from_bytes(bytes: Self::Array) -> Result<Self, Self::Error> {
        Ok(Self {
            dio1: bytes[0] & 0x01 != 0,
            dio2: bytes[0] & 0x02 != 0,
            dio3: bytes[0] & 0x04 != 0,
        })
    }
}

impl ToByteArray for DioOutputEnable {
    type Error = Infallible;
    type Array = [u8; 1];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok([(self.dio1 as u8) | ((self.dio2 as u8) << 1) | ((self.dio3 as u8) << 2)])
    }
}

impl FromByteArray for DioInputEnable {
    type Error = Infallible;
    type Array = [u8; 1];

    fn from_bytes(bytes: Self::Array) -> Result<Self, Self::Error> {
        Ok(Self {
            dio1: bytes[0] & 0x01 != 0,
            dio2: bytes[0] & 0x02 != 0,
            dio3: bytes[0] & 0x04 != 0,
        })
    }
}

impl ToByteArray for DioInputEnable {
    type Error = Infallible;
    type Array = [u8; 1];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok([(self.dio1 as u8) | ((self.dio2 as u8) << 1) | ((self.dio3 as u8) << 2)])
    }
}

impl FromByteArray for DioPullUpControl {
    type Error = Infallible;
    type Array = [u8; 1];

    fn from_bytes(bytes: Self::Array) -> Result<Self, Self::Error> {
        Ok(Self {
            dio1: bytes[0] & 0x01 != 0,
            dio2: bytes[0] & 0x02 != 0,
            dio3: bytes[0] & 0x04 != 0,
        })
    }
}

impl ToByteArray for DioPullUpControl {
    type Error = Infallible;
    type Array = [u8; 1];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok([(self.dio1 as u8) | ((self.dio2 as u8) << 1) | ((self.dio3 as u8) << 2)])
    }
}

impl FromByteArray for DioPullDownControl {
    type Error = Infallible;
    type Array = [u8; 1];

    fn from_bytes(bytes: Self::Array) -> Result<Self, Self::Error> {
        Ok(Self {
            dio1: bytes[0] & 0x01 != 0,
            dio2: bytes[0] & 0x02 != 0,
            dio3: bytes[0] & 0x04 != 0,
        })
    }
}

impl ToByteArray for DioPullDownControl {
    type Error = Infallible;
    type Array = [u8; 1];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok([(self.dio1 as u8) | ((self.dio2 as u8) << 1) | ((self.dio3 as u8) << 2)])
    }
}

impl FromByteArray for Dio3OutputVoltage {
    type Error = Infallible;
    type Array = [u8; 1];

    fn from_bytes(bytes: Self::Array) -> Result<Self, Self::Error> {
        Ok(Self {
            voltage: bytes[0] & 0x07,
        })
    }
}

impl ToByteArray for Dio3OutputVoltage {
    type Error = Infallible;
    type Array = [u8; 1];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok([self.voltage & 0x07])
    }
}
