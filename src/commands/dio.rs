//! DIO and IRQ control commands
//!
//! This module contains commands for configuring and controlling:
//! - DIO pin mapping and functionality
//! - IRQ generation and handling
//! - RF switch control via DIO2
//! - TCXO control via DIO3
//!
//! The SX126x has 3 configurable DIO pins and 10 possible interrupt sources.
//! Each interrupt can be mapped to any DIO pin, and multiple interrupts
//! can be mapped to the same pin (OR function).

use core::convert::Infallible;

use crate::{Command, FromByteArray, NoParameters, ToByteArray};

bitflags::bitflags! {
    /// IRQ flags for the SX126x radio
    #[derive(Debug, Clone, Copy)]
    pub struct IrqMask: u16 {
        /// Packet transmission completed
        const TX_DONE = 1 << 0;
        /// Packet received
        const RX_DONE = 1 << 1;
        /// Preamble detected
        const PREAMBLE_DETECTED = 1 << 2;
        /// Valid sync word detected (FSK) / Valid header received (LoRa)
        const SYNC_WORD_VALID = 1 << 3;
        /// Header CRC error (LoRa)
        const HEADER_ERROR = 1 << 4;
        /// Payload CRC error
        const CRC_ERROR = 1 << 5;
        /// Channel activity detection finished
        const CAD_DONE = 1 << 6;
        /// Channel activity detected
        const CAD_DETECTED = 1 << 7;
        /// RX or TX timeout
        const TIMEOUT = 1 << 8;
    }
}

impl ToByteArray for IrqMask {
    type Error = Infallible;
    type Array = [u8; 2];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok(self.bits().to_be_bytes())
    }
}

impl FromByteArray for IrqMask {
    type Error = Infallible;
    type Array = [u8; 2];

    fn from_bytes(bytes: Self::Array) -> Result<Self, Self::Error> {
        Ok(IrqMask::from_bits_truncate(u16::from_be_bytes(bytes)))
    }
}

/// DIO and IRQ configuration parameters
///
/// Used to configure which interrupts are enabled and how they
/// are mapped to DIO pins.
#[derive(Debug, Clone, Copy)]
pub struct DioIrqConfig {
    /// IRQ enable mask
    pub irq_mask: IrqMask,

    /// DIO1 interrupt mapping mask
    /// IRQ appears on DIO1 if corresponding bits set in both masks
    pub dio1_mask: IrqMask,

    /// DIO2 interrupt mapping mask
    /// IRQ appears on DIO2 if corresponding bits set in both masks
    /// Ignored if DIO2 configured for RF switch control
    pub dio2_mask: IrqMask,

    /// DIO3 interrupt mapping mask
    /// IRQ appears on DIO3 if corresponding bits set in both masks
    /// Ignored if DIO3 configured for TCXO control
    pub dio3_mask: IrqMask,
}

impl ToByteArray for DioIrqConfig {
    type Error = Infallible;
    type Array = [u8; 8];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        let mut bytes = [0u8; 8];
        bytes[0..2].copy_from_slice(&self.irq_mask.bits().to_be_bytes());
        bytes[2..4].copy_from_slice(&self.dio1_mask.bits().to_be_bytes());
        bytes[4..6].copy_from_slice(&self.dio2_mask.bits().to_be_bytes());
        bytes[6..8].copy_from_slice(&self.dio3_mask.bits().to_be_bytes());
        Ok(bytes)
    }
}

/// SetDioIrqParams command (0x08)
///
/// Configures the mapping between interrupt sources and DIO pins.
///
/// # Important Notes
/// - IRQs must be enabled in irq_mask to be generated
/// - IRQ must be mapped to a DIO to appear on that pin
/// - Multiple IRQs can be mapped to same DIO (OR function)
/// - DIO2/3 mappings ignored if used for RF switch/TCXO
#[derive(Debug, Clone)]
pub struct SetDioIrqParams {
    /// DIO and IRQ configuration parameters
    pub config: DioIrqConfig,
}

impl Command for SetDioIrqParams {
    type IdType = u8;
    type CommandParameters = DioIrqConfig;
    type ResponseParameters = NoParameters;

    fn id() -> Self::IdType {
        0x08
    }

    fn invoking_parameters(self) -> Self::CommandParameters {
        self.config
    }
}

/// GetIrqStatus command (0x12)
///
/// Returns the current state of all interrupt flags.
/// Each bit corresponds to an interrupt source as defined
/// in IrqMask.
///
/// # Important Notes
/// - Flags remain set until explicitly cleared
/// - Reading status does not clear flags
/// - Use ClearIrqStatus to clear flags
#[derive(Debug, Clone)]
pub struct GetIrqStatus;

impl Command for GetIrqStatus {
    type IdType = u8;
    type CommandParameters = NopParameters<1>;
    type ResponseParameters = IrqMask;

    fn id() -> Self::IdType {
        0x12
    }

    fn invoking_parameters(self) -> Self::CommandParameters {
        Self::CommandParameters::default()
    }
}

/// ClearIrqStatus command (0x02)
///
/// Clears specified interrupt flags.
///
/// # Important Notes
/// - Only clears flags with corresponding mask bits set
/// - Multiple flags can be cleared in single command
/// - Clearing flag removes it from IRQ register and DIO
#[derive(Debug, Clone)]
pub struct ClearIrqStatus {
    /// IRQ flags to clear
    pub irq_mask: IrqMask,
}

impl Command for ClearIrqStatus {
    type IdType = u8;
    type CommandParameters = IrqMask;
    type ResponseParameters = NoParameters;

    fn id() -> Self::IdType {
        0x02
    }

    fn invoking_parameters(self) -> Self::CommandParameters {
        self.irq_mask
    }
}

/// RF switch control configuration
#[derive(Debug, Clone, Copy)]
pub struct RfSwitchConfig {
    /// Enable RF switch control on DIO2
    /// - true = DIO2 controls RF switch
    /// - false = DIO2 available for IRQ mapping
    pub enable: bool,
}

impl ToByteArray for RfSwitchConfig {
    type Error = Infallible;
    type Array = [u8; 1];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok([self.enable as u8])
    }
}

/// SetDio2AsRfSwitchCtrl command (0x9D)
///
/// Configures DIO2 to automatically control an RF switch.
///
/// # Important Notes
/// - When enabled, DIO2 = 1 in TX mode, 0 otherwise
/// - Overrides any IRQ mapping to DIO2
/// - DIO2 changes state a few μs before PA ramp-up/down
#[derive(Debug, Clone)]
pub struct SetDio2AsRfSwitchCtrl {
    /// RF switch configuration
    pub config: RfSwitchConfig,
}

impl Command for SetDio2AsRfSwitchCtrl {
    type IdType = u8;
    type CommandParameters = RfSwitchConfig;
    type ResponseParameters = NoParameters;

    fn id() -> Self::IdType {
        0x9D
    }

    fn invoking_parameters(self) -> Self::CommandParameters {
        self.config
    }
}

/// TCXO voltage options
///
/// Available voltage options for TCXO power supply.
/// VBAT must be at least 200mV higher than selected voltage.
#[derive(Debug, Clone, Copy)]
pub enum TcxoVoltage {
    /// 1.6V (min VBAT = 1.8V)
    V1_6 = 0x00,
    /// 1.7V (min VBAT = 1.9V)
    V1_7 = 0x01,
    /// 1.8V (min VBAT = 2.0V)
    V1_8 = 0x02,
    /// 2.2V (min VBAT = 2.4V)
    V2_2 = 0x03,
    /// 2.4V (min VBAT = 2.6V)
    V2_4 = 0x04,
    /// 2.7V (min VBAT = 2.9V)
    V2_7 = 0x05,
    /// 3.0V (min VBAT = 3.2V)
    V3_0 = 0x06,
    /// 3.3V (min VBAT = 3.5V)
    V3_3 = 0x07,
}

/// TCXO control configuration
#[derive(Debug, Clone, Copy)]
pub struct TcxoConfig {
    /// TCXO supply voltage
    pub voltage: TcxoVoltage,

    /// Timeout in steps of 15.625 μs
    /// Time to wait for TCXO to stabilize
    /// Chip waits this long after enabling TCXO
    /// before starting operation
    pub delay: u32,
}

impl ToByteArray for TcxoConfig {
    type Error = Infallible;
    type Array = [u8; 5];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        let mut bytes = [0u8; 5];
        bytes[0] = self.voltage as u8;
        bytes[1..5].copy_from_slice(&self.delay.to_be_bytes());
        Ok(bytes)
    }
}

/// SetDio3AsTcxoCtrl command (0x97)
///
/// Configures DIO3 to control an external TCXO.
///
/// # Important Notes
/// - DIO3 provides regulated voltage for TCXO
/// - VBAT must be ≥ voltage + 200mV
/// - Up to 4mA available for TCXO
/// - ~70μA quiescent current
/// - Takes up to 100μs to reach regulated voltage
/// - Chip waits specified delay after enabling TCXO
/// - Overrides any IRQ mapping to DIO3
/// - Complete reset required to return to XOSC mode
#[derive(Debug, Clone)]
pub struct SetDio3AsTcxoCtrl {
    /// TCXO configuration
    pub config: TcxoConfig,
}

impl Command for SetDio3AsTcxoCtrl {
    type IdType = u8;
    type CommandParameters = TcxoConfig;
    type ResponseParameters = NoParameters;

    fn id() -> Self::IdType {
        0x97
    }

    fn invoking_parameters(self) -> Self::CommandParameters {
        self.config
    }
}

/// NopParameters for use with read commands that send few zeros
/// before the data are ready. One example of such command is
/// GetIrqStatus.
#[derive(Debug, Copy, Clone, Default)]
pub struct NopParameters<const N: usize>;

impl<const N: usize> ToByteArray for NopParameters<N> {
    type Error = Infallible;

    type Array = [u8; N];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok([0; N])
    }
}
impl<const N: usize> FromByteArray for NopParameters<N> {
    type Error = Infallible;

    type Array = [u8; N];

    fn from_bytes(_bytes: Self::Array) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}
