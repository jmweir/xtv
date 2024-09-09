use std::{
    fs,
    marker::Sized
};
use ::serde::{
    de::DeserializeOwned,
    Serialize
};

pub trait FileBacked {
    fn path() -> String;
}

pub trait AsToml {
    fn load() -> Result<Self, Box<dyn std::error::Error>> where Self: Sized;
    fn save(&self) -> Result<(), Box<dyn std::error::Error>>;
}

impl<T: FileBacked + Serialize + DeserializeOwned> AsToml for T {
    fn load() -> Result<Self, Box<dyn std::error::Error>> {
        toml::from_str::<Self>(&fs::read_to_string(Self::path())?)
            .map_err(Box::<dyn std::error::Error>::from)
    }

    fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        fs::write(Self::path(), toml::to_string(self)?)
            .map_err(Box::<dyn std::error::Error>::from)
    }
}
