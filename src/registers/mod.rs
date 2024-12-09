//! Register definitions for the SX126x radio
//! Generated from DS_SX1261-2_V1.2.pdf datasheet

mod dio;
mod packet;
mod rf;
mod system;

pub use dio::*;
pub use packet::*;
pub use rf::*;
pub use system::*;
