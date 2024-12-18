//! SX126x Radio Device Interface
//! 
//! This module provides a high-level interface for interacting with SX126x series radio devices
//! through SPI communication. It supports both synchronous and asynchronous operations.
//! 
//! The interface is built around the `Device<SPI>` struct which wraps an SPI interface and
//! provides methods for:
//! - Reading and writing device registers
//! - Reading and writing to the device's buffer
//! - Executing radio commands
//! 
//! # Example
//! ```no_run
//! use sx126x::Device;
//! 
//! // Create device with SPI interface
//! let spi = // ... SPI implementation
//! let mut device = Device::new(spi);
//! 
//! // Read a register
//! let value: SomeRegister = device.read_register()?;
//! 
//! // Write to buffer
//! device.write_buffer(0, &[0x01, 0x02, 0x03])?;
//! ```

use core::convert::Infallible;

use regiface::{
    errors::Error as RegifaceError, ByteArray, Command, FromByteArray, ReadableRegister,
    ToByteArray, WritableRegister,
};

/// Main device interface for the SX126x radio.
/// 
/// This struct wraps an SPI interface and provides methods to interact with the radio.
/// It supports both synchronous operations through the embedded-hal traits and
/// asynchronous operations through embedded-hal-async.
pub struct Device<SPI> {
    spi: SPI,
}

impl<SPI> Device<SPI> {
    /// Creates a new Device instance wrapping the provided SPI interface.
    /// 
    /// # Arguments
    /// * `spi` - An SPI interface implementing the required embedded-hal traits
    pub fn new(spi: SPI) -> Self {
        Self { spi }
    }

    /// Releases the underlying SPI device.
    /// 
    /// This method consumes the Device instance and returns the wrapped SPI interface.
    pub fn release(self) -> SPI {
        self.spi
    }
}

impl<SPI> Device<SPI>
where
    SPI: embedded_hal::spi::SpiDevice,
{
    /// Reads a register value from the device.
    /// 
    /// # Type Parameters
    /// * `R` - Register type implementing ReadableRegister with u16 ID
    /// 
    /// # Errors
    /// * `RegifaceError::BusError` - SPI communication failed
    /// * `RegifaceError::DeserializationError` - Failed to parse register value
    pub fn read_register<R>(&mut self) -> Result<R, RegifaceError>
    where
        R: ReadableRegister<IdType = u16>,
    {
        let header = &mut [0x1D, 0x00, 0x00, 0x00];
        header[1..].copy_from_slice(&R::id().to_be_bytes());

        let mut raw_value = R::Array::new();

        self.spi
            .transaction(&mut [
                embedded_hal::spi::Operation::Write(header.as_slice()),
                embedded_hal::spi::Operation::Read(raw_value.as_mut()),
            ])
            .map_err(|_| RegifaceError::BusError)?;

        R::from_bytes(raw_value).map_err(|_| RegifaceError::DeserializationError)
    }

    /// Writes a value to a device register.
    /// 
    /// # Type Parameters
    /// * `R` - Register type implementing WritableRegister with u16 ID
    /// 
    /// # Arguments
    /// * `register` - The register value to write
    /// 
    /// # Errors
    /// * `RegifaceError::BusError` - SPI communication failed
    pub fn write_register<R>(&mut self, register: R) -> Result<(), RegifaceError>
    where
        R: WritableRegister<IdType = u16, Error = Infallible>,
    {
        let header = &mut [0x0D, 0x00, 0x00];
        header[1..].copy_from_slice(&R::id().to_be_bytes());

        let raw_value = register.to_bytes().unwrap();

        self.spi
            .transaction(&mut [
                embedded_hal::spi::Operation::Write(header.as_slice()),
                embedded_hal::spi::Operation::Write(raw_value.as_ref()),
            ])
            .map_err(|_| RegifaceError::BusError)
    }

    /// Writes bytes to the device's buffer at a specified offset.
    /// 
    /// # Arguments
    /// * `offset` - Starting position in the buffer
    /// * `bytes` - Data to write
    /// 
    /// # Errors
    /// * `RegifaceError::BusError` - SPI communication failed
    pub fn write_buffer(&mut self, offset: u8, bytes: &[u8]) -> Result<(), RegifaceError> {
        let header = &mut [0x0E, offset];

        self.spi
            .transaction(&mut [
                embedded_hal::spi::Operation::Write(header.as_slice()),
                embedded_hal::spi::Operation::Write(bytes),
            ])
            .map_err(|_| RegifaceError::BusError)
    }

    /// Reads bytes from the device's buffer starting at a specified offset.
    /// 
    /// # Arguments
    /// * `offset` - Starting position in the buffer to read from
    /// * `bytes` - Buffer to store read data
    /// 
    /// # Errors
    /// * `RegifaceError::BusError` - SPI communication failed
    pub fn read_buffer(&mut self, offset: u8, bytes: &mut [u8]) -> Result<(), RegifaceError> {
        let header = &mut [0x1E, offset, 0x00];

        self.spi
            .transaction(&mut [
                embedded_hal::spi::Operation::Write(header.as_slice()),
                embedded_hal::spi::Operation::Read(bytes),
            ])
            .map_err(|_| RegifaceError::BusError)
    }

    /// Executes a command on the device.
    /// 
    /// # Type Parameters
    /// * `C` - Command type implementing the Command trait with u8 ID
    /// 
    /// # Arguments
    /// * `command` - The command to execute
    /// 
    /// # Returns
    /// Command response parameters on success
    /// 
    /// # Errors
    /// * `RegifaceError::BusError` - SPI communication failed
    /// * `RegifaceError::DeserializationError` - Failed to parse command response
    pub fn execute_command<C>(&mut self, command: C) -> Result<C::ResponseParameters, RegifaceError>
    where
        C: Command<IdType = u8>,
        C::CommandParameters: ToByteArray<Error = Infallible>,
    {
        let request = command.invoking_parameters().to_bytes().unwrap();
        let mut raw_response = <C::ResponseParameters as FromByteArray>::Array::new();

        self.spi
            .transaction(&mut [
                embedded_hal::spi::Operation::Write(&[C::id()]),
                embedded_hal::spi::Operation::Write(request.as_ref()),
                embedded_hal::spi::Operation::Read(raw_response.as_mut()),
            ])
            .map_err(|_| RegifaceError::BusError)?;

        C::ResponseParameters::from_bytes(raw_response)
            .map_err(|_| RegifaceError::DeserializationError)
    }
}

impl<SPI> Device<SPI>
where
    SPI: embedded_hal_async::spi::SpiDevice,
{
    /// Asynchronously reads a register value from the device.
    /// 
    /// This is the async version of [`read_register`](Device::read_register).
    pub async fn read_register_async<R>(&mut self) -> Result<R, RegifaceError>
    where
        R: ReadableRegister<IdType = u16>,
    {
        let header = &mut [0x1D, 0x00, 0x00, 0x00];
        header[1..].copy_from_slice(&R::id().to_be_bytes());

        let mut raw_value = R::Array::new();

        self.spi
            .transaction(&mut [
                embedded_hal_async::spi::Operation::Write(header.as_slice()),
                embedded_hal_async::spi::Operation::Read(raw_value.as_mut()),
            ])
            .await
            .map_err(|_| RegifaceError::BusError)?;

        R::from_bytes(raw_value).map_err(|_| RegifaceError::DeserializationError)
    }

    /// Asynchronously writes a value to a device register.
    /// 
    /// This is the async version of [`write_register`](Device::write_register).
    pub async fn write_register_async<R>(&mut self, register: R) -> Result<(), RegifaceError>
    where
        R: WritableRegister<IdType = u16, Error = Infallible>,
    {
        let header = &mut [0x0D, 0x00, 0x00];
        header[1..].copy_from_slice(&R::id().to_be_bytes());

        let raw_value = register.to_bytes().unwrap();

        self.spi
            .transaction(&mut [
                embedded_hal_async::spi::Operation::Write(header.as_slice()),
                embedded_hal_async::spi::Operation::Write(raw_value.as_ref()),
            ])
            .await
            .map_err(|_| RegifaceError::BusError)
    }

    /// Asynchronously writes bytes to the device's buffer at a specified offset.
    /// 
    /// This is the async version of [`write_buffer`](Device::write_buffer).
    pub async fn write_buffer_async(
        &mut self,
        offset: u8,
        bytes: &[u8],
    ) -> Result<(), RegifaceError> {
        let header = &mut [0x0E, offset];

        self.spi
            .transaction(&mut [
                embedded_hal_async::spi::Operation::Write(header.as_slice()),
                embedded_hal_async::spi::Operation::Write(bytes),
            ])
            .await
            .map_err(|_| RegifaceError::BusError)
    }

    /// Asynchronously reads bytes from the device's buffer starting at a specified offset.
    /// 
    /// This is the async version of [`read_buffer`](Device::read_buffer).
    pub async fn read_buffer_async(
        &mut self,
        offset: u8,
        bytes: &mut [u8],
    ) -> Result<(), RegifaceError> {
        let header = &mut [0x1E, offset, 0x00];

        self.spi
            .transaction(&mut [
                embedded_hal_async::spi::Operation::Write(header.as_slice()),
                embedded_hal_async::spi::Operation::Read(bytes),
            ])
            .await
            .map_err(|_| RegifaceError::BusError)
    }

    /// Asynchronously executes a command on the device.
    /// 
    /// This is the async version of [`execute_command`](Device::execute_command).
    pub async fn execute_command_async<C>(
        &mut self,
        command: C,
    ) -> Result<C::ResponseParameters, RegifaceError>
    where
        C: Command<IdType = u8>,
        C::CommandParameters: ToByteArray<Error = Infallible>,
    {
        let request = command.invoking_parameters().to_bytes().unwrap();
        let mut raw_response = <C::ResponseParameters as FromByteArray>::Array::new();

        self.spi
            .transaction(&mut [
                embedded_hal_async::spi::Operation::Write(&[C::id()]),
                embedded_hal_async::spi::Operation::Write(request.as_ref()),
                embedded_hal_async::spi::Operation::Read(raw_response.as_mut()),
            ])
            .await
            .map_err(|_| RegifaceError::BusError)?;

        C::ResponseParameters::from_bytes(raw_response)
            .map_err(|_| RegifaceError::DeserializationError)
    }
}
