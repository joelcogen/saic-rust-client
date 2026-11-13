use crate::{models::*, CommandOptions, Error, SaicClient};
use reqwest::Method;
use sha2::{Digest, Sha256};

pub(crate) struct Operation {
    pub path: String,
    pub body: zeroize::Zeroizing<String>,
    pub method: Method,
    pub events: bool,
    pub allow_empty_result: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Read {
    UserTimezone,
    Vehicles,
    Cached,
    Live,
    Charging,
    Management,
    Settings,
    BatteryHeating,
}
impl Read {
    pub fn operation(self, vin: &str) -> Result<Operation, Error> {
        if !matches!(self, Self::Vehicles | Self::UserTimezone) && vin.trim().is_empty() {
            return Err(Error::Configuration("VIN must not be empty"));
        }
        let hash = hex::encode(Sha256::digest(vin));
        let path = match self {
            Self::UserTimezone => "/user/timezone",
            Self::Vehicles => "/vehicle/list",
            Self::Cached => "/vehicle/status/cache",
            Self::Live => "/vehicle/status",
            Self::Charging => "/vehicle/charging/status",
            Self::Management => "/vehicle/charging/mgmtData",
            Self::Settings => "/vehicle/charging/setting",
            Self::BatteryHeating => "/charging/batteryHeating",
        };
        let settings = self == Self::Settings;
        let path = if matches!(self, Self::Vehicles | Self::UserTimezone) || settings {
            path.to_owned()
        } else {
            format!(
                "{path}?vin={hash}{}",
                if matches!(self, Self::Cached | Self::Live) {
                    "&vehStatusReqType=2"
                } else {
                    ""
                }
            )
        };
        Ok(Operation {
            allow_empty_result: false,
            path,
            method: if settings { Method::POST } else { Method::GET },
            body: zeroize::Zeroizing::new(if settings {
                serde_json::json!({"vin":hash,"altngChrgCrntReq":0,"onBdChrgTrgtSOCReq":0,"tboxV2XSpSOCReq":0}).to_string()
            } else {
                String::new()
            }),
            events: matches!(
                self,
                Self::Live | Self::Charging | Self::Management | Self::Settings
            ),
        })
    }
}

macro_rules! read_method {
    ($name:ident, $kind:ident, $response:ty, $doc:literal) => {
        #[doc = $doc]
        pub async fn $name(
            &mut self,
            vin: &str,
            options: CommandOptions,
        ) -> Result<$response, Error> {
            self.read(Read::$kind, vin, options).await
        }
    };
}
impl SaicClient {
    /// Explicit account timezone lookup. Commands never invoke this read.
    pub async fn get_user_timezone(&mut self) -> Result<UserTimezoneResp, Error> {
        self.read(Read::UserTimezone, "", CommandOptions::default())
            .await
    }
    /// Discover account vehicles and their raw model capability declarations.
    pub async fn vehicles(&mut self) -> Result<VehicleListResp, Error> {
        self.read(Read::Vehicles, "", CommandOptions::default())
            .await
    }
    read_method!(
        vehicle_status_cached,
        Cached,
        VehicleStatusCacheResp,
        "Read only SAIC's cache. Never falls back to live status, even on failure."
    );
    read_method!(
        vehicle_status_live,
        Live,
        VehicleStatusResp,
        "Explicit live refresh, which may wake the vehicle. No background polling."
    );
    read_method!(
        charging_status,
        Charging,
        ChargeStatusResp,
        "Explicit charging status read, with bounded event continuation."
    );
    read_method!(
        charging_management,
        Management,
        ChrgMgmtDataResp,
        "Explicit charging management data read."
    );
    read_method!(
        charging_settings,
        Settings,
        ChargingSettingResp,
        "Read settings using the protocol's POST with all three request values set to ignore (0)."
    );
    read_method!(
        battery_heating_schedule,
        BatteryHeating,
        ScheduledBatteryHeatingResp,
        "Read the stored battery-heating schedule; start_time retains UTC epoch milliseconds."
    );
}
