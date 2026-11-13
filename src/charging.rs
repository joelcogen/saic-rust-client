//! Independent charging and cable-lock controls without implicit status reads.
use crate::Command;

impl Command {
    /// Start discharge when `stop_v2x` is false; stop when true.
    /// `supported` is caller-supplied V2X capability; unknown must be false.
    /// Does not change charging or cable lock and never reads status.
    pub fn control_v2x(stop_v2x: bool, supported: bool) -> Result<Self, crate::Error> {
        if !supported {
            return Err(crate::Error::Unsupported(
                "V2X is not supported by this vehicle",
            ));
        }
        Ok(Self::Charging {
            charging: 0,
            port_lock: 0,
            v2x: if stop_v2x { 2 } else { 1 },
        })
    }

    /// Set discharge target, preserving charging target and current limit with
    /// ignore codes. `supported` declares V2X target capability; unknown is false.
    /// No companion values or status reads are required by this protocol.
    pub fn set_v2x_target_battery_soc(
        target: TargetBatterySoc,
        supported: bool,
    ) -> Result<Self, crate::Error> {
        if !supported {
            return Err(crate::Error::Unsupported(
                "V2X target SoC is not supported by this vehicle",
            ));
        }
        Ok(Self::ChargingSettings {
            target_soc: 0,
            current_limit: 0,
            v2x_soc: target as u8,
        })
    }

    /// Start charging when `stop_charging` is false; stop when true.
    /// Does not change the charging-port lock or refresh status.
    pub fn control_charging(stop_charging: bool) -> Self {
        Self::Charging {
            charging: if stop_charging { 2 } else { 1 },
            port_lock: 0,
            v2x: 0,
        }
    }

    /// Lock the charging port when `unlock` is false; unlock when true.
    /// Does not stop charging or refresh status.
    pub fn control_charging_port_lock(unlock: bool) -> Self {
        Self::Charging {
            charging: 0,
            port_lock: if unlock { 2 } else { 1 },
            v2x: 0,
        }
    }
}

/// Supported target percentages from the current Python client's enum.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum TargetBatterySoc {
    P40 = 1,
    P50 = 2,
    P60 = 3,
    P70 = 4,
    P80 = 5,
    P90 = 6,
    P100 = 7,
}

impl TargetBatterySoc {
    pub fn from_percentage(percentage: i16) -> Result<Self, crate::Error> {
        match percentage {
            40 => Ok(Self::P40),
            50 => Ok(Self::P50),
            60 => Ok(Self::P60),
            70 => Ok(Self::P70),
            80 => Ok(Self::P80),
            90 => Ok(Self::P90),
            100 => Ok(Self::P100),
            _ => Err(crate::Error::Configuration(
                "unsupported target SoC percentage",
            )),
        }
    }
}

/// Supported AC charging-current limits. Max is the vehicle's maximum, not an amperage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ChargeCurrentLimit {
    A6 = 1,
    A8 = 2,
    A16 = 3,
    Max = 4,
}

impl std::str::FromStr for ChargeCurrentLimit {
    type Err = crate::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_uppercase().as_str() {
            "6A" => Ok(Self::A6),
            "8A" => Ok(Self::A8),
            "16A" => Ok(Self::A16),
            "MAX" => Ok(Self::Max),
            _ => Err(crate::Error::Configuration(
                "unsupported charging-current limit",
            )),
        }
    }
}

impl Command {
    /// Set target SoC, preserving the current limit with the protocol ignore code.
    /// `supported` must come from caller-supplied model capability information;
    /// unknown support must not be treated as true. No state is fetched.
    pub fn set_target_battery_soc(
        target: TargetBatterySoc,
        supported: bool,
    ) -> Result<Self, crate::Error> {
        if !supported {
            return Err(crate::Error::Configuration(
                "target SoC is not supported by this vehicle",
            ));
        }
        Ok(Self::ChargingSettings {
            target_soc: target as u8,
            current_limit: 0,
            v2x_soc: 0,
        })
    }

    /// Set current limit, preserving target SoC with the protocol ignore code.
    /// `supported` is explicit caller-supplied capability information.
    /// No vehicle or charging status is fetched, including on failure.
    pub fn set_charge_current_limit(
        limit: ChargeCurrentLimit,
        supported: bool,
    ) -> Result<Self, crate::Error> {
        if !supported {
            return Err(crate::Error::Configuration(
                "current limit is not supported by this vehicle",
            ));
        }
        Ok(Self::ChargingSettings {
            target_soc: 0,
            current_limit: limit as u8,
            v2x_soc: 0,
        })
    }
}

#[cfg(test)]
mod specs {
    use super::*;

    #[test]
    fn invalid_values_and_unsupported_capabilities_are_rejected() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../tests/fixtures/charge_settings.json")).unwrap();
        for value in fixture["invalid_targets"].as_array().unwrap() {
            assert!(TargetBatterySoc::from_percentage(value.as_i64().unwrap() as i16).is_err());
        }
        for value in fixture["invalid_limits"].as_array().unwrap() {
            assert!(value
                .as_str()
                .unwrap()
                .parse::<ChargeCurrentLimit>()
                .is_err());
        }
        assert_eq!(
            "  max ".parse::<ChargeCurrentLimit>().unwrap(),
            ChargeCurrentLimit::Max
        );
        assert!(Command::set_target_battery_soc(TargetBatterySoc::P80, false).is_err());
        assert!(Command::set_charge_current_limit(ChargeCurrentLimit::A6, false).is_err());
        for stop in [false, true] {
            assert!(matches!(
                Command::control_v2x(stop, false),
                Err(crate::Error::Unsupported(_))
            ));
        }
        assert!(matches!(
            Command::set_v2x_target_battery_soc(TargetBatterySoc::P80, false),
            Err(crate::Error::Unsupported(_))
        ));
        // All-ignore is a settings read, so it must never pass through commands.
        for (target_soc, current_limit, v2x_soc) in [(0, 0, 0), (8, 0, 0), (0, 5, 0), (0, 0, 8)] {
            assert!(Command::ChargingSettings {
                target_soc,
                current_limit,
                v2x_soc
            }
            .body("SYNTHETICVIN")
            .is_err());
        }
    }
}

/// Validated wall-clock time with minute precision. Charging schedules use it
/// unchanged; battery-heating schedules convert it with an explicit timezone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScheduleTime {
    pub(crate) hour: u8,
    pub(crate) minute: u8,
}

impl ScheduleTime {
    pub fn new(hour: u8, minute: u8) -> Result<Self, crate::Error> {
        if hour > 23 || minute > 59 {
            return Err(crate::Error::Configuration(
                "schedule time must be 00:00 through 23:59",
            ));
        }
        Ok(Self { hour, minute })
    }
}

impl std::str::FromStr for ScheduleTime {
    type Err = crate::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let b = value.as_bytes();
        if b.len() != 5 || b[2] != b':' || ![b[0], b[1], b[3], b[4]].iter().all(u8::is_ascii_digit)
        {
            return Err(crate::Error::Configuration("schedule time must be HH:MM"));
        }
        Self::new(
            (b[0] - b'0') * 10 + b[1] - b'0',
            (b[3] - b'0') * 10 + b[4] - b'0',
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ScheduledChargingMode {
    UntilConfiguredTime = 1,
    Disabled = 2,
    UntilConfiguredSoc = 3,
}

impl std::str::FromStr for ScheduledChargingMode {
    type Err = crate::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "until-configured-time" => Ok(Self::UntilConfiguredTime),
            "disabled" => Ok(Self::Disabled),
            "until-configured-soc" => Ok(Self::UntilConfiguredSoc),
            _ => Err(crate::Error::Configuration(
                "invalid charging schedule mode",
            )),
        }
    }
}

impl Command {
    /// Configure the vehicle's schedule, including disabling it. Both times are
    /// explicit even when disabled or charging until the already configured SoC.
    /// Overnight and equal bounds pass through; no host scheduling or refresh occurs.
    /// `supports_target_soc` is caller-supplied; unknown support must be false.
    pub fn set_schedule_charging(
        start: ScheduleTime,
        end: ScheduleTime,
        mode: ScheduledChargingMode,
        supports_target_soc: bool,
    ) -> Result<Self, crate::Error> {
        if mode == ScheduledChargingMode::UntilConfiguredSoc && !supports_target_soc {
            return Err(crate::Error::Unsupported(
                "until-configured-SoC requires target SoC support",
            ));
        }
        Ok(Self::ChargingSchedule {
            start_hour: start.hour,
            start_minute: start.minute,
            end_hour: end.hour,
            end_minute: end.minute,
            public_station: 1,
            mode: mode as u8,
            supports_target_soc,
        })
    }
}

#[cfg(test)]
mod schedule_specs {
    use super::*;

    #[test]
    fn schedule_validation_includes_raw_command_payloads() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../tests/fixtures/charge_schedule.json")).unwrap();
        for value in fixture["invalid_times"].as_array().unwrap() {
            assert!(value.as_str().unwrap().parse::<ScheduleTime>().is_err());
        }
        for (h, m) in [(24, 0), (0, 60), (255, 255)] {
            assert!(ScheduleTime::new(h, m).is_err());
        }
        let start = "22:00".parse().unwrap();
        let end = "06:00".parse().unwrap();
        assert!(matches!(
            Command::set_schedule_charging(
                start,
                end,
                ScheduledChargingMode::UntilConfiguredSoc,
                false
            ),
            Err(crate::Error::Unsupported(_))
        ));
        for (field, value) in [
            ("rsvanStHour", 24),
            ("rsvanSpHour", 24),
            ("rsvanStMintue", 60),
            ("rsvanSpMintue", 60),
            ("tboxReserCtrlReq", 0),
            ("tboxReserCtrlReq", 4),
            ("tboxAdpPubChrgSttnReq", 0),
        ] {
            let mut payload = fixture["actions"][0]["payload"].clone();
            payload["kind"] = "ChargingSchedule".into();
            payload["supports_target_soc"] = true.into();
            payload[field] = value.into();
            let command: Command = serde_json::from_value(payload).unwrap();
            assert!(command.body("SYNTHETICVIN").is_err());
        }
        let mut payload = fixture["actions"][8]["payload"].clone();
        payload["kind"] = "ChargingSchedule".into();
        payload["supports_target_soc"] = false.into();
        let command: Command = serde_json::from_value(payload).unwrap();
        assert!(matches!(
            command.body("SYNTHETICVIN"),
            Err(crate::Error::Unsupported(_))
        ));
        for mode in [
            ScheduledChargingMode::Disabled,
            ScheduledChargingMode::UntilConfiguredTime,
        ] {
            assert!(Command::set_schedule_charging(start, end, mode, false)
                .unwrap()
                .body("SYNTHETICVIN")
                .is_ok());
        }
    }
}
