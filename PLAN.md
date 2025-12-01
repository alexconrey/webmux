# PLAN

## Objective
Create a web server in Rust that can interact with any number of serial connections that are present in a given config.yaml file.

## Status: ✅ COMPLETED

## Implementation Summary

### Core Features Implemented
- ✅ Rust-based web server using Axum framework
- ✅ YAML-based configuration system for serial connections
- ✅ Support for multiple simultaneous serial connections
- ✅ RESTful API for serial communication
- ✅ WebSocket support for real-time bidirectional streaming
- ✅ Optional per-connection logging with hex/ASCII output
- ✅ Graceful shutdown handling
- ✅ Cross-platform support (Linux, macOS, Windows)

### Technical Stack
- **Web Framework**: Axum 0.7 with WebSocket support
- **Async Runtime**: Tokio
- **Serial Communication**: tokio-serial, serialport
- **Configuration**: serde, serde_yaml
- **Logging**: tracing, tracing-subscriber

### Project Structure
```
src/
├── main.rs              - Application entry point
├── config/mod.rs        - Configuration parsing
├── serial/
│   ├── mod.rs          - Serial manager
│   └── connection.rs   - Individual connection handler
├── web/
│   ├── mod.rs          - Web server setup
│   └── handlers.rs     - API endpoint handlers
└── logging/mod.rs      - Serial data logging
```

### API Endpoints
- `GET /health` - Health check
- `GET /api/connections` - List all connections
- `GET /api/connections/:name` - Get connection info
- `POST /api/connections/:name/send` - Send data to serial port
- `GET /api/connections/:name/stats` - Get connection statistics
- `WS /api/connections/:name/ws` - WebSocket for real-time streaming

### Configuration Features
- Per-connection serial parameters (baud rate, data bits, stop bits, parity, flow control)
- Enable/disable individual connections
- Optional logging with custom file paths
- Flexible web server binding (host/port)
- Connection descriptions for documentation

### Testing & Quality Assurance
- ✅ Comprehensive unit test suite (29 tests in Rust)
- ✅ Integration tests (13 tests covering API workflows)
- ✅ CLI tests (11 tests for command-line interface)
- ✅ End-to-end Playwright UI tests (10 tests across 3 browsers)
- ✅ Mock device simulators for testing without hardware
- ✅ Docker-based testing environment with virtual serial ports
- ✅ GitHub Actions CI/CD pipeline for automated testing

### CI/CD Pipeline
- ✅ Automated Rust tests (cargo test, clippy)
- ✅ Automated Playwright UI tests
- ✅ Multi-platform release builds (macOS x86/ARM, Linux x86/ARM, Windows)
- ✅ Docker image builds and publishing
- ✅ Artifact retention and test result uploads

### Next Steps (Future Enhancements)
- Add authentication/authorization
- TLS/SSL support
- Dynamic connection management (hot reload)
- Serial port auto-discovery
- Connection health monitoring and auto-reconnect
- Metrics and Prometheus endpoints

---

## Session History

### Session: UI Test Fixes & CI/CD Setup (December 2024)

**Objective**: Fix failing Playwright UI tests, optimize test performance, and establish CI/CD pipeline.

#### Problems Addressed

1. **Playwright Test Failures**
   - **Issue**: 10/10 tests failing with visibility errors and empty connection info
   - **Root Causes**:
     - Vue.js app not fully mounted before tests ran
     - xterm.js DOM renderer polluting `textContent()` with inline CSS
     - Backend API only returning `{name, exists}` instead of full config
     - No intelligent waiting for API data to load
   - **Solutions**:
     - Added `await page.waitForSelector('#app', { state: 'visible' })` in `beforeEach`
     - Changed from `textContent()` to `innerText()` for terminal content reading
     - Updated `ConnectionInfo` struct to include all config fields (port, baud_rate, data_bits, stop_bits, parity)
     - Added `config()` getter method to `SerialConnection`
     - Rewrote `get_connection_info()` handler to return full configuration
     - Replaced arbitrary `waitForTimeout()` with `expect().toBeAttached()` checks
     - Used `toPass()` polling for dynamic content instead of fixed delays

2. **Mock Device Response Mismatch**
   - **Issue**: Tests expected "AVAILABLE COMMANDS" but mock device returned "COMMANDS:"
   - **Solution**: Updated test expectations to match actual mock device output format

3. **WebKit Multiple Device Switching Failure**
   - **Issue**: WebSocket race condition when switching between devices in WebKit
   - **Root Cause**: WebSocket closing in only 30ms wasn't enough time before opening new connection
   - **Solution**: Added 500ms delay after disconnecting to allow WebSocket cleanup

4. **Test Performance Optimization**
   - **Issue**: Tests taking 37.3s with many artificial delays
   - **Improvements**:
     - Removed 5x 5000ms visibility check delays
     - Removed 1x 10000ms timeout
     - Removed 8x 1000ms API load delays → replaced with `toBeAttached()` checks
     - Removed terminal response delays → replaced with `toPass()` polling
     - Kept only 500ms delay for WebSocket cleanup (necessary)
   - **Result**: Tests now run in 21.8s (42% faster)

5. **Rust Unit Test Failures**
   - **Issue**: Tests using old `ConnectionInfo` structure with `exists` field
   - **Solution**: Updated all test files to use new structure with config fields

6. **README Formatting Issues**
   - **Issue**: API docs outdated, project structure tree malformed, broken link
   - **Solutions**:
     - Updated "Get Connection Info" API documentation
     - Fixed project structure tree with proper Unicode box-drawing
     - Fixed CORS link by removing line number reference

7. **GitHub Actions Deprecation**
   - **Issue**: Using deprecated `actions/upload-artifact@v3` and `actions/cache@v3`
   - **Solution**: Updated to v4 of both actions (6 total updates)

#### Files Modified

**Test Files**:
- `tests/ui/webmux.spec.ts` - Playwright UI tests with intelligent waits and polling
- `src/web/handler_tests.rs` - Unit tests for web handlers
- `src/web/tests.rs` - Web module unit tests
- `tests/integration_tests.rs` - Integration test suite

**Backend Files**:
- `src/web/handlers.rs` - Updated `ConnectionInfo` struct and `get_connection_info()` handler
- `src/serial/connection.rs` - Added `config()` getter method

**Documentation**:
- `README.md` - Fixed API docs, project structure, and links

**CI/CD**:
- `.github/workflows/test.yml` - Updated to latest action versions
- `.github/workflows/release.yml` - Multi-platform release pipeline

#### Test Results

**Before Fixes**:
- Playwright: 0/10 passing
- Rust tests: 42 failing

**After Fixes**:
- Playwright: 10/10 passing (Chromium, Firefox, WebKit)
- Rust unit tests: 29/29 passing
- Integration tests: 13/13 passing
- CLI tests: 11/11 passing
- **Total: 63/63 tests passing**

#### Key Learnings

1. **Vue Mount Timing**: Always wait for framework initialization before running UI tests
2. **Terminal Content Reading**: Use `innerText()` instead of `textContent()` with xterm.js DOM renderer
3. **Intelligent Waits**: Replace arbitrary timeouts with `expect().toBeAttached()` and `toPass()` polling
4. **WebSocket Cleanup**: Allow sufficient time for WebSocket connections to fully close before opening new ones
5. **API Contract Consistency**: Ensure backend returns all data that frontend expects
6. **Build Caching**: Run `cargo clean` when code changes don't seem to take effect
7. **Mock Device Testing**: Use virtual serial ports and mock devices for hardware-free testing
8. **Action Versions**: Keep GitHub Actions up to date to avoid deprecation warnings

#### Commands Reference

```bash
# Run all tests
cargo test                    # Rust tests (53 total)
npm test                      # Playwright UI tests (10 total)

# Run specific test suites
cargo test --lib             # Unit tests only
cargo test --test integration_tests  # Integration tests only
npx playwright test --headed # UI tests with browser visible

# Performance and quality
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release

# Docker testing environment
docker-compose up --build    # Start all services with mock devices
docker-compose ps           # Check service status
docker-compose logs server  # View server logs

# Cleanup
cargo clean                 # Clear build cache
docker-compose down         # Stop all containers
```