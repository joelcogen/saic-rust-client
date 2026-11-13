//! Typed wire data from the pinned Python reference; see THIRD_PARTY_NOTICES.md.
//! Numeric measurements retain protocol units/scaling. Numeric status codes are
//! deliberately open integers: unknown values are never coerced to off/zero.
//! Missing or null fields remain None; unknown fields are retained in extra.
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fmt;

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct VehicleModelConfiguration {
    #[serde(rename = "itemCode")]
    pub item_code: Option<String>,
    #[serde(rename = "itemName")]
    pub item_name: Option<String>,
    #[serde(rename = "itemValue")]
    pub item_value: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}
impl fmt::Debug for VehicleModelConfiguration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VehicleModelConfiguration")
            .finish_non_exhaustive()
    }
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct SubAccount {
    #[serde(rename = "authorizationCardType")]
    pub authorization_card_type: Option<i64>,
    #[serde(rename = "btKeyStatus")]
    pub bt_key_status: Option<i64>,
    #[serde(rename = "locationAuthorization")]
    pub location_authorization: Option<i64>,
    #[serde(rename = "modelName")]
    pub model_name: Option<String>,
    #[serde(rename = "operationType")]
    pub operation_type: Option<i64>,
    #[serde(rename = "status")]
    pub status: Option<i64>,
    #[serde(rename = "subaccountId")]
    pub subaccount_id: Option<i64>,
    #[serde(rename = "subscriberId")]
    pub subscriber_id: Option<i64>,
    #[serde(rename = "userAccount")]
    pub user_account: Option<String>,
    #[serde(rename = "userName")]
    pub user_name: Option<String>,
    #[serde(rename = "validityEndTime")]
    pub validity_end_time: Option<i64>,
    #[serde(rename = "validityStartTime")]
    pub validity_start_time: Option<i64>,
    #[serde(rename = "vin")]
    pub vin: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}
impl fmt::Debug for SubAccount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SubAccount").finish_non_exhaustive()
    }
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct VinInfo {
    #[serde(rename = "bindTime")]
    pub bind_time: Option<i64>,
    #[serde(rename = "brandName")]
    pub brand_name: Option<String>,
    #[serde(rename = "colorName")]
    pub color_name: Option<String>,
    #[serde(rename = "isActivate")]
    pub is_activate: Option<bool>,
    #[serde(rename = "isCurrentVehicle")]
    pub is_current_vehicle: Option<bool>,
    #[serde(rename = "isSubaccount")]
    pub is_subaccount: Option<bool>,
    #[serde(rename = "modelName")]
    pub model_name: Option<String>,
    #[serde(rename = "modelYear")]
    pub model_year: Option<String>,
    #[serde(rename = "name")]
    pub name: Option<String>,
    #[serde(rename = "series")]
    pub series: Option<String>,
    #[serde(rename = "vin")]
    pub vin: Option<String>,
    #[serde(rename = "subAccountList")]
    pub sub_account_list: Option<Vec<SubAccount>>,
    #[serde(rename = "vehicleModelConfiguration")]
    pub vehicle_model_configuration: Option<Vec<VehicleModelConfiguration>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}
impl fmt::Debug for VinInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VinInfo").finish_non_exhaustive()
    }
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct VehicleListResp {
    #[serde(rename = "vinList")]
    pub vin_list: Option<Vec<VinInfo>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}
impl fmt::Debug for VehicleListResp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VehicleListResp").finish_non_exhaustive()
    }
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct BasicVehicleStatus {
    #[serde(rename = "batteryVoltage")]
    pub battery_voltage: Option<i64>,
    #[serde(rename = "bonnetStatus")]
    pub bonnet_status: Option<i64>,
    #[serde(rename = "bootStatus")]
    pub boot_status: Option<i64>,
    #[serde(rename = "canBusActive")]
    pub can_bus_active: Option<i64>,
    #[serde(rename = "clstrDspdFuelLvlSgmt")]
    pub clstr_dspd_fuel_lvl_sgmt: Option<i64>,
    #[serde(rename = "currentJourneyId")]
    pub current_journey_id: Option<i64>,
    #[serde(rename = "currentJourneyDistance")]
    pub current_journey_distance: Option<i64>,
    #[serde(rename = "dippedBeamStatus")]
    pub dipped_beam_status: Option<i64>,
    #[serde(rename = "driverDoor")]
    pub driver_door: Option<i64>,
    #[serde(rename = "driverWindow")]
    pub driver_window: Option<i64>,
    #[serde(rename = "engineStatus")]
    pub engine_status: Option<i64>,
    #[serde(rename = "extendedData1")]
    pub extended_data1: Option<i64>,
    #[serde(rename = "extendedData2")]
    pub extended_data2: Option<i64>,
    #[serde(rename = "exteriorTemperature")]
    pub exterior_temperature: Option<i64>,
    #[serde(rename = "frontLeftSeatHeatLevel")]
    pub front_left_seat_heat_level: Option<i64>,
    #[serde(rename = "frontLeftTyrePressure")]
    pub front_left_tyre_pressure: Option<i64>,
    #[serde(rename = "frontRightSeatHeatLevel")]
    pub front_right_seat_heat_level: Option<i64>,
    #[serde(rename = "frontRightTyrePressure")]
    pub front_right_tyre_pressure: Option<i64>,
    #[serde(rename = "fuelLevelPrc")]
    pub fuel_level_prc: Option<i64>,
    #[serde(rename = "fuelRange")]
    pub fuel_range: Option<i64>,
    #[serde(rename = "fuelRangeElec")]
    pub fuel_range_elec: Option<i64>,
    #[serde(rename = "handBrake")]
    pub hand_brake: Option<i64>,
    #[serde(rename = "interiorTemperature")]
    pub interior_temperature: Option<i64>,
    #[serde(rename = "lastKeySeen")]
    pub last_key_seen: Option<i64>,
    #[serde(rename = "lockStatus")]
    pub lock_status: Option<i64>,
    #[serde(rename = "mainBeamStatus")]
    pub main_beam_status: Option<i64>,
    #[serde(rename = "mileage")]
    pub mileage: Option<i64>,
    #[serde(rename = "passengerDoor")]
    pub passenger_door: Option<i64>,
    #[serde(rename = "passengerWindow")]
    pub passenger_window: Option<i64>,
    #[serde(rename = "powerMode")]
    pub power_mode: Option<i64>,
    #[serde(rename = "rearLeftDoor")]
    pub rear_left_door: Option<i64>,
    #[serde(rename = "rearLeftTyrePressure")]
    pub rear_left_tyre_pressure: Option<i64>,
    #[serde(rename = "rearLeftWindow")]
    pub rear_left_window: Option<i64>,
    #[serde(rename = "rearRightDoor")]
    pub rear_right_door: Option<i64>,
    #[serde(rename = "rearRightTyrePressure")]
    pub rear_right_tyre_pressure: Option<i64>,
    #[serde(rename = "rearRightWindow")]
    pub rear_right_window: Option<i64>,
    #[serde(rename = "remoteClimateStatus")]
    pub remote_climate_status: Option<i64>,
    #[serde(rename = "rmtHtdRrWndSt")]
    pub rmt_htd_rr_wnd_st: Option<i64>,
    #[serde(rename = "secondRowLeftSeatHeatLevel")]
    pub second_row_left_seat_heat_level: Option<i64>,
    #[serde(rename = "secondRowRightSeatHeatLevel")]
    pub second_row_right_seat_heat_level: Option<i64>,
    #[serde(rename = "sideLightStatus")]
    pub side_light_status: Option<i64>,
    #[serde(rename = "steeringHeatLevel")]
    pub steering_heat_level: Option<i64>,
    #[serde(rename = "steeringWheelHeatFailureReason")]
    pub steering_wheel_heat_failure_reason: Option<i64>,
    #[serde(rename = "sunroofStatus")]
    pub sunroof_status: Option<i64>,
    #[serde(rename = "timeOfLastCANBUSActivity")]
    pub time_of_last_canbus_activity: Option<i64>,
    #[serde(rename = "vehElecRngDsp")]
    pub veh_elec_rng_dsp: Option<i64>,
    #[serde(rename = "vehicleAlarmStatus")]
    pub vehicle_alarm_status: Option<i64>,
    #[serde(rename = "wheelTyreMonitorStatus")]
    pub wheel_tyre_monitor_status: Option<i64>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}
impl fmt::Debug for BasicVehicleStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BasicVehicleStatus").finish_non_exhaustive()
    }
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ExtendedVehicleStatus {
    #[serde(rename = "alertDataSum")]
    pub alert_data_sum: Option<Vec<Value>>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}
impl fmt::Debug for ExtendedVehicleStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExtendedVehicleStatus")
            .finish_non_exhaustive()
    }
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct VehicleStatusResp {
    #[serde(rename = "basicVehicleStatus")]
    pub basic_vehicle_status: Option<BasicVehicleStatus>,
    #[serde(rename = "extendedVehicleStatus")]
    pub extended_vehicle_status: Option<ExtendedVehicleStatus>,
    #[serde(rename = "gpsPosition")]
    pub gps_position: Option<GpsPosition>,
    #[serde(rename = "statusTime")]
    pub status_time: Option<i64>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}
impl fmt::Debug for VehicleStatusResp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VehicleStatusResp").finish_non_exhaustive()
    }
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct VehicleStatusCacheResp {
    #[serde(rename = "basicVehicleStatus")]
    pub basic_vehicle_status: Option<BasicVehicleStatus>,
    #[serde(rename = "gpsPosition")]
    pub gps_position: Option<GpsPosition>,
    #[serde(rename = "statusTime")]
    pub status_time: Option<i64>,
    #[serde(rename = "onlineStatus")]
    pub online_status: Option<i64>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}
impl fmt::Debug for VehicleStatusCacheResp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VehicleStatusCacheResp")
            .finish_non_exhaustive()
    }
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ChargingStatus {
    #[serde(rename = "chargingCurrent")]
    pub charging_current: Option<i64>,
    #[serde(rename = "chargingDuration")]
    pub charging_duration: Option<i64>,
    #[serde(rename = "chargingElectricityPhase")]
    pub charging_electricity_phase: Option<i64>,
    #[serde(rename = "chargingGunState")]
    pub charging_gun_state: Option<i64>,
    #[serde(rename = "chargingPileID")]
    pub charging_pile_id: Option<String>,
    #[serde(rename = "chargingPileSupplier")]
    pub charging_pile_supplier: Option<String>,
    #[serde(rename = "chargingState")]
    pub charging_state: Option<i64>,
    #[serde(rename = "chargingTimeLevelPrc")]
    pub charging_time_level_prc: Option<i64>,
    #[serde(rename = "chargingType")]
    pub charging_type: Option<i64>,
    #[serde(rename = "chargingVoltage")]
    pub charging_voltage: Option<i64>,
    #[serde(rename = "endTime")]
    pub end_time: Option<i64>,
    #[serde(rename = "fotaLowestVoltage")]
    pub fota_lowest_voltage: Option<i64>,
    #[serde(rename = "fuelRangeElec")]
    pub fuel_range_elec: Option<i64>,
    #[serde(rename = "lastChargeEndingPower")]
    pub last_charge_ending_power: Option<i64>,
    #[serde(rename = "mileage")]
    pub mileage: Option<i64>,
    #[serde(rename = "mileageOfDay")]
    pub mileage_of_day: Option<i64>,
    #[serde(rename = "mileageSinceLastCharge")]
    pub mileage_since_last_charge: Option<i64>,
    #[serde(rename = "powerLevelPrc")]
    pub power_level_prc: Option<i64>,
    #[serde(rename = "powerUsageOfDay")]
    pub power_usage_of_day: Option<i64>,
    #[serde(rename = "powerUsageSinceLastCharge")]
    pub power_usage_since_last_charge: Option<i64>,
    #[serde(rename = "realtimePower")]
    pub realtime_power: Option<i64>,
    #[serde(rename = "startTime")]
    pub start_time: Option<i64>,
    #[serde(rename = "staticEnergyConsumption")]
    pub static_energy_consumption: Option<i64>,
    #[serde(rename = "totalBatteryCapacity")]
    pub total_battery_capacity: Option<i64>,
    #[serde(rename = "workingCurrent")]
    pub working_current: Option<i64>,
    #[serde(rename = "workingVoltage")]
    pub working_voltage: Option<i64>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}
impl fmt::Debug for ChargingStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChargingStatus").finish_non_exhaustive()
    }
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ChargeStatusResp {
    #[serde(rename = "chargingStatus")]
    pub charging_status: Option<ChargingStatus>,
    #[serde(rename = "gpsPosition")]
    pub gps_position: Option<GpsPosition>,
    #[serde(rename = "statusTime")]
    pub status_time: Option<i64>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}
impl fmt::Debug for ChargeStatusResp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChargeStatusResp").finish_non_exhaustive()
    }
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ChrgMgmtData {
    #[serde(rename = "bmsAdpPubChrgSttnDspCmd")]
    pub bms_adp_pub_chrg_sttn_dsp_cmd: Option<i64>,
    #[serde(rename = "bmsAltngChrgCrntDspCmd")]
    pub bms_altng_chrg_crnt_dsp_cmd: Option<i64>,
    #[serde(rename = "bmsChrgCtrlDspCmd")]
    pub bms_chrg_ctrl_dsp_cmd: Option<i64>,
    #[serde(rename = "bmsChrgOtptCrntReq")]
    pub bms_chrg_otpt_crnt_req: Option<i64>,
    #[serde(rename = "bmsChrgOtptCrntReqV")]
    pub bms_chrg_otpt_crnt_req_v: Option<i64>,
    #[serde(rename = "bmsChrgSpRsn")]
    pub bms_chrg_sp_rsn: Option<i64>,
    #[serde(rename = "bmsChrgSts")]
    pub bms_chrg_sts: Option<i64>,
    #[serde(rename = "bmsDsChrgSpRsn")]
    pub bms_ds_chrg_sp_rsn: Option<i64>,
    #[serde(rename = "bmsEstdElecRng")]
    pub bms_estd_elec_rng: Option<i64>,
    #[serde(rename = "bmsOnBdChrgTrgtSOCDspCmd")]
    pub bms_on_bd_chrg_trgt_soc_dsp_cmd: Option<i64>,
    #[serde(rename = "bmsPackCrnt")]
    pub bms_pack_crnt: Option<i64>,
    #[serde(rename = "bmsPackCrntV")]
    pub bms_pack_crnt_v: Option<i64>,
    #[serde(rename = "bmsPackSOCDsp")]
    pub bms_pack_soc_dsp: Option<i64>,
    #[serde(rename = "bmsPackVol")]
    pub bms_pack_vol: Option<i64>,
    #[serde(rename = "bmsPTCHeatReqDspCmd")]
    pub bms_ptc_heat_req_dsp_cmd: Option<i64>,
    #[serde(rename = "bmsPTCHeatResp")]
    pub bms_ptc_heat_resp: Option<i64>,
    #[serde(rename = "bmsPTCHeatSpRsn")]
    pub bms_ptc_heat_sp_rsn: Option<i64>,
    #[serde(rename = "bmsReserCtrlDspCmd")]
    pub bms_reser_ctrl_dsp_cmd: Option<i64>,
    #[serde(rename = "bmsReserSpHourDspCmd")]
    pub bms_reser_sp_hour_dsp_cmd: Option<i64>,
    #[serde(rename = "bmsReserSpMintueDspCmd")]
    pub bms_reser_sp_mintue_dsp_cmd: Option<i64>,
    #[serde(rename = "bmsReserStHourDspCmd")]
    pub bms_reser_st_hour_dsp_cmd: Option<i64>,
    #[serde(rename = "bmsReserStMintueDspCmd")]
    pub bms_reser_st_mintue_dsp_cmd: Option<i64>,
    #[serde(rename = "ccuEleccLckCtrlDspCmd")]
    pub ccu_elecc_lck_ctrl_dsp_cmd: Option<i64>,
    #[serde(rename = "ccuOffBdChrgrPlugOn")]
    pub ccu_off_bd_chrgr_plug_on: Option<i64>,
    #[serde(rename = "ccuOnbdChrgrPlugOn")]
    pub ccu_onbd_chrgr_plug_on: Option<i64>,
    #[serde(rename = "chrgngAddedElecRng")]
    pub chrgng_added_elec_rng: Option<i64>,
    #[serde(rename = "chrgngAddedElecRngV")]
    pub chrgng_added_elec_rng_v: Option<i64>,
    #[serde(rename = "chrgngDoorOpenCnd")]
    pub chrgng_door_open_cnd: Option<i64>,
    #[serde(rename = "chrgngDoorPosSts")]
    pub chrgng_door_pos_sts: Option<i64>,
    #[serde(rename = "chrgngRmnngTime")]
    pub chrgng_rmnng_time: Option<i64>,
    #[serde(rename = "chrgngRmnngTimeV")]
    pub chrgng_rmnng_time_v: Option<i64>,
    #[serde(rename = "chrgngSpdngTime")]
    pub chrgng_spdng_time: Option<i64>,
    #[serde(rename = "chrgngSpdngTimeV")]
    pub chrgng_spdng_time_v: Option<i64>,
    #[serde(rename = "clstrElecRngToEPT")]
    pub clstr_elec_rng_to_ept: Option<i64>,
    #[serde(rename = "disChrgngRmnngTime")]
    pub dis_chrgng_rmnng_time: Option<i64>,
    #[serde(rename = "disChrgngRmnngTimeV")]
    pub dis_chrgng_rmnng_time_v: Option<i64>,
    #[serde(rename = "imcuChrgngEstdElecRng")]
    pub imcu_chrgng_estd_elec_rng: Option<i64>,
    #[serde(rename = "imcuChrgngEstdElecRngV")]
    pub imcu_chrgng_estd_elec_rng_v: Option<i64>,
    #[serde(rename = "imcuDschrgngEstdElecRng")]
    pub imcu_dschrgng_estd_elec_rng: Option<i64>,
    #[serde(rename = "imcuDschrgngEstdElecRngV")]
    pub imcu_dschrgng_estd_elec_rng_v: Option<i64>,
    #[serde(rename = "imcuVehElecRng")]
    pub imcu_veh_elec_rng: Option<i64>,
    #[serde(rename = "imcuVehElecRngV")]
    pub imcu_veh_elec_rng_v: Option<i64>,
    #[serde(rename = "onBdChrgrAltrCrntInptCrnt")]
    pub on_bd_chrgr_altr_crnt_inpt_crnt: Option<i64>,
    #[serde(rename = "onBdChrgrAltrCrntInptVol")]
    pub on_bd_chrgr_altr_crnt_inpt_vol: Option<i64>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}
impl fmt::Debug for ChrgMgmtData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChrgMgmtData").finish_non_exhaustive()
    }
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct RvsChargeStatus {
    #[serde(rename = "chargingDuration")]
    pub charging_duration: Option<i64>,
    #[serde(rename = "chargingElectricityPhase")]
    pub charging_electricity_phase: Option<i64>,
    #[serde(rename = "chargingGunState")]
    pub charging_gun_state: Option<i64>,
    #[serde(rename = "chargingPileID")]
    pub charging_pile_id: Option<String>,
    #[serde(rename = "chargingPileSupplier")]
    pub charging_pile_supplier: Option<String>,
    #[serde(rename = "chargingType")]
    pub charging_type: Option<i64>,
    #[serde(rename = "endTime")]
    pub end_time: Option<i64>,
    #[serde(rename = "extendedData1")]
    pub extended_data1: Option<i64>,
    #[serde(rename = "extendedData2")]
    pub extended_data2: Option<i64>,
    #[serde(rename = "extendedData3")]
    pub extended_data3: Option<String>,
    #[serde(rename = "extendedData4")]
    pub extended_data4: Option<String>,
    #[serde(rename = "fotaLowestVoltage")]
    pub fota_lowest_voltage: Option<i64>,
    #[serde(rename = "fuelRangeElec")]
    pub fuel_range_elec: Option<i64>,
    #[serde(rename = "lastChargeEndingPower")]
    pub last_charge_ending_power: Option<i64>,
    #[serde(rename = "mileage")]
    pub mileage: Option<i64>,
    #[serde(rename = "mileageOfDay")]
    pub mileage_of_day: Option<i64>,
    #[serde(rename = "mileageSinceLastCharge")]
    pub mileage_since_last_charge: Option<i64>,
    #[serde(rename = "powerUsageOfDay")]
    pub power_usage_of_day: Option<i64>,
    #[serde(rename = "powerUsageSinceLastCharge")]
    pub power_usage_since_last_charge: Option<i64>,
    #[serde(rename = "realtimePower")]
    pub realtime_power: Option<i64>,
    #[serde(rename = "startTime")]
    pub start_time: Option<i64>,
    #[serde(rename = "staticEnergyConsumption")]
    pub static_energy_consumption: Option<i64>,
    #[serde(rename = "totalBatteryCapacity")]
    pub total_battery_capacity: Option<i64>,
    #[serde(rename = "workingCurrent")]
    pub working_current: Option<i64>,
    #[serde(rename = "workingVoltage")]
    pub working_voltage: Option<i64>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}
impl fmt::Debug for RvsChargeStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RvsChargeStatus").finish_non_exhaustive()
    }
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ChrgMgmtDataResp {
    #[serde(rename = "chrgMgmtData")]
    pub chrg_mgmt_data: Option<ChrgMgmtData>,
    #[serde(rename = "rvsChargeStatus")]
    pub rvs_charge_status: Option<RvsChargeStatus>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}
impl fmt::Debug for ChrgMgmtDataResp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChrgMgmtDataResp").finish_non_exhaustive()
    }
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ChargingSettingResp {
    #[serde(rename = "bmsAltngChrgCrntDspCmd")]
    pub bms_altng_chrg_crnt_dsp_cmd: Option<i64>,
    #[serde(rename = "bmsAltngChrgCrntResp")]
    pub bms_altng_chrg_crnt_resp: Option<i64>,
    #[serde(rename = "bmsChrgTrgtSOCResp")]
    pub bms_chrg_trgt_soc_resp: Option<i64>,
    #[serde(rename = "bmsEstdElecRng")]
    pub bms_estd_elec_rng: Option<i64>,
    #[serde(rename = "bmsOnBdChrgTrgtSOCDspCmd")]
    pub bms_on_bd_chrg_trgt_soc_dsp_cmd: Option<i64>,
    #[serde(rename = "bmsPackCrnt")]
    pub bms_pack_crnt: Option<i64>,
    #[serde(rename = "imcuDschrgTrgtSOCDspCmd")]
    pub imcu_dschrg_trgt_soc_dsp_cmd: Option<i64>,
    #[serde(rename = "imcuDschrgTrgtSOCResp")]
    pub imcu_dschrg_trgt_soc_resp: Option<i64>,
    #[serde(rename = "rvcReqSts")]
    pub rvc_req_sts: Option<ProtocolValue>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}
impl fmt::Debug for ChargingSettingResp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChargingSettingResp")
            .finish_non_exhaustive()
    }
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ScheduledBatteryHeatingResp {
    #[serde(rename = "startTime")]
    pub start_time: Option<i64>,
    #[serde(rename = "status")]
    pub status: Option<i64>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}
impl fmt::Debug for ScheduledBatteryHeatingResp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScheduledBatteryHeatingResp")
            .finish_non_exhaustive()
    }
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct GpsPosition {
    #[serde(rename = "gpsStatus")]
    pub gps_status: Option<i64>,
    #[serde(rename = "timeStamp")]
    pub time_stamp: Option<i64>,
    #[serde(rename = "wayPoint")]
    pub way_point: Option<WayPoint>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}
impl fmt::Debug for GpsPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GpsPosition").finish_non_exhaustive()
    }
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct WayPoint {
    #[serde(rename = "hdop")]
    pub hdop: Option<i64>,
    #[serde(rename = "heading")]
    pub heading: Option<i64>,
    #[serde(rename = "position")]
    pub position: Option<Position>,
    #[serde(rename = "satellites")]
    pub satellites: Option<i64>,
    #[serde(rename = "speed")]
    pub speed: Option<i64>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}
impl fmt::Debug for WayPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WayPoint").finish_non_exhaustive()
    }
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Position {
    #[serde(rename = "altitude")]
    pub altitude: Option<i64>,
    #[serde(rename = "latitude")]
    pub latitude: Option<i64>,
    #[serde(rename = "longitude")]
    pub longitude: Option<i64>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}
impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Position").finish_non_exhaustive()
    }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ProtocolValue {
    Number(i64),
    Text(String),
}
impl VinInfo {
    /// Raw model capability declaration. Absence is unknown, never support.
    /// Unknown codes/values are preserved; callers decide which controls apply.
    pub fn capability(&self, code: &str) -> Option<&VehicleModelConfiguration> {
        self.vehicle_model_configuration
            .as_ref()?
            .iter()
            .find(|entry| {
                entry
                    .item_code
                    .as_ref()
                    .is_some_and(|c| c.trim().eq_ignore_ascii_case(code.trim()))
            })
    }
}

#[cfg(test)]
mod specs {
    use super::*;
    use serde_json::json;

    #[test]
    fn discovery_keeps_multiple_vehicles_and_unknown_capabilities() {
        let vehicles: VehicleListResp = serde_json::from_value(json!({"vinList":[
            {"vin":"synthetic-one", "vehicleModelConfiguration":[{"itemCode":"HeatedSeat", "itemValue":"99"},{"itemCode":"FutureControl","itemValue":null}]},
            {"vin":"synthetic-two"}
        ]})).unwrap();
        let list = vehicles.vin_list.unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(
            list[0]
                .capability("heatedseat")
                .unwrap()
                .item_value
                .as_deref(),
            Some("99")
        );
        assert!(list[0]
            .capability("FutureControl")
            .unwrap()
            .item_value
            .is_none());
        assert!(list[1].capability("HeatedSeat").is_none());
        assert!(!format!("{:?}", list[0]).contains("synthetic"));
        let empty: VehicleListResp = serde_json::from_value(json!({"vinList":[]})).unwrap();
        assert!(empty.vin_list.unwrap().is_empty());
        assert!(serde_json::from_value::<VehicleListResp>(json!({"vinList":"invalid"})).is_err());
    }

    #[test]
    fn all_responses_preserve_missing_values_timestamps_and_unknown_codes() {
        let value = json!({"statusTime":1700000000,"onlineStatus":99,"basicVehicleStatus":{"lockStatus":123,"batteryVoltage":null,"future":42},"gpsPosition":{"timeStamp":1700000001}});
        let cached: VehicleStatusCacheResp = serde_json::from_value(value.clone()).unwrap();
        let live: VehicleStatusResp = serde_json::from_value(value).unwrap();
        assert_eq!(cached.status_time, Some(1700000000));
        assert_eq!(cached.online_status, Some(99));
        let basic = cached.basic_vehicle_status.unwrap();
        assert_eq!(basic.lock_status, Some(123));
        assert_eq!(basic.battery_voltage, None);
        assert_eq!(basic.engine_status, None);
        assert_eq!(basic.extra["future"], 42);
        assert_eq!(live.gps_position.unwrap().time_stamp, Some(1700000001));
        assert!(live.extended_vehicle_status.is_none());
        let charge: ChargeStatusResp = serde_json::from_value(
            json!({"statusTime":1700000002,"chargingStatus":{"chargingState":999}}),
        )
        .unwrap();
        assert_eq!(charge.status_time, Some(1700000002));
        assert_eq!(charge.charging_status.unwrap().charging_state, Some(999));
        let mgmt: ChrgMgmtDataResp = serde_json::from_value(
            json!({"chrgMgmtData":{"bmsChrgSts":999,"bmsPackCrnt":0},"futureTime":1700000003}),
        )
        .unwrap();
        assert_eq!(
            mgmt.chrg_mgmt_data.as_ref().unwrap().bms_chrg_sts,
            Some(999)
        );
        assert_eq!(mgmt.chrg_mgmt_data.unwrap().bms_pack_crnt, Some(0));
        assert_eq!(mgmt.extra["futureTime"], 1700000003);
        let settings: ChargingSettingResp =
            serde_json::from_value(json!({"bmsOnBdChrgTrgtSOCDspCmd":999,"rvcReqSts":"AQ=="}))
                .unwrap();
        assert_eq!(settings.bms_on_bd_chrg_trgt_soc_dsp_cmd, Some(999));
        assert!(matches!(settings.rvc_req_sts, Some(ProtocolValue::Text(_))));
        let heating: ScheduledBatteryHeatingResp =
            serde_json::from_value(json!({"startTime":1700000000123_i64,"status":999})).unwrap();
        assert_eq!(heating.start_time, Some(1700000000123));
        assert_eq!(heating.status, Some(999));
        assert!(!format!("{heating:?}").contains("170000"));
    }
}
