use rainmeter::api::RmApi;
use crate::{schedule_data::RotationData, rm_write::{write_to_skin, ToRM, SplatinkType, TimeBarOptions, RmObject, ObjectType, MeasureType, PluginType, MeasureOptions}, splatfest_data::SplatfestData, rm_structure::{RmStructure, Download}};
extern crate serde;
use self::serde::Deserialize;

#[allow(non_snake_case)]
pub struct Measure {
    pub rm_api: RmApi,
    pub measure_type: SplatinkType,
    prev_sche: String,
    schedules: Option<RotationData>,
    pub RESOURCE_DIR: String,
    pub SKIN_PATH: String,
}

const SCHEDULE_JSON_NAME: &str = "Schedules Json.json";
const JSON_FILE_DIR: &str = "";
const SCHEDULE_JSON_SOURCE: JsonSource = JsonSource::Web;
// const SCHEDULE_JSON_SOURCE: JsonSource = JsonSource::Storage("Drizzle Tricolor 9_10");
const SPLATFEST_JSON_SOURCE: JsonSource = JsonSource::Web;
// const SPLATFEST_JSON_SOURCE: JsonSource = JsonSource::Storage("Splatfest Results");
#[derive(PartialEq)]
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
        // if let Some(ref schedules) = self.schedules {
        //     if SCHEDULE_JSON_SOURCE != JsonSource::Web || chrono::Local::now() > schedules.data.regularSchedules.nodes.first().unwrap().endTime {
        //         match self.pull_schedules() {
        //             Ok(s) => {
        //                 match Self::parse_json::<RotationData>(&s) {
        //                     Ok(source) => {
        //                         if let Some(ref mut schedules) = self.schedules {
        //                             if source != *schedules {
        //                                 self.rm_api.log(crate::rainmeter::api::LogType::Notice, "Schedules out of date");
        //                                 *schedules = source;
        //                                 let _ = self.rewrite_file().map_err(|e| self.rm_api.log(crate::rainmeter::api::LogType::Error, e));
        //                             }
        //                         }
        //                     },
        //                     Err(e) => {self.rm_api.log(crate::rainmeter::api::LogType::Error, e);}
        //                 }
        //             },
        //             Err(e) => {self.rm_api.log(crate::rainmeter::api::LogType::Error, e);}
        //         }
                
        //     }
        // }
        if let Some(ref schedules) = self.schedules {
            match schedules.data.regularSchedules.nodes.first().ok_or("Local Regular Schedule Has No Elements".to_string())
                .and_then(|event|{
                    if SCHEDULE_JSON_SOURCE != JsonSource::Web || chrono::Local::now() > event.endTime + chrono::Duration::seconds(20) {
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
                                                    Ok(Some(source))
                                                } else {Ok(None)}
                                            },
                                            Err(e) => {
                                                Err(e)
                                            }
                                        }
                                    },
                                    JsonSource::Storage(_) => {
                                        if &source != schedules {
                                            Ok(Some(source))
                                        } else {Ok(None)}
                                    }
                                }
                            })
                    } else {Ok(None)}
                }) {
                    Ok(Some(replacement)) => {
                        self.rm_api.log(crate::rainmeter::api::LogType::Notice, "Schedules out of date");
                        self.schedules = Some(replacement);
                        self.rewrite_file().map_err(|e| self.rm_api.log(crate::rainmeter::api::LogType::Error, e)).ok();
                    },
                    Err(e) => {self.rm_api.log(crate::rainmeter::api::LogType::Error, e);}
                    _ => {}
                }
        }
    }

    fn parse_json<'a, T: Deserialize<'a>>(json: &'a str) -> Result<T, String> {
        serde_json::from_str(json).map_err(|e| format!("Failed To Parse: {e:?}"))
    }

    fn populate_schedules(&mut self) {
        // match self.read_local_schedules().or_else(|e| {
        //         self.rm_api.log(crate::rainmeter::api::LogType::Error, e);
        //         if let JsonSource::Web = SCHEDULE_JSON_SOURCE {
        //             self.pull_schedules()
        //         } else {
        //             Err("Set To Read Local Schedule File".to_string())
        //         }
        //     })
        // {
        //     Ok(s) => {
        //         self.schedules = Self::parse_json(&s)
        //             .map_err(|e| self.rm_api.log(crate::rainmeter::api::LogType::Error, e))
        //             .ok();
        //     },
        //     Err(e) => {self.rm_api.log(crate::rainmeter::api::LogType::Error, e);}
        // }
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
                reqwest::blocking::get("https://splatoon3.ink/data/schedules.json").map_err(|e| format!("Failed To Fetch Json: {e:?}"))?
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
                reqwest::blocking::get("https://splatoon3.ink/data/festivals.json").map_err(|e| format!("Failed To Fetch Json: {e:?}"))?
                    .text().map_err(|e| format!("Failed To Build Text: {e:?}"))
            },
            JsonSource::Storage(name) => {
                self.rm_api.log(crate::rainmeter::api::LogType::Notice, "Pulling splatfests from storage...");
                std::fs::read_to_string(format!("{JSON_FILE_DIR}/{name}.json")).map_err(|e| format!("Failed To Read File: {e:?}"))
            },
        }
    }

    fn rewrite_file(&self) -> Result<(), String> {
        // if let Some(ref schedules) = self.schedules {
        //     std::fs::write(
        //         format!("{}/{SCHEDULE_JSON_NAME}", self.RESOURCE_DIR),
        //         serde_json::to_string(schedules).map_err(|e| format!("Failed To Serialize: {e:?}"))?).map_err(|e| format!("Failed To Write To File: {e:?}")
        //     )?;
        //     let ref splatfests = Self::parse_json::<SplatfestData>(&self.pull_splatfests()?)?;
        //     self.rm_api.log(crate::rainmeter::api::LogType::Notice, "Formatting data...");
        //     let structure = RmStructure::generate(schedules, splatfests);
        //     for ele in structure.download(&self.RESOURCE_DIR) {
        //         let _ = ele.map_err(|e| self.rm_api.log(crate::rainmeter::api::LogType::Error, e));
        //     }
        //     self.rm_api.log(crate::rainmeter::api::LogType::Notice, "Rewriting file...");
        //     write_to_skin(self.SKIN_PATH.as_str(), {
        //         let mut ret = Vec::new();
        //         ret.push(RmObject::new(ObjectType::Measure(
        //             MeasureType::Plugin(PluginType::Splatink(self.measure_type.clone())),
        //             MeasureOptions::default()
        //         )).prefix_name_owned("SplatinkCore"));
        //         ret.append(&mut structure.to_rm());
        //         ret
        //     }).map_err(|e| format!("Failed To Write To File: {e:?}"))?;
        //     self.rm_api.execute_self("!Refresh");
        //     Ok(())
        // } else {
        //     Err("Failed To Rewrite File: No Schedule".to_string())
        // }

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
                    .map(|ref splatfests| {
                        self.rm_api.log(crate::rainmeter::api::LogType::Notice, "Building Structure...");
                        RmStructure::generate(schedules, splatfests)
                    })
            )
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
