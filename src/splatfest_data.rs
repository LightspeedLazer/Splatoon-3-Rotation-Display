#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused)]

extern crate serde;
extern crate chrono;

use crate::schedule_data::{Player, image};

use self::serde::{Deserialize, Serialize};
use self::chrono::{DateTime, Local};

#[derive(Debug, Deserialize)]
pub struct SplatfestData {
    pub US: region,
    pub EU: region,
    pub JP: region,
    pub AP: region,
}

#[derive(Debug, Deserialize)]
pub struct region {
    pub data: data,
}

#[derive(Debug, Deserialize)]
pub struct data {
    pub festRecords: nodes,
    pub currentPlayer: Player,
}

#[derive(Debug, Deserialize)]
pub struct nodes {
    pub nodes: Vec<splatfest>,
}

#[derive(Debug, Deserialize)]
pub struct splatfest {
    pub __splatoon3ink_id: String,
    pub id: String,
    pub state: String,
    pub startTime: DateTime<Local>,
    pub endTime: DateTime<Local>,
    pub title: String,
    pub lang: String,
    pub image: image,
    pub playerResult: Option<()>,
    pub teams: (team, team, team),
    pub myTeam: Option<()>,
    pub __typename: String,
    pub isVotable: bool,
    pub undecidedVotes: Option<Votes>,
}

#[derive(Debug, Deserialize)]
pub struct team {
    pub result: Option<result>,
    pub id: String,
    pub teamName: String,
    pub color: Color,
    pub image: image,
    pub myVoteState: Option<()>,
    pub preVotes: Option<Votes>,
    pub votes: Option<Votes>,
    pub role: Option<String>
}

#[derive(Debug, Deserialize)]
pub struct result {
    pub __typename: String,
    pub isWinner: bool,
    pub horagaiRatio: f64,
    pub isHoragaiRatioTop: bool,
    pub voteRatio: f64,
    pub isVoteRatioTop: bool,
    pub regularContributionRatio: f64,
    pub isRegularContributionRatioTop: bool,
    pub challengeContributionRatio: f64,
    pub isChallengeContributionRatioTop: bool,
    pub tricolorContributionRatio: Option<f64>,
    pub isTricolorContributionRatioTop: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Color {
    pub a: f64,
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

#[derive(Debug, Deserialize)]
pub struct Votes {
    pub totalCount: usize,
}
