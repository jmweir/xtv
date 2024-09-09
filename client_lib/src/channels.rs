use std::{
    ops::Index,
    collections::{
        HashMap,
        hash_map::Keys
    }
};
use home::home_dir;
use ::serde::{
    Deserialize,
    Deserializer,
    Serialize
};
use super::utils::FileBacked;


#[derive(Clone,Debug,Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Channel {
    #[serde(rename = "callSignVoiceOverHint")]
    name: String,
    number: u16,
    call_sign: String,
    #[serde(rename = "isHD")]
    hd: bool
}

impl Channel {
    pub fn number(&self) -> u16 {
        self.number
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn call_sign(&self) -> &String {
        &self.call_sign
    }
}

impl<'de> Deserialize<'de> for Channel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: serde_json::Value = Deserialize::deserialize(deserializer)?;
        Ok(
            Self {
                name: value["callSignVoiceOverHint"].as_str().unwrap().to_string(),
                number: value["number"].as_u64().unwrap() as u16,
                call_sign: value["callSign"].as_str().unwrap().to_string().to_uppercase(),
                hd: value["isHD"].as_bool().unwrap()
            }
        )
    }
}

#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct ChannelMap(HashMap<String,Vec<Channel>>);

impl ChannelMap {
    pub fn keys(&self) -> Keys<'_, String, Vec<Channel>> {
        self.0.keys()
    }

    pub fn get(&self, k: &String) -> Option<&Vec<Channel>> {
        self.0.get(k)
    }

    pub fn get_mut(&mut self, k: &String) -> Option<&mut Vec<Channel>> {
        self.0.get_mut(k)
    }

    pub fn insert(&mut self, k: String, v: Vec<Channel>) -> Option<Vec<Channel>> {
        self.0.insert(k, v)
    }    
    
    pub fn new() -> ChannelMap {
        ChannelMap(HashMap::<String, Vec<Channel>>::new())
    }    
}

impl Index<&String> for ChannelMap {
    type Output = Vec<Channel>;

    #[inline]
    fn index(&self, key: &String) -> &Vec<Channel> {
        self.get(key).expect("no entry found for key")
    }
}

impl FileBacked for ChannelMap {
    fn path() -> String {
        format!("{}/.config/xtv/channels", home_dir().unwrap().display())
    }
}
