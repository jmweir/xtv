mod channels;
mod devices;
mod oauth2;
mod recordings;
mod response;
mod search;
mod serde;
mod server;
mod utils;

use std::{
    rc::Rc,
    cell::{
        Ref,
        RefCell,
    },
    fmt,
    collections::HashMap,
    ops::Deref,
};

use channels::ChannelMap;
use clap::ValueEnum;
use convert_case::{
    Case,
    Casing
};
pub use devices::{
    Device,
    DeviceMap
};
use home::home_dir;
use self::oauth2::{
    authenticate,
    refresh,
    Token
};
use recordings::Recording;
use reqwest::{
    Method,
    RequestBuilder,
    Response
};
use response::XTVResponse;
use search::SearchResult;
use ::serde::{
    Deserialize,
    Serialize
};
use utils::{
    AsToml,
    FileBacked
};


pub struct XTVClient {
    config: Rc<RefCell<Config>>,
    client: reqwest::Client,
    channel_map: Rc<RefCell<Option<ChannelMap>>>,
    device_map: Rc<RefCell<Option<DeviceMap>>>
}

#[derive(Clone,Debug,Deserialize,Serialize)]
struct Config {
    api_host: String,
    oauth: self::oauth2::Config,
    token: Option<self::oauth2::Token>
}

impl FileBacked for Config {
    fn path() -> String {
        format!("{}/.config/xtv/config", home_dir().unwrap().display())
    }
}

#[derive(Debug)]
pub enum KeyCode {
    Play,
    Pause,
    Stop,
    FastForward,
    Rewind,
    Exit,
}

impl fmt::Display for KeyCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug,Clone,Deserialize,ValueEnum,PartialEq)]
pub enum TuningTarget {
    Channel,
    Recording,
    VOD
}

impl fmt::Display for TuningTarget {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl XTVClient {
    pub fn new() -> Result<XTVClient, Box<dyn std::error::Error>> {
        Ok(
            XTVClient {
                config: Rc::new(RefCell::new(Config::load()?)),
                client: reqwest::Client::new(),
                channel_map: Rc::new(RefCell::new(match ChannelMap::load() {
                    Ok(map) => Some(map),
                    Err(_) => None
                })),
                device_map: Rc::new(RefCell::new(match DeviceMap::load() {
                    Ok(map) => Some(map),
                    Err(_) => None
                }))
            }
        )
    }
    
    pub async fn token(&self) -> Result<String, Box<dyn std::error::Error>> {
        self.config.borrow_mut().token = Some(self.get_token().await?);
        match &self.config.borrow().token {
            Some(token) => Ok(token.access().to_string()),
            None => Err(String::from("No token available").into())
        }
    }

    pub async fn tune(&self, target: &TuningTarget, id: &String, device: &Device) -> Result<Response, Box<dyn std::error::Error>> {
        let url = format!("/devices/{}/remote/tune/{}/", device.id(), target.to_string().to_lowercase());

        let id_param = match target {
            TuningTarget::Channel => "channelNumber",
            TuningTarget::Recording => "mediaId",
            TuningTarget::VOD => "mediaId"
        };

        let id = match target {
            TuningTarget::Channel => match id.parse::<u16>() {
                Ok(x) => x.to_string(),
                Err(_) => {
                    let channel_map = self.channels().await?;
                    match channel_map.get(&id.to_uppercase()) {
                        None => id.to_string(),
                        Some(v) => v[0].number().to_string()
                    }
                }
            },
            _ => id.to_string()
        };

        self.post(url, &HashMap::from([(id_param, &*id.to_string())])).await
    }

    pub async fn recordings(&self, device: &Device) -> Result<Vec<Recording>, Box<dyn std::error::Error>> {
        Ok(
            self.get(format!("/devices/{}/recordings/completed/", device.id()), &HashMap::new())
                .await?
                .json::<XTVResponse>()
                .await?
                .recordings()
        )
    }

    pub async fn devices(&self) -> Result<impl Deref<Target = DeviceMap> + '_, Box<dyn std::error::Error>> {
        if self.device_map.borrow().is_none() {        
            let devices = self.get("/devices/".to_string(), &HashMap::new())
                .await?
                .json::<XTVResponse>()
                .await?
                .devices();

            let device_map = devices.iter()
                .fold(DeviceMap::new(), |mut map, device| {
                    map.insert(device.name().to_string(), device.clone());
                    map
                });

            *self.device_map.borrow_mut() = Some(device_map);
        }

        Ok(Ref::map(self.device_map.borrow(), |borrow| { borrow.as_ref().unwrap() }))
    }

    pub async fn channels(&self) -> Result<impl Deref<Target = ChannelMap> + '_, Box<dyn std::error::Error>> {
        if self.channel_map.borrow().is_none() {
            let channels = self.get("/channelmap/".to_string(), &HashMap::new())
                .await?
                .json::<XTVResponse>()
                .await?
                .channels();

            let channel_map = channels.iter()
                .fold(ChannelMap::new(), |mut map, channel| {
                    match map.get_mut(&channel.call_sign()) {
                        Some(v) => { v.push(channel.clone()); map },
                        None => { map.insert(channel.call_sign().clone(), vec![channel.clone()]); map }
                    }
                });

            *self.channel_map.borrow_mut() = Some(channel_map);
        }

        Ok(Ref::map(self.channel_map.borrow(), |borrow| { borrow.as_ref().unwrap() }))
    }

    pub async fn press_key(&self, code: KeyCode, device: &Device) -> Result<Response, Box<dyn std::error::Error>> {
        let code = code.to_string().to_case(Case::UpperSnake);

        let params = HashMap::from([("keyCode", &*code)]);

        self.post(format!("/devices/{}/remote/processKey/", device.id()), &params).await
    }

    pub async fn search(&self, query: &String) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
        Ok(
            self.get(format!("/search/term/"), &HashMap::from([("query", &*query.to_string())]))
                .await?
                .json::<XTVResponse>()
                .await?
                .search_results()
        )        
    }

    pub async fn lookup_device(&self, name: &str) -> Result<Device, Box<dyn std::error::Error>> {
        match self.devices().await?.get(&name.to_string()) {
            Some(device) => Ok(device.clone()),
            None => Err("Device not found")?
        }
    }

    async fn get(&self, endpoint: String, query: &HashMap<&str,&str>) -> Result<Response, Box<dyn std::error::Error>> {
        self.request(Method::GET, endpoint).await?
            .query(query)
            .send()
            .await
            .map_err(Box::<dyn std::error::Error>::from)
    }

    async fn post(&self, endpoint: String, params: &HashMap<&str, &str>) -> Result<Response, Box<dyn std::error::Error>> {
        self.request(Method::POST, endpoint).await?
            .form(params)
            .send()
            .await
            .map_err(Box::<dyn std::error::Error>::from)
    }

    async fn request(&self, method: Method, endpoint: String) -> Result<RequestBuilder, Box<dyn std::error::Error>> {
        let url = format!("https://{}{}", self.config.borrow().api_host, endpoint);

        self.config.borrow_mut().token = Some(self.get_token().await?);

        let req = self.client.request(method.clone(), url)
            .bearer_auth(self.config.borrow().token.as_ref().unwrap().access());

        Ok(req)
    }

    async fn get_token(&self) -> Result<Token, Box<dyn std::error::Error>> {
        match &self.config.borrow().token {
            Some(token) if !token.is_expired() => Ok(token.clone()),
            Some(token) => match refresh(&self.config.borrow().oauth, token.refresh().to_string()).await {
                Ok(token) => Ok(token),
                Err(_) => authenticate(&self.config.borrow().oauth).await
            },
            None => authenticate(&self.config.borrow().oauth).await
        }
    }    

}

impl Drop for XTVClient {
    fn drop(&mut self) {
        self.config.borrow().save().unwrap();
        if let Some(channels) = &*self.channel_map.borrow() {
            channels.save().unwrap();
        }
        if let Some(devices) = &*self.device_map.borrow() {
            devices.save().unwrap();
        }
    }
}
