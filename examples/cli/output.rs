//! Deliberate terminal presentation, separate from redacted library diagnostics.
//! Never serialize a response wholesale: `extra`, subaccounts and raw command
//! data/messages can contain authentication or account data.
use saic_client::{models::*, CommandOutcome, CommandState, MessageId, UnknownReason};

pub trait Output {
    fn output(&self) -> String;
}
impl<T: Output> Output for Option<T> {
    fn output(&self) -> String {
        self.as_ref()
            .map(Output::output)
            .unwrap_or_else(|| "missing".into())
    }
}
impl<T: Output> Output for Vec<T> {
    fn output(&self) -> String {
        if self.is_empty() {
            return "empty (0 entries)".into();
        }
        self.iter()
            .enumerate()
            .map(|(i, v)| format!("Entry {}:\n{}", i + 1, v.output()))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
impl Output for String {
    fn output(&self) -> String {
        if self.is_empty() {
            return "empty".into();
        }
        // Keep VINs copyable, but escape terminal controls in server text.
        self.chars()
            .flat_map(|c| {
                if c.is_control() {
                    c.escape_default().collect::<Vec<_>>()
                } else {
                    vec![c]
                }
            })
            .collect()
    }
}
impl Output for i64 {
    fn output(&self) -> String {
        self.to_string()
    }
}
impl Output for bool {
    fn output(&self) -> String {
        self.to_string()
    }
}
impl Output for ProtocolValue {
    fn output(&self) -> String {
        match self {
            Self::Number(n) => n.output(),
            Self::Text(s) => s.output(),
        }
    }
}
impl Output for MessageId {
    fn output(&self) -> String {
        // Preserve string/integer distinction for message-status input.
        match self {
            Self::Text(s) => serde_json::to_string(s).expect("string serializes"),
            Self::Number(n) => n.to_string(),
        }
    }
}
macro_rules! fields {
    ($ty:ty, $($field:ident => $label:literal),+ $(,)?) => {
        impl Output for $ty {
            fn output(&self) -> String {
                vec![$(format!("{}: {}", $label, self.$field.output())),+].join("\n")
            }
        }
    };
}
fields!(VehicleListResp, vin_list => "Vehicles");
fields!(VinInfo,
    vin => "VIN", name => "Name", brand_name => "Brand", model_name => "Model",
    model_year => "Model year", series => "Series", color_name => "Color",
    is_current_vehicle => "Current vehicle", is_activate => "Activated",
    vehicle_model_configuration => "Capabilities (raw declarations)");
fields!(VehicleModelConfiguration, item_code => "Capability code", item_name => "Capability name", item_value => "Capability value (raw; unknown values retained)");
fields!(VehicleStatusCacheResp, status_time => "Status timestamp (Unix seconds)", online_status => "Online status (raw code; not proof vehicle is awake)", basic_vehicle_status => "Vehicle status", gps_position => "GPS");
fields!(VehicleStatusResp, status_time => "Status timestamp (Unix seconds)", basic_vehicle_status => "Vehicle status", gps_position => "GPS");
fields!(BasicVehicleStatus,
    battery_voltage => "Battery voltage (raw protocol units)",
    mileage => "Mileage (raw protocol units)",
    fuel_level_prc => "Fuel level (raw protocol units)",
    fuel_range => "Fuel range (raw protocol units)",
    fuel_range_elec => "Electric range (raw protocol units)",
    interior_temperature => "Interior temperature (raw protocol units)",
    exterior_temperature => "Exterior temperature (raw protocol units)",
    lock_status => "Lock status (raw code; unknown values retained)",
    bonnet_status => "Bonnet (raw code)", boot_status => "Boot (raw code)",
    driver_door => "Driver door (raw code)", passenger_door => "Passenger door (raw code)",
    rear_left_door => "Rear left door (raw code)", rear_right_door => "Rear right door (raw code)",
    driver_window => "Driver window (raw code)", passenger_window => "Passenger window (raw code)",
    rear_left_window => "Rear left window (raw code)", rear_right_window => "Rear right window (raw code)",
    sunroof_status => "Sunroof (raw code)", engine_status => "Engine (raw code)",
    power_mode => "Power mode (raw code)", hand_brake => "Hand brake (raw code)",
    remote_climate_status => "Climate (raw code)", rmt_htd_rr_wnd_st => "Rear window heating (raw code)",
    front_left_seat_heat_level => "Front left seat heating (raw level)", front_right_seat_heat_level => "Front right seat heating (raw level)",
    second_row_left_seat_heat_level => "Rear left seat heating (raw level)", second_row_right_seat_heat_level => "Rear right seat heating (raw level)",
    steering_heat_level => "Steering heating (raw level)",
    front_left_tyre_pressure => "Front left tyre pressure (raw protocol units)", front_right_tyre_pressure => "Front right tyre pressure (raw protocol units)",
    rear_left_tyre_pressure => "Rear left tyre pressure (raw protocol units)", rear_right_tyre_pressure => "Rear right tyre pressure (raw protocol units)",
    time_of_last_canbus_activity => "Last CAN bus activity timestamp (raw)");
fields!(GpsPosition, gps_status => "GPS status (raw code)", time_stamp => "GPS timestamp (raw)", way_point => "Waypoint");
fields!(WayPoint, position => "Position", speed => "Speed (raw protocol units)", heading => "Heading (raw protocol units)", satellites => "Satellites");
fields!(Position, latitude => "Latitude (raw protocol units)", longitude => "Longitude (raw protocol units)", altitude => "Altitude (raw protocol units)");
fields!(ChargeStatusResp, status_time => "Status timestamp (Unix seconds)", charging_status => "Charging status", gps_position => "GPS");
fields!(ChargingStatus,
    charging_state => "Charging state (raw code; unknown values retained)",
    power_level_prc => "Battery level (raw protocol units)",
    charging_current => "Charging current (raw protocol units)", charging_voltage => "Charging voltage (raw protocol units)",
    working_current => "Working current (raw protocol units)", working_voltage => "Working voltage (raw protocol units)",
    realtime_power => "Power (raw protocol units)", charging_duration => "Charging duration (raw protocol units)",
    charging_gun_state => "Charging gun (raw code)", charging_type => "Charging type (raw code)",
    fuel_range_elec => "Electric range (raw protocol units)",
    start_time => "Start timestamp (raw)", end_time => "End timestamp (raw)");
fields!(ChrgMgmtDataResp, chrg_mgmt_data => "Charging management", rvs_charge_status => "Charging session");
fields!(ChrgMgmtData,
    bms_chrg_sts => "Charging state (raw code; unknown values retained)",
    bms_pack_soc_dsp => "Battery SoC (raw protocol units)", bms_pack_crnt => "Pack current (raw protocol units)", bms_pack_crnt_v => "Pack current validity (raw code)", bms_pack_vol => "Pack voltage (raw protocol units)",
    bms_on_bd_chrg_trgt_soc_dsp_cmd => "Charge target (raw code)", bms_altng_chrg_crnt_dsp_cmd => "Current limit (raw code)",
    bms_reser_ctrl_dsp_cmd => "Schedule mode (raw code)",
    bms_reser_st_hour_dsp_cmd => "Schedule start hour", bms_reser_st_mintue_dsp_cmd => "Schedule start minute", bms_reser_sp_hour_dsp_cmd => "Schedule end hour", bms_reser_sp_mintue_dsp_cmd => "Schedule end minute",
    bms_ptc_heat_req_dsp_cmd => "Battery heat request (raw code)", bms_ptc_heat_resp => "Battery heat response (raw code)", bms_ptc_heat_sp_rsn => "Battery heat stop reason (raw code)",
    chrgng_rmnng_time => "Charging remaining time (raw protocol units)", chrgng_rmnng_time_v => "Charging remaining time validity (raw code)",
    dis_chrgng_rmnng_time => "Discharging remaining time (raw protocol units)", dis_chrgng_rmnng_time_v => "Discharging remaining time validity (raw code)",
    ccu_elecc_lck_ctrl_dsp_cmd => "Cable lock (raw code)");
fields!(RvsChargeStatus, charging_duration => "Charging duration (raw protocol units)", charging_gun_state => "Charging gun (raw code)", charging_type => "Charging type (raw code)", start_time => "Start timestamp (raw)", end_time => "End timestamp (raw)");

fn code(value: Option<i64>, meanings: &[(i64, &str)]) -> String {
    match value {
        None => "missing".into(),
        Some(n) => match meanings.iter().find(|(v, _)| *v == n) {
            Some((_, label)) => format!("{label} (code {n})"),
            None => format!("unknown (code {n})"),
        },
    }
}
impl Output for ChargingSettingResp {
    fn output(&self) -> String {
        let targets = [
            (1, "40%"),
            (2, "50%"),
            (3, "60%"),
            (4, "70%"),
            (5, "80%"),
            (6, "90%"),
            (7, "100%"),
        ];
        format!("Charge target: {}\nCurrent limit: {}\nV2X target: {}\nCurrent response (raw code): {}\nCharge target response (raw code): {}\nV2X target response (raw code): {}\nEstimated electric range (raw protocol units): {}\nPack current (raw protocol units): {}\nRequest status (raw protocol value): {}",
            code(self.bms_on_bd_chrg_trgt_soc_dsp_cmd, &targets),
            code(self.bms_altng_chrg_crnt_dsp_cmd, &[(1,"6 A"),(2,"8 A"),(3,"16 A"),(4,"MAX")]),
            code(self.imcu_dschrg_trgt_soc_dsp_cmd, &targets),
            self.bms_altng_chrg_crnt_resp.output(), self.bms_chrg_trgt_soc_resp.output(), self.imcu_dschrg_trgt_soc_resp.output(), self.bms_estd_elec_rng.output(), self.bms_pack_crnt.output(), self.rvc_req_sts.output())
    }
}
impl Output for ScheduledBatteryHeatingResp {
    fn output(&self) -> String {
        format!(
            "Heating schedule: {}\nStart timestamp (UTC Unix milliseconds): {}",
            code(self.status, &[(0, "disabled"), (1, "enabled")]),
            self.start_time.output()
        )
    }
}
fields!(UserTimezoneResp, timezone => "Account timezone");
fields!(AlarmSwitchResp, alarm_switch_list => "Alarm switches");
impl Output for AlarmSwitchStatus {
    fn output(&self) -> String {
        format!(
            "Alarm type: {}\nAlarm switch: {}\nFunction switch: {}",
            code(
                self.alarm_type,
                &[(0, "vehicle-fault"), (2, "geofence"), (3, "vehicle-start")]
            ),
            code(self.alarm_switch, &[(0, "off"), (1, "on")]),
            code(self.function_switch, &[(0, "off"), (1, "on")])
        )
    }
}
fields!(MessageResp, alarm_number => "Alarm count", command_number => "Command count", news_number => "News count", records_number => "Record count", total_number => "Total count", messages => "Messages");
fields!(MessageEntity, message_id => "Message ID (JSON)", title => "Title", content => "Content", message_type => "Message type", message_time => "Message time (server text)", create_time => "Created timestamp (raw)", read_status => "Read status (raw code; unknown values retained)", vin => "VIN");
impl Output for CommandOutcome {
    fn output(&self) -> String {
        let state = match &self.state {
            CommandState::Completed => "Completed by API".into(),
            CommandState::Rejected => "Rejected by API".into(),
            CommandState::InvalidSession => "Invalid session; log in explicitly".into(),
            CommandState::Unknown(reason) => {
                let reason = match reason {
                    UnknownReason::Timeout => "timed out".into(),
                    UnknownReason::Cancelled => "cancelled".into(),
                    UnknownReason::Failure(error) => error.to_string(),
                };
                format!("Unknown outcome ({reason}); do not blindly replay")
            }
        };
        format!("{state}; physical position unverified (requested state is not confirmed).\nSAIC code: {}\nHTTP status: {}", self.saic_code.output(), self.http_status.map(i64::from).output())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::DeserializeOwned;
    use serde_json::json;

    fn render<T: DeserializeOwned + Output + std::fmt::Debug>(value: serde_json::Value) -> String {
        let value: T = serde_json::from_value(value).unwrap();
        let text = value.output();
        assert!(!text.contains("{ .. }"));
        assert!(!text.contains("SECRET"));
        let diagnostic = format!("{value:?}");
        assert!(diagnostic.ends_with("{ .. }"));
        assert!(!diagnostic.contains("SECRET"));
        text
    }

    #[test]
    fn discovery_lists_full_copyable_vins_names_and_models_without_account_data() {
        let text = render::<VehicleListResp>(json!({
        "token":"SECRET", "vinList":[
            {"vin":"SYNTHETICVIN00001","name":"First","modelName":"MG4", "password":"SECRET",
             "subAccountList":[{"userAccount":"SECRET","userName":"SECRET"}],
             "vehicleModelConfiguration":[{"itemCode":"HeatedSeat","itemValue":"99","token":"SECRET"}]},
            {"vin":"SYNTHETICVIN00002","name":"Second","modelName":"MG5"},
            {"vin":null}
        ]}));
        for expected in [
            "VIN: SYNTHETICVIN00001",
            "VIN: SYNTHETICVIN00002",
            "Name: First",
            "Model: MG5",
            "VIN: missing",
            "Name: missing",
            "99",
            "Entry 3:",
        ] {
            assert!(text.contains(expected), "{expected}: {text}");
        }
        assert_eq!(
            render::<VehicleListResp>(json!({"vinList":[]})),
            "Vehicles: empty (0 entries)"
        );
        assert_eq!(render::<VehicleListResp>(json!({})), "Vehicles: missing");
        let vehicle: VinInfo = serde_json::from_value(json!({"vin":"SYNTHETICVIN00001"})).unwrap();
        assert!(!format!("{vehicle:?}").contains("SYNTHETIC"));
    }

    #[test]
    fn status_preserves_missing_zero_unknown_codes_scaling_and_timestamps() {
        let value = json!({"statusTime":1700000000,"onlineStatus":99,
            "basicVehicleStatus":{"batteryVoltage":125,"interiorTemperature":0,"lockStatus":999,"accessToken":"SECRET"},
            "gpsPosition":{"timeStamp":1700000001,"wayPoint":{"position":{"latitude":123456,"longitude":234567,"authorization":"SECRET"}}},
            "extendedVehicleStatus":{"alertDataSum":["SECRET"]},"refreshToken":"SECRET"});
        for text in [
            render::<VehicleStatusCacheResp>(value.clone()),
            render::<VehicleStatusResp>(value),
        ] {
            for expected in [
                "Status timestamp (Unix seconds): 1700000000",
                "Battery voltage (raw protocol units): 125",
                "Interior temperature (raw protocol units): 0",
                "unknown values retained): 999",
                "Mileage (raw protocol units): missing",
                "Latitude (raw protocol units): 123456",
            ] {
                assert!(text.contains(expected), "{expected}: {text}");
            }
        }
        assert!(render::<VehicleStatusCacheResp>(json!({})).contains("Vehicle status: missing"));
        let text = render::<ChargeStatusResp>(
            json!({"statusTime":1700000002,"chargingStatus":{"chargingState":999,"chargingCurrent":123,"workingVoltage":0,"token":"SECRET"}}),
        );
        assert!(text.contains("Status timestamp (Unix seconds): 1700000002"));
        assert!(text.contains("Charging current (raw protocol units): 123"));
        assert!(text.contains("unknown values retained): 999"));
        assert!(text.contains("Working voltage (raw protocol units): 0"));
        assert!(render::<ChargeStatusResp>(json!({})).contains("Charging status: missing"));
    }

    #[test]
    fn management_settings_and_heating_show_returned_values_without_defaults() {
        let text = render::<ChrgMgmtDataResp>(
            json!({"chrgMgmtData":{"bmsPackCrnt":0,"bmsChrgSts":999,"bmsReserStHourDspCmd":23,"token":"SECRET"},"rvsChargeStatus":{"startTime":1234,"extendedData3":"SECRET"}}),
        );
        for expected in [
            "Pack current (raw protocol units): 0",
            "unknown values retained): 999",
            "Schedule start hour: 23",
            "Start timestamp (raw): 1234",
        ] {
            assert!(text.contains(expected));
        }
        assert!(render::<ChrgMgmtDataResp>(json!({})).contains("Charging management: missing"));
        let text = render::<ChargingSettingResp>(
            json!({"bmsOnBdChrgTrgtSOCDspCmd":5,"bmsAltngChrgCrntDspCmd":3,"imcuDschrgTrgtSOCDspCmd":999,"rvcReqSts":"AQ==","token":"SECRET"}),
        );
        for expected in [
            "Charge target: 80% (code 5)",
            "Current limit: 16 A (code 3)",
            "V2X target: unknown (code 999)",
            "Request status (raw protocol value): AQ==",
            "Pack current (raw protocol units): missing",
        ] {
            assert!(text.contains(expected), "{text}");
        }
        assert!(render::<ChargingSettingResp>(json!({})).contains("Charge target: missing"));
        let text = render::<ScheduledBatteryHeatingResp>(
            json!({"startTime":1700000000123_i64,"status":999,"token":"SECRET"}),
        );
        assert!(text.contains("Heating schedule: unknown (code 999)"));
        assert!(text.contains("Start timestamp (UTC Unix milliseconds): 1700000000123"));
        assert!(render::<ScheduledBatteryHeatingResp>(json!({"status":0}))
            .contains("disabled (code 0)"));
        assert!(
            render::<ScheduledBatteryHeatingResp>(json!({})).contains("Heating schedule: missing")
        );
    }

    #[test]
    fn newer_reads_and_messages_have_explicit_output_and_escape_terminal_controls() {
        assert_eq!(
            render::<UserTimezoneResp>(json!({"timezone":"Europe/Brussels","token":"SECRET"})),
            "Account timezone: Europe/Brussels"
        );
        assert_eq!(
            render::<UserTimezoneResp>(json!({})),
            "Account timezone: missing"
        );
        let text = render::<AlarmSwitchResp>(
            json!({"alarmSwitchList":[{"alarmType":2,"alarmSwitch":1,"functionSwitch":0},{"alarmType":99,"alarmSwitch":7}],"token":"SECRET"}),
        );
        for expected in [
            "geofence (code 2)",
            "on (code 1)",
            "off (code 0)",
            "unknown (code 99)",
            "unknown (code 7)",
            "Function switch: missing",
        ] {
            assert!(text.contains(expected));
        }
        assert!(
            render::<AlarmSwitchResp>(json!({"alarmSwitchList":[]})).contains("empty (0 entries)")
        );
        let text = render::<MessageResp>(
            json!({"alarmNumber":0,"totalNumber":2,"messages":[{"messageId":"001","title":"Test\nTitle","content":"Synthetic message\u{001b}[31m","createTime":1234,"messageTime":"2023-11-14","vin":"SYNTHETICVIN00001","contentIdList":["SECRET"],"token":"SECRET"},{"messageId":42}]}),
        );
        for expected in [
            "Message ID (JSON): \"001\"",
            "Message ID (JSON): 42",
            "Title: Test\\nTitle",
            "Synthetic message\\u{1b}[31m",
            "Created timestamp (raw): 1234",
            "Alarm count: 0",
        ] {
            assert!(text.contains(expected), "{expected}: {text}");
        }
        assert!(!text.contains('\x1b'));
        assert!(render::<MessageResp>(json!({"messages":[]})).contains("empty (0 entries)"));
        assert_eq!(None::<MessageResp>.output(), "missing");
    }

    #[test]
    fn all_command_outcomes_are_clear_and_never_dump_server_data() {
        for (state, expected) in [
            (CommandState::Completed, "Completed by API"),
            (CommandState::Rejected, "Rejected by API"),
            (CommandState::InvalidSession, "Invalid session"),
            (
                CommandState::Unknown(UnknownReason::Timeout),
                "Unknown outcome (timed out)",
            ),
            (
                CommandState::Unknown(UnknownReason::Cancelled),
                "Unknown outcome (cancelled)",
            ),
            (
                CommandState::Unknown(UnknownReason::Failure(saic_client::Error::Transport)),
                "Unknown outcome (HTTP transport failed)",
            ),
        ] {
            let outcome = CommandOutcome {
                state,
                saic_code: Some(3),
                http_status: Some(200),
                event_id: Some("SECRET".into()),
                message: Some("SECRET".into()),
                data: Some(json!({"token":"SECRET"})),
            };
            let text = outcome.output();
            assert!(text.contains(expected));
            assert!(text.contains("physical position unverified"));
            assert!(text.contains("SAIC code: 3\nHTTP status: 200"));
            assert!(!text.contains("SECRET"));
            assert!(!format!("{outcome:?}").contains("SECRET"));
        }
        assert!(CommandOutcome::default()
            .output()
            .contains("SAIC code: missing\nHTTP status: missing"));
    }
}
