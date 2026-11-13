//! Independent charging and cable-lock controls without implicit status reads.
use crate::Command;

impl Command {
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
        // All-ignore is a settings read, so it must never pass through commands.
        for (target_soc, current_limit, v2x_soc) in [(0, 0, 0), (8, 0, 0), (0, 5, 0), (1, 1, 1)] {
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
