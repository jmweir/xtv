use chrono::{
    DateTime,
    offset::Local
};
use derive_getters::Getters;
use ::serde::Deserialize;


#[derive(Clone,Debug,Deserialize,Getters)]
#[serde(rename_all = "camelCase")]
pub struct Recording {
    title: String,    
    #[serde(with = "super::serde::chrono_rfc2822")]
    date_recorded: DateTime<Local>,
    media_id: String,
}
