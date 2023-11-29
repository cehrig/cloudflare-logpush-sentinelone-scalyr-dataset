use chrono::DateTime;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum TimestampTypes {
    String(String),
    Num(u64),
}

#[derive(Debug, Deserialize)]
pub(crate) struct Timestamps {
    #[serde(rename = "EdgeStartTimestamp")]
    edge_start_timestamp: Option<TimestampTypes>,
    #[serde(rename = "EventTimestampMs")]
    event_timestamp_ms: Option<u64>,
}

impl Timestamps {
    pub fn get(&self) -> u64 {
        match self {
            Timestamps {
                edge_start_timestamp: Some(stamp),
                ..
            } => match stamp {
                TimestampTypes::String(stamp) => {
                    DateTime::parse_from_rfc3339(stamp)
                        .map(|s| s.timestamp())
                        .unwrap_or_default() as u64
                        * 1_000_000_000
                }
                TimestampTypes::Num(num) => *num,
            },
            Timestamps {
                event_timestamp_ms: Some(stamp),
                ..
            } => *stamp * 1_000_000,
            _ => 0,
        }
    }
}
