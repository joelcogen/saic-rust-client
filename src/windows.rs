//! Window controls from the current JSON API; vehicle support is caller-owned.
use crate::{Command, ControlParameter, Error};

/// Front windows are named by role; rear windows by physical side.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VehicleWindowId {
    Sunroof,
    Driver,
    Passenger,
    RearLeft,
    RearRight,
}

const WINDOWS: [VehicleWindowId; 5] = [
    VehicleWindowId::Sunroof,
    VehicleWindowId::Driver,
    VehicleWindowId::Passenger,
    VehicleWindowId::RearLeft,
    VehicleWindowId::RearRight,
];

/// Actions for all four door windows together, excluding the sunroof.
/// Open uses the MG S6 EV value; support on other models is not established.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DoorWindowsAction {
    Close,
    Ventilate,
    Open,
}

impl Command {
    pub fn close_driver_window() -> Self {
        window_command(&[VehicleWindowId::Driver], "AA==")
    }

    pub fn control_sunroof(should_open: bool) -> Self {
        window_command(&[VehicleWindowId::Sunroof], open_value(should_open))
    }

    /// Select one or more windows. Empty and duplicate selections are rejected.
    /// Open uses protocol action 3, distinct from `control_door_windows` action 2.
    /// Single-window control is not supported on every model (including MG S6 EV).
    /// No existing state is needed and no status refresh is performed.
    pub fn control_windows(should_open: bool, windows: &[VehicleWindowId]) -> Result<Self, Error> {
        if windows.is_empty()
            || windows
                .iter()
                .enumerate()
                .any(|(i, w)| windows[..i].contains(w))
        {
            return Err(Error::Configuration(
                "select at least one window without duplicates",
            ));
        }
        Ok(window_command(windows, open_value(should_open)))
    }

    /// Close, ventilate or open all four door windows, leaving the sunroof alone.
    /// API completion does not verify physical position. Never refreshes status.
    pub fn control_door_windows(action: DoorWindowsAction) -> Self {
        window_command(
            &WINDOWS[1..],
            match action {
                DoorWindowsAction::Close => "AA==",
                DoorWindowsAction::Ventilate => "AQ==",
                DoorWindowsAction::Open => "Ag==",
            },
        )
    }
}

fn open_value(should_open: bool) -> &'static str {
    if should_open {
        "Aw=="
    } else {
        "AA=="
    }
}

fn window_command(windows: &[VehicleWindowId], action: &str) -> Command {
    // Zero is an unselected window, not an instruction to close that window.
    let mut parameters: Vec<_> = WINDOWS
        .iter()
        .zip(8..=12)
        .map(|(window, id)| ControlParameter {
            id,
            value: if windows.contains(window) {
                "AQ=="
            } else {
                "AA=="
            }
            .into(),
        })
        .collect();
    parameters.push(ControlParameter {
        id: 13,
        value: action.into(),
    });
    Command::Vehicle {
        request_type: "3".into(),
        parameters,
    }
}

#[cfg(test)]
mod specs {
    use super::*;

    #[test]
    fn every_selection_and_action_matches_independent_fixtures() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../tests/fixtures/windows.json")).unwrap();
        for case in fixture["selections"].as_array().unwrap() {
            let windows: Vec<_> = case["indices"]
                .as_array()
                .unwrap()
                .iter()
                .map(|i| WINDOWS[i.as_u64().unwrap() as usize])
                .collect();
            let actual = Command::control_windows(case["open"].as_bool().unwrap(), &windows);
            if case["invalid"] == true {
                assert!(actual.is_err());
            } else {
                let body: serde_json::Value =
                    serde_json::from_str(&actual.unwrap().body("SYNTHETICVIN").unwrap()).unwrap();
                assert_eq!(body["rvcParams"], case["payload"]["rvcParams"]);
                assert_eq!(body["rvcReqType"], "3");
                assert_eq!(body["vin"], fixture["vin_hash"]);
            }
        }
        let commands = [
            Command::close_driver_window(),
            Command::control_sunroof(false),
            Command::control_sunroof(true),
            Command::control_door_windows(DoorWindowsAction::Close),
            Command::control_door_windows(DoorWindowsAction::Ventilate),
            Command::control_door_windows(DoorWindowsAction::Open),
        ];
        for (command, case) in commands.iter().zip(fixture["helpers"].as_array().unwrap()) {
            let mut body: serde_json::Value =
                serde_json::from_str(&command.body("SYNTHETICVIN").unwrap()).unwrap();
            assert_eq!(
                body.as_object_mut().unwrap().remove("vin").unwrap(),
                fixture["vin_hash"]
            );
            assert_eq!(body, case["payload"]);
        }
    }
}
