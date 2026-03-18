# WAE Monitoring API Example

This example demonstrates how to use the WAE Monitoring API to interact with monitoring services.

## Features
- Example usage of WAE Monitoring API
- Integration with HTTP services
- Basic monitoring setup

## Requirements
- Rust 1.75+
- Tokio runtime

## Usage
1. Clone the repository
2. Run the example:
   ```bash
   cargo run
   ```

## API Endpoints
- `/api/health` - Health check endpoint
- `/api/metrics` - Metrics endpoint
- `/api/alerts` - Alerts endpoint

## Configuration
The example uses default configuration values. You can modify the code to use your specific monitoring setup.

## Dependencies
- wae-monitoring - WAE Monitoring service
- wae-https - WAE HTTPS service
- tokio - Async runtime
- serde_json - JSON serialization
- http - HTTP types
- hyper - HTTP client/server
