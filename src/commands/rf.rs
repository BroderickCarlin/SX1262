//! RF, modulation and packet commands
//!
//! This module contains commands for configuring RF parameters, modulation settings,
//! and packet handling. These commands control:
//! - RF frequency configuration
//! - Packet type selection (LoRa/FSK)
//! - TX power and ramping
//! - Modulation parameters
//! - Packet formatting
//! - Channel Activity Detection (CAD)
//! - Buffer management
//!
//! Most configuration commands must be issued while in STDBY_RC mode.

use core::convert::Infallible;

use regiface::FromByteArray;

use crate::{Command, NoParameters, ToByteArray};

/// RF frequency configuration parameters
///
/// Used to set the RF frequency for both TX and RX operations.
/// The frequency is calculated as: RF = frequency_in_hz * FXTAL / 2^25
/// where FXTAL is typically 32MHz.
#[derive(Debug, Clone, Copy)]
pub struct RfFrequencyConfig {
    /// RF frequency in Hz
    /// Valid range: 150MHz to 960MHz
    pub frequency: u32,
}

impl ToByteArray for RfFrequencyConfig {
    type Error = Infallible;
    type Array = [u8; 4];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        // Frequency register = (Frequency * 2^25) / FXTAL
        let f = ((self.frequency as u64 * (1_u64 << 25)) / 32_000_000) as u32;

        Ok(f.to_be_bytes())
    }
}

/// SetRfFrequency command (0x86)
///
/// Sets the RF frequency for both TX and RX operations. In RX mode,
/// the command automatically configures the necessary IF frequency offset.
///
/// # Important Notes
/// - Must be called while in STDBY_RC mode
/// - For frequencies below 400MHz, some bandwidths may not be available
/// - The frequency resolution (PLL step) is ~0.95Hz
#[derive(Debug, Clone)]
pub struct SetRfFrequency {
    /// RF frequency configuration
    pub config: RfFrequencyConfig,
}

impl Command for SetRfFrequency {
    type IdType = u8;
    type CommandParameters = RfFrequencyConfig;
    type ResponseParameters = NoParameters;

    fn id() -> Self::IdType {
        0x86
    }

    fn invoking_parameters(self) -> Self::CommandParameters {
        self.config
    }
}

/// Packet type options for radio configuration
#[derive(Debug, Clone, Copy)]
pub enum PacketType {
    /// GFSK packet type (0x00)
    /// Supports bit rates from 0.6 to 300kbps
    Gfsk = 0x00,

    /// LoRa packet type (0x01)
    /// Supports spreading factors 5-12 and bandwidths 7.8-500kHz
    LoRa = 0x01,
}

impl FromByteArray for PacketType {
    type Error = Infallible;
    type Array = [u8; 1];

    fn from_bytes(bytes: Self::Array) -> Result<Self, Self::Error> {
        Ok(match bytes[0] {
            0x00 => Self::Gfsk,
            0x01 => Self::LoRa,
            _ => Self::LoRa,
        })
    }
}

impl ToByteArray for PacketType {
    type Error = Infallible;
    type Array = [u8; 1];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok([self as u8])
    }
}

/// SetPacketType command (0x8A)
///
/// Sets the packet type (LoRa or GFSK) and associated modem configuration.
///
/// # Important Notes
/// - Must be the first command in the radio configuration sequence
/// - Must be called while in STDBY_RC mode
/// - Parameters from previous mode are not retained
/// - Modulation and packet parameters must be reconfigured after changing type
#[derive(Debug, Clone)]
pub struct SetPacketType {
    /// Packet type selection
    pub packet_type: PacketType,
}

impl Command for SetPacketType {
    type IdType = u8;
    type CommandParameters = PacketType;
    type ResponseParameters = NoParameters;

    fn id() -> Self::IdType {
        0x8A
    }

    fn invoking_parameters(self) -> Self::CommandParameters {
        self.packet_type
    }
}

/// GetPacketType command (0x11)
///
/// Returns the current packet type configuration.
#[derive(Debug, Clone)]
pub struct GetPacketType;

impl Command for GetPacketType {
    type IdType = u8;
    type CommandParameters = NoParameters;
    type ResponseParameters = PacketType;

    fn id() -> Self::IdType {
        0x11
    }

    fn invoking_parameters(self) -> Self::CommandParameters {
        NoParameters::default()
    }
}

/// Power amplifier ramp time options
#[derive(Debug, Clone, Copy)]
pub enum RampTime {
    /// 10 μs ramp time
    Micros10 = 0x00,
    /// 20 μs ramp time
    Micros20 = 0x01,
    /// 40 μs ramp time
    Micros40 = 0x02,
    /// 80 μs ramp time
    Micros80 = 0x03,
    /// 200 μs ramp time
    Micros200 = 0x04,
    /// 800 μs ramp time
    Micros800 = 0x05,
    /// 1700 μs ramp time
    Micros1700 = 0x06,
    /// 3400 μs ramp time
    Micros3400 = 0x07,
}

/// TX parameters configuration
#[derive(Debug, Clone, Copy)]
pub struct TxParams {
    /// Output power in dBm
    /// - SX1261: -17 to +14 dBm
    /// - SX1262: -9 to +22 dBm
    ///
    /// Power selection depends on PA configuration set by SetPaConfig
    pub power: i8,

    /// Power amplifier ramp time
    /// Longer ramp times reduce spectral spreading but increase
    /// packet time-on-air
    pub ramp_time: RampTime,
}

impl ToByteArray for TxParams {
    type Error = Infallible;
    type Array = [u8; 2];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok([self.power as u8, self.ramp_time as u8])
    }
}

/// SetTxParams command (0x8E)
///
/// Sets the TX output power and PA ramp time.
///
/// # Important Notes
/// - Power range depends on PA configuration (SX1261/SX1262)
/// - Power is set in 1dB steps
/// - Ramp time affects spectral emissions and time-on-air
/// - Must be configured after SetPaConfig
#[derive(Debug, Clone)]
pub struct SetTxParams {
    /// TX parameters configuration
    pub params: TxParams,
}

impl Command for SetTxParams {
    type IdType = u8;
    type CommandParameters = TxParams;
    type ResponseParameters = NoParameters;

    fn id() -> Self::IdType {
        0x8E
    }

    fn invoking_parameters(self) -> Self::CommandParameters {
        self.params
    }
}

/// GFSK modulation parameters
///
/// Configures the modulation settings for GFSK packet type.
/// The radio calculates internal register values from these parameters:
/// - Bit rate register = (32 * FXTAL) / bit_rate
/// - Frequency deviation register = (deviation * 2^25) / FXTAL
///   where FXTAL is typically 32MHz
///
/// # Important Notes
/// - Ensure bandwidth > 2 * (frequency_deviation + bit_rate/2)
/// - Pulse shaping affects spectral efficiency and occupied bandwidth
/// - Higher bit rates require wider bandwidths
#[derive(Debug, Clone, Copy)]
pub struct GfskModParams {
    /// Bit rate in bits per second
    /// Valid range: 600 bps to 300 kbps
    pub bit_rate: u32,
    /// Pulse shape filtering for spectral efficiency
    pub pulse_shape: GfskPulseShape,
    /// RX bandwidth setting for channel filtering
    pub bandwidth: GfskBandwidth,
    /// Frequency deviation in Hz
    /// Maximum deviation should be < 0.5 * bandwidth
    pub freq_deviation: u32,
}

/// GFSK pulse shape options for spectral shaping
///
/// Gaussian filtering reduces spectral spreading but increases
/// intersymbol interference. Higher BT products reduce ISI
/// at the cost of wider bandwidth.
#[derive(Debug, Clone, Copy)]
pub enum GfskPulseShape {
    /// No pulse shaping filter
    NoFilter = 0x00,
    /// Gaussian filter, BT = 0.3
    /// Minimum bandwidth, maximum ISI
    Bt03 = 0x08,
    /// Gaussian filter, BT = 0.5
    /// Balanced bandwidth/ISI tradeoff
    Bt05 = 0x09,
    /// Gaussian filter, BT = 0.7
    /// Reduced ISI, wider bandwidth
    Bt07 = 0x0A,
    /// Gaussian filter, BT = 1.0
    /// Minimum ISI, maximum bandwidth
    Bt1 = 0x0B,
}

/// GFSK receiver bandwidth options
///
/// Sets the channel filter bandwidth. Should be selected based on:
/// - Signal bandwidth (2 * (freq_deviation + bit_rate/2))
/// - Adjacent channel rejection requirements
/// - Expected frequency error
///
/// Wider bandwidths allow higher data rates but reduce selectivity
#[derive(Debug, Clone, Copy)]
pub enum GfskBandwidth {
    /// 4.8 kHz Double-Side Bandwidth
    Bw48 = 0x1F,
    /// 5.8 kHz Double-Side Bandwidth
    Bw58 = 0x17,
    /// 7.3 kHz Double-Side Bandwidth
    Bw73 = 0x0F,
    /// 9.7 kHz Double-Side Bandwidth
    Bw97 = 0x1E,
    /// 11.7 kHz Double-Side Bandwidth
    Bw117 = 0x16,
    /// 14.6 kHz Double-Side Bandwidth
    Bw146 = 0x0E,
    /// 29.3 kHz Double-Side Bandwidth
    Bw293 = 0x0D,
    /// 39 kHz Double-Side Bandwidth
    Bw39 = 0x1C,
    /// 46.9 kHz Double-Side Bandwidth
    Bw469 = 0x14,
    /// 58.6 kHz Double-Side Bandwidth
    Bw586 = 0x0C,
    /// 78.2 kHz Double-Side Bandwidth
    Bw782 = 0x1B,
    /// 93.8 kHz Double-Side Bandwidth
    Bw938 = 0x13,
    /// 117.3 kHz Double-Side Bandwidth
    Bw1173 = 0x0B,
    /// 156.2 kHz Double-Side Bandwidth
    Bw1562 = 0x1A,
    /// 1872 kHz Double-Side Bandwidth
    Bw1872 = 0x12,
    /// 232.3 kHz Double-Side Bandwidth
    Bw2323 = 0x0A,
    /// 312.0 kHz Double-Side Bandwidth
    Bw3120 = 0x19,
    /// 373.6 kHz Double-Side Bandwidth
    Bw3736 = 0x11,
    /// 467.0 kHz Double-Side Bandwidth
    Bw4670 = 0x09,
}

/// LoRa modulation parameters
///
/// Configures the modulation settings for LoRa packet type.
/// Parameter selection affects:
/// - Sensitivity vs time-on-air tradeoff
/// - Maximum packet length
/// - Required receive window length
///
/// # Important Notes
/// - Higher spreading factors increase sensitivity but reduce data rate
/// - Wider bandwidths increase data rate but reduce sensitivity
/// - Enable low data rate optimization when symbol length ≥ 16.38ms
/// - Coding rate adds redundancy at the cost of time-on-air
#[derive(Debug, Clone, Copy)]
pub struct LoRaModParams {
    /// Spreading Factor (chip/symbol)
    pub spreading_factor: SpreadingFactor,
    /// Signal bandwidth
    pub bandwidth: LoRaBandwidth,
    /// Coding rate for forward error correction
    pub coding_rate: CodingRate,
    /// Low Data Rate optimization
    /// Required when symbol length ≥ 16.38ms
    pub low_data_rate_opt: bool,
}

/// LoRa spreading factor options
///
/// Sets the number of chips per symbol. Higher spreading factors:
/// - Increase sensitivity
/// - Increase time-on-air
/// - Reduce maximum packet length
/// - Reduce tolerance to frequency offset
///
/// SF5/SF6 have restrictions on header and CRC usage
#[derive(Debug, Clone, Copy)]
pub enum SpreadingFactor {
    /// SF5 - 32 chips/symbol
    /// Fastest data rate, shortest range
    SF5 = 5,
    /// SF6 - 64 chips/symbol
    SF6 = 6,
    /// SF7 - 128 chips/symbol
    SF7 = 7,
    /// SF8 - 256 chips/symbol
    SF8 = 8,
    /// SF9 - 512 chips/symbol
    SF9 = 9,
    /// SF10 - 1024 chips/symbol
    SF10 = 10,
    /// SF11 - 2048 chips/symbol
    SF11 = 11,
    /// SF12 - 4096 chips/symbol
    /// Slowest data rate, longest range
    SF12 = 12,
}

/// LoRa bandwidth options
///
/// Sets the signal bandwidth. Wider bandwidths:
/// - Increase data rate
/// - Reduce sensitivity
/// - Increase tolerance to frequency offset
///
/// Some bandwidths may not be available below 400MHz
#[derive(Debug, Clone, Copy)]
pub enum LoRaBandwidth {
    /// 7.81 kHz bandwidth
    Bw7 = 0x00,
    /// 10.42 kHz bandwidth
    Bw10 = 0x08,
    /// 15.63 kHz bandwidth
    Bw15 = 0x01,
    /// 20.83 kHz bandwidth
    Bw20 = 0x09,
    /// 31.25 kHz bandwidth
    Bw31 = 0x02,
    /// 41.67 kHz bandwidth
    Bw41 = 0x0a,
    /// 62.50 kHz bandwidth
    Bw62 = 0x03,
    /// 125 kHz bandwidth
    Bw125 = 0x04,
    /// 250 kHz bandwidth
    Bw250 = 0x05,
    /// 500 kHz bandwidth
    /// Highest data rate, lowest sensitivity
    Bw500 = 0x06,
}

/// LoRa coding rate options
///
/// Sets the Forward Error Correction (FEC) rate.
/// Higher coding rates:
/// - Increase reliability in noisy conditions
/// - Increase time-on-air
/// - Reduce effective data rate
#[derive(Debug, Clone, Copy)]
pub enum CodingRate {
    /// 4/5 coding rate
    /// Lowest redundancy (1.25x overhead)
    Cr45 = 0x01,
    /// 4/6 coding rate
    /// 1.5x overhead
    Cr46 = 0x02,
    /// 4/7 coding rate
    /// 1.75x overhead
    Cr47 = 0x03,
    /// 4/8 coding rate
    /// Highest redundancy (2x overhead)
    Cr48 = 0x04,
}

/// Modulation parameters configuration
///
/// Configures the radio modulation based on the selected packet type.
/// The parameters are interpreted differently for GFSK and LoRa modes:
///
/// # GFSK Mode Parameters
/// Uses GfskModParams to configure:
/// - Bit rate (600 bps to 300 kbps)
/// - Pulse shaping filter
/// - RX bandwidth (4.8 to 467 kHz)
/// - Frequency deviation
///
/// # LoRa Mode Parameters
/// Uses LoRaModParams to configure:
/// - Spreading factor (SF5-SF12)
/// - Bandwidth (7.8 to 500 kHz)
/// - Coding rate (4/5 to 4/8)
/// - Low data rate optimization
///
/// # Important Notes
/// - Parameters must match the selected packet type
/// - Configuration affects sensitivity, range, and data rate
/// - Some parameter combinations may be invalid or suboptimal
#[derive(Debug, Clone)]
pub enum ModulationParams {
    /// GFSK modulation configuration
    Gfsk(GfskModParams),
    /// LoRa modulation configuration
    LoRa(LoRaModParams),
}

impl ToByteArray for ModulationParams {
    type Error = Infallible;
    type Array = [u8; 8];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        let mut bytes = [0u8; 8];
        match self {
            ModulationParams::Gfsk(params) => {
                // Bit rate = (32 * FXTAL) / bit_rate
                let br_val = (32 * 32_000_000) / params.bit_rate;
                bytes[0..3].copy_from_slice(&br_val.to_be_bytes()[1..]);
                bytes[3] = params.pulse_shape as u8;
                bytes[4] = params.bandwidth as u8;
                // Frequency deviation register = (Frequency deviation * 2^25) / FXTAL
                let fdev = ((params.freq_deviation as u64 * (1_u64 << 25)) / 32_000_000) as u32;
                bytes[5..8].copy_from_slice(&fdev.to_be_bytes()[1..]);
            }
            ModulationParams::LoRa(params) => {
                bytes[0] = params.spreading_factor as u8;
                bytes[1] = params.bandwidth as u8;
                bytes[2] = params.coding_rate as u8;
                bytes[3] = params.low_data_rate_opt as u8;
            }
        }
        Ok(bytes)
    }
}

/// SetModulationParams command (0x8B)
///
/// Configures the modulation parameters for the selected packet type.
/// Must be called after SetPacketType and before SetPacketParams.
///
/// # Important Notes
/// - Parameters interpretation depends on packet type
/// - For LoRa, low data rate optimization should be enabled for
///   symbol times ≥ 16.38ms
/// - For GFSK, ensure bandwidth > 2*(frequency_deviation + bit_rate/2)
#[derive(Debug, Clone)]
pub struct SetModulationParams {
    /// Modulation parameters
    pub params: ModulationParams,
}

impl Command for SetModulationParams {
    type IdType = u8;
    type CommandParameters = ModulationParams;
    type ResponseParameters = NoParameters;

    fn id() -> Self::IdType {
        0x8B
    }

    fn invoking_parameters(self) -> Self::CommandParameters {
        self.params
    }
}

/// Packet parameters configuration
///
/// Parameters interpretation depends on the packet type:
///
/// # GFSK Mode
/// - params[0-1]: Preamble length in bits
/// - params[2]: Preamble detector length (0=Off, 4-7=8-32 bits)
/// - params[3]: Sync word length in bits
/// - params[4]: Address filtering (0=Off, 1=Node, 2=Node+Broadcast)
/// - params[5]: Packet type (0=Fixed, 1=Variable)
/// - params[6]: Payload length
/// - params[7]: CRC type
/// - params[8]: Whitening enable
///
/// # LoRa Mode
/// - params[0-1]: Preamble length in symbols
/// - params[2]: Header type (0=Variable/Explicit, 1=Fixed/Implicit)
/// - params[3]: Payload length
/// - params[4]: CRC enable
/// - params[5]: IQ inversion enable
#[derive(Debug, Clone)]
pub struct PacketParams {
    /// Raw packet parameters array
    pub params: [u8; 9],
}

impl ToByteArray for PacketParams {
    type Error = Infallible;
    type Array = [u8; 9];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok(self.params)
    }
}

/// SetPacketParams command (0x8C)
///
/// Configures the packet parameters for the selected packet type.
/// Must be called after SetModulationParams.
///
/// # Important Notes
/// - Parameters interpretation depends on packet type
/// - For GFSK with address filtering, max payload is 254 bytes
/// - For LoRa implicit header, payload length must match on TX/RX
/// - Preamble detector length must be shorter than sync word
#[derive(Debug, Clone)]
pub struct SetPacketParams {
    /// Packet parameters
    pub params: PacketParams,
}

impl Command for SetPacketParams {
    type IdType = u8;
    type CommandParameters = PacketParams;
    type ResponseParameters = NoParameters;

    fn id() -> Self::IdType {
        0x8C
    }

    fn invoking_parameters(self) -> Self::CommandParameters {
        self.params
    }
}

/// Channel Activity Detection (CAD) parameters
/// LoRa mode only
#[derive(Debug, Clone, Copy)]
pub struct CadParams {
    /// Number of symbols for CAD detection (0=1, 1=2, 2=4, 3=8, 4=16)
    pub cad_symbol_num: u8,
    /// Detection peak threshold
    pub cad_detect_peak: u8,
    /// Detection minimum threshold
    pub cad_detect_min: u8,
    /// Exit mode (0=CAD only, 1=CAD + RX)
    pub cad_exit_mode: u8,
    /// Timeout in 15.625μs steps (CAD_RX mode only)
    pub cad_timeout: u32,
}

impl ToByteArray for CadParams {
    type Error = Infallible;
    type Array = [u8; 8];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        let mut bytes = [0u8; 8];
        bytes[0] = self.cad_symbol_num;
        bytes[1] = self.cad_detect_peak;
        bytes[2] = self.cad_detect_min;
        bytes[3] = self.cad_exit_mode;
        bytes[4..8].copy_from_slice(&self.cad_timeout.to_be_bytes());
        Ok(bytes)
    }
}

/// SetCadParams command (0x88)
///
/// Configures the Channel Activity Detection parameters.
/// Only available in LoRa packet type.
///
/// # Important Notes
/// - CAD can detect both preamble and data symbols
/// - Detection thresholds depend on SF/BW and symbol count
/// - In CAD_RX mode, device stays in RX if activity detected
#[derive(Debug, Clone)]
pub struct SetCadParams {
    /// CAD parameters
    pub params: CadParams,
}

impl Command for SetCadParams {
    type IdType = u8;
    type CommandParameters = CadParams;
    type ResponseParameters = NoParameters;

    fn id() -> Self::IdType {
        0x88
    }

    fn invoking_parameters(self) -> Self::CommandParameters {
        self.params
    }
}

/// Buffer base addresses configuration
#[derive(Debug, Clone, Copy)]
pub struct BufferBaseAddressConfig {
    /// TX base address in data buffer (0-255)
    pub tx_base_addr: u8,
    /// RX base address in data buffer (0-255)
    pub rx_base_addr: u8,
}

impl ToByteArray for BufferBaseAddressConfig {
    type Error = Infallible;
    type Array = [u8; 2];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok([self.tx_base_addr, self.rx_base_addr])
    }
}

/// SetBufferBaseAddress command (0x8F)
///
/// Sets the base addresses for TX and RX data in the 256-byte data buffer.
///
/// # Important Notes
/// - Buffer is cleared in Sleep mode
/// - In RX, if packet exceeds allocated space it can overwrite TX area
/// - Base addresses can be anywhere in 0-255 range
#[derive(Debug, Clone)]
pub struct SetBufferBaseAddress {
    /// Buffer base addresses configuration
    pub config: BufferBaseAddressConfig,
}

impl Command for SetBufferBaseAddress {
    type IdType = u8;
    type CommandParameters = BufferBaseAddressConfig;
    type ResponseParameters = NoParameters;

    fn id() -> Self::IdType {
        0x8F
    }

    fn invoking_parameters(self) -> Self::CommandParameters {
        self.config
    }
}

/// LoRa symbol number timeout configuration
#[derive(Debug, Clone, Copy)]
pub struct LoRaSymbNumTimeout {
    /// Number of symbols to validate reception
    /// 0 = Validate on first symbol
    /// 1-255 = Wait for specified symbols before timeout
    pub symb_num: u8,
}

impl ToByteArray for LoRaSymbNumTimeout {
    type Error = Infallible;
    type Array = [u8; 1];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok([self.symb_num])
    }
}

/// SetLoRaSymbNumTimeout command (0xA0)
///
/// Sets the number of symbols to wait for valid LoRa reception.
/// Used to avoid false detection on noise.
///
/// # Important Notes
/// - Only available in LoRa packet type
/// - 0 = Accept first symbol detection
/// - >0 = Wait for specified symbols before timeout
/// - Helps prevent false detections in noisy environments
#[derive(Debug, Clone)]
pub struct SetLoRaSymbNumTimeout {
    /// LoRa symbol timeout configuration
    pub config: LoRaSymbNumTimeout,
}

impl Command for SetLoRaSymbNumTimeout {
    type IdType = u8;
    type CommandParameters = LoRaSymbNumTimeout;
    type ResponseParameters = NoParameters;

    fn id() -> Self::IdType {
        0xA0
    }

    fn invoking_parameters(self) -> Self::CommandParameters {
        self.config
    }
}
