#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused)]

extern crate chrono;
extern crate serde;
use self::serde::Deserialize;
use self::chrono::prelude::*;

use super::*;

type Releases = Vec<Release>;

#[derive(Deserialize)]
struct Release {
    url: String,
    assets_url: String,
    upload_url: String,
    html_url: String,
    id: u32,
    author: Author,
    node_id: String,
    tag_name: String,
    target_commitish: String,
    name: String,
    draft: bool,
    prerelease: bool,
    created_at: DateTime<Local>,
    published_at: DateTime<Local>,
    assets: Vec<Asset>,
    tarball_url: String,
    zipball_url: String,
    body: String,
    reactions: Reactions
}

#[derive(Deserialize)]
struct Author {
    login: String,
    id: u32,
    node_id: String,
    avatar_url: String,
    gravatar_id: String,
    url: String,
    html_url: String,
    followers_url: String,
    following_url: String,
    gists_url: String,
    starred_url: String,
    subscriptions_url: String,
    organizations_url: String,
    repos_url: String,
    events_url: String,
    received_events_url: String,
    #[serde(rename = "type")]
    type_: String,
    site_admin: bool,
}

#[derive(Deserialize)]
struct Asset {
    url: String,
    id: u32,
    node_id: String,
    name: String,
    label: String,
    uploader: Author,
    content_type: String,
    state: String,
    size: u32,
    download_count: u32,
    created_at: DateTime<Local>,
    published_at: DateTime<Local>,
    browser_download_url: String,
}

#[derive(Deserialize)]
struct Reactions {
    url: String,
    total_count: u32,
    #[serde(rename = "+1")]
    pos_one: u32,
    #[serde(rename = "-1")]
    neg_one: u32,
    laugh: u32,
    hooray: u32,
    confused: u32,
    heart: u32,
    rocket: u32,
    eyes: u32,
}