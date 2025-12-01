# WebMux UI Tests

This directory contains Playwright end-to-end tests for the WebMux web interface.

## Prerequisites

- Node.js (v18 or later)
- WebMux server running with virtual devices configured
- Playwright browsers installed

## Setup

1. Install dependencies:
```bash
npm install
```

2. Install Playwright browsers:
```bash
npx playwright install
```

## Running Tests

### Run all tests (headless):
```bash
npm test
```

### Run tests in headed mode (see browser):
```bash
npm run test:headed
```

### Run tests in debug mode:
```bash
npm run test:debug
```

### Run tests with UI mode (interactive):
```bash
npm run test:ui
```

### Run specific test file:
```bash
npx playwright test webmux.spec.ts
```

### Run specific test:
```bash
npx playwright test -g "can connect to a device"
```

## Test Coverage

The UI tests cover:

- **UI Elements**: Verifies all main interface elements are present
- **Connection Management**:
  - Listing available devices
  - Selecting devices
  - Connecting and disconnecting
  - Multiple device switching
- **Terminal Interaction**:
  - Sending commands via keyboard input
  - Receiving responses from mock devices
  - Command buffering and execution
  - Quick command buttons
- **Statistics Display**: Connection stats, bytes sent/received, uptime
- **Terminal Controls**: Clear button functionality

## Test Configuration

The tests are configured in [playwright.config.ts](../../playwright.config.ts):

- **Base URL**: http://127.0.0.1:8080
- **Browsers**: Chromium, Firefox, WebKit
- **Web Server**: Automatically starts WebMux server with `config.virtual.yaml`
- **Retries**: 2 retries in CI, 0 locally
- **Screenshots**: Only on failure
- **Traces**: On first retry

## CI Integration

The tests are designed to run in CI environments:

```yaml
# Example GitHub Actions workflow
- name: Install dependencies
  run: npm install

- name: Install Playwright browsers
  run: npx playwright install --with-deps

- name: Run Playwright tests
  run: npm test
```

## Writing New Tests

When adding new tests:

1. Place test files in `tests/ui/` with `.spec.ts` extension
2. Use descriptive test names
3. Follow the existing test structure
4. Include proper waits for WebSocket connections
5. Clean up by disconnecting from devices

Example test structure:
```typescript
test('descriptive test name', async ({ page }) => {
  // Navigate
  await page.goto('/');

  // Interact
  await page.locator('#connection-select').selectOption('iot_sensor');
  await page.getByRole('button', { name: /connect/i }).click();

  // Assert
  await expect(page.locator('.status.connected')).toBeVisible();
});
```

## Debugging Tests

### Visual debugging:
```bash
npm run test:debug
```

### View test report:
```bash
npx playwright show-report
```

### Generate trace:
```bash
npx playwright test --trace on
```

## Mock Device Commands

The tests interact with mock devices that support these commands:
- `HELP` - Show available commands
- `STATUS` - Show device status
- `VERSION` - Show device version
- `TEMP` - Get temperature reading (for iot_sensor)

These commands are implemented in the mock device binary and will respond with appropriate data.
