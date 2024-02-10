#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused)]

extern crate chrono;
extern crate serde;
use self::serde::Deserialize;
use self::chrono::prelude::*;

use super::*;

pub type Releases = Vec<Release>;

#[derive(Deserialize)]
pub struct Release {
    pub url: String,
    pub assets_url: String,
    pub upload_url: String,
    pub html_url: String,
    pub id: u32,
    pub author: Author,
    pub node_id: String,
    pub tag_name: String,
    pub target_commitish: String,
    pub name: String,
    pub draft: bool,
    pub prerelease: bool,
    pub created_at: DateTime<Local>,
    pub published_at: DateTime<Local>,
    pub assets: Vec<Asset>,
    pub tarball_url: String,
    pub zipball_url: String,
    pub body: String,
    pub reactions: Option<Reactions>
}

#[derive(Deserialize)]
pub struct Author {
    pub login: String,
    pub id: u32,
    pub node_id: String,
    pub avatar_url: String,
    pub gravatar_id: String,
    pub url: String,
    pub html_url: String,
    pub followers_url: String,
    pub following_url: String,
    pub gists_url: String,
    pub starred_url: String,
    pub subscriptions_url: String,
    pub organizations_url: String,
    pub repos_url: String,
    pub events_url: String,
    pub received_events_url: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub site_admin: bool,
}

#[derive(Deserialize)]
pub struct Asset {
    pub url: String,
    pub id: u32,
    pub node_id: String,
    pub name: String,
    pub label: Option<String>,
    pub uploader: Author,
    pub content_type: String,
    pub state: String,
    pub size: u32,
    pub download_count: u32,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
    pub browser_download_url: String,
}

#[derive(Deserialize)]
pub struct Reactions {
    pub url: String,
    pub total_count: u32,
    #[serde(rename = "+1")]
    pub pos_one: u32,
    #[serde(rename = "-1")]
    pub neg_one: u32,
    pub laugh: u32,
    pub hooray: u32,
    pub confused: u32,
    pub heart: u32,
    pub rocket: u32,
    pub eyes: u32,
}