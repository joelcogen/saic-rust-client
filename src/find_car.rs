//! Find-car controls without implicit status reads.
use crate::{Command, ControlParameter};

/// Audible modes require explicit selection. The default only flashes lights.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FindCarMode {
    #[default]
    LightsOnly,
    /// Sound the horn without flashing lights.
    HornOnly,
    /// Sound the horn and flash lights.
    HornAndLights,
    /// Disable find-car, including both horn and lights.
    Stop,
}

impl Command {
    /// Build a find-car request. The pinned API exposes no duration setting.
    /// This does not refresh vehicle or charging status.
    pub fn find_my_car(mode: FindCarMode) -> Self {
        let (enabled, horn, lights) = match mode {
            FindCarMode::LightsOnly => (true, false, true),
            FindCarMode::HornOnly => (true, true, false),
            FindCarMode::HornAndLights => (true, true, true),
            FindCarMode::Stop => (false, false, false),
        };
        let mut parameters: Vec<_> = [(1, enabled), (2, horn), (3, lights)]
            .into_iter()
            .map(|(id, value)| ControlParameter {
                id,
                value: if value { "AQ==" } else { "AA==" }.into(),
            })
            .collect();
        parameters.push(ControlParameter {
            id: 255,
            value: "AAAAAA==".into(),
        });
        Self::Vehicle {
            request_type: "0".into(),
            parameters,
        }
    }
}

#[cfg(test)]
mod specs {
    use super::*;

    #[test]
    fn all_modes_match_independent_fixtures() {
        let fixtures: Vec<serde_json::Value> =
            serde_json::from_str(include_str!("../tests/fixtures/find_car.json")).unwrap();
        let modes = [
            FindCarMode::LightsOnly,
            FindCarMode::HornOnly,
            FindCarMode::HornAndLights,
            FindCarMode::Stop,
        ];
        assert_eq!(fixtures.len(), modes.len());
        assert_eq!(FindCarMode::default(), FindCarMode::LightsOnly);
        for (mode, case) in modes.into_iter().zip(fixtures) {
            let mut body: serde_json::Value =
                serde_json::from_str(&Command::find_my_car(mode).body("SYNTHETICVIN").unwrap())
                    .unwrap();
            body.as_object_mut().unwrap().remove("vin");
            assert_eq!(body, case["payload"]);
        }
    }
}
