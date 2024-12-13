//! Status commands
//!
//! This module contains commands for monitoring device status and performance:
//! - Device operating mode and command status
//! - Signal strength measurements
//! - Packet reception status
//! - Error detection and handling
//! - Communication statistics
//!
//! These commands can be used to monitor device operation and
//! diagnose issues during development and operation.

use core::convert::Infallible;

use regiface::FromByteArray;

use crate::{Command, NoParameters};

/// GetStatus command (0xC0)
///
/// Returns the current device status including:
/// - Operating mode (bits 6:4)
///   - 0x0: Unused
///   - 0x2: STDBY_RC
///   - 0x3: STDBY_XOSC
///   - 0x4: FS
///   - 0x5: RX
///   - 0x6: TX
/// - Command status (bits 3:1)
///   - 0x0: Reserved
///   - 0x2: Data available
///   - 0x3: Command timeout
///   - 0x4: Command processing error
///   - 0x5: Failure to execute
///   - 0x6: TX done
#[derive(Debug, Clone)]
pub struct GetStatus;

impl Command for GetStatus {
    type IdType = u8;
    type CommandParameters = NoParameters;
    type ResponseParameters = u8;

    fn id() -> Self::IdType {
        0xC0
    }

    fn invoking_parameters(self) -> Self::CommandParameters {
        NoParameters::default()
    }
}

/// GetRssiInst command (0x15)
///
/// Returns instantaneous RSSI value during reception.
///
/// # RSSI Calculation
/// Signal power in dBm = -value/2
///
/// # Important Notes
/// - Only valid in RX mode
/// - Updates continuously during reception
/// - Accuracy typically ±2dB
#[derive(Debug, Clone)]
pub struct GetRssiInst;

impl Command for GetRssiInst {
    type IdType = u8;
    type CommandParameters = NoParameters;
    type ResponseParameters = u8;

    fn id() -> Self::IdType {
        0x15
    }

    fn invoking_parameters(self) -> Self::CommandParameters {
        NoParameters::default()
    }
}

/// RX buffer status response
///
/// Contains information about received packet in buffer.
#[derive(Debug, Clone, Copy)]
pub struct RxBufferStatus {
    /// Length of received payload in bytes
    pub payload_length: u8,

    /// Buffer pointer to first byte of payload
    /// Offset from RxBaseAddress
    pub buffer_pointer: u8,
}

impl FromByteArray for RxBufferStatus {
    type Error = Infallible;
    type Array = [u8; 2];

    fn from_bytes(bytes: Self::Array) -> Result<Self, Self::Error> {
        Ok(Self {
            payload_length: bytes[0],
            buffer_pointer: bytes[1],
        })
    }
}

/// GetRxBufferStatus command (0x13)
///
/// Returns status of received packet in buffer.
///
/// # Important Notes
/// - Valid after RxDone interrupt
/// - Returns payload length and start address
/// - Data remains valid until next RX operation
/// - Use with ReadBuffer to retrieve payload
#[derive(Debug, Clone)]
pub struct GetRxBufferStatus;

impl Command for GetRxBufferStatus {
    type IdType = u8;
    type CommandParameters = NoParameters;
    type ResponseParameters = RxBufferStatus;

    fn id() -> Self::IdType {
        0x13
    }

    fn invoking_parameters(self) -> Self::CommandParameters {
        NoParameters::default()
    }
}

/// Packet status response
///
/// Contains status information about received packet.
/// Interpretation depends on packet type (LoRa/FSK).
#[derive(Debug, Clone, Copy)]
pub struct PacketStatus {
    /// Status bytes array:
    /// FSK Mode:
    /// - status[0]: RxStatus
    ///   - bit 7: Preamble error
    ///   - bit 6: Sync error
    ///   - bit 5: Addr error
    ///   - bit 4: CRC error
    ///   - bit 3: Length error
    ///   - bit 2: Abort error
    ///   - bit 1: Packet received
    ///   - bit 0: Packet sent
    /// - status[1]: RssiSync (-value/2 dBm)
    /// - status[2]: RssiAvg (-value/2 dBm)
    ///
    /// LoRa Mode:
    /// - status[0]: RssiPkt (-value/2 dBm)
    /// - status[1]: SnrPkt (value/4 dB)
    /// - status[2]: SignalRssiPkt (-value/2 dBm)
    pub status: [u8; 3],
}

impl FromByteArray for PacketStatus {
    type Error = Infallible;
    type Array = [u8; 3];

    fn from_bytes(bytes: Self::Array) -> Result<Self, Self::Error> {
        Ok(Self { status: bytes })
    }
}

/// GetPacketStatus command (0x14)
///
/// Returns detailed status of received packet.
///
/// # Important Notes
/// - Valid after RxDone interrupt
/// - Status interpretation depends on packet type
/// - RSSI/SNR values latched at different times
/// - FSK: RssiSync at sync word, RssiAvg over payload
/// - LoRa: RssiPkt average over header+payload
#[derive(Debug, Clone)]
pub struct GetPacketStatus;

impl Command for GetPacketStatus {
    type IdType = u8;
    type CommandParameters = NoParameters;
    type ResponseParameters = PacketStatus;

    fn id() -> Self::IdType {
        0x14
    }

    fn invoking_parameters(self) -> Self::CommandParameters {
        NoParameters::default()
    }
}

/// Device errors response
///
/// Contains flags for various error conditions.
#[derive(Debug, Clone, Copy)]
pub struct DeviceErrors {
    /// RC64k calibration error
    pub rc64k_calib_err: bool,
    /// RC13M calibration error
    pub rc13m_calib_err: bool,
    /// PLL calibration error
    pub pll_calib_err: bool,
    /// ADC calibration error
    pub adc_calib_err: bool,
    /// Image calibration error
    pub img_calib_err: bool,
    /// XOSC startup error
    /// Normal with TCXO at startup
    pub xosc_start_err: bool,
    /// PLL lock error
    pub pll_lock_err: bool,
    /// PA ramping error
    pub pa_ramp_err: bool,
}

impl FromByteArray for DeviceErrors {
    type Error = Infallible;
    type Array = [u8; 2];

    fn from_bytes(bytes: Self::Array) -> Result<Self, Self::Error> {
        Ok(Self {
            rc64k_calib_err: bytes[1] & 0b1 != 0,
            rc13m_calib_err: bytes[1] & 0b10 != 0,
            pll_calib_err: bytes[1] & 0b100 != 0,
            adc_calib_err: bytes[1] & 0b1000 != 0,
            img_calib_err: bytes[1] & 0b1_0000 != 0,
            xosc_start_err: bytes[1] & 0b10_0000 != 0,
            pll_lock_err: bytes[1] & 0b100_0000 != 0,
            pa_ramp_err: bytes[0] & 0b1 != 0,
        })
    }
}

/// GetDeviceErrors command (0x17)
///
/// Returns error flags for various conditions.
///
/// # Important Notes
/// - Errors persist until explicitly cleared
/// - XOSC_START_ERR normal with TCXO at startup
/// - Multiple errors may be set simultaneously
/// - Use ClearDeviceErrors to clear flags
#[derive(Debug, Clone)]
pub struct GetDeviceErrors;

impl Command for GetDeviceErrors {
    type IdType = u8;
    type CommandParameters = NoParameters;
    type ResponseParameters = DeviceErrors;

    fn id() -> Self::IdType {
        0x17
    }

    fn invoking_parameters(self) -> Self::CommandParameters {
        NoParameters::default()
    }
}

/// ClearDeviceErrors command (0x07)
///
/// Clears all device error flags.
///
/// # Important Notes
/// - Clears all errors simultaneously
/// - Cannot clear errors individually
/// - Should be called after handling errors
#[derive(Debug, Clone)]
pub struct ClearDeviceErrors;

impl Command for ClearDeviceErrors {
    type IdType = u8;
    type CommandParameters = NoParameters;
    type ResponseParameters = NoParameters;

    fn id() -> Self::IdType {
        0x07
    }

    fn invoking_parameters(self) -> Self::CommandParameters {
        NoParameters::default()
    }
}

/// Statistics response
///
/// Contains packet reception statistics.
#[derive(Debug, Clone, Copy)]
pub struct Stats {
    /// Number of packets received
    /// Increments for all received packets
    pub packets_received: u16,

    /// Number of packets with CRC error
    /// Increments when CRC check fails
    pub packets_crc_error: u16,

    /// Number of packets with header error
    /// LoRa: Header CRC error
    /// FSK: Invalid length field
    pub packets_header_error: u16,
}

impl FromByteArray for Stats {
    type Error = Infallible;
    type Array = [u8; 6];

    fn from_bytes(bytes: Self::Array) -> Result<Self, Self::Error> {
        Ok(Self {
            packets_received: u16::from_be_bytes(bytes[0..2].try_into().unwrap()),
            packets_crc_error: u16::from_be_bytes(bytes[2..4].try_into().unwrap()),
            packets_header_error: u16::from_be_bytes(bytes[4..6].try_into().unwrap()),
        })
    }
}

/// GetStats command (0x10)
///
/// Returns packet reception statistics.
///
/// # Important Notes
/// - Stats persist through sleep mode
/// - Reset with ResetStats command
/// - Useful for monitoring link quality
/// - CRC/header error rates indicate issues
#[derive(Debug, Clone)]
pub struct GetStats;

impl Command for GetStats {
    type IdType = u8;
    type CommandParameters = NoParameters;
    type ResponseParameters = Stats;

    fn id() -> Self::IdType {
        0x10
    }

    fn invoking_parameters(self) -> Self::CommandParameters {
        NoParameters::default()
    }
}

/// ResetStats command (0x00)
///
/// Resets all packet reception statistics to zero.
///
/// # Important Notes
/// - Resets all counters simultaneously
/// - Cannot reset counters individually
/// - Use before starting new test/monitoring
#[derive(Debug, Clone)]
pub struct ResetStats;

impl Command for ResetStats {
    type IdType = u8;
    type CommandParameters = NoParameters;
    type ResponseParameters = NoParameters;

    fn id() -> Self::IdType {
        0x00
    }

    fn invoking_parameters(self) -> Self::CommandParameters {
        NoParameters::default()
    }
}
