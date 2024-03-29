extern crate chrono;
extern crate serde;
extern crate reqwest;
use self::chrono::Duration;
use self::serde::Deserialize;
use self::reqwest::blocking::{ClientBuilder, Client};
use rainmeter::api::RmApi;
use crate::{github_data::Releases, rm_structure::{Download, RmStructure}, rm_write::{write_to_skin, MeasureOptions, MeasureType, ObjectType, PluginType, RmObject, SplatinkType, TimeBarOptions, ToRM}, schedule_data::RotationData, splatfest_data::SplatfestData};

#[allow(non_snake_case)]
pub struct Measure {
    pub rm_api: RmApi,
    pub measure_type: SplatinkType,
    prev_sche: String,
    schedules: Option<RotationData>,
    pub RESOURCE_DIR: String,
    pub SKIN_PATH: String,
    web_pull_cooldown: Duration,
    web_pull_cooldown_set: u32,
    web_client: Client,
}

const SCHEDULE_URL: &str = "https://splatoon3.ink/data/schedules.json";
const SPLATFEST_URL: &str = "https://splatoon3.ink/data/festivals.json";
const GITHUB_RELEASES_URL: &str = "https://api.github.com/repos/LightspeedLazer/Splatoon-3-Rotation-Display/releases";

const SCHEDULE_JSON_NAME: &str = "Schedules Json.json";
const JSON_FILE_DIR: &str = "";
const SCHEDULE_JSON_SOURCE: JsonSource = JsonSource::Web;
// const SCHEDULE_JSON_SOURCE: JsonSource = JsonSource::Storage("Drizzle Tricolor 9_10");
const SPLATFEST_JSON_SOURCE: JsonSource = JsonSource::Web;
// const SPLATFEST_JSON_SOURCE: JsonSource = JsonSource::Storage("Splatfest Results");
#[derive(PartialEq)]
#[allow(unused)]
enum JsonSource<'a> {
    Web,
    Storage(&'a str),
}

#[allow(non_snake_case)]
impl Measure {
    pub fn new(api: RmApi) -> Measure {
        let RESOURCE_DIR = api.read_path("DONTNAMESOMETHINGTHIS", "@Resources");
        let SKIN_PATH = api.read_path("DONTNAMESOMETHINGTHIS", "Splatoon3RotationDisplay.ini");
        Measure {
            rm_api: api,
            measure_type: SplatinkType::Core("RegSche".to_string()),
            prev_sche: "RegSche".to_string(),
            schedules: None,
            RESOURCE_DIR,
            SKIN_PATH,
            web_pull_cooldown: Duration::seconds(0),
            web_pull_cooldown_set: 2,
            web_client: ClientBuilder::new().user_agent("Splatoon-3-Rotation-Display").build().unwrap(),
        }
    }
    pub fn dispose(&self) {}
    #[allow(unused)]
    pub fn reload(&mut self, rm_api: RmApi, max_value: &mut f64) {
        self.rm_api = rm_api;
        let type_string = self.rm_api.read_string("Type", "Core", None);
        let selected_sche = self.rm_api.read_string("Sche", "RegSche", None);
        let start_time = self.rm_api.read_int("StartTime", 0) as i64;
        let end_time = self.rm_api.read_int("EndTime", 0) as i64;

        self.measure_type = if type_string == "TimeBar" {
            SplatinkType::TimeBar(TimeBarOptions{
                start_time,
                end_time
            })
        } else {
            SplatinkType::Core(selected_sche.clone())
        };  

        if let SplatinkType::Core(_) = self.measure_type {
            self.check();
        }
    }
    pub fn execute_bang(&mut self, args: String) {
        let iter: Vec<&str> = args.split_whitespace().collect();

        if let SplatinkType::Core(_) = self.measure_type {
            #[allow(clippy::single_match)]
            match iter[0].to_lowercase().as_str() {
                "refreshfile" => {
                    let _ = self.rewrite_file().map_err(|e| self.rm_api.log(crate::rainmeter::api::LogType::Error, e));
                },
                "repulldata" => {
                    let _ = std::fs::remove_file(format!("{}/{SCHEDULE_JSON_NAME}", self.RESOURCE_DIR));
                    self.rm_api.execute_self("!CommandMeasure SplatinkCore refreshfile");
                },
                "redrawsche" => {
                    self.measure_type = SplatinkType::Core(iter[1].to_string());
                    self.rm_api.execute_self("!UpdateMeasure SplatinkCore");
                },
                _ => {},
            }
        }
    }
    pub fn update(&mut self) -> f64 {
        match self.measure_type.clone() {
            SplatinkType::Core(sche) => {
                if SCHEDULE_JSON_SOURCE == JsonSource::Web {
                    self.check();
                }
                if sche != self.prev_sche {
                    self.rm_api.execute_self(&format!("!HideMeterGroup {}", self.prev_sche));
                    self.rm_api.execute_self(&format!("!ShowMeterGroup {}", sche));
                    self.rm_api.execute_self("!Redraw");
                    self.prev_sche = sche.to_string();
                }
            },
            SplatinkType::TimeBar(ref o) => {
                return (chrono::Local::now().timestamp() - o.start_time).max(0) as f64 / (o.end_time - o.start_time) as f64;
            },
        };
        0.5
    }
    pub fn get_string(&mut self) -> Option<String> {
        match self.measure_type {
            SplatinkType::Core(_) => Some(self.prev_sche.clone()),
            SplatinkType::TimeBar(_) => None,
        }
    }

    fn check(&mut self) {
        if let None = self.schedules {
            self.populate_schedules();
        }
        if let Some(ref schedules) = self.schedules {
            match schedules.data.regularSchedules.nodes.first().ok_or("Local Regular Schedule Has No Elements".to_string())
                .and_then(|event|{
                    if SCHEDULE_JSON_SOURCE != JsonSource::Web || chrono::Local::now() > event.endTime {
                        if self.web_pull_cooldown_set == 2 {self.rm_api.log(crate::rainmeter::api::LogType::Notice, "--------Schedules out of date--------");}
                        if !self.web_pull_cooldown.is_zero() {
                            Ok(Err(false))
                        } else {
                            self.pull_schedules()
                            .and_then(|json|
                                Self::parse_json::<RotationData>(&json)
                            )
                            .and_then(|source| {
                                match SCHEDULE_JSON_SOURCE {
                                    JsonSource::Web => {
                                        match source.data.regularSchedules.nodes.first().ok_or("Web Regular Schedule Has No Elements".to_string()) {
                                            Ok(event) => {
                                                if chrono::Local::now() <= event.endTime {
                                                    Ok(Ok(source))
                                                } else {Ok(Err(true))}
                                            },
                                            Err(e) => {
                                                Err(e)
                                            }
                                        }
                                    },
                                    JsonSource::Storage(_) => {
                                        if &source != schedules {
                                            Ok(Ok(source))
                                        } else {Ok(Err(false))}
                                    }
                                }
                            })
                        }
                    } else {Ok(Err(false))}
                }) {
                    Ok(Ok(replacement)) => {
                        self.schedules = Some(replacement);
                        self.rewrite_file().map_err(|e| self.rm_api.log(crate::rainmeter::api::LogType::Error, e)).ok();
                    }
                    varible => {
                        match varible {
                            Ok(Err(false)) => {}
                            varible2 => {
                                match varible2 {
                                    Err(e) => {self.rm_api.log(crate::rainmeter::api::LogType::Error, e);}
                                    Ok(Err(true)) => {self.rm_api.log(crate::rainmeter::api::LogType::Warning, "Web schedule hasn't been updated yet".to_string());}
                                    _ => {}
                                }
                                if self.web_pull_cooldown.is_zero() {
                                    self.web_pull_cooldown = Duration::seconds(2_i64.pow(self.web_pull_cooldown_set));
                                    self.web_pull_cooldown_set = (self.web_pull_cooldown_set + 1).min(10);
                                    self.rm_api.log(crate::rainmeter::api::LogType::Notice, format!("Web requesting on cooldown for {:02}:{:02}", self.web_pull_cooldown.num_minutes(), self.web_pull_cooldown.num_seconds() % 60));
                                }
                            }
                        }
                        if !self.web_pull_cooldown.is_zero() {
                            self.web_pull_cooldown = self.web_pull_cooldown - Duration::seconds(1);
                        }
                    }
                }
        }
    }

    fn parse_json<'a, T: Deserialize<'a>>(json: &'a str) -> Result<T, String> {
        serde_json::from_str(json).map_err(|e| format!("Failed To Parse: {e:?}"))
    }

    fn populate_schedules(&mut self) {
        self.schedules = self.read_local_schedules()
            .or_else(|e| {
                self.rm_api.log(crate::rainmeter::api::LogType::Warning, e);
                if let JsonSource::Web = SCHEDULE_JSON_SOURCE {
                    self.pull_schedules()
                } else {
                    Err("Set To Read Local Schedule File".to_string())
                }
            })
            .and_then(|json| 
                Self::parse_json(&json)
            )
            .map_err(|e| self.rm_api.log(crate::rainmeter::api::LogType::Error, e))
            .ok();
    }

    fn read_local_schedules(&self) -> Result<String, String> {
        self.rm_api.log(crate::rainmeter::api::LogType::Notice, "Reading local schedule file...");
        std::fs::read_to_string(format!("{}/{SCHEDULE_JSON_NAME}", self.RESOURCE_DIR)).map_err(|e| format!("Failed To Read File: {e:?}"))
    }

    fn pull_schedules(&self) -> Result<String, String> {
        match SCHEDULE_JSON_SOURCE {
            JsonSource::Web => {
                self.rm_api.log(crate::rainmeter::api::LogType::Notice, "Pulling schedules from web...");
                self.web_client.get(SCHEDULE_URL).send().map_err(|e| format!("Failed To Fetch Json: {e:?}"))?
                    .text().map_err(|e| format!("Failed To Build Text: {e:?}"))
            },
            JsonSource::Storage(name) => {
                self.rm_api.log(crate::rainmeter::api::LogType::Notice, "Pulling schedules from storage...");
                std::fs::read_to_string(format!("{JSON_FILE_DIR}/{name}.json")).map_err(|e| format!("Failed To Read File: {e:?}"))
            },
        }
    }

    fn pull_splatfests(&self) -> Result<String, String> {
        match SPLATFEST_JSON_SOURCE {
            JsonSource::Web => {
                self.rm_api.log(crate::rainmeter::api::LogType::Notice, "Pulling splatfests from web...");
                self.web_client.get(SPLATFEST_URL).send().map_err(|e| format!("Failed To Fetch Json: {e:?}"))?
                    .text().map_err(|e| format!("Failed To Build Text: {e:?}"))
            },
            JsonSource::Storage(name) => {
                self.rm_api.log(crate::rainmeter::api::LogType::Notice, "Pulling splatfests from storage...");
                std::fs::read_to_string(format!("{JSON_FILE_DIR}/{name}.json")).map_err(|e| format!("Failed To Read File: {e:?}"))
            },
        }
    }

    fn pull_releases(&self) -> Result<String, String> {
        self.rm_api.log(crate::rainmeter::api::LogType::Notice, "Pulling releases from web...");
        self.web_client.get(GITHUB_RELEASES_URL).send().map_err(|e| format!("Failed To Fetch Json: {e:?}"))?
            .text().map_err(|e| format!("Failed To Build Text: {e:?}"))
    }

    fn rewrite_file(&self) -> Result<(), String> {
        self.schedules.as_ref().ok_or("Failed To Rewrite File: No Schedule".to_string())
            .and_then(|schedules|                       // Write internal to Local
                serde_json::to_string(schedules).map_err(|e| format!("Failed To Serialize: {e:?}"))
                    .and_then(|serialized_json|
                        std::fs::write(
                            format!("{}/{SCHEDULE_JSON_NAME}", self.RESOURCE_DIR),
                            serialized_json
                        ).map_err(|e| format!("Failed To Write To File: {e:?}"))
                        .map(|_| schedules)
                    )
            )
            .and_then(|schedules|                       // Build Structure
                self.pull_splatfests()
                    .and_then(|json|
                        Self::parse_json::<SplatfestData>(&json)
                    )
                    .map(|splatfests| (schedules, splatfests))
                )
            .and_then(|(schedules, splatfests)|
                self.pull_releases()
                    .and_then(|json|
                        Self::parse_json::<Releases>(&json)
                    )
                    .map(|releases| (schedules, splatfests, releases))
                )
            .map(|(schedules, splatfests, releases)| {
                self.rm_api.log(crate::rainmeter::api::LogType::Notice, "Building Structure...");
                RmStructure::generate(schedules, &splatfests, &releases)
            })
            .map(|structure|{                           // Download Images
                self.rm_api.log(crate::rainmeter::api::LogType::Notice, "Downloading missing images...");
                for ele in structure.download(&self.RESOURCE_DIR) {
                    let _ = ele.map_err(|e| self.rm_api.log(crate::rainmeter::api::LogType::Warning, e));
                }
                structure
            })
            .and_then(|structure| {                     // Write to file
                self.rm_api.log(crate::rainmeter::api::LogType::Notice, "Rewriting file...");
                write_to_skin(self.SKIN_PATH.as_str(), {
                    let mut ret = Vec::new();
                    ret.push(RmObject::new(ObjectType::Measure(
                        MeasureType::Plugin(PluginType::Splatink(
                            if structure.schedules.iter().any(|s| SplatinkType::Core(s.get_id().to_string()) == self.measure_type) {
                                self.measure_type.clone()
                            } else {
                                SplatinkType::Core(match structure.schedules.iter().next() {
                                    Some(s) => s.get_id().to_string(),
                                    None => "nonewhat".to_string()
                                })
                            }
                        )),
                        MeasureOptions::default()
                    )).prefix_name_owned("SplatinkCore"));
                    ret.append(&mut structure.to_rm());
                    ret
                }).map_err(|e| format!("Failed To Write To File: {e:?}"))
            })
            .map(|_| self.rm_api.execute_self("!Refresh"))
    }
}
