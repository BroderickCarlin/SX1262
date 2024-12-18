#![no_std]
//! SX126x Radio Driver
//!
//! This crate provides a type-safe interface for the Semtech SX1261/2 sub-GHz radio transceivers.
//! The SX1261/2 are highly integrated, long range, low power radio transceivers designed for use
//! in ISM band applications.
//!
//! # Features
//! - Frequency range: 150-960 MHz
//! - Modulation support:
//!   - LoRa: SF5-12, BW 7.8-500kHz
//!   - (G)FSK: BR 0.6-300kbps
//! - Output power:
//!   - SX1261: -17 to +15 dBm
//!   - SX1262: -9 to +22 dBm
//! - Receive sensitivity down to -148 dBm
//! - Integrated voltage regulation (DC-DC or LDO)
//! - Programmable DIO pins for interrupts and control
//!
//! # Architecture
//! The driver is organized into several modules:
//!
//! - [`device`]: Main device interface for hardware interaction
//!   - Provides high-level API for register access and command execution
//!   - Manages SPI communication with the radio
//!
//! - [`registers`]: Register definitions for direct hardware access
//!   - [`registers::rf`]: RF-related registers (frequency, power, etc)
//!   - [`registers::packet`]: Packet handling registers
//!   - [`registers::dio`]: Digital I/O configuration registers
//!   - [`registers::system`]: System configuration registers
//!
//! - [`commands`]: Command interface for radio control
//!   - [`commands::rf`]: RF and modulation configuration
//!   - [`commands::dio`]: DIO and interrupt control
//!   - [`commands::operational`]: Operating mode control
//!   - [`commands::status`]: Status monitoring and statistics
//!
//! # Usage
//! The driver uses the `regiface` crate to provide a type-safe interface
//! for register access and command execution. The main entry point is the
//! [`Device`] struct which wraps an SPI interface and provides methods for
//! interacting with the radio.
//!
//! Configuration follows a specific sequence:
//!
//! 1. Create a new [`Device`] instance with your SPI interface
//! 2. Set operating mode (STDBY_RC for configuration)
//! 3. Configure packet type (LoRa/FSK)
//! 4. Set RF frequency and modulation parameters
//! 5. Configure packet format and processing
//! 6. Set up DIO pins and interrupts
//! 7. Enter RX/TX mode for operation
//!
//! # Important Notes
//! - Most configuration must be done in STDBY_RC mode
//! - Packet type must be set before other RF configuration
//! - PA configuration depends on device type (SX1261/2)
//! - TCXO configuration requires special handling
//! - Some registers have interdependencies
//!
//! # Example
//! ```no_run
//! use embedded_hal::spi::SpiDevice;
//! use sx1262::{Device, commands::{SetStandby, StandbyConfig}, Error};
//!
//! fn configure_radio<SPI: SpiDevice>(spi: SPI) -> Result<Device<SPI>, Error> {
//!     let mut device = Device::new(spi);
//!     
//!     // Set to STDBY_RC mode for configuration
//!     device.execute_command( SetStandby { config: StandbyConfig::Rc})?;
//!     
//!     Ok(device)
//! }
//! ```

pub use regiface::errors::Error;
use regiface::*;

pub mod commands;
pub mod device;
pub mod registers;

pub use commands::*;
pub use device::Device;
pub use registers::*;
