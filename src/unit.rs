use serde_with::{DeserializeFromStr, SerializeDisplay};
use strum::{Display, EnumString};

/// A metric unit.
#[derive(PartialEq, Clone, Debug, Display, EnumString, SerializeDisplay, DeserializeFromStr)]
#[strum(serialize_all = "lowercase")]
pub enum Unit {
    Float,
    Integer,
    Percentage,
    Seconds,
    Milliseconds,
    Bytes,
    #[strum(serialize = "bytes/sec")]
    BytesPerSec,
    #[strum(serialize = "bits/sec")]
    BitsPerSec,
    IOPS,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(Unit::Float, "float")]
    #[case(Unit::Integer, "integer")]
    #[case(Unit::Percentage, "percentage")]
    #[case(Unit::Seconds, "seconds")]
    #[case(Unit::Milliseconds, "milliseconds")]
    #[case(Unit::Bytes, "bytes")]
    #[case(Unit::BytesPerSec, "bytes/sec")]
    #[case(Unit::BitsPerSec, "bits/sec")]
    #[case(Unit::IOPS, "iops")]
    fn test_unit(#[case] unit: Unit, #[case] unit_str: &str) {
        assert_eq!(unit.to_string(), unit_str);
        assert_eq!(unit, unit_str.parse().unwrap());
        assert_eq!(unit, serde_json::from_value(unit_str.into()).unwrap());
        assert_eq!(serde_json::to_value(unit).unwrap(), unit_str);
    }
}
