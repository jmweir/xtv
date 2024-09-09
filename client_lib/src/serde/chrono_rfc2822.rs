use std::fmt;
use serde::de;
use chrono::{
    DateTime,
    offset::{Utc, Local},
    naive::NaiveDateTime
};

pub fn deserialize<'de, D>(d: D) -> Result<DateTime<Local>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct DateTimeVisitor;

        impl<'de> de::Visitor<'de> for DateTimeVisitor {
            type Value = DateTime<Local>;
        
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a formatted date and time string or a unix timestamp")
            }
        
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let ndt = NaiveDateTime::parse_from_str(value, "%a, %e %b %Y %H:%M:%S UTC").map_err(E::custom)?;
                Ok(DateTime::<Local>::from(DateTime::<Utc>::from_utc(ndt, Utc)))
            }
        }
        
        d.deserialize_str(DateTimeVisitor)
    }
