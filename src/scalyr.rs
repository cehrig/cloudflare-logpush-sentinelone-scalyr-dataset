use serde::Serialize;
use serde_json::value::RawValue;

#[derive(Serialize, Debug)]
pub(crate) struct ScalyrSkel {
    session: String,
    #[serde(rename = "sessionInfo")]
    session_info: ScalyrSessionInfo,
    events: Vec<ScalyrEvents>,
}

impl ScalyrSkel {
    pub fn new(session_name: String, events: Vec<ScalyrEvents>) -> Self {
        ScalyrSkel {
            session: nano_id::base64::<21>(),
            session_info: ScalyrSessionInfo::new(session_name),
            events,
        }
    }
}

#[derive(Serialize, Debug)]
struct ScalyrSessionInfo {
    session_name: String,
}

impl ScalyrSessionInfo {
    fn new(session_name: String) -> Self {
        ScalyrSessionInfo { session_name }
    }
}

#[derive(Serialize, Debug)]
pub(crate) struct ScalyrEvents {
    ts: String,
    attrs: Box<RawValue>,
}

impl ScalyrEvents {
    pub fn new(ts: String, attrs: Box<RawValue>) -> Self {
        ScalyrEvents { ts, attrs }
    }
}
