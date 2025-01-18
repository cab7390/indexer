/// Utility functions for deserializing API responses

use chrono::{DateTime, Utc};
use serde::{de::Visitor, Deserializer};

pub fn api_date<'de, D: Deserializer<'de>>(deserializer: D) -> Result<DateTime<Utc>, D::Error> {
    struct ApiDateVisitor;
    impl Visitor<'_> for ApiDateVisitor {
        type Value = DateTime<Utc>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a date string in the following format: `%a %b %d %T %z %Y`")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            DateTime::parse_from_str(v, "%a %b %d %T %z %Y")
                .map_err(E::custom)
                .map(|dt| dt.with_timezone(&Utc))
        }
    }
    deserializer.deserialize_str(ApiDateVisitor)
}

pub fn api_bool<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
    struct ApiBoolVisitor;
    impl Visitor<'_> for ApiBoolVisitor {
        type Value = bool;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("either a string with the value \"true\" or \"false\" or an integer with the value `1` or `0`")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            match v {
                "true" => Ok(true),
                "false" => Ok(false),
                _ => Err(E::custom(format!(
                    "String must be \"true\" or \"false\". Got: {:#?}",
                    v
                ))),
            }
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            match v {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(E::custom(format!(
                    "Integer must be `0` or `1`. Got: {:#?}",
                    v
                ))),
            }
        }
    }

    deserializer.deserialize_any(ApiBoolVisitor)
}

pub fn api_option_str<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    struct ApiOptionStrVisitor;
    impl Visitor<'_> for ApiOptionStrVisitor {
        type Value = Option<String>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("either an empty string or one with a value")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            if v.is_empty() {
                Ok(None)
            } else {
                Ok(Some(v.to_string()))
            }
        }
    }

    deserializer.deserialize_str(ApiOptionStrVisitor)
}

pub fn api_option_u64<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<u64>, D::Error> {
    struct ApiOptionU64Visitor;
    impl Visitor<'_> for ApiOptionU64Visitor {
        type Value = Option<u64>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a u64 integer")
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            if v == 0 {
                Ok(None)
            } else {
                Ok(Some(v))
            }
        }
    }

    deserializer.deserialize_u64(ApiOptionU64Visitor)
}

pub fn api_option_u32<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<u32>, D::Error> {
    struct ApiOptionU32Visitor;
    impl Visitor<'_> for ApiOptionU32Visitor {
        type Value = Option<u32>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a u32 integer")
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            if v == 0 {
                Ok(None)
            } else {
                u32::try_from(v).map(Some).map_err(E::custom)
            }
        }
    }

    deserializer.deserialize_u64(ApiOptionU32Visitor)
}
