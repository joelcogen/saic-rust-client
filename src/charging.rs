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
