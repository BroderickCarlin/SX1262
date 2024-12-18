//! Packet handling related registers
//!
//! This module contains registers for configuring packet handling features including:
//! - Data whitening configuration
//! - CRC calculation settings
//! - Sync word detection
//! - Address filtering
//! - IQ signal configuration
//!
//! These registers are used in conjunction with the packet parameters commands
//! to define the complete packet format.

use core::convert::Infallible;

use regiface::{register, FromByteArray, ReadableRegister, ToByteArray, WritableRegister};

/// Whitening initial value register (address: 0x06B8)
///
/// Sets the initial value for the whitening LFSR used in FSK mode.
/// Data whitening helps maintain DC balance and avoid long sequences
/// of identical bits.
///
/// The whitening process uses a 9-bit LFSR with polynomial x^9 + x^5 + 1.
/// The LSB of the LFSR output is XORed with the MSB of the data.
///
/// # Note
/// Whitening is only required when user data has high correlation
/// with long strings of 0's and 1's. If data is already random,
/// whitening is not needed.
#[register(0x06B8u16)]
#[derive(Debug, Clone, Copy, ReadableRegister, WritableRegister)]
pub struct WhiteningInitialValue {
    /// Initial value for whitening LFSR
    /// Default: 0x0100
    pub value: u16,
}

impl Default for WhiteningInitialValue {
    fn default() -> Self {
        Self { value: 0x0100 }
    }
}

/// CRC initial value register (address: 0x06BC)
///
/// Sets the initial value for CRC calculation in FSK mode.
/// The CRC can be configured for different standard implementations:
///
/// # Examples
/// - IBM CRC-16: Initial value = 0xFFFF
/// - CCITT CRC-16: Initial value = 0x1D0F
///
/// The initial value is used as the starting state for the CRC calculation.
#[register(0x06BCu16)]
#[derive(Debug, Clone, Copy, ReadableRegister, WritableRegister)]
pub struct CrcInitialValue {
    /// Initial CRC value
    /// Default: 0x1D0F
    pub value: u16,
}

impl Default for CrcInitialValue {
    fn default() -> Self {
        Self { value: 0x1D0F }
    }
}

/// CRC polynomial register (address: 0x06BE)
///
/// Sets the polynomial used for CRC calculation in FSK mode.
/// The polynomial can be configured for different standard CRC implementations:
///
/// # Examples
/// - IBM CRC-16: Polynomial = 0x8005
/// - CCITT CRC-16: Polynomial = 0x1021
///
/// # Note
/// The CRC configuration (polynomial and initial value) must match between
/// transmitter and receiver for proper packet validation.
#[register(0x06BEu16)]
#[derive(Debug, Clone, Copy, ReadableRegister, WritableRegister)]
pub struct CrcPolynomial {
    /// CRC polynomial value
    /// Default: 0x1021
    pub value: u16,
}

impl Default for CrcPolynomial {
    fn default() -> Self {
        Self { value: 0x1021 }
    }
}

/// FSK sync word register (address: 0x06C0)
///
/// Contains the sync word used for FSK packet detection and synchronization.
/// The sync word helps identify valid packets and reject noise.
///
/// # Important Notes
/// - Sync word length is configurable from 0-8 bytes
/// - Preamble detector length must be shorter than sync word length
/// - Sync word must match exactly between TX and RX
#[register(0x06C0u16)]
#[derive(Debug, Clone, Copy, ReadableRegister, WritableRegister)]
pub struct SyncWord {
    /// 8-byte sync word value
    /// Each byte is written to consecutive addresses starting at 0x06C0
    pub value: [u8; 8],
}

/// Node address register for FSK mode (address: 0x06CD)
///
/// Sets the node address for address filtering in FSK mode.
/// Used in conjunction with address filtering configuration in
/// packet parameters.
///
/// # Address Filtering Modes
/// - Disabled: No filtering
/// - Node: Accept packets matching node address
/// - Node+Broadcast: Accept packets matching node or broadcast address
///
/// # Note
/// When address filtering is enabled, maximum payload length is 254 bytes.
#[register(0x06CDu16)]
#[derive(Debug, Clone, Copy, ReadableRegister, WritableRegister, Default)]
pub struct NodeAddress {
    /// Node address for filtering
    /// Default: 0x00
    pub address: u8,
}

/// Broadcast address register for FSK mode (address: 0x06CE)
///
/// Sets the broadcast address for address filtering in FSK mode.
/// Packets with this address will be accepted when Node+Broadcast
/// filtering is enabled.
#[register(0x06CEu16)]
#[derive(Debug, Clone, Copy, ReadableRegister, WritableRegister, Default)]
pub struct BroadcastAddress {
    /// Broadcast address for filtering
    /// Default: 0x00
    pub address: u8,
}

/// IQ polarity setup register (address: 0x0736)
///
/// Controls IQ signal configuration for LoRa modulation.
/// Can be used to invert IQ signals for network compatibility.
///
/// # Important Notes
/// - For inverted IQ operation, bit 2 must be:
///   - 0 when using inverted IQ
///   - 1 when using standard IQ
/// - Setting affects packet reception and network compatibility
#[register(0x0736u16)]
#[derive(Debug, Clone, Copy, ReadableRegister, WritableRegister, Default)]
pub struct IqPolaritySetup {
    /// IQ mode selection
    /// - false = Standard IQ (default)
    /// - true = Inverted IQ
    pub inverted_iq: bool,
}

/// LoRa sync word register (address: 0x0740)
///
/// Sets the LoRa sync word for network identification.
/// Different sync words can be used to create separate networks
/// that won't interfere with each other.
///
/// # Standard Values
/// - Public Network: 0x3444
/// - Private Network: 0x1424
///
/// # Note
/// Sync word must match between all devices in the same network.
#[register(0x0740u16)]
#[derive(Debug, Clone, Copy, ReadableRegister, WritableRegister)]
pub struct LoraSyncWord {
    /// Sync word value
    /// Default: 0x1424
    pub value: u16,
}

impl Default for LoraSyncWord {
    fn default() -> Self {
        Self { value: 0x1424 }
    }
}

impl FromByteArray for WhiteningInitialValue {
    type Error = Infallible;
    type Array = [u8; 2];

    fn from_bytes(bytes: Self::Array) -> Result<Self, Self::Error> {
        Ok(Self {
            value: u16::from_be_bytes(bytes),
        })
    }
}

impl ToByteArray for WhiteningInitialValue {
    type Error = Infallible;
    type Array = [u8; 2];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok(self.value.to_be_bytes())
    }
}

impl FromByteArray for CrcInitialValue {
    type Error = Infallible;
    type Array = [u8; 2];

    fn from_bytes(bytes: Self::Array) -> Result<Self, Self::Error> {
        Ok(Self {
            value: u16::from_be_bytes(bytes),
        })
    }
}

impl ToByteArray for CrcInitialValue {
    type Error = Infallible;
    type Array = [u8; 2];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok(self.value.to_be_bytes())
    }
}

impl FromByteArray for CrcPolynomial {
    type Error = Infallible;
    type Array = [u8; 2];

    fn from_bytes(bytes: Self::Array) -> Result<Self, Self::Error> {
        Ok(Self {
            value: u16::from_be_bytes(bytes),
        })
    }
}

impl ToByteArray for CrcPolynomial {
    type Error = Infallible;
    type Array = [u8; 2];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok(self.value.to_be_bytes())
    }
}

impl FromByteArray for SyncWord {
    type Error = Infallible;
    type Array = [u8; 8];

    fn from_bytes(bytes: Self::Array) -> Result<Self, Self::Error> {
        Ok(Self { value: bytes })
    }
}

impl ToByteArray for SyncWord {
    type Error = Infallible;
    type Array = [u8; 8];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok(self.value)
    }
}

impl FromByteArray for NodeAddress {
    type Error = Infallible;
    type Array = [u8; 1];

    fn from_bytes(bytes: Self::Array) -> Result<Self, Self::Error> {
        Ok(Self { address: bytes[0] })
    }
}

impl ToByteArray for NodeAddress {
    type Error = Infallible;
    type Array = [u8; 1];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok([self.address])
    }
}

impl FromByteArray for BroadcastAddress {
    type Error = Infallible;
    type Array = [u8; 1];

    fn from_bytes(bytes: Self::Array) -> Result<Self, Self::Error> {
        Ok(Self { address: bytes[0] })
    }
}

impl ToByteArray for BroadcastAddress {
    type Error = Infallible;
    type Array = [u8; 1];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok([self.address])
    }
}

impl FromByteArray for IqPolaritySetup {
    type Error = Infallible;
    type Array = [u8; 1];

    fn from_bytes(bytes: Self::Array) -> Result<Self, Self::Error> {
        Ok(Self {
            inverted_iq: bytes[0] & 0x01 != 0,
        })
    }
}

impl ToByteArray for IqPolaritySetup {
    type Error = Infallible;
    type Array = [u8; 1];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok([self.inverted_iq as u8])
    }
}

impl FromByteArray for LoraSyncWord {
    type Error = Infallible;
    type Array = [u8; 2];

    fn from_bytes(bytes: Self::Array) -> Result<Self, Self::Error> {
        Ok(Self {
            value: u16::from_be_bytes(bytes),
        })
    }
}

impl ToByteArray for LoraSyncWord {
    type Error = Infallible;
    type Array = [u8; 2];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok(self.value.to_be_bytes())
    }
}
