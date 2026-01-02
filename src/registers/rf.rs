//! RF-related registers
//!
//! This module contains registers related to RF configuration and operation including:
//! - Random number generation
//! - TX modulation parameters
//! - RX gain control
//! - Power amplifier configuration
//! - Over-current protection

use core::convert::Infallible;

use regiface::{register, FromByteArray, ReadableRegister, ToByteArray, WritableRegister};

/// Random number generator register (address: 0x0819)
///
/// Provides access to the internal 32-bit random number generator. This can be used
/// for generating random numbers for protocol implementations or other purposes.
/// The random number is generated from thermal noise in the RF frontend.
///
/// Reading this register returns a new random 32-bit value each time.
#[register(0x0819u16)]
#[derive(Debug, Clone, Copy, ReadableRegister)]
pub struct RandomNumber {
    /// 32-bit random number value
    pub value: u32,
}

/// TX modulation register (address: 0x0889)
///
/// Controls transmit modulation parameters, particularly for LoRa bandwidth optimization.
/// This register must be configured correctly based on the selected bandwidth to ensure
/// optimal modulation quality.
///
/// # Important Notes
/// - Must be configured before each packet transmission
#[register(0x0889u16)]
#[derive(Debug, Clone, Copy, ReadableRegister, WritableRegister)]
pub struct TxModulation {
    pub data: u8,
}

impl TxModulation {
    /// Apply the workaround for LoRa 500kHz bandwidth optimization.
    /// This should be set to true when using LoRa with 500kHz bandwidth,
    /// and false for all other LoRa bandwidths and (G)FSK.
    ///
    /// See the datasheet chapter 15.1 for more details.
    pub fn apply_lora_500khz_optimization(&mut self, do_optimize: bool) {
        if do_optimize {
            // clear bit 2 for lora 500kHz
            self.data &= 0xFB;
        } else {
            // set bit 2 for (G)FSK and other LoRa bandwidths
            self.data |= 0x04;
        }
    }
}

impl Default for TxModulation {
    fn default() -> Self {
        // Preserve the datasheet default (bit 0) and default to the non-500kHz setting (bit 2 set).
        Self { data: 0x01 | 0x04 }
    }
}

/// Error type for RX gain mode conversion
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidGainMode(pub u8);

/// RX gain register (address: 0x08AC)
///
/// Controls the receiver gain configuration, allowing tradeoff between power consumption
/// and sensitivity. The gain mode affects both power consumption and receiver sensitivity:
///
/// # Power Saving Mode
/// - Lower power consumption (~4.2mA in DC-DC mode)
/// - Reduced sensitivity (~3dB worse than boosted)
///
/// # Boosted Mode
/// - Higher power consumption (~4.8mA in DC-DC mode)
/// - Maximum sensitivity
///
/// Note: The RX Gain setting is not retained when waking from sleep mode. To include this
/// register in retention memory, additional configuration is required.
#[register(0x08ACu16)]
#[derive(Debug, Clone, Copy, ReadableRegister, WritableRegister)]
pub enum RxGain {
    /// Power saving gain mode (~4.2mA in DC-DC mode)
    /// Lower power consumption but reduced sensitivity
    PowerSaving,
    /// Boosted gain mode (~4.8mA in DC-DC mode)
    /// Maximum sensitivity but higher power consumption
    Boosted,
}

impl Default for RxGain {
    fn default() -> Self {
        Self::PowerSaving
    }
}

impl RxGain {
    /// Convert a raw byte value to RxGainMode
    pub fn from_byte(value: u8) -> Result<Self, InvalidGainMode> {
        match value {
            0x94 => Ok(Self::PowerSaving),
            0x96 => Ok(Self::Boosted),
            invalid => Err(InvalidGainMode(invalid)),
        }
    }

    /// Convert RxGainMode to its raw byte value
    pub fn to_byte(self) -> u8 {
        match self {
            Self::PowerSaving => 0x94,
            Self::Boosted => 0x96,
        }
    }
}

/// TX clamp configuration register (address: 0x08D8)
///
/// Controls the Power Amplifier (PA) clamping threshold to protect against
/// over-voltage conditions, particularly important for the SX1262 high-power PA.
///
/// # Important Notes
/// - For SX1262: Set to "<value> | 0x1E" (see 15.2.2 in datasheet)
/// - For SX1261: Use default value
/// - Must be configured after power-on reset or wake from cold start
#[register(0x08D8u16)]
#[derive(Debug, Clone, Copy, ReadableRegister, WritableRegister)]
pub struct TxClampConfig {
    config: u8,
}

impl TxClampConfig {
    pub fn apply_sx1262_workaround(&mut self) {
        self.config |= 0x1E;
    }
}

impl Default for TxClampConfig {
    fn default() -> Self {
        Self {
            config: 0xC8 | 0x1E, // Default value for SX1262 with workaround applied
        }
    }
}

/// OCP (Over Current Protection) configuration register (address: 0x08E7)
///
/// Sets the over-current protection threshold for the power amplifier.
/// The threshold is automatically configured when SetPaConfig() is called,
/// but can be manually adjusted if needed.
///
/// # Current Limit Calculation
/// Current limit = threshold * 2.5mA
///
/// # Default Values
/// - SX1261: 0x18 (60mA)
/// - SX1262: 0x38 (140mA)
///
/// # Important Notes
/// - When using SX1261 in DC-DC mode, the current limit should account for
///   supply voltage as current draw is inversely proportional to VBAT
/// - Value is automatically reconfigured when SetPaConfig() is called
#[register(0x08E7u16)]
#[derive(Debug, Clone, Copy, ReadableRegister, WritableRegister)]
pub struct OcpConfiguration {
    /// OCP current limit in steps of 2.5mA
    /// - Range: 0x00-0xFF (0-637.5mA)
    /// - Default SX1261: 0x18 (60mA)
    /// - Default SX1262: 0x38 (140mA)
    pub threshold: u8,
}

impl Default for OcpConfiguration {
    fn default() -> Self {
        Self {
            threshold: 0x18, // Default to SX1261 value
        }
    }
}

impl FromByteArray for RandomNumber {
    type Error = Infallible;
    type Array = [u8; 4];

    fn from_bytes(bytes: Self::Array) -> Result<Self, Self::Error> {
        Ok(Self {
            value: u32::from_be_bytes(bytes),
        })
    }
}

impl FromByteArray for TxModulation {
    type Error = Infallible;
    type Array = [u8; 1];

    fn from_bytes(bytes: Self::Array) -> Result<Self, Self::Error> {
        Ok(Self { data: bytes[0] })
    }
}

impl ToByteArray for TxModulation {
    type Error = Infallible;
    type Array = [u8; 1];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok([self.data])
    }
}

impl FromByteArray for RxGain {
    type Error = InvalidGainMode;
    type Array = [u8; 1];

    fn from_bytes(bytes: Self::Array) -> Result<Self, Self::Error> {
        Self::from_byte(bytes[0])
    }
}

impl ToByteArray for RxGain {
    type Error = Infallible;
    type Array = [u8; 1];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok([self.to_byte()])
    }
}

impl FromByteArray for TxClampConfig {
    type Error = Infallible;
    type Array = [u8; 1];

    fn from_bytes(bytes: Self::Array) -> Result<Self, Self::Error> {
        Ok(Self { config: bytes[0] })
    }
}

impl ToByteArray for TxClampConfig {
    type Error = Infallible;
    type Array = [u8; 1];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok([self.config])
    }
}

impl FromByteArray for OcpConfiguration {
    type Error = Infallible;
    type Array = [u8; 1];

    fn from_bytes(bytes: Self::Array) -> Result<Self, Self::Error> {
        Ok(Self {
            threshold: bytes[0],
        })
    }
}

impl ToByteArray for OcpConfiguration {
    type Error = Infallible;
    type Array = [u8; 1];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok([self.threshold])
    }
}
