use chrono::{NaiveDate, NaiveDateTime, Utc};

pub trait TimeFactory {
    fn now(&self) -> NaiveDateTime;
}

pub struct RealTimeFactory {}

impl TimeFactory for RealTimeFactory {
    fn now(&self) -> NaiveDateTime {
        Utc::now().naive_local()
    }
}

pub struct FrozenTimeFactory {
    time: NaiveDateTime,
}

impl FrozenTimeFactory {
    pub fn new(y: i32, m: u32, d: u32, h: u32, i: u32) -> FrozenTimeFactory {
        FrozenTimeFactory {
            time: NaiveDate::from_ymd(y, m, d).and_hms(h, i, 0),
        }
    }
}

impl TimeFactory for FrozenTimeFactory {
    fn now(&self) -> NaiveDateTime {
        self.time
    }
}
