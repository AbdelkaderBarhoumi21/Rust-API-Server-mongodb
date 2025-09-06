use std::time::SystemTime;

use chrono::Utc;
use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize)]
pub struct Booking {
    pub _id: ObjectId,
    pub owner: ObjectId,
    pub start_time: DateTime,
    pub duration_in_minutes: u8,
    pub cancelled: bool,
}

#[derive(Debug, Serialize, Deserialize)]

pub struct BooKingRequest {
    pub owner: String,
    pub start_time: String,
    pub duration_in_minutes: u8,
}

impl TryFrom<BooKingRequest> for Booking {
    type Error = Box<dyn std::error::Error>;
    fn try_from(item: BooKingRequest) -> Result<Self, Self::Error> {
        //RFC 3339 C’est un format standard pour représenter une date et une heure. "2025-09-06T18:30:00+02:00"
        //DateTime<FixedOffset> => contient une date + heure + fuseau horaire fixe (+02:00).
        //parse_from_rfc3339 → "2025-09-06T18:30:00+02:00" → DateTime<FixedOffset>.
        //with_timezone(&Utc) => Convertit ton DateTime<FixedOffset> en DateTime<Utc>. 2025-09-06T18:30:00+02:00 =>2025-09-06T16:30:00Z (UTC).
        //into() Convertit le DateTime<Utc> en SystemTime
        let chrono_datetime: SystemTime = chrono::DateTime::parse_from_rfc3339(&item.start_time)
            .map_err(|err| format!("Failed to parse satrt _time : {}", err))?
            .with_timezone(&Utc)
            .into();

        Ok(Self {
            _id: ObjectId::new(),
            owner: ObjectId::parse_str(&item.owner).expect("Failed to parse owner"),
            start_time: DateTime::from(chrono_datetime),
            duration_in_minutes: item.duration_in_minutes,
            cancelled: false,
        })
    }
}
