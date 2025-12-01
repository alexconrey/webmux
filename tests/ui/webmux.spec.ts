import { test, expect } from '@playwright/test';

test.describe('WebMux UI', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await expect(page).toHaveTitle('WebMux');
    // Wait for Vue to mount by checking for the app container
    await page.waitForSelector('#app', { state: 'visible' });
  });

  test('displays the main interface elements', async ({ page }) => {
    // Check for header
    await expect(page.locator('h1')).toHaveText('WebMux');

    // Check for connection selector
    await expect(page.locator('#connection-select')).toBeVisible();

    // Check for connect button
    await expect(page.getByRole('button', { name: /connect/i })).toBeVisible();

    // Check for terminal
    await expect(page.locator('#terminal')).toBeVisible();

    // Check for info panel headers using h3 locators
    await expect(page.locator('aside.info-panel h3').nth(0)).toContainText('Connection Info');
    await expect(page.locator('aside.info-panel h3').nth(1)).toContainText('Statistics');
    await expect(page.locator('aside.info-panel h3').nth(2)).toContainText('Quick Commands');
  });

  test('lists available connections', async ({ page }) => {
    const select = page.locator('#connection-select');

    // Wait for connections to load by checking for a specific device option
    await expect(select.locator('option[value="iot_sensor"]')).toBeAttached();

    // Get all option texts
    const options = await select.locator('option').allTextContents();

    // Should have placeholder
    expect(options[0]).toBe('Select a device...');

    // Should list mock devices from config.virtual.yaml
    expect(options).toContain('iot_sensor');
    expect(options).toContain('embedded_mcu');
    expect(options).toContain('industrial_plc');
  });

  test('connect button is disabled without device selection', async ({ page }) => {
    const connectButton = page.getByRole('button', { name: /connect/i });
    await expect(connectButton).toBeDisabled();
  });

  test('can select a device and enable connect button', async ({ page }) => {
    const select = page.locator('#connection-select');
    const connectButton = page.getByRole('button', { name: /connect/i });

    // Wait for connections to load
    await expect(select.locator('option[value="iot_sensor"]')).toBeAttached();

    // Select a device
    await select.selectOption('iot_sensor');

    // Connect button should be enabled
    await expect(connectButton).toBeEnabled();
  });

  test('can connect to a device and interact with terminal', async ({ page }) => {
    const select = page.locator('#connection-select');
    const connectButton = page.getByRole('button', { name: /connect/i });

    // Wait for connections to load
    await expect(select.locator('option[value="iot_sensor"]')).toBeAttached();

    // Select iot_sensor device
    await select.selectOption('iot_sensor');

    // Connect to the device
    await connectButton.click();

    // Wait for connection status to change
    await expect(page.locator('.status.connected')).toBeVisible();
    await expect(page.locator('.status')).toContainText('Connected');

    // Button should change to Disconnect
    await expect(page.getByRole('button', { name: /disconnect/i })).toBeVisible();

    // Connection info should be populated - use more specific locators
    await expect(page.locator('.info-content .info-item').filter({ hasText: 'Name:' })).toContainText('iot_sensor');
    await expect(page.locator('.info-content .info-item').filter({ hasText: 'Port:' })).toContainText('ttyVIOT0');
    await expect(page.locator('.info-content .info-item').filter({ hasText: 'Baud Rate:' })).toContainText('115200');

    // Focus the terminal by clicking on it
    await page.locator('#terminal').click();

    // Type HELP command into the terminal
    await page.keyboard.type('HELP');

    // Press Enter to send the command
    await page.keyboard.press('Enter');

    // Wait for response from mock device by polling terminal content
    await expect(async () => {
      const terminalContent = await page.locator('#terminal').innerText();
      expect(terminalContent).toContain('COMMANDS:');
    }).toPass({ timeout: 3000 });
  });

  test('can send quick commands', async ({ page }) => {
    const select = page.locator('#connection-select');
    const connectButton = page.getByRole('button', { name: /connect/i });

    // Wait for connections to load
    await expect(select.locator('option[value="iot_sensor"]')).toBeAttached();

    // Select and connect to device
    await select.selectOption('iot_sensor');
    await connectButton.click();

    // Wait for connection
    await expect(page.locator('.status.connected')).toBeVisible();

    // Click on a quick command button (e.g., STATUS)
    const statusButton = page.getByRole('button', { name: 'STATUS' });
    await statusButton.click();

    // Wait for response by polling terminal content
    await expect(async () => {
      const terminalContent = await page.locator('#terminal').innerText();
      expect(terminalContent).toContain('STATUS');
    }).toPass({ timeout: 2000 });
  });

  test('displays statistics after connection', async ({ page }) => {
    const select = page.locator('#connection-select');
    const connectButton = page.getByRole('button', { name: /connect/i });

    // Wait for connections to load
    await expect(select.locator('option[value="iot_sensor"]')).toBeAttached();

    // Select and connect
    await select.selectOption('iot_sensor');
    await connectButton.click();

    // Wait for connection
    await expect(page.locator('.status.connected')).toBeVisible();

    // Statistics should be visible
    await expect(page.locator('.info-panel')).toContainText('Bytes Received');
    await expect(page.locator('.info-panel')).toContainText('Bytes Sent');
    await expect(page.locator('.info-panel')).toContainText('Uptime');
  });

  test('can disconnect from device', async ({ page }) => {
    const select = page.locator('#connection-select');
    const connectButton = page.getByRole('button', { name: /connect/i });

    // Wait for connections to load
    await expect(select.locator('option[value="iot_sensor"]')).toBeAttached();

    // Connect
    await select.selectOption('iot_sensor');
    await connectButton.click();
    await expect(page.locator('.status.connected')).toBeVisible();

    // Disconnect
    const disconnectButton = page.getByRole('button', { name: /disconnect/i });
    await disconnectButton.click();

    // Status should change to Disconnected
    await expect(page.locator('.status.disconnected')).toBeVisible();
    await expect(page.locator('.status')).toContainText('Disconnected');

    // Button should change back to Connect
    await expect(page.getByRole('button', { name: /^connect$/i })).toBeVisible();
  });

  test('can clear terminal', async ({ page }) => {
    const select = page.locator('#connection-select');
    const connectButton = page.getByRole('button', { name: /connect/i });

    // Wait for connections to load
    await expect(select.locator('option[value="iot_sensor"]')).toBeAttached();

    // Connect and send a command
    await select.selectOption('iot_sensor');
    await connectButton.click();
    await expect(page.locator('.status.connected')).toBeVisible();

    // Send some data
    await page.locator('#terminal').click();
    await page.keyboard.type('TEST');
    await page.keyboard.press('Enter');

    // Click Clear button
    const clearButton = page.getByRole('button', { name: 'Clear' });
    await clearButton.click();

    // Wait for clear message to appear by polling
    await expect(async () => {
      const terminalContent = await page.locator('#terminal').innerText();
      expect(terminalContent).toContain('Terminal cleared');
    }).toPass({ timeout: 2000 });
  });

  test('multiple device switching', async ({ page }) => {
    const select = page.locator('#connection-select');
    const connectButton = page.getByRole('button', { name: /connect/i });

    // Wait for connections to load
    await expect(select.locator('option[value="iot_sensor"]')).toBeAttached();

    // Connect to first device
    await select.selectOption('iot_sensor');
    await connectButton.click();
    await expect(page.locator('.status.connected')).toBeVisible();

    // Verify connection info shows iot_sensor
    await expect(page.locator('.info-content .info-item').filter({ hasText: 'Name:' })).toContainText('iot_sensor');

    // Disconnect
    const disconnectButton = page.getByRole('button', { name: /disconnect/i });
    await disconnectButton.click();
    await expect(page.locator('.status.disconnected')).toBeVisible();

    // Wait for WebSocket to fully close before connecting to next device
    await page.waitForTimeout(500);

    // Connect to different device
    await select.selectOption('embedded_mcu');
    await page.getByRole('button', { name: /^connect$/i }).click();
    await expect(page.locator('.status.connected')).toBeVisible();

    // Verify connection info shows embedded_mcu
    await expect(page.locator('.info-content .info-item').filter({ hasText: 'Name:' })).toContainText('embedded_mcu');
  });
});
