# SX1262 Radio Driver

A type-safe embedded-hal driver for the Semtech SX1261/2 sub-GHz radio transceivers. This crate provides a robust interface for controlling these highly integrated, long range, low power radio transceivers designed for ISM band applications.

## Features

- **Frequency Range**: 150-960 MHz
- **Modulation Support**:
  - LoRa: SF5-12, BW 7.8-500kHz
  - (G)FSK: BR 0.6-300kbps
- **Output Power**:
  - SX1261: -17 to +15 dBm
  - SX1262: -9 to +22 dBm
- **High Sensitivity**: Down to -148 dBm
- **Power Management**: Integrated voltage regulation (DC-DC or LDO)
- **Flexible I/O**: Programmable DIO pins for interrupts and control
- **`no_std` Compatible**: Suitable for embedded systems
- **Type-safe Interface**: Built on `regiface` for reliable register access

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
sx1262 = "0.1.0"
```

## Usage

The main entry point is the `Device` type which wraps an SPI interface and provides methods for register access and command execution:

```rust
use sx1262::{Device, commands::operational::SetStandby};
use embedded_hal::spi::SpiDevice;

fn configure_radio<SPI: SpiDevice>(spi: SPI) -> Result<Device<SPI>, SPI::Error> {
    // Create new device instance
    let mut device = Device::new(spi);
    
    // Read/write registers
    let reg_value = device.read_register(/* register */)?;
    device.write_register(/* register */)?;
    
    // Execute commands
    device.execute_command(/* command */)?;
    
    Ok(device)
}
```

The driver is organized into modules for registers and commands:

- **device**: Main interface for hardware interaction
  - Wraps SPI communication
  - Provides register access methods
  - Handles command execution

- **registers**: Hardware register definitions
  - rf: Frequency, power, etc.
  - packet: Packet handling
  - dio: Digital I/O configuration
  - system: System configuration

- **commands**: Control interface
  - rf: RF/modulation configuration
  - dio: Interrupt control
  - operational: Mode control
  - status: Monitoring and statistics

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
