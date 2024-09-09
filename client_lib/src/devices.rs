use std::{
    collections::{
        HashMap,
        hash_map::Values
    },
    ops::Index
};
use derive_getters::Getters;
use home::{
    home_dir
};
use ::serde::{
    Deserialize,
    Serialize
};
use super::utils::{
    FileBacked
};


#[derive(Clone,Debug,Deserialize,Getters,Serialize)]
pub struct Device {
    #[serde(rename = "deviceId")]
    id: String,
    #[serde(rename = "deviceName")]
    name: String
}

#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct DeviceMap(HashMap<String,Device>);

impl DeviceMap {
    pub fn get(&self, k: &String) -> Option<&Device> {
        self.0.get(k)
    }

    pub fn insert(&mut self, k: String, v: Device) -> Option<Device> {
        self.0.insert(k, v)
    }

    pub fn values(&self) ->  Values<'_, String, Device> {
        self.0.values()
    }

    pub fn new() -> DeviceMap {
        DeviceMap(HashMap::<String, Device>::new())
    }
}

impl FileBacked for DeviceMap {
    fn path() -> String {
        format!("{}/.config/xtv/devices", home_dir().unwrap().display())
    }
}

impl Index<&String> for DeviceMap {
    type Output = Device;

    #[inline]
    fn index(&self, key: &String) -> &Device {
        self.get(key).expect("no entry found for key")
    }
}

// pub struct Devices;

// impl Devices {
//     pub const MEDIA_ROOM: Device = Device { id: String::from("x1-6851522893570396146") };
//     pub const LIVING_ROOM: Device = Device { id: String::from("x1-8863577650658625914") };
// }
