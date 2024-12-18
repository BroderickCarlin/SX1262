use core::convert::Infallible;

use embedded_hal::spi::{Operation, SpiDevice};
use regiface::{
    errors::Error as RegifaceError, ByteArray, Command, FromByteArray, ReadableRegister,
    ToByteArray, WritableRegister,
};

/// Main device interface for the SX126x radio
pub struct Device<SPI> {
    spi: SPI,
}

impl<SPI> Device<SPI> {
    /// Create a new Device instance
    pub fn new(spi: SPI) -> Self {
        Self { spi }
    }

    /// Release the underlying SPI device
    pub fn release(self) -> SPI {
        self.spi
    }
}

impl<SPI> Device<SPI>
where
    SPI: SpiDevice,
{
    /// Read a register value
    pub fn read_register<R>(&mut self) -> Result<R, RegifaceError>
    where
        R: ReadableRegister<IdType = u16>,
    {
        let header = &mut [0x1D, 0x00, 0x00, 0x00];
        header[1..].copy_from_slice(&R::id().to_be_bytes());

        let mut raw_value = R::Array::new();

        self.spi
            .transaction(&mut [
                Operation::Write(header.as_slice()),
                Operation::Read(raw_value.as_mut()),
            ])
            .map_err(|_| RegifaceError::BusError)?;

        R::from_bytes(raw_value).map_err(|_| RegifaceError::DeserializationError)
    }

    /// Write a register value
    pub fn write_register<R>(&mut self, register: R) -> Result<(), RegifaceError>
    where
        R: WritableRegister<IdType = u16, Error = Infallible>,
    {
        let header = &mut [0x0D, 0x00, 0x00];
        header[1..].copy_from_slice(&R::id().to_be_bytes());

        let raw_value = register.to_bytes().unwrap();

        self.spi
            .transaction(&mut [
                Operation::Write(header.as_slice()),
                Operation::Write(raw_value.as_ref()),
            ])
            .map_err(|_| RegifaceError::BusError)
    }

    /// Write the provided bytes into the buffer at the given offset
    pub fn write_buffer(&mut self, offset: u8, bytes: &[u8]) -> Result<(), RegifaceError> {
        let header = &mut [0x0E, offset];

        self.spi
            .transaction(&mut [Operation::Write(header.as_slice()), Operation::Write(bytes)])
            .map_err(|_| RegifaceError::BusError)
    }

    /// Fill the provided buffer with bytes starting at the given offset
    pub fn read_buffer(&mut self, offset: u8, bytes: &mut [u8]) -> Result<(), RegifaceError> {
        let header = &mut [0x1E, offset, 0x00];

        self.spi
            .transaction(&mut [Operation::Write(header.as_slice()), Operation::Read(bytes)])
            .map_err(|_| RegifaceError::BusError)
    }

    /// Execute a command
    pub fn execute_command<C>(&mut self, command: C) -> Result<C::ResponseParameters, RegifaceError>
    where
        C: Command<IdType = u8>,
        C::CommandParameters: ToByteArray<Error = Infallible>,
    {
        let request = command.invoking_parameters().to_bytes().unwrap();
        let mut raw_response = <C::ResponseParameters as FromByteArray>::Array::new();

        self.spi
            .transaction(&mut [
                Operation::Write(&[C::id()]),
                Operation::Write(request.as_ref()),
                Operation::Read(raw_response.as_mut()),
            ])
            .map_err(|_| RegifaceError::BusError)?;

        C::ResponseParameters::from_bytes(raw_response)
            .map_err(|_| RegifaceError::DeserializationError)
    }
}
