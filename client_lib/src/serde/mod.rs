pub mod chrono_rfc2822;

use ::serde::{
    de::{
        MapAccess
    }
};
use serde_json::{
    Value,
    map::Map
};


pub fn to_map<'a, M>(mut access: M) -> Result<Map<String,Value>, M::Error>
where
    M: MapAccess<'a>
    {
        let mut value = Map::with_capacity(access.size_hint().unwrap_or(0));

        while let Some((k, v)) = access.next_entry()? {
            value.insert(k, v);
        }

        Ok(value)
    }
