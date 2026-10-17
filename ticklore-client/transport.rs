use std::io::{Read, Write};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;

const LENGTH_PREFIX_SIZE: usize = size_of::<u32>();

const MAXIMUM_MESSAGE_LENGTH: u32 = 4 * 1024 * 1024;

pub const PROTOCOL_VERSION: u32 = 1;

pub fn write_handshake<W : Write>(w: &mut W, version: &[u8]) -> Result<(), TransportError> {
    if version.len() > MAXIMUM_MESSAGE_LENGTH as usize {
        return Err(TransportError::MessageTooLarge);
    }

    let mut payload: Vec<u8> = Vec::with_capacity(2 * LENGTH_PREFIX_SIZE + version.len());
    payload.extend_from_slice(&PROTOCOL_VERSION.to_be_bytes());
    payload.extend_from_slice(&(version.len() as u32).to_be_bytes());
    payload.extend_from_slice(version);
    w.write_all(&payload)?;

    Ok(())
}

pub fn read_handshake_reply<R : Read>(r: &mut R) -> Result<HandshakeReply, TransportError> {
    let mut raw_reply: [u8; 1] = [0];
    r.read_exact(&mut raw_reply)?;

    match raw_reply[0] {
        0 => Ok(HandshakeReply::Accepted),
        1 => Ok(HandshakeReply::ProtocolRejected),
        2 => Ok(HandshakeReply::VersionRejected),
        _ => Err(TransportError::UnknownHandshakeReply)
    }
}

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

#[derive(Clone, Copy, Debug)]
pub enum HandshakeReply {
    Accepted,
    ProtocolRejected,
    VersionRejected
}

#[derive(Deserialize, Serialize)]
pub enum ClientMessage<SE> {
    ShadowEvent(SE),
    Keepalive
}

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("failed to serialize message: {0}")]
    Bincode(#[from] bincode::Error),

    #[error("I/O: {0}")]
    IO(#[from] std::io::Error),

    #[error("the supplied message is too large")]
    MessageTooLarge,

    #[error("the server sent an unknown handshake reply")]
    UnknownHandshakeReply
}
