#![allow(dead_code)]
use crate::bid::{BidError, BidResult};

// Minimal helpers to keep loader.rs lean and consistent
#[inline]
pub fn from_fs<T>(_r: std::io::Result<T>) -> BidResult<T> {
    _r.map_err(|_| BidError::PluginError)
}

#[inline]
pub fn from_toml<T>(_r: Result<T, toml::de::Error>) -> BidResult<T> {
    _r.map_err(|_| BidError::PluginError)
}

#[inline]
pub fn or_plugin_err<T>(opt: Option<T>) -> BidResult<T> {
    opt.ok_or(BidError::PluginError)
}
