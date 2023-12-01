mod scalyr;
mod time;

use crate::scalyr::{ScalyrEvents, ScalyrSkel};
use crate::time::Timestamps;
use flate2::read::GzDecoder;
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::value::RawValue;
use std::io::Read;
use worker::*;

#[derive(Deserialize)]
struct DestinationTest {
    content: String,
}

enum InputType {
    DestinationTest,
    Lines(Vec<String>),
}

async fn decode<'a>(bytes: Vec<u8>) -> Result<InputType> {
    let mut gz = GzDecoder::new(&bytes[..]);
    let mut data = String::new();
    let _ = gz.read_to_string(&mut data)?;

    // Destination test request
    if matches!(serde_json::from_str::<DestinationTest>(&data), Ok(test) if test.content == "test")
    {
        return Ok(InputType::DestinationTest);
    }

    Ok(InputType::Lines(
        data.split('\n').map(str::to_string).collect(),
    ))
}

fn events(name: String, lines: Vec<String>) -> Result<ScalyrSkel> {
    let events = lines
        .into_iter()
        .filter(|l| !l.is_empty())
        .map_while(|l| {
            let Ok(timestamp) = serde_json::from_str::<Timestamps>(l.as_str()) else {
                console_log!("can't read timestamp: {:?}", l);
                return None;
            };

            Some((timestamp, l))
        })
        .map_while(|(t, l)| match RawValue::from_string(l) {
            Ok(raw) => Some(ScalyrEvents::new(t.get().to_string(), raw)),
            Err(ex) => {
                console_log!("can't deserialize log line: {:?}", ex);
                None
            }
        })
        .collect::<Vec<ScalyrEvents>>();

    Ok(ScalyrSkel::new(name, events))
}

async fn send(token: String, data: String) -> Result<StatusCode> {
    let client = reqwest::Client::new();
    let fetch = client
        .post("https://app.scalyr.com/api/addEvents")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token.to_string()))
        .body(data);

    let response = fetch
        .send()
        .await
        .map_err(|ex| Error::RustError(format!("error from upstream: {:?}", ex)))?;

    Ok(response.status())
}

#[event(fetch)]
async fn main(mut req: Request, env: Env, _: Context) -> Result<Response> {
    if req.method() != Method::Post {
        return Err("must be POST request".into());
    }

    let name = req.path().replace('/', "");
    if name.is_empty() {
        return Err("must pass name as path".into());
    }

    let Ok(token) = env.secret("TOKEN") else {
        return Err("must pass token".into());
    };

    let InputType::Lines(lines) = decode(req.bytes().await?).await? else {
        return Response::empty();
    };

    let len = lines.len();
    let skel = events(name, lines)?;
    let status = send(token.to_string(), serde_json::to_string(&skel)?).await?;

    console_log!("sent {}, upstream: {}", len, status);

    Response::empty()
}
