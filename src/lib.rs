use std::time::SystemTime;
use thiserror::Error;
use varint_rs::{VarintReader,VarintWriter};
use serde::{Serialize,Deserialize};

// These definitions go in their own crate so that I can absolutely 100% guarantee that the encoder
// and the decoder are using the same definition.
//
// I *think* Rust would guarantee this anyway if I defined the same struct with the same fields in 
// the same order in two different crates, but I am not taking *any* chances.

#[derive(Serialize, Deserialize)]
pub struct ModLogEntry {
    pub timestamp: SystemTime,
    pub channel_id: u64,
    pub moderator_id: u64,
    pub moderator_name: String,
    pub moderator_discrim: u16,
    pub action: ModLogAction,
}

#[derive(Serialize, Deserialize)]
pub enum ModLogAction {
    DeleteMessage(ModLogMessage),
    DeleteHour(Vec<ModLogMessage>),
    Reason(String),
}

#[derive(Serialize, Deserialize)]
pub struct ModLogMessage {
    pub id: u64,
    pub content: String,
    pub attachments: Vec<String>,
}

#[derive(Error, Debug)]
pub enum EncodeError {
    Io(#[from] std::io::Error),
    Serde(#[from] serde_json::Error),
}

impl std::fmt::Display for EncodeError {
    fn fmt(&self,fmt:&mut std::fmt::Formatter<'_>)->std::fmt::Result {
        match self {
            EncodeError::Io(e) => {
                fmt.write_str("Error writing modlog message to disk: ")?;
                std::fmt::Display::fmt(&e, fmt)
            },
            EncodeError::Serde(e) => {
                fmt.write_str("Error serializing modlog message: ")?;
                std::fmt::Display::fmt(&e, fmt)
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum DecodeError {
    Io(#[from] std::io::Error),
    Serde(#[from] serde_json::Error),
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self,fmt:&mut std::fmt::Formatter<'_>)->std::fmt::Result {
        match self {
            DecodeError::Io(e) => {
                fmt.write_str("Error reading modlog message from disk: ")?;
                std::fmt::Display::fmt(&e, fmt)
            },
            DecodeError::Serde(e) => {
                fmt.write_str("Error deserializing modlog message: ")?;
                std::fmt::Display::fmt(&e, fmt)
            }
        }
    }
}

// binary mod log format (messagepack)
/*
impl ModLogEntry {
    pub fn read(mut f: &mut dyn std::io::Read) -> Result<Self, DecodeError> {
        let count = f.read_usize_varint()?;
        let mut buf = vec![0u8; count]; // i could do this more efficiently with a boxed slice but not
                                    // without both unsafe{} code and nightly-only compiler
                                    // features.
        f.read_exact(&mut buf)?;
        return Ok(rmp_serde::from_slice(&buf)?);

    }
    pub fn write(&self, mut f: &mut dyn std::io::Write) -> Result<(), EncodeError> {
        let data = rmp_serde::to_vec(self)?;
        f.write_usize_varint(data.len())?;
        f.write_all(&data)?;
        Ok(())
    }
}
*/

impl ModLogEntry {
    pub fn read(f: &mut dyn std::io::BufRead) -> Result<Self, DecodeError> {
        let mut s = String::new();
        f.read_line(&mut s)?;
        Ok(serde_json::from_slice(s.as_bytes())?)
    }
    pub fn write(&self, f: &mut dyn std::io::Write) -> Result<(), EncodeError> {
        serde_json::to_writer(&mut *f, self)?;
        f.write_all(b"\n")?;
        Ok(())
    }
}
