use std::fmt;
use super::channels::Channel;
use super::devices::Device;
use super::recordings::Recording;
use super::search::SearchResult;
use ::serde::{
    de::{
        Deserializer,
        DeserializeOwned,
        Error,
        MapAccess,
        Visitor
    },
    Deserialize
};


pub enum XTVResponse {
    Channels(Vec<Channel>),
    Devices(Vec<Device>),
    Recordings(Vec<Recording>),
    SearchResults(Vec<SearchResult>),
}

impl XTVResponse {
    pub fn channels(&self) -> Vec<Channel> {
        if let XTVResponse::Channels(chan) = self { chan.to_vec() } else { panic!("Not channels!") }
    }
    pub fn devices(&self) -> Vec<Device> {
        if let XTVResponse::Devices(dev) = self { dev.to_vec() } else { panic!("Not devices!") }
    }
    pub fn recordings(&self) -> Vec<Recording> {
        if let XTVResponse::Recordings(rec) = self { rec.to_vec() } else { panic!("Not recordings!") }
    }    
    pub fn search_results(&self) -> Vec<SearchResult> {
        if let XTVResponse::SearchResults(res) = self { res.to_vec() } else { panic!("Not search results!") }
    }
    fn from_value<C: Fn(T) -> Self, T: DeserializeOwned>(value: serde_json::Value, cons: C) -> Self {
        cons(serde_json::from_value::<T>(value).unwrap())
    }
}

impl<'de> Deserialize<'de> for XTVResponse {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct XTVResponseVisitor;

        impl<'de> Visitor<'de> for XTVResponseVisitor {
            type Value = XTVResponse;
        
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an XTV response json string")
            }
        
            fn visit_map<M>(self, access: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let value = super::serde::to_map::<M>(access)?;

                match value["_type"].as_str().unwrap() {
                    "Enumeration/ChannelMap" => Ok(XTVResponse::from_value(value["_embedded"]["channels"].clone(), XTVResponse::Channels)),
                    "Enumeration/Device" => Ok(XTVResponse::from_value(value["_embedded"]["devices"].clone(), XTVResponse::Devices)),
                    "Enumeration/Recording" => Ok(XTVResponse::from_value(value["_embedded"]["recordings"].clone(), XTVResponse::Recordings)),
                    "Enumeration/SearchResult" => Ok(XTVResponse::from_value(value["_embedded"]["results"].clone(), XTVResponse::SearchResults)),
                    _ => Err(M::Error::custom("Unknown type"))
                }
            }
        }
                
        d.deserialize_map(XTVResponseVisitor)
    }
}
