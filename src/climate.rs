//! Stateless climate payload builders, submitted with `SaicClient::execute_command`.
use crate::{Command, ControlParameter, Error};

/// Caller-selected gateway temperature encoding. No vehicle discovery is implicit.
#[derive(Clone, Copy, Debug)]
pub enum TemperatureProfile {
    /// EH32 series: 17–31 °C, indices 3–17.
    Eh32,
    /// MZS3E series: 16–31 °C, indices 2–17.
    Mzs3e,
    /// Gateway fallback: 16–28 °C, indices 2–14.
    Standard,
}

impl TemperatureProfile {
    pub fn encode(self, celsius: i16) -> Result<u8, Error> {
        let (min, max, offset) = match self {
            Self::Eh32 => (17, 31, 3),
            Self::Mzs3e => (16, 31, 2),
            Self::Standard => (16, 28, 2),
        };
        if !(min..=max).contains(&celsius) {
            return Err(Error::Configuration(
                "temperature outside vehicle profile bounds",
            ));
        }
        Ok((celsius - min + offset) as u8)
    }
}

impl Command {
    /// Turn only the rear-window heater on or off. No status refresh is implicit.
    pub fn set_rear_window_heater(enabled: bool) -> Self {
        Self::Vehicle {
            request_type: "32".into(),
            parameters: vec![
                byte_parameter(23, u8::from(enabled)),
                ControlParameter {
                    id: 255,
                    value: "AAAAAA==".into(),
                },
            ],
        }
    }

    /// Turn only the steering-wheel heater on or off, without a PARAMS_MAX marker.
    /// Follows the gateway payload; no status refresh is implicit.
    pub fn set_steering_wheel_heater(enabled: bool) -> Self {
        Self::Vehicle {
            request_type: "8".into(),
            parameters: vec![byte_parameter(24, u8::from(enabled))],
        }
    }

    /// Start AC at an explicit whole-degree Celsius target.
    pub fn start_ac(profile: TemperatureProfile, celsius: i16) -> Result<Self, Error> {
        Self::control_climate(2, None, i16::from(profile.encode(celsius)?))
    }

    /// Reissue AC start at the target, even if it is unchanged or AC was off.
    /// This is not a stored preference or a temperature-only patch.
    pub fn set_ac_temperature(profile: TemperatureProfile, celsius: i16) -> Result<Self, Error> {
        Self::start_ac(profile, celsius)
    }

    /// Stop cabin AC without a temperature parameter.
    pub fn stop_ac() -> Self {
        Self::control_climate(0, Some(false), 0).expect("constant climate values")
    }

    /// Start ventilation with AC explicitly off.
    pub fn start_ac_blowing() -> Self {
        Self::control_climate(1, Some(false), 0).expect("constant climate values")
    }

    /// Start front defrost with the reference fixed temperature index of 8.
    pub fn start_front_defrost() -> Self {
        Self::control_climate(5, Some(true), 8).expect("constant climate values")
    }

    /// The gateway stops front defrost by stopping AC entirely.
    pub fn stop_front_defrost() -> Self {
        Self::stop_ac()
    }

    /// Explicit protocol parameters, each in 0..=255 (not Celsius).
    /// Vehicle support is the caller's responsibility. `None` omits AC on/off.
    /// As in the pinned Python client, fan zero forces AC off and omits temperature.
    /// Validation occurs even for a temperature that fan zero would omit.
    pub fn control_climate(
        fan_speed: i16,
        ac_on: Option<bool>,
        temperature_idx: i16,
    ) -> Result<Self, Error> {
        let fan = u8::try_from(fan_speed)
            .map_err(|_| Error::Configuration("fan must be a protocol byte"))?;
        let temperature = u8::try_from(temperature_idx)
            .map_err(|_| Error::Configuration("temperature index must be a protocol byte"))?;
        let mut parameters = vec![byte_parameter(19, fan)];
        if fan > 0 {
            parameters.push(byte_parameter(20, temperature));
        }
        if let Some(on) = if fan == 0 { Some(false) } else { ac_on } {
            parameters.push(byte_parameter(22, u8::from(on)));
        }
        parameters.push(ControlParameter {
            id: 255,
            value: "AAAAAA==".into(),
        });
        Ok(Self::Vehicle {
            request_type: "6".into(),
            parameters,
        })
    }
}

fn byte_parameter(id: u8, value: u8) -> ControlParameter {
    const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    ControlParameter {
        id,
        value: format!(
            "{}{}==",
            BASE64[(value >> 2) as usize] as char,
            BASE64[((value & 3) << 4) as usize] as char
        ),
    }
}

#[cfg(test)]
mod specs {
    use super::*;
    use serde_json::Value;

    #[test]
    fn independent_heaters_match_exact_reference_fixtures() {
        let fixtures: Value =
            serde_json::from_str(include_str!("../tests/fixtures/heaters.json")).unwrap();
        assert_eq!(fixtures.as_array().unwrap().len(), 4);
        for case in fixtures.as_array().unwrap() {
            let enabled = case["enabled"].as_bool().unwrap();
            let command = match case["heater"].as_str().unwrap() {
                "rear-window" => Command::set_rear_window_heater(enabled),
                "steering-wheel" => Command::set_steering_wheel_heater(enabled),
                _ => panic!("unknown heater"),
            };
            let mut body: Value =
                serde_json::from_str(&command.body("SYNTHETICVIN").unwrap()).unwrap();
            body.as_object_mut().unwrap().remove("vin");
            assert_eq!(body, case["payload"]);
        }
    }

    #[test]
    fn payloads_match_independent_python_fixtures() {
        let fixtures: Value =
            serde_json::from_str(include_str!("../tests/fixtures/climate.json")).unwrap();
        for case in fixtures.as_array().unwrap() {
            let args = &case["args"];
            let command = Command::control_climate(
                args[0].as_i64().unwrap() as i16,
                args[1].as_bool(),
                args[2].as_i64().unwrap() as i16,
            );
            if case["invalid"] == true {
                assert!(command.is_err());
                continue;
            }
            let command = command.unwrap();
            let mut body: Value =
                serde_json::from_str(&command.body("SYNTHETICVIN").unwrap()).unwrap();
            body.as_object_mut().unwrap().remove("vin");
            assert_eq!(body, case["payload"]);
        }
    }

    #[test]
    fn profiles_reject_outside_bounds_and_encode_every_degree() {
        for (profile, min, max, offset) in [
            (TemperatureProfile::Eh32, 17, 31, 3),
            (TemperatureProfile::Mzs3e, 16, 31, 2),
            (TemperatureProfile::Standard, 16, 28, 2),
        ] {
            for c in min..=max {
                assert_eq!(profile.encode(c).unwrap(), (c - min + offset) as u8);
                assert_eq!(
                    Command::start_ac(profile, c).unwrap().body("vin").unwrap(),
                    Command::set_ac_temperature(profile, c)
                        .unwrap()
                        .body("vin")
                        .unwrap()
                );
            }
            for c in [i16::MIN, min - 1, max + 1, i16::MAX] {
                assert!(Command::start_ac(profile, c).is_err());
                assert!(Command::set_ac_temperature(profile, c).is_err());
            }
        }
        for invalid in [-1, 256, i16::MIN, i16::MAX] {
            assert!(Command::control_climate(invalid, None, 8).is_err());
            assert!(Command::control_climate(0, None, invalid).is_err());
            assert!(Command::control_climate(2, None, invalid).is_err());
        }
    }

    #[test]
    fn named_modes_match_protocol_parameters() {
        for (command, fan, ac, temp) in [
            (
                Command::start_ac(TemperatureProfile::Standard, 22).unwrap(),
                2,
                None,
                8,
            ),
            (Command::stop_ac(), 0, Some(false), 0),
            (Command::start_ac_blowing(), 1, Some(false), 0),
            (Command::start_front_defrost(), 5, Some(true), 8),
            (Command::stop_front_defrost(), 0, Some(false), 0),
        ] {
            assert_eq!(
                command.body("vin").unwrap(),
                Command::control_climate(fan, ac, temp)
                    .unwrap()
                    .body("vin")
                    .unwrap()
            );
        }
    }
}
