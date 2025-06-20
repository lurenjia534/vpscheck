# vpscheck

A lightweight system metrics server built with [Actix Web](https://actix.rs/). It exposes a JSON endpoint for quickly checking the health of a VPS.

## Building

```bash
cargo build --release
```

## Running

Launch the server (listens on port `8080` by default):

```bash
cargo run --release
```

## Querying Metrics

Requesting `http://localhost:8080/metrics` returns current system statistics in JSON. Example output:

```json
{"uptime_days":0,"load":[1.49,0.44,0.15],"cpu":0.19,"mem_used":"471.64 MiB","mem_total":"9.93 GiB","disk_used_gib":13.52,"disk_total_gib":62.44,"rx_rate":0,"tx_rate":0,"rx_total_gib":0.0166,"tx_total_gib":0.00014,"swap_used_mib":0.0,"swap_total_mib":0.0,"tcp":7,"udp":2,"processes":15,"threads":24}
```


## Streaming Metrics over WebSocket

Connect to `ws://localhost:8080/ws` to receive the same metrics as a JSON string every second.
This endpoint is useful for dashboards that need live updates.

An integration test in `tests/ws_test.rs` ensures the WebSocket service works correctly.
