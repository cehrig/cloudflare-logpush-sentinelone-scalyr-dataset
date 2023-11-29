# Cloudflare Logpush to SentinelOne Scalyr Dataset

Example Cloudflare Workers script written in Rust to ingest Cloudflare Logpush data into SentinelOne Scalyr Dataset via `addEvents`.

All attributes of Cloudflare Logpush loglines will be mapped to the `attrs` object, so we can filter on them in the Dataset UI.

## Worker Setup
Create a `wrangler.toml` file in the root directory

```
name = "scalyr"
main = "build/worker/shim.mjs"
compatibility_date = "2023-10-02"
account_id = "<Your Cloudflare Account Tag>"
workers_dev = false
routes = [
    { pattern = "<Worker Route>", zone_name = "<Your Cloudflare zone name>", custom_domain = true }
]

[build]
command = "cargo install -q worker-build && worker-build --release"
```

This will create a new Worker script `scalyr` that executes on `https://<Worker Route>`

## Deploy & Configure Worker
Make sure you have 
- [Rust](https://www.rust-lang.org/tools/install) installed
- [Cloudflare Wrangler](https://developers.cloudflare.com/workers/wrangler/install-and-update/) installed

Deploy the worker with
```
$ wrangler deploy
```

The Worker needs your Scalyr API Token in a Secret called `TOKEN`
```
$ wrangler secret put TOKEN
```

## Logpush Setup
Create a Cloudflare Logpush job. This has been tested with the HTTP dataset.

Replace
- Zone tag
- E-Mail address and Global API key
- Worker Route with the URL configured in the `wrangler.toml` above
- Identifier, this will get mapped into the `session_name` field in Scalyr
```
curl -s -XPOST "https://api.cloudflare.com/client/v4/zones/<Your Cloudflare Zone Tag>/logpush/jobs" \
-H "X-Auth-Email: <Your E-Mail address>" \
-H "X-Auth-Key: <Your Global API key>" \
-d '{
 "name":"<Logpush Job Name>",
 "destination_conf":"https://<Worker Route>/<Identifier>",
 "dataset": "http_requests",
 "logpull_options":"fields=ClientIP,ClientRequestHost,ClientRequestMethod,ClientRequestURI,EdgeEndTimestamp,EdgeResponseBytes,EdgeResponseStatus,EdgeStartTimestamp,RayID&timestamps=rfc3339"
}'
```

Make sure to add any other field you are interested in and that you enable the Logpush Job (can be done in Cloudflare Dashboard too)

## Debug & Todo
The script will log the response status from Scalyr to console
```
POST https://scalyr.example.com/logpush_test - Ok @ 11/29/2023, 11:21:31 PM
  (log) upstream: 200 OK
```

According to the Scalyr Docs there's a 6MB limit per request body. 
This Worker doesn't chunk Logpush data yet, so might need to invest some additional love on this front.