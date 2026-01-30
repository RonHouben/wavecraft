//! Latency benchmarks for IPC communication

use bridge::{IpcHandler, ParameterHost};
use desktop::AppState;
use protocol::{IpcRequest, RequestId, METHOD_GET_PARAMETER};
use std::time::Instant;

#[test]
fn bench_ipc_latency() {
    let state = AppState::new();
    let handler = IpcHandler::new(state);

    // Warm up
    for _ in 0..10 {
        let request = IpcRequest::new(
            RequestId::Number(0),
            METHOD_GET_PARAMETER,
            Some(serde_json::json!({"id": "gain"})),
        );
        let _ = handler.handle_request(request);
    }

    // Measure latency for 100 requests
    let iterations = 100;
    let mut latencies = Vec::with_capacity(iterations);

    for i in 0..iterations {
        let request = IpcRequest::new(
            RequestId::Number(i as i64),
            METHOD_GET_PARAMETER,
            Some(serde_json::json!({"id": "gain"})),
        );

        let start = Instant::now();
        let _ = handler.handle_request(request);
        let duration = start.elapsed();

        latencies.push(duration.as_micros() as f64 / 1000.0); // Convert to milliseconds
    }

    // Calculate statistics
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let min = latencies[0];
    let max = latencies[latencies.len() - 1];
    let avg = latencies.iter().sum::<f64>() / latencies.len() as f64;
    let p50 = latencies[latencies.len() / 2];
    let p95 = latencies[(latencies.len() as f64 * 0.95) as usize];
    let p99 = latencies[(latencies.len() as f64 * 0.99) as usize];

    println!("\n=== IPC Latency Benchmark (Handler Only) ===");
    println!("Iterations: {}", iterations);
    println!("Min:  {:.3} ms", min);
    println!("Avg:  {:.3} ms", avg);
    println!("p50:  {:.3} ms", p50);
    println!("p95:  {:.3} ms", p95);
    println!("p99:  {:.3} ms", p99);
    println!("Max:  {:.3} ms", max);

    // Assert performance targets (handler only, not including WebView overhead)
    assert!(p50 < 0.1, "p50 latency too high: {:.3} ms", p50);
    assert!(p95 < 0.5, "p95 latency too high: {:.3} ms", p95);
}

#[test]
fn bench_json_parsing_latency() {
    let state = AppState::new();
    let handler = IpcHandler::new(state);

    let request_json = r#"{"jsonrpc":"2.0","id":1,"method":"getParameter","params":{"id":"gain"}}"#;

    // Warm up
    for _ in 0..10 {
        let _ = handler.handle_json(request_json);
    }

    // Measure latency for 100 requests
    let iterations = 100;
    let mut latencies = Vec::with_capacity(iterations);

    for _ in 0..iterations {
        let start = Instant::now();
        let _ = handler.handle_json(request_json);
        let duration = start.elapsed();

        latencies.push(duration.as_micros() as f64 / 1000.0); // Convert to milliseconds
    }

    // Calculate statistics
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let avg = latencies.iter().sum::<f64>() / latencies.len() as f64;
    let p50 = latencies[latencies.len() / 2];
    let p95 = latencies[(latencies.len() as f64 * 0.95) as usize];

    println!("\n=== JSON Parsing + Handler Benchmark ===");
    println!("Iterations: {}", iterations);
    println!("Avg:  {:.3} ms", avg);
    println!("p50:  {:.3} ms", p50);
    println!("p95:  {:.3} ms", p95);

    // Assert performance targets
    assert!(p50 < 0.2, "p50 latency too high: {:.3} ms", p50);
    assert!(p95 < 1.0, "p95 latency too high: {:.3} ms", p95);
}

#[test]
fn bench_set_parameter_latency() {
    let state = AppState::new();
    let handler = IpcHandler::new(state);

    // Warm up
    for _ in 0..10 {
        let request_json = r#"{"jsonrpc":"2.0","id":1,"method":"setParameter","params":{"id":"gain","value":0.5}}"#;
        let _ = handler.handle_json(request_json);
    }

    // Measure latency
    let iterations = 100;
    let mut latencies = Vec::with_capacity(iterations);

    for _ in 0..iterations {
        let request_json = r#"{"jsonrpc":"2.0","id":1,"method":"setParameter","params":{"id":"gain","value":0.5}}"#;

        let start = Instant::now();
        let _ = handler.handle_json(request_json);
        let duration = start.elapsed();

        latencies.push(duration.as_micros() as f64 / 1000.0);
    }

    // Calculate statistics
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let p50 = latencies[latencies.len() / 2];
    let p95 = latencies[(latencies.len() as f64 * 0.95) as usize];

    println!("\n=== Set Parameter Benchmark ===");
    println!("p50:  {:.3} ms", p50);
    println!("p95:  {:.3} ms", p95);

    assert!(p50 < 0.2, "p50 latency too high: {:.3} ms", p50);
    assert!(p95 < 1.0, "p95 latency too high: {:.3} ms", p95);
}
