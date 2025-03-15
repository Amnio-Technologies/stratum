use anyhow::Result;
use crc::{Crc, CRC_16_IBM_3740};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum I2CError {
    #[error("Message too short")]
    MessageTooShort,

    #[error("Invalid start byte")]
    InvalidStartByte,

    #[error("Message length mismatch")]
    LengthMismatch,

    #[error("CRC mismatch")]
    CRCMismatch,
}

#[derive(Debug)]
pub struct I2CMessage<'a> {
    start_byte: u8,
    module_address: u8,
    command_id: u8,
    payload_length: u8,
    payload: &'a [u8],
    crc: [u8; I2CMessage::CRC_LEN],
}

impl<'a> I2CMessage<'a> {
    // Amnio Communication (AC)
    const START_BYTE: u8 = 0xAC;
    const CRC_LEN: usize = 2;

    /// Compute CRC-16 checksum (big-endian order)
    fn compute_crc16(data: &[u8]) -> [u8; I2CMessage::CRC_LEN] {
        Crc::<u16>::new(&CRC_16_IBM_3740)
            .checksum(data)
            .to_be_bytes()
    }

    pub fn new(module_address: u8, command_id: u8, payload: &'a [u8]) -> Self {
        let payload_length = payload.len() as u8;

        let mut message_data = Vec::with_capacity(4 + payload.len());
        message_data.push(Self::START_BYTE);
        message_data.push(module_address);
        message_data.push(command_id);
        message_data.push(payload_length);
        message_data.extend_from_slice(payload);

        let crc = Self::compute_crc16(&message_data);

        Self {
            start_byte: Self::START_BYTE,
            module_address,
            command_id,
            payload_length,
            payload,
            crc,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(4 + self.payload.len() + Self::CRC_LEN);
        data.push(self.start_byte);
        data.push(self.module_address);
        data.push(self.command_id);
        data.push(self.payload_length);
        data.extend_from_slice(self.payload);
        data.extend_from_slice(&self.crc);
        data
    }

    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, I2CError> {
        if bytes.len() < 6 {
            return Err(I2CError::MessageTooShort);
        }

        if bytes[0] != Self::START_BYTE {
            return Err(I2CError::InvalidStartByte);
        }

        let payload_length = bytes[3];

        let expected_length = 4 + payload_length as usize + Self::CRC_LEN;
        if bytes.len() < expected_length {
            return Err(I2CError::LengthMismatch);
        }

        let crc_received = &bytes[bytes.len() - Self::CRC_LEN..];
        let computed_crc = Self::compute_crc16(&bytes[..bytes.len() - Self::CRC_LEN]);

        if computed_crc != [crc_received[0], crc_received[1]] {
            return Err(I2CError::CRCMismatch);
        }

        Ok(Self {
            start_byte: bytes[0],
            module_address: bytes[1],
            command_id: bytes[2],
            payload_length,
            payload: &bytes[4..4 + payload_length as usize],
            crc: [crc_received[0], crc_received[1]],
        })
    }
}
