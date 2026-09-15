use std::io::{Read, Write};

use serde::de::DeserializeOwned;
use serde::Serialize;
use thiserror::Error;

const LENGTH_PREFIX_SIZE: usize = size_of::<u32>();

const MAXIMUM_MESSAGE_LENGTH: u32 = 4 * 1024 * 1024;

pub fn read_transport<M : DeserializeOwned, R : Read>(r: &mut R) -> Result<M, TransportError> {
    let mut raw_message_length: [u8; LENGTH_PREFIX_SIZE] = [0; LENGTH_PREFIX_SIZE];
    r.read_exact(&mut raw_message_length)?;

    let message_length = u32::from_be_bytes(raw_message_length);
    if message_length > MAXIMUM_MESSAGE_LENGTH {
        return Err(TransportError::MessageTooLarge);
    }

    let mut raw_message = vec![0; message_length as usize];
    r.read_exact(&mut raw_message)?;

    let message: M = bincode::deserialize(&raw_message)?;
    Ok(message)
}


pub fn write_transport<W : Write>(w: &mut W, message: impl Serialize) -> Result<(), TransportError> {
    let mut payload: Vec<u8> = vec![0; LENGTH_PREFIX_SIZE];
    bincode::serialize_into(&mut payload, &message)?;

    let message_length = payload.len() - LENGTH_PREFIX_SIZE;
    if message_length > MAXIMUM_MESSAGE_LENGTH as usize {
        return Err(TransportError::MessageTooLarge);
    }

    payload[..LENGTH_PREFIX_SIZE].copy_from_slice(&(message_length as u32).to_be_bytes());
    w.write_all(&payload)?;

    Ok(())
}

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("failed to serialize message: {0}")]
    Bincode(#[from] bincode::Error),

    #[error("I/O: {0}")]
    IO(#[from] std::io::Error),

    #[error("the supplied message is too large")]
    MessageTooLarge
}
