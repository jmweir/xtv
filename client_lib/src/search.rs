use std::fmt;
use derive_getters::Getters;
use ::serde::{
    Deserialize,
    Deserializer,
    de::{
        Error,
        MapAccess,
        Visitor,
    }
};


#[derive(Clone,Getters)]
pub struct SearchResult {
    name: String,
    subtitle: String,
    entity: Entity,
}

#[derive(Clone,Debug,Deserialize,Getters)]
#[serde(rename_all = "camelCase")]
pub struct Entity {
    merlin_id: u64,
    name: String,
    description: String
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<'de> Deserialize<'de> for SearchResult {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SearchResultVisitor;

        impl<'de> Visitor<'de> for SearchResultVisitor {
            type Value = SearchResult;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an XTV search result")
            }

            fn visit_map<M>(self, access: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let value = super::serde::to_map::<M>(access)?;

                Ok(
                    SearchResult {
                        name: value["name"].as_str().unwrap().to_string(),
                        subtitle: value["subtitle"].as_str().unwrap().to_string(),
                        entity: serde_json::from_value::<Entity>(value["_embedded"]["entity"].clone()).map_err(M::Error::custom)?
                    }
                )
            }
        }
  
        d.deserialize_map(SearchResultVisitor)
    }
}
