//! Operational mode commands
//!
//! This module contains commands for controlling the radio's operating modes:
//! - Sleep mode for minimum power consumption
//! - Standby modes (RC and XOSC) for configuration
//! - Frequency synthesis mode for PLL locking
//! - Transmit and receive modes
//! - Duty cycling and timeout control
//! - Power amplifier configuration
//! - Calibration procedures
//!
//! Most configuration commands must be issued in STDBY_RC mode.
//! Mode transitions have specific timing requirements detailed in
//! the documentation for each command.

use bitflags::bitflags;
use core::convert::Infallible;

use crate::{Command, NoParameters, ToByteArray};

bitflags! {
    /// Sleep configuration options
    ///
    /// Controls behavior during sleep mode, including configuration
    /// retention and wake-up sources.
    #[derive(Debug, Clone, Copy)]
    pub struct SleepConfig: u8 {
        /// When set, configuration is retained in sleep mode (warm start)
        /// When clear, cold start - all registers reset to defaults
        const WARM_START = 1 << 2;
        /// When set, device can wake up on RTC timeout
        /// When clear, wake on NSS falling edge only
        const RTC_WAKEUP = 1;
    }
}

impl ToByteArray for SleepConfig {
    type Error = Infallible;
    type Array = [u8; 1];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok([self.bits()])
    }
}

/// SetSleep command (0x84)
///
/// Puts the radio into sleep mode for minimum power consumption.
///
/// # Important Notes
/// - Can only be issued from STDBY mode
/// - Takes ~500μs to save configuration before sleep
/// - No SPI commands should be sent during this time
/// - Device unresponsive until woken by NSS or RTC
/// - Current consumption ~160nA (cold) / 600nA (warm)
#[derive(Debug, Clone)]
pub struct SetSleep {
    /// Sleep configuration
    pub config: SleepConfig,
}

impl Command for SetSleep {
    type IdType = u8;
    type CommandParameters = SleepConfig;
    type ResponseParameters = NoParameters;

    fn id() -> Self::IdType {
        0x84
    }

    fn invoking_parameters(self) -> Self::CommandParameters {
        self.config
    }
}

/// Standby mode configuration
///
/// Selects which oscillator to use in standby mode.
#[derive(Debug, Clone, Copy)]
pub enum StandbyConfig {
    /// Device running on RC13M (~0.6mA)
    /// Used for configuration and lower power
    Rc = 0,

    /// Device running on XTAL 32MHz (~0.8mA)
    /// Required for faster transition to TX/RX
    Xosc = 1,
}

impl ToByteArray for StandbyConfig {
    type Error = Infallible;
    type Array = [u8; 1];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok([self as u8])
    }
}

/// SetStandby command (0x80)
///
/// Puts the radio into standby mode for configuration.
///
/// # Important Notes
/// - Default mode after power-up/reset is STDBY_RC
/// - Most configuration must be done in STDBY_RC
/// - STDBY_XOSC provides faster transition to TX/RX
/// - DC-DC configuration only possible in STDBY_RC
#[derive(Debug, Clone)]
pub struct SetStandby {
    /// Standby mode configuration
    pub config: StandbyConfig,
}

impl Command for SetStandby {
    type IdType = u8;
    type CommandParameters = StandbyConfig;
    type ResponseParameters = NoParameters;

    fn id() -> Self::IdType {
        0x80
    }

    fn invoking_parameters(self) -> Self::CommandParameters {
        self.config
    }
}

/// SetFs command (0xC1)
///
/// Puts the radio into frequency synthesis mode.
/// PLL is locked to the configured frequency.
///
/// # Important Notes
/// - Used for testing/debugging PLL
/// - Automatically entered during TX/RX transitions
/// - BUSY goes low when PLL locked
/// - Takes ~40μs from STDBY_XOSC
#[derive(Debug, Clone)]
pub struct SetFs;

impl Command for SetFs {
    type IdType = u8;
    type CommandParameters = NoParameters;
    type ResponseParameters = NoParameters;

    fn id() -> Self::IdType {
        0xC1
    }

    fn invoking_parameters(self) -> Self::CommandParameters {
        NoParameters::default()
    }
}

/// Timeout configuration for Tx/Rx operations
///
/// Used to automatically terminate TX/RX operations
/// after specified period.
#[derive(Debug, Clone, Copy)]
pub struct Timeout(pub u32);

impl ToByteArray for Timeout {
    type Error = Infallible;
    type Array = [u8; 4];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok(self.0.to_be_bytes())
    }
}

/// SetTx command (0x83)
///
/// Puts the radio into transmit mode.
///
/// # Important Notes
/// - PA ramps according to SetTxParams ramp time
/// - BUSY low when PA ramped and transmission starts
/// - Returns to configured fallback mode after:
///   - Packet transmitted (TxDone IRQ)
///   - Timeout period elapsed
/// - Timeout = 0x000000 disables timeout
#[derive(Debug, Clone)]
pub struct SetTx {
    /// Timeout in steps of 15.625 μs
    /// Maximum timeout is 262s
    pub timeout: Timeout,
}

impl Command for SetTx {
    type IdType = u8;
    type CommandParameters = Timeout;
    type ResponseParameters = NoParameters;

    fn id() -> Self::IdType {
        0x83
    }

    fn invoking_parameters(self) -> Self::CommandParameters {
        self.timeout
    }
}

/// RX operation mode
#[derive(Debug, Clone, Copy)]
pub enum RxMode {
    /// Return after receiving a single packet
    Single,
    /// Continuous reception until stopped by command
    Continuous,
    /// Return after timeout or packet reception
    /// Timeout in steps of 15.625 μs (max 262s)
    Timed(u32),
}

impl From<RxMode> for Timeout {
    fn from(mode: RxMode) -> Self {
        match mode {
            RxMode::Single => Timeout(0x000000),
            RxMode::Continuous => Timeout(0xFFFFFF),
            RxMode::Timed(timeout) => Timeout(timeout),
        }
    }
}

/// SetRx command (0x82)
///
/// Puts the radio into receive mode.
///
/// # Important Notes
/// - BUSY low when RX enabled and ready
/// - Returns to configured fallback mode after:
///   - Packet received (RxDone IRQ)
///   - Timeout period elapsed (Timed mode only)
/// - Timeout disabled once valid packet detected
#[derive(Debug, Clone)]
pub struct SetRx {
    /// RX operation mode
    pub mode: RxMode,
}

impl Command for SetRx {
    type IdType = u8;
    type CommandParameters = Timeout;
    type ResponseParameters = NoParameters;

    fn id() -> Self::IdType {
        0x82
    }

    fn invoking_parameters(self) -> Self::CommandParameters {
        self.mode.into()
    }
}

bitflags! {
    /// StopTimerOnPreamble configuration
    ///
    /// Controls when RX timeout timer is stopped.
    #[derive(Debug, Clone, Copy)]
    pub struct StopTimerOnPreambleConfig: u8 {
        /// When set, stop timer on preamble detection
        /// When clear, stop on Sync/Header (default)
        const STOP_ON_PREAMBLE = 1;
    }
}

impl ToByteArray for StopTimerOnPreambleConfig {
    type Error = Infallible;
    type Array = [u8; 1];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok([self.bits()])
    }
}

/// StopTimerOnPreamble command (0x9F)
///
/// Configures when RX timeout timer is stopped.
///
/// # Important Notes
/// - Default is to stop on Sync/Header detection
/// - Stopping on preamble may cause extended RX
///   if false detection occurs
#[derive(Debug, Clone)]
pub struct StopTimerOnPreamble {
    /// Stop on preamble configuration
    pub config: StopTimerOnPreambleConfig,
}

impl Command for StopTimerOnPreamble {
    type IdType = u8;
    type CommandParameters = StopTimerOnPreambleConfig;
    type ResponseParameters = NoParameters;

    fn id() -> Self::IdType {
        0x9F
    }

    fn invoking_parameters(self) -> Self::CommandParameters {
        self.config
    }
}

/// RxDutyCycle configuration
///
/// Controls periodic wake-up for packet reception.
#[derive(Debug, Clone, Copy)]
pub struct RxDutyCycleConfig {
    /// RX period in steps of 15.625 μs
    /// Time radio spends in RX mode
    pub rx_period: u32,

    /// Sleep period in steps of 15.625 μs
    /// Time radio spends in sleep mode
    pub sleep_period: u32,
}

impl ToByteArray for RxDutyCycleConfig {
    type Error = Infallible;
    type Array = [u8; 8];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        let mut bytes = [0u8; 8];
        bytes[0..4].copy_from_slice(&self.rx_period.to_be_bytes());
        bytes[4..8].copy_from_slice(&self.sleep_period.to_be_bytes());
        Ok(bytes)
    }
}

/// SetRxDutyCycle command (0x94)
///
/// Configures periodic wake-up for packet reception.
///
/// # Important Notes
/// - Context saved before sleep
/// - ~1ms overhead for save/restore
/// - Timer restarted with 2*rx + sleep on preamble
/// - Loop terminates on:
///   - Packet received (RxDone IRQ)
///   - SetStandby command during RX
#[derive(Debug, Clone)]
pub struct SetRxDutyCycle {
    /// Duty cycle configuration
    pub config: RxDutyCycleConfig,
}

impl Command for SetRxDutyCycle {
    type IdType = u8;
    type CommandParameters = RxDutyCycleConfig;
    type ResponseParameters = NoParameters;

    fn id() -> Self::IdType {
        0x94
    }

    fn invoking_parameters(self) -> Self::CommandParameters {
        self.config
    }
}

/// SetCad command (0xC5)
///
/// Puts radio into Channel Activity Detection mode.
/// LoRa mode only.
///
/// # Important Notes
/// - Detects LoRa preamble or data symbols
/// - Returns to STDBY_RC after detection
/// - Triggers CADDone and optionally CADDetected IRQs
/// - Parameters set by SetCadParams command
#[derive(Debug, Clone)]
pub struct SetCad;

impl Command for SetCad {
    type IdType = u8;
    type CommandParameters = NoParameters;
    type ResponseParameters = NoParameters;

    fn id() -> Self::IdType {
        0xC5
    }

    fn invoking_parameters(self) -> Self::CommandParameters {
        NoParameters::default()
    }
}

/// SetTxContinuousWave command (0xD1)
///
/// Puts radio into continuous wave (RF tone) transmission.
///
/// # Important Notes
/// - Test command for measuring RF performance
/// - Transmits unmodulated carrier at set frequency
/// - Stays in TX until mode changed by command
#[derive(Debug, Clone)]
pub struct SetTxContinuousWave;

impl Command for SetTxContinuousWave {
    type IdType = u8;
    type CommandParameters = NoParameters;
    type ResponseParameters = NoParameters;

    fn id() -> Self::IdType {
        0xD1
    }

    fn invoking_parameters(self) -> Self::CommandParameters {
        NoParameters::default()
    }
}

/// SetTxInfinitePreamble command (0xD2)
///
/// Puts radio into infinite preamble transmission.
///
/// # Important Notes
/// - Test command for measuring modulation
/// - FSK: Alternating 0/1 sequence
/// - LoRa: Continuous preamble symbols
/// - Stays in TX until mode changed by command
#[derive(Debug, Clone)]
pub struct SetTxInfinitePreamble;

impl Command for SetTxInfinitePreamble {
    type IdType = u8;
    type CommandParameters = NoParameters;
    type ResponseParameters = NoParameters;

    fn id() -> Self::IdType {
        0xD2
    }

    fn invoking_parameters(self) -> Self::CommandParameters {
        NoParameters::default()
    }
}

/// Regulator mode configuration
///
/// Selects voltage regulator configuration.
#[derive(Debug, Clone, Copy)]
pub enum RegulatorMode {
    /// Only LDO used for all modes
    /// - Lower cost (no inductor needed)
    /// - Higher power consumption
    LdoOnly = 0,

    /// DC-DC+LDO used for STBY_XOSC, FS, RX and TX
    /// - Requires external inductor
    /// - ~50% lower power consumption
    /// - LDO remains active as backup
    DcDcLdo = 1,
}

impl ToByteArray for RegulatorMode {
    type Error = Infallible;
    type Array = [u8; 1];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok([self as u8])
    }
}

/// SetRegulatorMode command (0x96)
///
/// Configures the voltage regulator mode.
///
/// # Important Notes
/// - Must be configured in STDBY_RC mode
/// - DC-DC requires 15μH inductor
/// - LDO remains active with DC-DC as backup
/// - Mode persists until changed or sleep
#[derive(Debug, Clone)]
pub struct SetRegulatorMode {
    /// Regulator mode selection
    pub mode: RegulatorMode,
}

impl Command for SetRegulatorMode {
    type IdType = u8;
    type CommandParameters = RegulatorMode;
    type ResponseParameters = NoParameters;

    fn id() -> Self::IdType {
        0x96
    }

    fn invoking_parameters(self) -> Self::CommandParameters {
        self.mode
    }
}

bitflags! {
    /// Calibration configuration
    ///
    /// Selects which blocks to calibrate.
    #[derive(Debug, Clone, Copy)]
    pub struct CalibrationConfig: u8 {
        /// RC64k oscillator calibration
        const RC64K = 1 << 0;
        /// RC13M oscillator calibration
        const RC13M = 1 << 1;
        /// PLL calibration
        const PLL = 1 << 2;
        /// ADC pulse calibration
        const ADC_PULSE = 1 << 3;
        /// ADC bulk N calibration
        const ADC_BULK_N = 1 << 4;
        /// ADC bulk P calibration
        const ADC_BULK_P = 1 << 5;
        /// Image rejection calibration
        const IMAGE = 1 << 6;
    }
}

impl ToByteArray for CalibrationConfig {
    type Error = Infallible;
    type Array = [u8; 1];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok([self.bits()])
    }
}

/// Calibrate command (0x89)
///
/// Triggers calibration of selected blocks.
///
/// # Important Notes
/// - Must be called in STDBY_RC mode
/// - Takes up to 3.5ms for full calibration
/// - BUSY high during calibration
/// - Automatically performed at power-up
/// - Required after configuration changes
#[derive(Debug, Clone)]
pub struct Calibrate {
    /// Calibration configuration
    pub config: CalibrationConfig,
}

impl Command for Calibrate {
    type IdType = u8;
    type CommandParameters = CalibrationConfig;
    type ResponseParameters = NoParameters;

    fn id() -> Self::IdType {
        0x89
    }

    fn invoking_parameters(self) -> Self::CommandParameters {
        self.config
    }
}

/// Image calibration configuration
///
/// Defines frequency range for image calibration.
#[derive(Debug, Clone, Copy)]
pub struct ImageCalibConfig {
    /// Start frequency code
    pub freq1: u8,
    /// Stop frequency code
    pub freq2: u8,
}

impl ToByteArray for ImageCalibConfig {
    type Error = Infallible;
    type Array = [u8; 2];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok([self.freq1, self.freq2])
    }
}

/// CalibrateImage command (0x98)
///
/// Calibrates image rejection for frequency range.
///
/// # Important Notes
/// - Must be called in STDBY_RC mode
/// - Calibration valid between freq1 and freq2
/// - Default calibration for 902-928MHz
/// - Required after changing frequency band
/// - Special handling needed with TCXO
#[derive(Debug, Clone)]
pub struct CalibrateImage {
    /// Image calibration configuration
    pub config: ImageCalibConfig,
}

impl Command for CalibrateImage {
    type IdType = u8;
    type CommandParameters = ImageCalibConfig;
    type ResponseParameters = NoParameters;

    fn id() -> Self::IdType {
        0x98
    }

    fn invoking_parameters(self) -> Self::CommandParameters {
        self.config
    }
}

/// Device selection for PA configuration
#[derive(Debug, Clone, Copy)]
pub enum DeviceSelect {
    /// SX1262 device (+22dBm max)
    Sx1262 = 0,
    /// SX1261 device (+15dBm max)
    Sx1261 = 1,
}

/// PA configuration parameters
#[derive(Debug, Clone, Copy)]
pub struct PaConfig {
    /// PA duty cycle (controls efficiency)
    /// See datasheet for optimal values
    pub duty_cycle: u8,

    /// HP max (SX1262 only)
    /// Controls maximum output power
    /// Range 0x00-0x07
    pub hp_max: u8,

    /// Device selection
    pub device_sel: DeviceSelect,

    /// PA LUT (always 0x01)
    pub pa_lut: u8,
}

impl ToByteArray for PaConfig {
    type Error = Infallible;
    type Array = [u8; 4];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok([
            self.duty_cycle,
            self.hp_max,
            self.device_sel as u8,
            self.pa_lut,
        ])
    }
}

/// SetPaConfig command (0x95)
///
/// Configures the power amplifier.
///
/// # Important Notes
/// - Must be configured before SetTxParams
/// - Different optimal settings for power levels
/// - Affects efficiency and harmonics
/// - SX1261: duty_cycle ≤ 0x04 below 400MHz
/// - SX1262: duty_cycle ≤ 0x04 all frequencies
#[derive(Debug, Clone)]
pub struct SetPaConfig {
    /// PA configuration
    pub config: PaConfig,
}

impl Command for SetPaConfig {
    type IdType = u8;
    type CommandParameters = PaConfig;
    type ResponseParameters = NoParameters;

    fn id() -> Self::IdType {
        0x95
    }

    fn invoking_parameters(self) -> Self::CommandParameters {
        self.config
    }
}

/// Fallback mode after Rx/Tx
///
/// Defines mode to enter after packet operation.
#[derive(Debug, Clone, Copy)]
pub enum FallbackMode {
    /// Go to FS mode
    /// Fastest transition to next TX/RX
    Fs = 0x40,

    /// Go to STDBY_XOSC mode
    /// Medium power, medium transition
    StdbyXosc = 0x30,

    /// Go to STDBY_RC mode (default)
    /// Lowest power, slowest transition
    StdbyRc = 0x20,
}

impl ToByteArray for FallbackMode {
    type Error = Infallible;
    type Array = [u8; 1];

    fn to_bytes(self) -> Result<Self::Array, Self::Error> {
        Ok([self as u8])
    }
}

/// SetRxTxFallbackMode command (0x93)
///
/// Configures mode entered after RX/TX operation.
///
/// # Important Notes
/// - Default is STDBY_RC
/// - FS mode allows fastest transition
/// - Affects power consumption when idle
/// - Takes effect after TxDone/RxDone/Timeout
#[derive(Debug, Clone)]
pub struct SetRxTxFallbackMode {
    /// Fallback mode selection
    pub mode: FallbackMode,
}

impl Command for SetRxTxFallbackMode {
    type IdType = u8;
    type CommandParameters = FallbackMode;
    type ResponseParameters = NoParameters;

    fn id() -> Self::IdType {
        0x93
    }

    fn invoking_parameters(self) -> Self::CommandParameters {
        self.mode
    }
}
