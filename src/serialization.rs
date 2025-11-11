use crate::error::{Error, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;

/// Data format for serialization (matching Modal's proto enum)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataFormat {
    /// CBOR format (preferred, more efficient)
    Cbor = 4,
    /// Pickle format (legacy, Python-compatible)
    Pickle = 1,
}

impl Default for DataFormat {
    fn default() -> Self {
        // Default to CBOR for Rust SDK
        DataFormat::Cbor
    }
}

/// Get the preferred data format from environment or config
/// 
/// Checks MODAL_PAYLOAD_FORMAT environment variable (defaults to "cbor")
pub fn get_preferred_data_format() -> DataFormat {
    match std::env::var("MODAL_PAYLOAD_FORMAT")
        .unwrap_or_else(|_| "cbor".to_string())
        .to_lowercase()
        .as_str()
    {
        "pickle" => DataFormat::Pickle,
        "cbor" | _ => DataFormat::Cbor,
    }
}

/// Serialize a value to CBOR format
pub fn to_cbor<T: Serialize>(v: &T) -> Result<Vec<u8>> {
    serde_cbor::to_vec(v).map_err(|e| Error::Serialization(format!("CBOR serialization failed: {}", e)))
}

/// Deserialize a value from CBOR format
pub fn from_cbor<T: DeserializeOwned>(b: &[u8]) -> Result<T> {
    serde_cbor::from_slice(b).map_err(|e| Error::Deserialization(format!("CBOR deserialization failed: {}", e)))
}

/// Serialize function arguments
/// 
/// In Python, args are serialized as (args, kwargs) tuple.
/// For Rust, we serialize the args directly using the preferred format.
pub fn serialize_args<T: Serialize>(args: &T) -> Result<Vec<u8>> {
    serialize_args_with_format(args, get_preferred_data_format())
}

/// Serialize function arguments with a specific format
pub fn serialize_args_with_format<T: Serialize>(args: &T, format: DataFormat) -> Result<Vec<u8>> {
    match format {
        DataFormat::Cbor => to_cbor(args),
        DataFormat::Pickle => {
            // Note: Rust doesn't have a direct pickle implementation
            // For now, we'll use CBOR and note that pickle support would require
            // a pickle library or conversion layer
            Err(Error::Serialization(
                "Pickle format not yet supported in Rust SDK. Use CBOR format instead.".to_string(),
            ))
        }
    }
}

/// Deserialize function result
pub fn deserialize_result<T: DeserializeOwned>(b: &[u8]) -> Result<T> {
    deserialize_result_with_format(b, get_preferred_data_format())
}

/// Deserialize function result with a specific format
pub fn deserialize_result_with_format<T: DeserializeOwned>(b: &[u8], format: DataFormat) -> Result<T> {
    match format {
        DataFormat::Cbor => from_cbor(b),
        DataFormat::Pickle => {
            Err(Error::Deserialization(
                "Pickle format not yet supported in Rust SDK. Use CBOR format instead.".to_string(),
            ))
        }
    }
}

/// Serialize data format enum to proto i32 value
pub fn data_format_to_proto(format: DataFormat) -> i32 {
    format as i32
}

/// Parse data format from proto i32 value
pub fn data_format_from_proto(value: i32) -> DataFormat {
    match value {
        1 => DataFormat::Pickle,
        4 => DataFormat::Cbor,
        _ => DataFormat::Cbor, // Default to CBOR for unknown values
    }
}

