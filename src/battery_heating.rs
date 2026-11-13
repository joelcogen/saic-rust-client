//! Battery heating controls and Python-compatible local schedule conversion.
use crate::{Command, Error, ScheduleTime};
use chrono::{DateTime, Duration, LocalResult, Offset, TimeZone, Timelike, Utc};

impl Command {
    /// Switch heating independently; never read status or account timezone.
    pub fn control_battery_heating(enable: bool) -> Self {
        Self::BatteryHeating {
            request: if enable { 1 } else { 2 },
        }
    }

    /// Disable without fetching the existing schedule or vehicle status.
    pub fn disable_schedule_battery_heating() -> Self {
        Self::BatteryHeatingSchedule {
            start_time: 0,
            status: 0,
        }
    }

    /// Schedule the next occurrence of `start` in the supplied timezone.
    /// `now` is explicit for deterministic caller clock handling. Named IANA zones
    /// and fixed offsets are supported via chrono's TimeZone implementations.
    /// Matches Python datetime.replace/add: fold is retained today, reset tomorrow;
    /// nonexistent spring times use the offset before the gap.
    pub fn enable_schedule_battery_heating<T: TimeZone>(
        start: ScheduleTime,
        timezone: &T,
        now: DateTime<Utc>,
    ) -> Result<Self, Error> {
        let local = now.with_timezone(timezone);
        let wall_now = now
            .naive_utc()
            .checked_add_signed(Duration::seconds(
                local.offset().fix().local_minus_utc().into(),
            ))
            .ok_or(Error::Configuration("heating date out of range"))?;
        let mut wall = wall_now
            .with_hour(start.hour.into())
            .and_then(|v| v.with_minute(start.minute.into()))
            .and_then(|v| v.with_second(0))
            .and_then(|v| v.with_nanosecond(0))
            .ok_or(Error::Configuration("invalid heating start time"))?;
        let mut fold = matches!(timezone.from_local_datetime(&wall_now), LocalResult::Ambiguous(_, late) if late.with_timezone(&Utc) == now);
        if wall < wall_now {
            wall = wall
                .checked_add_signed(Duration::days(1))
                .ok_or(Error::Configuration("heating date out of range"))?;
            fold = false;
        }
        let utc = match timezone.from_local_datetime(&wall) {
            LocalResult::Single(date) => date.with_timezone(&Utc),
            LocalResult::Ambiguous(early, late) => {
                if fold { late } else { early }.with_timezone(&Utc)
            }
            LocalResult::None => {
                // Python zoneinfo uses the pre-transition offset for fold=0 gaps.
                let mut offset = None;
                for minutes in 1..=2880 {
                    let before = wall
                        .checked_sub_signed(Duration::minutes(minutes))
                        .ok_or(Error::Configuration("heating date out of range"))?;
                    if let Some(date) = timezone.from_local_datetime(&before).earliest() {
                        offset = Some(date.offset().fix());
                        break;
                    }
                }
                offset
                    .and_then(|o| o.from_local_datetime(&wall).single())
                    .ok_or(Error::Configuration("unresolvable heating timezone"))?
                    .with_timezone(&Utc)
            }
        };
        let start_time = utc.timestamp_millis();
        if start_time <= 0 {
            return Err(Error::Configuration("heating start must follow Unix epoch"));
        }
        Ok(Self::BatteryHeatingSchedule {
            start_time,
            status: 1,
        })
    }
}

#[cfg(test)]
mod specs {
    use super::*;
    #[test]
    fn schedule_conversion_matches_python_fixtures() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../tests/fixtures/battery_heating.json")).unwrap();
        for case in fixture["schedules"].as_array().unwrap() {
            let start = case["start"].as_str().unwrap().parse().unwrap();
            let now = case["now"]
                .as_str()
                .unwrap()
                .parse::<DateTime<Utc>>()
                .unwrap();
            let zone = case["zone"].as_str().unwrap();
            let command = if let Ok(zone) = zone.parse::<chrono_tz::Tz>() {
                Command::enable_schedule_battery_heating(start, &zone, now)
            } else {
                Command::enable_schedule_battery_heating(
                    start,
                    &zone.parse::<chrono::FixedOffset>().unwrap(),
                    now,
                )
            }
            .unwrap();
            let mut body: serde_json::Value =
                serde_json::from_str(&command.body("SYNTHETICVIN").unwrap()).unwrap();
            body.as_object_mut().unwrap().remove("vin");
            assert_eq!(body, case["payload"], "{case}");
        }
        assert!(Command::enable_schedule_battery_heating(
            "23:59".parse().unwrap(),
            &chrono::FixedOffset::east_opt(3600).unwrap(),
            DateTime::<Utc>::MAX_UTC,
        )
        .is_err());
        for time in fixture["invalid_times"].as_array().unwrap() {
            assert!(time.as_str().unwrap().parse::<ScheduleTime>().is_err());
        }
        for command in [
            Command::BatteryHeating { request: 0 },
            Command::BatteryHeating { request: 3 },
            Command::BatteryHeatingSchedule {
                start_time: 1,
                status: 0,
            },
            Command::BatteryHeatingSchedule {
                start_time: 0,
                status: 1,
            },
            Command::BatteryHeatingSchedule {
                start_time: -1,
                status: 1,
            },
            Command::BatteryHeatingSchedule {
                start_time: 1,
                status: 2,
            },
        ] {
            assert!(command.body("SYNTHETICVIN").is_err());
        }
        assert!(Command::enable_schedule_battery_heating(
            "00:00".parse().unwrap(),
            &Utc,
            DateTime::from_timestamp(-86400, 0).unwrap()
        )
        .is_err());
    }
}
