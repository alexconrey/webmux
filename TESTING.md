# Testing Documentation

This document describes the test suite for the Terminal Access Server.

## Test Overview

The project includes comprehensive tests covering all major components:

- **42 total tests** across unit tests and integration tests
- **100% pass rate**
- Tests for configuration parsing, serial management, web API, and integration scenarios

## Running Tests

### Run All Tests
```bash
cargo test
```

### Run Specific Test Suites

**Unit Tests (Library)**
```bash
cargo test --lib
```

**Integration Tests**
```bash
cargo test --test integration_tests
```

**Specific Test**
```bash
cargo test test_config_parsing_valid
```

**With Output**
```bash
cargo test -- --nocapture
```

**With Verbose Output**
```bash
cargo test -- --nocapture --test-threads=1
```

## Test Coverage

### Configuration Module Tests (12 tests)
Located in: `src/config/tests.rs`

- ✅ Valid configuration parsing
- ✅ Multiple serial connections
- ✅ Duplicate connection name validation
- ✅ Invalid port number validation
- ✅ Disabled connections
- ✅ All parity variants (None, Odd, Even)
- ✅ All data bits variants (5, 6, 7, 8)
- ✅ All stop bits variants (1, 2)
- ✅ All flow control variants (None, Software, Hardware)

### Web Module Tests (17 tests)
Located in: `src/web/tests.rs` and `src/web/handler_tests.rs`

**API Tests:**
- ✅ Health check endpoint
- ✅ List connections (empty state)
- ✅ Get connection info (not found)
- ✅ Send data (connection not found)
- ✅ Get stats (connection not found)
- ✅ CORS headers
- ✅ API error serialization
- ✅ Error conversion from anyhow

**Handler Tests:**
- ✅ Data format defaults to text
- ✅ Text format parsing
- ✅ Hex format parsing
- ✅ Base64 format parsing
- ✅ Hex decoding with spaces
- ✅ Hex decoding without spaces
- ✅ Base64 decoding
- ✅ Invalid hex handling
- ✅ Invalid base64 handling
- ✅ Connection list serialization
- ✅ Connection info serialization

### Integration Tests (13 tests)
Located in: `tests/integration_tests.rs`

**API Workflow Tests:**
- ✅ Server health endpoint
- ✅ List connections workflow
- ✅ Connection info workflow
- ✅ Send data with text format
- ✅ Send data with hex format
- ✅ Send data with base64 format
- ✅ Get stats for nonexistent connection

**Configuration Tests:**
- ✅ Config example YAML structure matches spec
  - Validates IoT sensor configuration
  - Validates embedded MCU configuration
  - Validates industrial PLC configuration

**Serial Manager Tests:**
- ✅ Serial manager creation
- ✅ List empty connections
- ✅ Get nonexistent connection

**Utility Tests:**
- ✅ Data format conversions (text, hex, base64)
- ✅ All serial port configuration options

## Test Structure

```
terminal-access-server/
├── src/
│   ├── config/
│   │   ├── mod.rs
│   │   └── tests.rs          # Configuration tests
│   ├── web/
│   │   ├── mod.rs
│   │   ├── tests.rs           # Web API tests
│   │   └── handler_tests.rs  # Handler-specific tests
│   └── lib.rs                 # Library exports for testing
└── tests/
    └── integration_tests.rs   # Integration tests
```

## What Gets Tested

### 1. Configuration Parsing
- YAML parsing from config.example.yaml format
- All serial port parameters (baud rate, data bits, stop bits, parity, flow control)
- Server configuration (host, port)
- Logging configuration
- Validation rules (duplicate names, invalid ports)

### 2. Web API Endpoints
- Health check (`/health`)
- List connections (`/api/connections`)
- Get connection info (`/api/connections/:name`)
- Send data (`/api/connections/:name/send`)
- Get statistics (`/api/connections/:name/stats`)

### 3. Data Format Handling
- Text format: Plain strings → bytes
- Hex format: "48656c6c6f" → [0x48, 0x65, 0x6c, 0x6c, 0x6f]
- Base64 format: "SGVsbG8=" → "Hello"
- Error handling for invalid formats

### 4. Serial Port Configuration
Tests validate that all configuration options from config.example.yaml work:
- IoT sensor at 115200 baud
- Embedded MCU at 9600 baud
- Industrial PLC at 19200 baud with even parity and 2 stop bits

### 5. Error Handling
- Connection not found errors
- Invalid data format errors
- Configuration validation errors
- API error responses

## Mock vs Real Serial Ports

**Current Implementation:**
- Tests use the actual API without real serial port hardware
- Tests verify correct error handling when connections don't exist
- Config parsing tests use temporary YAML files

**Why No Serial Port Mocking:**
- Serial port hardware is not required to test the API layer
- Configuration and data format handling can be tested independently
- Real serial port testing requires physical hardware or virtual serial ports

## Running Tests Without Serial Hardware

All tests pass without any serial port hardware connected. This is by design:

1. **Config tests** use temporary files
2. **API tests** verify endpoint behavior and error handling
3. **Integration tests** validate the full request/response flow

The tests ensure the server will:
- Parse configurations correctly
- Handle API requests properly
- Convert data formats accurately
- Report appropriate errors

## Adding New Tests

### Unit Test Example
```rust
#[test]
fn test_new_feature() {
    // Arrange
    let config = create_test_config();

    // Act
    let result = new_feature(&config);

    // Assert
    assert!(result.is_ok());
}
```

### Async Test Example
```rust
#[tokio::test]
async fn test_async_feature() {
    let manager = SerialManager::new();
    let result = manager.some_async_operation().await;
    assert!(result.is_ok());
}
```

### API Test Example
```rust
#[tokio::test]
async fn test_new_endpoint() {
    let serial_manager = SerialManager::new();
    let app = web::create_router(serial_manager);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/new-endpoint")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
```

## Continuous Integration

To run tests in CI:

```yaml
# .github/workflows/test.yml
name: Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test --verbose
```

## Test Results Summary

```
Running unittests src/lib.rs
test result: ok. 29 passed; 0 failed

Running tests/integration_tests.rs
test result: ok. 13 passed; 0 failed

Total: 42 tests passed ✅
```

## Future Test Enhancements

Potential additions to the test suite:

- [ ] Virtual serial port testing using `socat` or similar
- [ ] WebSocket connection tests
- [ ] Performance/load testing
- [ ] Property-based testing with `proptest`
- [ ] Code coverage reporting with `tarpaulin`
- [ ] Benchmark tests with `criterion`
