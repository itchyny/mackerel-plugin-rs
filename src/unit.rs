use serde_derive::{Deserialize, Serialize};

/// Metric units
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Unit {
    #[serde(rename = "float")]
    Float,
    #[serde(rename = "integer")]
    Integer,
    #[serde(rename = "percentage")]
    Percentage,
    #[serde(rename = "bytes")]
    Bytes,
    #[serde(rename = "bytes/sec")]
    BytesPerSecond,
    #[serde(rename = "iops")]
    IOPS,
}

impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Unit::Float => write!(f, "float"),
            Unit::Integer => write!(f, "integer"),
            Unit::Percentage => write!(f, "percentage"),
            Unit::Bytes => write!(f, "bytes"),
            Unit::BytesPerSecond => write!(f, "bytes/sec"),
            Unit::IOPS => write!(f, "iops"),
        }
    }
}

impl<'a> From<&'a str> for Unit {
    fn from(src: &str) -> Unit {
        match src {
            "float" => Unit::Float,
            "integer" => Unit::Integer,
            "percentage" => Unit::Percentage,
            "bytes" => Unit::Bytes,
            "bytes/sec" => Unit::BytesPerSecond,
            "iops" => Unit::IOPS,
            x => panic!(
                "invalid unit: {} (should be one of float, integer, percentage, bytes, bytes/sec or iops)",
                x
            ),
        }
    }
}
