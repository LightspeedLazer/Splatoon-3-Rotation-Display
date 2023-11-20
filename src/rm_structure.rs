extern crate chrono;
use std::ops::{Mul, Div};

use crate::rm_write::{Color, ToRM, RmObject, ObjectType, MeterType, StringOptions, MeterOptions, ImageOptions, MeasureType, PluginType, SplatinkType, TimeBarOptions, MeasureOptions, BarOptions, BarOrientation, ToolTip};

const DISPLAY_TIME_FORMAT: &str = "%a %-I%P";
const TOOLTIP_TIME_FORMAT: &str = "%-m/%-d %-I%P";
use self::chrono::{DateTime, Local};

pub trait Download {
    fn download(&self, dir_path: &str) -> Vec<Result<(), String>>;
}
impl <T: Download> Download for Vec<T> {
    fn download(&self, dir_path: &str) -> Vec<Result<(), String>> {
        let mut ret = Vec::new();
        for ele in self {
            ret.append(&mut ele.download(dir_path));
        }
        ret
    }
}

#[derive(Clone)]
pub struct Image {
    pub url: String,
}
impl Image {
    pub fn download(&self, name: &str, dir_path: &str) -> Result<(), String> {
        let path = format!("{dir_path}/{name}.png");
        if !std::fs::read_dir(dir_path).map_err(|e| format!("Failed To Read Directory: {e:?}"))?
            .map(|x|
                x.unwrap()
                .file_name()
                .to_str()
                .unwrap()
                .to_string()
            )
            .collect::<Vec<_>>()
            .contains(&path)
        {
            std::fs::write(path, reqwest::blocking::get(&self.url)
                .map_err(|e| format!("Failed To Fetch {name}: {e:?}"))?
                .bytes()
                .map_err(|e| format!("Failed To Convert {name} To Bytes: {e:?}"))?
            ).map_err(|e| format!("Failed To Write To File: {e:?}"))?;
        }
        Ok(())
    }
}

fn new_timebar(start_time: &DateTime<Local>, end_time: &DateTime<Local>) -> Vec<RmObject> {
    let mut ret = Vec::new();
    ret.push(RmObject::new(ObjectType::Measure(
        MeasureType::Plugin(
            PluginType::Splatink(
                SplatinkType::TimeBar(TimeBarOptions {
                    start_time: start_time.timestamp(),
                    end_time: end_time.timestamp(),
                })
            )
        ),
        MeasureOptions::default()
    )).prefix_name_owned("TimeBarMeasure"));
    ret.push(RmObject::new(ObjectType::Meter(
        MeterType::Bar(
            BarOptions {
                bar_color: (150,50,50,255).into(),
                bar_orientation: BarOrientation::Horizontal
            }
        ),
        {
            let mut ret = MeterOptions::new();
            ret.size = (100,50).into();
            ret.measure_name = Some("Measure".to_string());
            ret.solid_color = Some((50,50,50,255).into());
            ret
        }
    )).prefix_name_owned("TimeBar"));
    ret.push(RmObject::new(ObjectType::Meter(
        MeterType::String(
            {
                let mut ret = StringOptions::default();
                ret.text = start_time.format(DISPLAY_TIME_FORMAT).to_string();
                ret
            }
        ),
        {
            let mut ret = MeterOptions::new();
            ret.pos = (25,25).into();
            ret.size = (50,50).into();
            ret.tool_tip = Some(ToolTip::new(start_time.format(TOOLTIP_TIME_FORMAT).to_string()));
            ret
        }
    )).prefix_name_owned("StartTime"));
    ret.push(RmObject::new(ObjectType::Meter(
        MeterType::String(
            {
                let mut ret = StringOptions::default();
                ret.text = end_time.format(DISPLAY_TIME_FORMAT).to_string();
                ret
            }
        ),
        {
            let mut ret = MeterOptions::new();
            ret.pos = (75,25).into();
            ret.size = (50,50).into();
            ret.tool_tip = Some(ToolTip::new(end_time.format(TOOLTIP_TIME_FORMAT).to_string()));
            ret
        }
    )).prefix_name_owned("EndTime"));
    ret
}

impl ToRM for (DateTime<Local>, DateTime<Local>) {
    fn to_rm(&self) -> Vec<RmObject> {
        new_timebar(&self.0, &self.1)
    }
}

pub struct RmStructure {
    pub schedules: Vec<Box<dyn Sche>>,
    pub splatfest: Option<Splatfest>,
}
impl RmStructure {
    pub fn generate(schedule_data: &crate::schedule_data::RotationData, splatfest_data: &crate::splatfest_data::SplatfestData) -> Self {
        let mut active_schedules: Vec<Box<dyn Sche>> = Vec::new();
        let mut active_ids = Vec::new();
        //-----------------------------Regular Schedule-----------------------------
        let mut regular_schedule = Vec::new();
        for ele in schedule_data.data.regularSchedules.nodes.iter() {
            if let Some(setting) = &ele.regularMatchSetting {
                regular_schedule.push(
                    VsEvent{
                        run_time: (ele.startTime.clone(), ele.endTime.clone()),
                        vs_setting: VsSetting{
                            vs_rule: VsRule{
                                name: setting.vsRule.name.clone(),
                            },
                            vs_stages: (
                                Stage{
                                    name: setting.vsStages.0.name.clone(),
                                    image: Image{
                                        url: setting.vsStages.0.image.url.clone()
                                    }
                                },
                                Stage{
                                    name: setting.vsStages.1.name.clone(),
                                    image: Image{
                                        url: setting.vsStages.1.image.url.clone()
                                    }
                                }
                            )
                        }
                    }
                );
            }
        }
        if !regular_schedule.is_empty() {
            active_ids.push("RegSche".to_string());
            active_schedules.push(
                Box::new(
                    Schedule::<VsEvent> {
                        title: "Regular".to_string(),
                        id: "RegSche".to_string(),
                        prev_sche: String::new(),
                        next_sche: String::new(),
                        events: regular_schedule,
                    }
                )
            );
        }
        //-----------------------------Anarchy Schedules-----------------------------
        let mut series_schedule = Vec::new();
        let mut open_schedule = Vec::new();
        for ele in schedule_data.data.bankaraSchedules.nodes.iter() {
            if let Some((series_setting, open_setting)) = &ele.bankaraMatchSettings {
                series_schedule.push(
                    VsEvent{
                        run_time: (ele.startTime.clone(), ele.endTime.clone()),
                        vs_setting: VsSetting{
                            vs_rule: VsRule{
                                name: series_setting.vsRule.name.clone(),
                            },
                            vs_stages: (
                                Stage{
                                    name: series_setting.vsStages.0.name.clone(),
                                    image: Image{
                                        url: series_setting.vsStages.0.image.url.clone()
                                    }
                                },
                                Stage{
                                    name: series_setting.vsStages.1.name.clone(),
                                    image: Image{
                                        url: series_setting.vsStages.1.image.url.clone()
                                    }
                                }
                            )
                        }
                    }
                );
                open_schedule.push(
                    VsEvent{
                        run_time: (ele.startTime.clone(), ele.endTime.clone()),
                        vs_setting: VsSetting{
                            vs_rule: VsRule{
                                name: open_setting.vsRule.name.clone(),
                            },
                            vs_stages: (
                                Stage{
                                    name: open_setting.vsStages.0.name.clone(),
                                    image: Image{
                                        url: open_setting.vsStages.0.image.url.clone()
                                    }
                                },
                                Stage{
                                    name: open_setting.vsStages.1.name.clone(),
                                    image: Image{
                                        url: open_setting.vsStages.1.image.url.clone()
                                    }
                                }
                            )
                        }
                    }
                );
            }
        }
        if !series_schedule.is_empty() {
            active_ids.push("BanSeriesSche".to_string());
            active_schedules.push(
                Box::new(
                    Schedule::<VsEvent> {
                        title: "Series".to_string(),
                        id: "BanSeriesSche".to_string(),
                        prev_sche: String::new(),
                        next_sche: String::new(),
                        events: series_schedule,
                    }
                )
            );
            
        }
        if !open_schedule.is_empty() {
            active_ids.push("BanOpenSche".to_string());
            active_schedules.push(
                Box::new(
                    Schedule::<VsEvent> {
                        title: "Open".to_string(),
                        id: "BanOpenSche".to_string(),
                        prev_sche: String::new(),
                        next_sche: String::new(),
                        events: open_schedule,
                    }
                )
            );
        }
        //-----------------------------X Schedule-----------------------------
        let mut x_schedule = Vec::new();
        for ele in schedule_data.data.xSchedules.nodes.iter() {
            if let Some(setting) = &ele.xMatchSetting {
                x_schedule.push(
                    VsEvent{
                        run_time: (ele.startTime.clone(), ele.endTime.clone()),
                        vs_setting: VsSetting{
                            vs_rule: VsRule{
                                name: setting.vsRule.name.clone(),
                            },
                            vs_stages: (
                                Stage{
                                    name: setting.vsStages.0.name.clone(),
                                    image: Image{
                                        url: setting.vsStages.0.image.url.clone()
                                    }
                                },
                                Stage{
                                    name: setting.vsStages.1.name.clone(),
                                    image: Image{
                                        url: setting.vsStages.1.image.url.clone()
                                    }
                                }
                            )
                        }
                    }
                );
            }
        }
        if !x_schedule.is_empty() {
            active_ids.push("xSche".to_string());
            active_schedules.push(
                Box::new(
                    Schedule::<VsEvent> {
                        title: "X Battles".to_string(),
                        id: "xSche".to_string(),
                        prev_sche: String::new(),
                        next_sche: String::new(),
                        events: x_schedule,
                    }
                )
            );
        }
        //-----------------------------Splatfest Schedules-----------------------------
        let mut splatfest_open_schedule = Vec::new();
        let mut splatfest_pro_schedule = Vec::new();
        for ele in schedule_data.data.festSchedules.nodes.iter() {
            if let Some((pro_setting, open_setting)) = &ele.festMatchSettings {
                splatfest_open_schedule.push(
                    VsEvent{
                        run_time: (ele.startTime.clone(), ele.endTime.clone()),
                        vs_setting: VsSetting{
                            vs_rule: VsRule{
                                name: open_setting.vsRule.name.clone(),
                            },
                            vs_stages: (
                                Stage{
                                    name: open_setting.vsStages.0.name.clone(),
                                    image: Image{
                                        url: open_setting.vsStages.0.image.url.clone()
                                    }
                                },
                                Stage{
                                    name: open_setting.vsStages.1.name.clone(),
                                    image: Image{
                                        url: open_setting.vsStages.1.image.url.clone()
                                    }
                                }
                            )
                        }
                    }
                );
                splatfest_pro_schedule.push(
                    VsEvent{
                        run_time: (ele.startTime.clone(), ele.endTime.clone()),
                        vs_setting: VsSetting{
                            vs_rule: VsRule{
                                name: pro_setting.vsRule.name.clone(),
                            },
                            vs_stages: (
                                Stage{
                                    name: pro_setting.vsStages.0.name.clone(),
                                    image: Image{
                                        url: pro_setting.vsStages.0.image.url.clone()
                                    }
                                },
                                Stage{
                                    name: pro_setting.vsStages.1.name.clone(),
                                    image: Image{
                                        url: pro_setting.vsStages.1.image.url.clone()
                                    }
                                }
                            )
                        }
                    }
                );
            }
        }
        if !splatfest_open_schedule.is_empty() {
            active_ids.push("SfOpenSche".to_string());
            active_schedules.push(
                Box::new(
                    Schedule::<VsEvent> {
                        title: "Open".to_string(),
                        id: "SfOpenSche".to_string(),
                        prev_sche: String::new(),
                        next_sche: String::new(),
                        events: splatfest_open_schedule,
                    }
                )
            );
        }
        if !splatfest_pro_schedule.is_empty() {
            active_ids.push("SfProSche".to_string());
            active_schedules.push(
                Box::new(
                    Schedule::<VsEvent> {
                        title: "Pro".to_string(),
                        id: "SfProSche".to_string(),
                        prev_sche: String::new(),
                        next_sche: String::new(),
                        events: splatfest_pro_schedule,
                    }
                )
            );
        }
        //-----------------------------Challenge Schedule-----------------------------
        if !schedule_data.data.eventSchedules.nodes.is_empty() {
            active_ids.push("ChalSche".to_string());
            let mut chal_schedule = Vec::new();
            for ele in schedule_data.data.eventSchedules.nodes.iter() {
                if let Some(rule) = &ele.leagueMatchSetting.vsRule {
                    chal_schedule.push(
                        ChalEvent{
                            run_time: {
                                let mut ret = Vec::new();
                                for period in ele.timePeriods.iter() {
                                    ret.push((period.startTime.clone(), period.endTime.clone()));
                                }
                                ret
                            },
                            vs_setting: VsSetting{
                                vs_rule: VsRule{
                                    name: rule.name.clone()
                                },
                                vs_stages: (
                                    Stage{
                                        name: ele.leagueMatchSetting.vsStages.0.name.clone(),
                                        image: Image{
                                            url: ele.leagueMatchSetting.vsStages.0.image.url.clone()
                                        }
                                    },
                                    Stage{
                                        name: ele.leagueMatchSetting.vsStages.1.name.clone(),
                                        image: Image{
                                            url: ele.leagueMatchSetting.vsStages.1.image.url.clone()
                                        }
                                    }
                                )
                            },
                            title: ele.leagueMatchSetting.leagueMatchEvent.name.clone(),
                            desc: ele.leagueMatchSetting.leagueMatchEvent.desc.replace("<br />", " ").replace("・", " *"),
                            details: ele.leagueMatchSetting.leagueMatchEvent.regulation.replace("<br />", " ").replace("・", " *"),
                        }
                    );
                }
            }
            active_schedules.push(
                Box::new(
                    Schedule::<ChalEvent> {
                        title: "Challenge".to_string(),
                        id: "ChalSche".to_string(),
                        prev_sche: String::new(),
                        next_sche: String::new(),
                        events: chal_schedule,
                    }
                )
            );
        }
        //-----------------------------Salmon Run Schedule-----------------------------
        // if !schedule_data.data.coopGroupingSchedule..nodes.is_empty() {
            active_ids.push("CoopSche".to_string());
            let mut coop_schedule = Vec::new();
            for ele in schedule_data.data.coopGroupingSchedule.regularSchedules.nodes.iter() {
                coop_schedule.push(
                    SalmonRunEvent{
                        run_time: (ele.startTime.clone(), ele.endTime.clone()),
                        coop_setting: SalmonRunSetting {
                            coop_stage: Stage{
                                name: ele.setting.coopStage.name.clone(),
                                image: Image{
                                    url: ele.setting.coopStage.image.url.clone()
                                }
                            },
                            weapons: {
                                let mut ret = Vec::new();
                                for weapon in ele.setting.weapons.iter() {
                                    ret.push(
                                        Weapon{
                                            name: weapon.name.clone(),
                                            image: Image{
                                                url: weapon.image.url.clone()
                                            }
                                        }
                                    )
                                }
                                ret
                            },
                            special: false
                        },
                        king_guess: ele.__splatoon3ink_king_salmonid_guess.clone()
                    }
                )
            }
            for ele in schedule_data.data.coopGroupingSchedule.bigRunSchedules.nodes.iter() {
                coop_schedule.push(
                    SalmonRunEvent{
                        run_time: (ele.startTime.clone(), ele.endTime.clone()),
                        coop_setting: SalmonRunSetting {
                            coop_stage: Stage{
                                name: ele.setting.coopStage.name.clone(),
                                image: Image{
                                    url: ele.setting.coopStage.image.url.clone()
                                }
                            },
                            weapons: {
                                let mut ret = Vec::new();
                                for weapon in ele.setting.weapons.iter() {
                                    ret.push(
                                        Weapon{
                                            name: weapon.name.clone(),
                                            image: Image{
                                                url: weapon.image.url.clone()
                                            }
                                        }
                                    )
                                }
                                ret
                            },
                            special: true
                        },
                        king_guess: ele.__splatoon3ink_king_salmonid_guess.clone()
                    }
                )
            }
            for ele in schedule_data.data.coopGroupingSchedule.teamContestSchedules.nodes.iter() {
                coop_schedule.push(
                    SalmonRunEvent{
                        run_time: (ele.startTime.clone(), ele.endTime.clone()),
                        coop_setting: SalmonRunSetting {
                            coop_stage: Stage{
                                name: ele.setting.coopStage.name.clone(),
                                image: Image{
                                    url: ele.setting.coopStage.image.url.clone()
                                }
                            },
                            weapons: {
                                let mut ret = Vec::new();
                                for weapon in ele.setting.weapons.iter() {
                                    ret.push(
                                        Weapon{
                                            name: weapon.name.clone(),
                                            image: Image{
                                                url: weapon.image.url.clone()
                                            }
                                        }
                                    )
                                }
                                ret
                            },
                            special: true
                        },
                        king_guess: "None".to_string()
                    }
                )
            }
            coop_schedule.sort_by(|a, b| a.run_time.0.partial_cmp(&b.run_time.0).unwrap());
            active_schedules.push(
                Box::new(
                    Schedule::<SalmonRunEvent> {
                        title: "Salmon Run".to_string(),
                        id: "CoopSche".to_string(),
                        prev_sche: String::new(),
                        next_sche: String::new(),
                        events: coop_schedule,
                    }
                )
            );
        // }
        for (i, sche) in active_schedules.iter_mut().enumerate() {
            sche.set_schedules(
                active_ids[(i as isize - 1).rem_euclid(active_ids.len() as isize) as usize].clone(),
                active_ids[(i as isize + 1).rem_euclid(active_ids.len() as isize) as usize].clone()
            )
        }

        let splatfest = match splatfest_data.US.data.festRecords.nodes.first() {
                Some(s) => {
                    if chrono::Local::now() < (s.endTime + chrono::Duration::days(1)) {
                        let teams = (
                            SplatfestTeam{
                                name: s.teams.0.teamName.clone(),
                                color: (
                                    s.teams.0.color.r.mul(255.0).round() as isize,
                                    s.teams.0.color.g.mul(255.0).round() as isize,
                                    s.teams.0.color.b.mul(255.0).round() as isize,
                                    s.teams.0.color.a.mul(255.0).round() as isize,
                                ).into(),
                                image: Image{
                                    url: s.teams.0.image.url.clone()
                                }
                            },
                            SplatfestTeam{
                                name: s.teams.1.teamName.clone(),
                                color: (
                                    s.teams.1.color.r.mul(255.0).round() as isize,
                                    s.teams.1.color.g.mul(255.0).round() as isize,
                                    s.teams.1.color.b.mul(255.0).round() as isize,
                                    s.teams.1.color.a.mul(255.0).round() as isize,
                                ).into(),
                                image: Image{
                                    url: s.teams.1.image.url.clone()
                                }
                            },
                            SplatfestTeam{
                                name: s.teams.2.teamName.clone(),
                                color: (
                                    s.teams.2.color.r.mul(255.0).round() as isize,
                                    s.teams.2.color.g.mul(255.0).round() as isize,
                                    s.teams.2.color.b.mul(255.0).round() as isize,
                                    s.teams.2.color.a.mul(255.0).round() as isize,
                                ).into(),
                                image: Image{
                                    url: s.teams.2.image.url.clone()
                                }
                            }
                        );
                        Some(
                            Splatfest{
                                run_time: (s.startTime.clone(), s.endTime.clone()),
                                title: s.title.clone(),
                                teams: teams.clone(),
                                state: {
                                    let mut ret = SplatfestState::MissingTricolor;
                                    if let Some(f) = &schedule_data.data.currentFest {
                                        ret = SplatfestState::Active(
                                            f.midtermTime.clone(),
                                            Stage{
                                                name: f.tricolorStage.name.clone(),
                                                image: Image{
                                                    url: f.tricolorStage.image.url.clone()
                                                }
                                            }
                                        );
                                    }
                                    if let Some(r0) = &s.teams.0.result {
                                        if let Some(r1) = &s.teams.1.result {
                                            if let Some(r2) = &s.teams.2.result {
                                                ret = SplatfestState::Finished(
                                                    SplatfestResults{
                                                        winner: {
                                                            let mut ret = teams.0.clone();
                                                            if r1.isWinner {
                                                                ret = teams.1.clone();
                                                            }
                                                            if r2.isWinner {
                                                                ret = teams.2.clone();
                                                            }
                                                            ret
                                                        },
                                                        team_results: (
                                                            SplatfestTeamResult{
                                                                sneak_peak: (r0.isHoragaiRatioTop,                  r0.horagaiRatio),
                                                                votes:      (r0.isVoteRatioTop,                     r0.voteRatio),
                                                                open:       (r0.isRegularContributionRatioTop,      r0.regularContributionRatio),
                                                                pro:        (r0.isChallengeContributionRatioTop,    r0.challengeContributionRatio),
                                                                tricolor:   (r0.isTricolorContributionRatioTop.unwrap_or(false),     r0.tricolorContributionRatio.unwrap_or(0.0)),
                                                            },
                                                            SplatfestTeamResult{
                                                                sneak_peak: (r1.isHoragaiRatioTop,                  r1.horagaiRatio),
                                                                votes:      (r1.isVoteRatioTop,                     r1.voteRatio),
                                                                open:       (r1.isRegularContributionRatioTop,      r1.regularContributionRatio),
                                                                pro:        (r1.isChallengeContributionRatioTop,    r1.challengeContributionRatio),
                                                                tricolor:   (r1.isTricolorContributionRatioTop.unwrap_or(false),     r1.tricolorContributionRatio.unwrap_or(0.0)),
                                                            },
                                                            SplatfestTeamResult{
                                                                sneak_peak: (r2.isHoragaiRatioTop,                  r2.horagaiRatio),
                                                                votes:      (r2.isVoteRatioTop,                     r2.voteRatio),
                                                                open:       (r2.isRegularContributionRatioTop,      r2.regularContributionRatio),
                                                                pro:        (r2.isChallengeContributionRatioTop,    r2.challengeContributionRatio),
                                                                tricolor:   (r2.isTricolorContributionRatioTop.unwrap_or(false),     r2.tricolorContributionRatio.unwrap_or(0.0)),
                                                            }
                                                        )
                                                    }
                                                );
                                            }
                                        }
                                    }
                                    ret
                                }
                            }
                        )
                    } else {
                        None
                    }
                },
                None => None,
        };
        RmStructure{
            schedules: active_schedules,
            splatfest: splatfest
        }
    }
}
impl ToRM for RmStructure {
    fn to_rm(&self) -> Vec<RmObject> {
        let mut ret = Vec::new();
        let mut schedules = {
            let mut ret = Vec::new();
            for ele in self.schedules.iter() {
                ret.append(&mut ele.to_rm());
            }
            ret
        };
        if let Some(splatfest) = &self.splatfest {
            match splatfest.state {
                SplatfestState::MissingTricolor | SplatfestState::Active(_, _) => {
                    for obj in schedules.iter_mut() {
                        if let ObjectType::Meter(_, ref mut o) = obj.object_type {
                            o.pos += (0,150).into();
                        }
                    }
                },
                SplatfestState::Finished(_) => {
                    for obj in schedules.iter_mut() {
                        if let ObjectType::Meter(_, ref mut o) = obj.object_type {
                            o.pos += (0,400).into();
                        }
                    }
                },
            }
            ret.append(&mut splatfest.to_rm());
        }
        ret.append(&mut schedules);
        ret
    }
}
impl Download for RmStructure {
    fn download(&self, dir_path: &str) -> Vec<Result<(), String>> {
        let mut ret = Vec::new();
        for ele in self.schedules.iter() {
            ret.append(&mut ele.download(dir_path));
        }
        if let Some(s) = &self.splatfest {
            ret.append(&mut s.download(dir_path));
        }
        ret
    }
}

pub trait Sche: ToRM + Download {
    fn set_schedules(&mut self, prev: String, next: String);
    fn get_id(&self) -> &str;
}

pub struct Schedule<T: ToRM + Download> {
    pub title: String,
    pub id: String,
    pub prev_sche: String,
    pub next_sche: String,
    pub events: Vec<T>,
}
impl <T: ToRM + Download> Sche for Schedule<T> {
    fn set_schedules(&mut self, prev: String, next: String) {
        self.prev_sche = prev;
        self.next_sche = next;
    }
    fn get_id(&self) -> &str {
        &self.id
    }
}
impl <T: ToRM + Download> Download for Schedule<T> {
    fn download(&self, dir_path: &str) -> Vec<Result<(), String>> {
        self.events.download(dir_path)
    }
}
impl <T: ToRM + Download> ToRM for Schedule<T> {
    fn to_rm(&self) -> Vec<RmObject> {
        let mut ret = Vec::new();
        ret.push(
            RmObject::new(
                ObjectType::Meter(
                    MeterType::Image(
                        ImageOptions{
                            image_name: format!("#@#Schedule Types/{}.png", self.prev_sche),
                            preseve_aspect_ratio: false,
                        }
                    ),
                    {
                        let mut ret = MeterOptions::new();
                        ret.pos = (0,0).into();
                        ret.size = (50,50).into();
                        ret.solid_color = Some((50,50,50,255).into());
                        ret.left_click_action.push(format!("!CommandMeasure SplatinkCore \"redrawsche {}\"", self.prev_sche));
                        ret
                    }
                )
            ).prefix_name_owned("PrevScheImage")
        );
        ret.push(
            RmObject::new(
                ObjectType::Meter(
                    MeterType::String(
                        {
                            let mut ret = StringOptions::default();
                            ret.text = "<<".to_string();
                            ret
                        }
                    ),
                    {
                        let mut ret = MeterOptions::new();
                        ret.pos = (75,25).into();
                        ret.size = (50,50).into();
                        ret.solid_color = Some((50,50,50,255).into());
                        ret.left_click_action.push(format!("!CommandMeasure SplatinkCore \"redrawsche {}\"", self.prev_sche));
                        ret
                    }
                )
            ).prefix_name_owned("PrevScheArrow")
        );
        ret.push(
            RmObject::new(
                ObjectType::Meter(
                    MeterType::Image(
                        ImageOptions{
                            image_name: format!("#@#Schedule Types/{}.png", self.id),
                            preseve_aspect_ratio: false,
                        }
                    ),
                    {
                        let mut ret = MeterOptions::new();
                        ret.pos = (100,0).into();
                        ret.size = (50,50).into();
                        ret.solid_color = Some((40,40,40,255).into());
                        ret
                    }
                )
            ).prefix_name_owned("CurrScheImage")
        );
        ret.push(
            RmObject::new(
                ObjectType::Meter(
                    MeterType::String(
                        {
                            let mut ret = StringOptions::default();
                            ret.text = self.title.clone();
                            ret
                        }
                    ),
                    {
                        let mut ret = MeterOptions::new();
                        ret.pos = (200,25).into();
                        ret.size = (100,50).into();
                        ret.solid_color = Some((40,40,40,255).into());
                        ret
                    }
                )
            ).prefix_name_owned("CurrScheTitle")
        );
        ret.push(
            RmObject::new(
                ObjectType::Meter(
                    MeterType::String(
                        {
                            let mut ret = StringOptions::default();
                            ret.text = ">>".to_string();
                            ret
                        }
                    ),
                    {
                        let mut ret = MeterOptions::new();
                        ret.pos = (275,25).into();
                        ret.size = (50,50).into();
                        ret.solid_color = Some((50,50,50,255).into());
                        ret.left_click_action.push(format!("!CommandMeasure SplatinkCore \"redrawsche {}\"", self.next_sche));
                        ret
                    }
                )
            ).prefix_name_owned("NextScheArrow")
        );
        ret.push(
            RmObject::new(
                ObjectType::Meter(
                    MeterType::Image(
                        ImageOptions{
                            image_name: format!("#@#Schedule Types/{}.png", self.next_sche),
                            preseve_aspect_ratio: false,
                        }
                    ),
                    {
                        let mut ret = MeterOptions::new();
                        ret.pos = (300,0).into();
                        ret.size = (50,50).into();
                        ret.solid_color = Some((50,50,50,255).into());
                        ret.left_click_action.push(format!("!CommandMeasure SplatinkCore \"redrawsche {}\"", self.next_sche));
                        ret
                    }
                )
            ).prefix_name_owned("NextScheImage")
        );


        ret.append(&mut {
            let mut ret = Vec::new();
            let mut vert_size_accum = 50;
            for (i, ele) in self.events.iter().enumerate() {
                let mut vert_size = 50;
                ret.append(&mut {
                    let mut ret = ele.to_rm();
                    for obj in ret.iter_mut() {
                        if let ObjectType::Meter(ref t, ref mut o) = obj.prefix_name_mut(&format!("{i}")).object_type {
                            vert_size = match t {
                                MeterType::String(s) if s.string_align == Some(crate::rm_write::StringAlign::CenterCenter) => vert_size.max(o.pos.y + o.size.y / 2),
                                _ => vert_size.max(o.pos.y + o.size.y),
                            };
                            o.pos += (0, vert_size_accum).into();
                        }
                    }
                    ret
                });
                vert_size_accum += vert_size;
            }
            ret
        });

        for obj in ret.iter_mut() {
            obj.prefix_name_mut(&self.id);
            if let ObjectType::Meter(_, ref mut o) = obj.object_type {
                o.groups.push(self.id.clone());
                o.scroll_down_action.push(format!("!CommandMeasure SplatinkCore \"redrawsche {}\"", self.next_sche));
                o.scroll_up_action.push(format!("!CommandMeasure SplatinkCore \"redrawsche {}\"", self.prev_sche));
                if self.id != "RegSche" {
                    o.hidden = true;
                }
            }
        }
        ret
    }
}

pub struct VsEvent {
    pub run_time: (DateTime<Local>, DateTime<Local>),
    pub vs_setting: VsSetting,
}
impl ToRM for VsEvent {
    fn to_rm(&self) -> Vec<RmObject> {
        let mut ret = Vec::new();
        ret.append(&mut {
            let mut ret = self.run_time.to_rm();

            ret
        });
        ret.append(&mut {
            let mut ret = self.vs_setting.to_rm();
            for obj in ret.iter_mut() {
                if let ObjectType::Meter(_, ref mut o) = obj.object_type {
                    o.pos += (100, 0).into();
                }
            }
            ret
        });
        ret
    }
}
impl Download for VsEvent {
    fn download(&self, dir_path: &str) -> Vec<Result<(), String>> {
        self.vs_setting.download(dir_path)
    }
}

pub struct VsSetting {
    pub vs_rule: VsRule,
    pub vs_stages: (Stage, Stage),
}
impl ToRM for VsSetting {
    fn to_rm(&self) -> Vec<RmObject> {
        let mut ret = Vec::new();
        ret.push({
            let mut ret = self.vs_rule.get_rm_object();
            ret.prefix_name_mut("Mode");
            if let ObjectType::Meter(_, ref mut o) = ret.object_type {
                o.pos += (0,0).into();
            }
            ret
        });
        ret.push({
            let mut ret = self.vs_stages.0.get_rm_object();
            ret.prefix_name_mut("Stage0");
            if let ObjectType::Meter(_, ref mut o) = ret.object_type {
                o.pos += (50,0).into();
            }
            ret
        });
        ret.push({
            let mut ret = self.vs_stages.1.get_rm_object();
            ret.prefix_name_mut("Stage1");
            if let ObjectType::Meter(_, ref mut o) = ret.object_type {
                o.pos += (150,0).into();
            }
            ret
        });
        ret
    }
}
impl Download for VsSetting {
    fn download(&self, dir_path: &str) -> Vec<Result<(), String>> {
        let mut ret = Vec::new();
        ret.append(&mut self.vs_stages.0.download(dir_path));
        ret.append(&mut self.vs_stages.1.download(dir_path));
        ret
    }
}

pub struct VsRule {
    pub name: String,
}
impl VsRule {
    pub fn get_rm_object(&self) -> RmObject {
        RmObject::new(
            ObjectType::Meter(
                MeterType::Image(
                    ImageOptions{
                        image_name: format!("#@#Modes/{}.png", self.name),
                        preseve_aspect_ratio: false,
                    }
                ),
                {
                    let mut ret = MeterOptions::new();
                    ret.size = (50,50).into();
                    ret.solid_color = Some((40,40,40,255).into());
                    ret.tool_tip = Some(ToolTip::new(self.name.clone()));
                    ret
                }
            )
        )
    }
}

pub struct Stage {
    pub name: String,
    pub image: Image,
}
impl Stage {
    pub fn get_rm_object(&self) -> RmObject {
        RmObject::new(
            ObjectType::Meter(
                MeterType::Image(
                    ImageOptions{
                        image_name: format!("#@#Stages/{}.png", self.name),
                        preseve_aspect_ratio: false,
                    }
                ),
                {
                    let mut ret = MeterOptions::new();
                    ret.size = (100,50).into();
                    ret.solid_color = Some((30,30,30,255).into());
                    ret.tool_tip = Some(ToolTip::new(self.name.clone()));
                    ret
                }
            )
        )
    }
}
impl Download for Stage {
    fn download(&self, dir_path: &str) -> Vec<Result<(), String>> {
        vec![self.image.download(&self.name, &format!("{dir_path}/Stages"))]
    }
}

pub struct ChalEvent {
    pub run_time: Vec<(DateTime<Local>, DateTime<Local>)>,
    pub vs_setting: VsSetting,
    pub title: String,
    pub desc: String,
    pub details: String,
}
impl ToRM for ChalEvent {
    fn to_rm(&self) -> Vec<RmObject> {
        let mut ret = Vec::new();
        ret.push(
            RmObject::new(
                ObjectType::Meter(
                    MeterType::Image(
                        ImageOptions{
                            image_name: String::new(),
                            preseve_aspect_ratio: false,
                        }
                    ),
                    {
                        let mut ret = MeterOptions::new();
                        ret.size = (100,150).into();
                        ret.solid_color = Some((50,50,50,255).into());
                        ret
                    }
                )
            ).prefix_name_owned("Background")
        );
        for (i, ele) in self.run_time.iter().enumerate() {
            ret.append(&mut {
                let mut ret = ele.to_rm();
                for obj in ret.iter_mut() {
                    obj.prefix_name_mut(&format!("Time{i}"));
                    if let ObjectType::Meter(_, ref mut o) = obj.object_type {
                        o.pos += (0,(75 - self.run_time.len() * 25 + 50 * i) as isize).into();
                    }
                }
                ret
            });
        }
        ret.append(&mut {
            let mut ret = self.vs_setting.to_rm();
            for obj in ret.iter_mut() {
                if let ObjectType::Meter(_, ref mut o) = obj.object_type {
                    o.pos += (100,0).into();
                }
            }
            ret
        });
        ret.push({
            RmObject::new(ObjectType::Meter(
                MeterType::String(
                    {
                        let mut ret = StringOptions::default();
                        ret.text = self.title.clone();
                        ret
                    }
                ),
                {
                    let mut ret = MeterOptions::new();
                    ret.pos = (225,75).into();
                    ret.size = (250,50).into();
                    ret.solid_color = Some((40,40,40,255).into());
                    ret.tool_tip = Some(ToolTip::new(self.details.clone()));
                    ret
                }
            ))
        }.prefix_name_owned("Title"));
        ret.push({
            RmObject::new(ObjectType::Meter(
                MeterType::String(
                    {
                        let mut ret = StringOptions::default();
                        ret.text = self.desc.clone();
                        ret.font_size = Some(10_f64);
                        ret.font_weight = Some(400);
                        ret
                    }
                ),
                {
                    let mut ret = MeterOptions::new();
                    ret.pos = (225,125).into();
                    ret.size = (250,50).into();
                    ret.solid_color = Some((40,40,40,255).into());
                    ret.tool_tip = Some(ToolTip::new(self.details.clone()));
                    ret
                }
            ))
        }.prefix_name_owned("Desc"));
        ret
    }
}
impl Download for ChalEvent {
    fn download(&self, dir_path: &str) -> Vec<Result<(), String>> {
        self.vs_setting.download(dir_path)
    }
}

pub struct SalmonRunEvent {
    pub run_time: (DateTime<Local>, DateTime<Local>),
    pub coop_setting: SalmonRunSetting,
    pub king_guess: String,
}
impl ToRM for SalmonRunEvent {
    fn to_rm(&self) -> Vec<RmObject> {
        let mut ret = Vec::new();
        ret.append(&mut {
            let mut ret = self.run_time.to_rm();
            for obj in ret.iter_mut() {
                if let ObjectType::Meter(_, ref mut o) = obj.object_type {
                    o.pos += (0,0).into();
                }
            }
            ret
        });
        ret.append(&mut {
            let mut ret = self.coop_setting.to_rm();
            for obj in ret.iter_mut() {
                if let ObjectType::Meter(_, ref mut o) = obj.object_type {
                    o.pos += (100, 0).into();
                }
            }
            ret
        });
        ret.push(
            RmObject::new(
                ObjectType::Meter(
                    MeterType::Image(
                        ImageOptions{
                            image_name: format!("#@#King Salmonids/{}.png", self.king_guess),
                            preseve_aspect_ratio: false,
                        }
                    ),
                    {
                        let mut ret = MeterOptions::new();
                        ret.pos = (400, 0).into();
                        ret.size = (50, 50).into();
                        ret.solid_color = Some((75,50,50,255).into());
                        ret.tool_tip = Some(ToolTip::new(self.king_guess.clone()));
                        ret
                    }
                )
            ).prefix_name_owned("King")
        );
        ret
    }
}
impl Download for SalmonRunEvent {
    fn download(&self, dir_path: &str) -> Vec<Result<(), String>> {
        self.coop_setting.download(dir_path)
    }
}

pub struct SalmonRunSetting {
    pub coop_stage: Stage,
    pub weapons: Vec<Weapon>,
    pub special: bool,
}
impl ToRM for SalmonRunSetting {
    fn to_rm(&self) -> Vec<RmObject> {
        let mut ret = Vec::new();
        ret.push({
            let mut ret = self.coop_stage.get_rm_object();
            ret.prefix_name_mut("Stage");
            if let ObjectType::Meter(_, ref mut o) = ret.object_type {
                o.pos += (0,0).into();
            }
            ret
        });
        ret.append(&mut {
            let mut ret = Vec::new();
            for (i, weapon) in self.weapons.iter().enumerate() {
                let mut obj = weapon.get_rm_object();
                obj.prefix_name_mut(&format!("Weapon{i}"));
                if let ObjectType::Meter(_, ref mut o) = obj.object_type {
                    o.pos += (100 + 50 * i as isize, 0).into();
                    if self.special {
                        o.solid_color = Some((150,150,30,255).into());
                    }
                }
                ret.push(obj);
            }
            ret
        });
        ret
    }
}
impl Download for SalmonRunSetting {
    fn download(&self, dir_path: &str) -> Vec<Result<(), String>> {
        let mut ret = Vec::new();
        ret.append(&mut self.coop_stage.download(dir_path));
        ret.append(&mut self.weapons.download(dir_path));
        ret
    }
}

pub struct Weapon {
    pub name: String,
    pub image: Image,
}
impl Weapon {
    pub fn get_rm_object(&self) -> RmObject {
        RmObject::new(
            ObjectType::Meter(
                MeterType::Image(ImageOptions {
                    image_name: format!("#@#Weapons/{}.png", self.name),
                    preseve_aspect_ratio: false,
                }),
                {
                    let mut ret = MeterOptions::new();
                    ret.size = (50,50).into();
                    ret.solid_color = Some((30,30,30,255).into());
                    ret.tool_tip = Some(ToolTip::new(self.name.clone()));
                    ret
                }
            )
        )
    }
}
impl Download for Weapon {
    fn download(&self, dir_path: &str) -> Vec<Result<(), String>> {
        vec![self.image.download(&self.name, &format!("{dir_path}/Weapons"))]
    }
}

pub struct Splatfest {
    pub run_time: (DateTime<Local>, DateTime<Local>),
    pub title: String,
    pub teams: (SplatfestTeam, SplatfestTeam, SplatfestTeam),
    pub state: SplatfestState,
}
impl ToRM for Splatfest {
    fn to_rm(&self) -> Vec<RmObject> {
        let mut ret = Vec::new();
        ret.push(
            RmObject::new(
                ObjectType::Meter(
                    MeterType::String(
                        {
                            let mut ret = StringOptions::default();
                            ret.text = self.title.clone();
                            ret
                        }
                    ),
                    {
                        let mut ret = MeterOptions::new();
                        ret.pos = (250, 25).into();
                        ret.size = (300, 50).into();
                        ret.solid_color = Some((50, 50, 50, 255).into());
                        ret
                    }
                )
            ).prefix_name_owned("Title")
        );
        for (i, team) in <[SplatfestTeam; 3]>::from(self.teams.clone()).iter().enumerate() {
            ret.push(
                RmObject::new(
                    ObjectType::Meter(
                        MeterType::Image(
                            ImageOptions{
                                image_name: format!("#@#Splatfest Teams/{}", team.name),
                                preseve_aspect_ratio: true,
                            }
                        ),
                        {
                            let mut ret = MeterOptions::new();
                            ret.pos = (100 + i as isize * 100, 50).into();
                            ret.size = (100, 50).into();
                            ret.solid_color = Some(team.color.clone());
                            ret
                        }
                    )
                ).prefix_name_owned(&format!("Team{i}Image"))
            );
            ret.push(
                RmObject::new(
                    ObjectType::Meter(
                        MeterType::String(
                            {
                                let mut ret = StringOptions::default();
                                ret.text = team.name.clone();
                                ret
                            }
                        ),
                        {
                            let mut ret = MeterOptions::new();
                            ret.pos = (150 + i as isize * 100, 125).into();
                            ret.size = (100, 50).into();
                            ret.solid_color = Some(team.color.clone());
                            ret
                        }
                    )
                ).prefix_name_owned(&format!("Team{i}Name"))
            );
        }
        match &self.state {
            SplatfestState::MissingTricolor => {
                ret.push(
                    RmObject::new(
                        ObjectType::Meter(
                            MeterType::Image(
                                ImageOptions{
                                    image_name: String::new(),
                                    preseve_aspect_ratio: false,
                                }
                            ),
                            {
                                let mut ret = MeterOptions::new();
                                ret.size = (100,150).into();
                                ret.solid_color = Some((50,50,50,255).into());
                                ret
                            }
                        )
                    ).prefix_name_owned("Background")
                );
                ret.append(&mut {
                    let mut ret = new_timebar(&self.run_time.0, &self.run_time.1);
                    for obj in ret.iter_mut() {
                        obj.prefix_name_mut("FullTerm");
                        if let ObjectType::Meter(_, ref mut o) = obj.object_type {
                            o.pos += (0,50).into();
                        }
                    }
                    ret
                });
            },
            SplatfestState::Active(mid_term, tricolor_stage) => {
                ret.append(&mut {
                    let mut ret = new_timebar(&self.run_time.0, mid_term);
                    for obj in ret.iter_mut() {
                        obj.prefix_name_mut("FirstTerm");
                        if let ObjectType::Meter(_, ref mut o) = obj.object_type {
                            o.pos += (0,0).into();
                        }
                    }
                    ret
                });
                ret.push({
                    let mut ret = tricolor_stage.get_rm_object();
                    ret.prefix_name_mut("TricolorStage");
                    if let ObjectType::Meter(_, ref mut o) = ret.object_type {
                        o.pos += (0,50).into();
                    }
                    ret
                });
                ret.append(&mut {
                    let mut ret = new_timebar(mid_term, &self.run_time.1);
                    for obj in ret.iter_mut() {
                        obj.prefix_name_mut("SecondTerm");
                        if let ObjectType::Meter(_, ref mut o) = obj.object_type {
                            o.pos += (0,100).into();
                        }
                    }
                    ret
                });
            },
            SplatfestState::Finished(results) => {
                ret.push(
                    RmObject::new(
                        ObjectType::Meter(
                            MeterType::String(
                                {
                                    let mut ret = StringOptions::default();
                                    ret.text = results.winner.name.clone();
                                    ret
                                }
                            ),
                            {
                                let mut ret = MeterOptions::new();
                                ret.pos = (50,25).into();
                                ret.size = (100,50).into();
                                ret.solid_color = Some(results.winner.color.clone());
                                ret
                            }
                        )
                    ).prefix_name_owned("WinnerTeamName")
                );
                ret.push(
                    RmObject::new(
                        ObjectType::Meter(
                            MeterType::Image(
                                ImageOptions{
                                    image_name: format!("#@#Splatfest Teams/{}", results.winner.name),
                                    preseve_aspect_ratio: false,
                                }
                            ),
                            {
                                let mut ret = MeterOptions::new();
                                ret.pos = (0,50).into();
                                ret.size = (100,100).into();
                                ret.solid_color = Some(results.winner.color.clone());
                                ret
                            }
                        )
                    ).prefix_name_owned("WinnerTeamImage")
                );
                ret.push(
                    RmObject::new(
                        ObjectType::Meter(
                            MeterType::String(
                                {
                                    let mut ret = StringOptions::default();
                                    ret.text = "Sneak Peak".to_string();
                                    ret
                                }
                            ),
                            {
                                let mut ret = MeterOptions::new();
                                ret.pos = (50, 175).into();
                                ret.size = (100, 50).into();
                                ret.solid_color = Some((50,50,50,255).into());
                                ret
                            }
                        )
                    ).prefix_name_owned("SneakPeakRow")
                );
                ret.push(
                    RmObject::new(
                        ObjectType::Meter(
                            MeterType::String(
                                {
                                    let mut ret = StringOptions::default();
                                    ret.text = "Votes".to_string();
                                    ret
                                }
                            ),
                            {
                                let mut ret = MeterOptions::new();
                                ret.pos = (50, 225).into();
                                ret.size = (100, 50).into();
                                ret.solid_color = Some((50,50,50,255).into());
                                ret
                            }
                        )
                    ).prefix_name_owned("VotesRow")
                );
                ret.push(
                    RmObject::new(
                        ObjectType::Meter(
                            MeterType::String(
                                {
                                    let mut ret = StringOptions::default();
                                    ret.text = "Open".to_string();
                                    ret
                                }
                            ),
                            {
                                let mut ret = MeterOptions::new();
                                ret.pos = (50, 275).into();
                                ret.size = (100, 50).into();
                                ret.solid_color = Some((50,50,50,255).into());
                                ret
                            }
                        )
                    ).prefix_name_owned("OpenRow")
                );
                ret.push(
                    RmObject::new(
                        ObjectType::Meter(
                            MeterType::String(
                                {
                                    let mut ret = StringOptions::default();
                                    ret.text = "Pro".to_string();
                                    ret
                                }
                            ),
                            {
                                let mut ret = MeterOptions::new();
                                ret.pos = (50, 325).into();
                                ret.size = (100, 50).into();
                                ret.solid_color = Some((50,50,50,255).into());
                                ret
                            }
                        )
                    ).prefix_name_owned("ProRow")
                );
                ret.push(
                    RmObject::new(
                        ObjectType::Meter(
                            MeterType::String(
                                {
                                    let mut ret = StringOptions::default();
                                    ret.text = "Tricolor".to_string();
                                    ret
                                }
                            ),
                            {
                                let mut ret = MeterOptions::new();
                                ret.pos = (50, 375).into();
                                ret.size = (100, 50).into();
                                ret.solid_color = Some((50,50,50,255).into());
                                ret
                            }
                        )
                    ).prefix_name_owned("TricolorRow")
                );
                for (i, result) in <[SplatfestTeamResult; 3]>::from(results.team_results.clone()).iter().enumerate() {
                    ret.append(&mut {
                        let mut ret = result.to_rm();
                        for obj in ret.iter_mut() {
                            obj.prefix_name_mut(&format!("Team{i}"));
                            if let ObjectType::Meter(_, ref mut o) = obj.object_type {
                                o.pos += (100 + 100 * i as isize,150).into();
                                o.solid_color = Some(<[SplatfestTeam; 3]>::from(self.teams.clone())[i].color.clone());
                            }
                        }
                        ret
                    });
                }
            },
        }

        for ele in ret.iter_mut() {
            ele.prefix_name_mut("Splatfest");
        }
        ret
    }
}
impl Download for Splatfest {
    fn download(&self, dir_path: &str) -> Vec<Result<(), String>> {
        let mut ret = Vec::new();
        ret.append(&mut self.teams.0.download(dir_path));
        ret.append(&mut self.teams.1.download(dir_path));
        ret.append(&mut self.teams.2.download(dir_path));
        if let SplatfestState::Active(_, s) = &self.state {
            ret.append(&mut s.download(dir_path));
        }
        ret
    }
}

#[derive(Clone)]
pub struct SplatfestTeam {
    pub name: String,
    pub color: Color,
    pub image: Image,
}
impl Download for SplatfestTeam {
    fn download(&self, dir_path: &str) -> Vec<Result<(), String>> {
        vec![self.image.download(&self.name, &format!("{dir_path}/Splatfest Teams"))]
    }
}

pub enum SplatfestState {
    MissingTricolor,
    Active(DateTime<Local>, Stage),
    Finished(SplatfestResults),
}

pub struct SplatfestResults {
    pub winner: SplatfestTeam,
    pub team_results: (SplatfestTeamResult, SplatfestTeamResult, SplatfestTeamResult),
}

#[derive(Clone)]
pub struct SplatfestTeamResult {
    pub sneak_peak: (bool, f64),
    pub votes: (bool, f64),
    pub open: (bool, f64),
    pub pro: (bool, f64),
    pub tricolor: (bool, f64),
}
impl ToRM for SplatfestTeamResult {
    fn to_rm(&self) -> Vec<RmObject> {
        let mut ret = Vec::new();
        ret.push(
            RmObject::new(
                ObjectType::Meter(
                    MeterType::String(
                        {
                            let mut ret = StringOptions::default();
                            ret.text = format!("{}%", self.sneak_peak.1.mul(10000.0).round().div(100.0));
                            if self.sneak_peak.0 {
                                ret.font_color = Some((150,150,50,255).into());
                            }
                            ret
                        }
                    ),
                    {
                        let mut ret = MeterOptions::new();
                        ret.pos = (50,25).into();
                        ret.size = (100,50).into();
                        ret.solid_color = Some((30,30,30,255).into());
                        ret
                    }
                )
            ).prefix_name_owned("SneakPeak")
        );
        ret.push(
            RmObject::new(
                ObjectType::Meter(
                    MeterType::String(
                        {
                            let mut ret = StringOptions::default();
                            ret.text = format!("{}%", self.votes.1.mul(10000.0).round().div(100.0));
                            if self.votes.0 {
                                ret.font_color = Some((150,150,50,255).into());
                            }
                            ret
                        }
                    ),
                    {
                        let mut ret = MeterOptions::new();
                        ret.pos = (50,75).into();
                        ret.size = (100,50).into();
                        ret.solid_color = Some((30,30,30,255).into());
                        ret
                    }
                )
            ).prefix_name_owned("Votes")
        );
        ret.push(
            RmObject::new(
                ObjectType::Meter(
                    MeterType::String(
                        {
                            let mut ret = StringOptions::default();
                            ret.text = format!("{}%", self.open.1.mul(10000.0).round().div(100.0));
                            if self.open.0 {
                                ret.font_color = Some((150,150,50,255).into());
                            }
                            ret
                        }
                    ),
                    {
                        let mut ret = MeterOptions::new();
                        ret.pos = (50,125).into();
                        ret.size = (100,50).into();
                        ret.solid_color = Some((30,30,30,255).into());
                        ret
                    }
                )
            ).prefix_name_owned("Open")
        );
        ret.push(
            RmObject::new(
                ObjectType::Meter(
                    MeterType::String(
                        {
                            let mut ret = StringOptions::default();
                            ret.text = format!("{}%", self.pro.1.mul(10000.0).round().div(100.0));
                            if self.pro.0 {
                                ret.font_color = Some((150,150,50,255).into());
                            }
                            ret
                        }
                    ),
                    {
                        let mut ret = MeterOptions::new();
                        ret.pos = (50,175).into();
                        ret.size = (100,50).into();
                        ret.solid_color = Some((30,30,30,255).into());
                        ret
                    }
                )
            ).prefix_name_owned("Pro")
        );
        ret.push(
            RmObject::new(
                ObjectType::Meter(
                    MeterType::String(
                        {
                            let mut ret = StringOptions::default();
                            ret.text = format!("{}%", self.tricolor.1.mul(10000.0).round().div(100.0));
                            if self.tricolor.0 {
                                ret.font_color = Some((150,150,50,255).into());
                            }
                            ret
                        }
                    ),
                    {
                        let mut ret = MeterOptions::new();
                        ret.pos = (50,225).into();
                        ret.size = (100,50).into();
                        ret.solid_color = Some((30,30,30,255).into());
                        ret
                    }
                )
            ).prefix_name_owned("Tricolor")
        );
        ret
    }
}
