//! Explicit lock and tailgate controls. These builders never read vehicle state.
use crate::{Command, ControlParameter, Error};

/// Supported unlock targets from the current Python client.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VehicleLockId {
    Tailgate,
    Doors,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LockAction {
    Lock,
    Unlock,
}

/// Powered movement, distinct from releasing the tailgate latch.
/// Vehicle support must be established by the caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TailgateAction {
    Open,
    Close,
}

impl Command {
    pub fn lock_vehicle() -> Self {
        Self::control_vehicle_locks(LockAction::Lock, None).expect("valid lock action")
    }

    pub fn unlock_vehicle() -> Self {
        Self::control_vehicle_locks(LockAction::Unlock, Some(VehicleLockId::Doors))
            .expect("valid unlock target")
    }

    /// Release the tailgate latch; does not request powered movement.
    pub fn open_tailgate() -> Self {
        Self::control_vehicle_locks(LockAction::Unlock, Some(VehicleLockId::Tailgate))
            .expect("valid unlock target")
    }

    /// Lock the vehicle without a target, or unlock one explicit supported target.
    /// Rejects targeted locking rather than silently ignoring the target.
    pub fn control_vehicle_locks(
        action: LockAction,
        target: Option<VehicleLockId>,
    ) -> Result<Self, Error> {
        let (request_type, parameters) = match (action, target) {
            (LockAction::Lock, None) => ("1", vec![]),
            (LockAction::Unlock, Some(target)) => (
                "2",
                vec![
                    parameter(4, "AA=="),
                    parameter(5, "AA=="),
                    parameter(6, "AA=="),
                    parameter(
                        7,
                        match target {
                            VehicleLockId::Tailgate => "Ag==",
                            VehicleLockId::Doors => "Aw==",
                        },
                    ),
                    parameter(255, "AAAAAA=="),
                ],
            ),
            _ => {
                return Err(Error::Configuration(
                    "lock requires no target; unlock requires doors or tailgate",
                ))
            }
        };
        Ok(Self::Vehicle {
            request_type: request_type.into(),
            parameters,
        })
    }

    /// Request powered opening or closing, without a parameter terminator.
    /// API completion does not establish the physical tailgate position.
    pub fn control_tailgate(action: TailgateAction) -> Self {
        Self::Vehicle {
            request_type: "9".into(),
            parameters: vec![parameter(
                31,
                match action {
                    TailgateAction::Open => "AQ==",
                    TailgateAction::Close => "Ag==",
                },
            )],
        }
    }
}

fn parameter(id: u8, value: &str) -> ControlParameter {
    ControlParameter {
        id,
        value: value.into(),
    }
}

#[cfg(test)]
mod specs {
    use super::*;

    #[test]
    fn payloads_match_reference_fixtures() {
        let fixtures: serde_json::Value =
            serde_json::from_str(include_str!("../tests/fixtures/locks.json")).unwrap();
        let commands = [
            Command::lock_vehicle(),
            Command::unlock_vehicle(),
            Command::open_tailgate(),
            Command::control_tailgate(TailgateAction::Open),
            Command::control_tailgate(TailgateAction::Close),
        ];
        assert_eq!(fixtures.as_array().unwrap().len(), commands.len());
        for (command, case) in commands.iter().zip(fixtures.as_array().unwrap()) {
            let mut body: serde_json::Value =
                serde_json::from_str(&command.body("SYNTHETICVIN").unwrap()).unwrap();
            body.as_object_mut().unwrap().remove("vin");
            assert_eq!(body, case["payload"]);
        }
    }

    #[test]
    fn invalid_lock_combinations_fail_locally() {
        assert!(Command::control_vehicle_locks(LockAction::Unlock, None).is_err());
        for target in [VehicleLockId::Doors, VehicleLockId::Tailgate] {
            assert!(Command::control_vehicle_locks(LockAction::Lock, Some(target)).is_err());
        }
    }
}
