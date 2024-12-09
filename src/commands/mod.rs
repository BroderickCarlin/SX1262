//! Radio command implementations
//!
//! This module contains the implementation of all SX126x radio commands.
//! Commands are organized into functional categories:
//!
//! # Command Categories
//! - [`dio`]: DIO and IRQ control commands
//!   - Configure DIO pin functionality
//!   - Map and control interrupts
//!   - Control RF switch and TCXO
//!
//! - [`operational`]: Operating mode control commands
//!   - Set operating modes (Sleep, Standby, etc)
//!   - Configure power management
//!   - Control calibration
//!   - Configure PA operation
//!
//! - [`rf`]: RF and packet configuration commands
//!   - Set frequency and modulation
//!   - Configure packet formatting
//!   - Control TX/RX parameters
//!   - Manage data buffering
//!
//! - [`status`]: Status and monitoring commands
//!   - Read device status
//!   - Monitor signal strength
//!   - Track packet statistics
//!   - Handle error conditions
//!
//! # Command Execution
//! Most commands have specific requirements for execution:
//! - Operating mode (usually STDBY_RC)
//! - Command sequencing (e.g. packet type before modulation)
//! - Parameter validation
//! - Timing constraints
//!
//! The BUSY line indicates when commands can be issued:
//! - High = Device busy, wait before sending command
//! - Low = Device ready for next command
//!
//! # Common Patterns
//! 1. Check BUSY is low before sending command
//! 2. Send command with required parameters
//! 3. Wait for BUSY to go low again
//! 4. Check status/errors if needed
//! 5. Proceed with next configuration step
//!
//! # Important Notes
//! - Commands cannot be sent during sleep mode
//! - Some commands require specific timing gaps
//! - Parameter ranges depend on operating conditions
//! - Error checking is recommended for critical commands
//! - BUSY must be monitored for reliable operation

mod dio;
mod operational;
mod rf;
mod status;

pub use dio::*;
pub use operational::*;
pub use rf::*;
pub use status::*;
