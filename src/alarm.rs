//! Current JSON alarm switches; no legacy PIN or transport.
use crate::{Command, Error};
use serde::{Deserialize, Serialize};

/// Switch types supported by the pinned current client.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
#[repr(u8)]
pub enum AlarmType {
    VehicleFault = 0,
    Geofence = 2,
    VehicleStart = 3,
}
impl From<AlarmType> for u8 {
    fn from(value: AlarmType) -> Self {
        value as u8
    }
}
impl TryFrom<u8> for AlarmType {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Error> {
        match value {
            0 => Ok(Self::VehicleFault),
            2 => Ok(Self::Geofence),
            3 => Ok(Self::VehicleStart),
            _ => Err(Error::Configuration("unsupported alarm type")),
        }
    }
}
impl std::str::FromStr for AlarmType {
    type Err = Error;
    fn from_str(value: &str) -> Result<Self, Error> {
        match value {
            "vehicle-fault" => Ok(Self::VehicleFault),
            "geofence" => Ok(Self::Geofence),
            "vehicle-start" => Ok(Self::VehicleStart),
            _ => Err(Error::Configuration(
                "expected vehicle-fault, geofence or vehicle-start",
            )),
        }
    }
}

/// Both flags are required; no existing setting is fetched or defaulted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AlarmSwitch {
    #[serde(rename = "alarmType")]
    pub alarm_type: AlarmType,
    #[serde(rename = "alarmSwitch", with = "flag")]
    pub alarm_enabled: bool,
    #[serde(rename = "functionSwitch", with = "flag")]
    pub function_enabled: bool,
}
mod flag {
    use serde::{Deserialize, Deserializer, Serializer};
    pub fn serialize<S: Serializer>(value: &bool, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(u8::from(*value))
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
        match u8::deserialize(deserializer)? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(serde::de::Error::custom("expected switch code 0 or 1")),
        }
    }
}

pub(crate) fn validate(switches: &[AlarmSwitch]) -> Result<(), Error> {
    if switches.is_empty() {
        return Err(Error::Configuration(
            "at least one alarm switch is required",
        ));
    }
    for (index, switch) in switches.iter().enumerate() {
        if switches[..index]
            .iter()
            .any(|s| s.alarm_type == switch.alarm_type)
        {
            return Err(Error::Configuration("duplicate alarm type"));
        }
    }
    Ok(())
}
impl Command {
    /// Set exactly the supplied switches in one PUT, without reads or retries.
    pub fn set_alarm_switches(switches: &[AlarmSwitch]) -> Result<Self, Error> {
        validate(switches)?;
        Ok(Self::AlarmSwitches {
            switches: switches.to_vec(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixtures_cover_explicit_flags_validation_and_open_read_codes() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../tests/fixtures/alarm.json")).unwrap();
        for case in fixture["cases"].as_array().unwrap() {
            let switch: AlarmSwitch = serde_json::from_value(case["switch"].clone()).unwrap();
            let command = Command::set_alarm_switches(&[switch]).unwrap();
            let body: serde_json::Value =
                serde_json::from_str(&command.body("SYNTHETICVIN").unwrap()).unwrap();
            assert_eq!(body["alarmSwitchList"], serde_json::json!([case["switch"]]));
            assert_eq!(body.as_object().unwrap().len(), 2);
            assert!(Command::set_alarm_switches(&[switch, switch]).is_err());
            assert!(Command::AlarmSwitches {
                switches: vec![switch, switch]
            }
            .body("vin")
            .is_err());
            assert!(command.body(" ").is_err());
        }
        assert!(Command::set_alarm_switches(&[]).is_err());
        assert!(Command::AlarmSwitches { switches: vec![] }
            .body("vin")
            .is_err());
        for invalid in fixture["invalid"].as_array().unwrap() {
            assert!(serde_json::from_value::<AlarmSwitch>(invalid.clone()).is_err());
        }
        let response: crate::models::AlarmSwitchResp =
            serde_json::from_value(fixture["response"].clone()).unwrap();
        let switches = response.alarm_switch_list.as_ref().unwrap();
        assert_eq!(switches[3].alarm_type, Some(99));
        assert_eq!(switches[3].alarm_switch, Some(7));
        assert_eq!(switches[3].function_switch, None);
        assert_eq!(switches[3].extra["future"], "synthetic");
        assert_eq!(switches[4].alarm_type, None);
        assert!(!format!("{response:?}").contains("synthetic"));
        for value in [
            serde_json::json!({}),
            serde_json::json!({"alarmSwitchList":null}),
        ] {
            assert!(
                serde_json::from_value::<crate::models::AlarmSwitchResp>(value)
                    .unwrap()
                    .alarm_switch_list
                    .is_none()
            );
        }
        assert!(serde_json::from_value::<crate::models::AlarmSwitchResp>(
            serde_json::json!({"alarmSwitchList":"bad"})
        )
        .is_err());
    }
}
