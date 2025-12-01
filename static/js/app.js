// WebMux - Vue.js Frontend with xterm.js
const { createApp, ref, computed, onMounted, onUnmounted, nextTick } = Vue;

createApp({
    setup() {
        // State
        const connections = ref([]);
        const selectedConnection = ref('');
        const isConnected = ref(false);
        const status = ref('Disconnected');
        const connectionInfo = ref(null);
        const stats = ref(null);
        const quickCommands = ref(['STATUS', 'VERSION', 'HELP', 'TEMP']);

        // WebSocket
        let ws = null;
        let statsInterval = null;

        // xterm.js terminal
        let term = null;
        let fitAddon = null;
        let currentLine = ''; // Buffer for current input line

        // Refs for DOM elements
        const terminalEl = ref(null);

        // Computed
        const statusClass = computed(() => {
            if (status.value === 'Connected') return 'connected';
            if (status.value === 'Error') return 'error';
            return 'disconnected';
        });

        const terminalTitle = computed(() => {
            return selectedConnection.value || 'No device selected';
        });

        // Methods
        const initTerminal = () => {
            if (!terminalEl.value) return;

            // Initialize xterm.js
            term = new Terminal({
                cursorBlink: true,
                fontSize: 14,
                fontFamily: 'Consolas, Monaco, "Courier New", monospace',
                theme: {
                    background: '#0c0c0c',
                    foreground: '#d4d4d4',
                    cursor: '#4ec9b0',
                    black: '#0c0c0c',
                    red: '#f48771',
                    green: '#4ec9b0',
                    yellow: '#dcdcaa',
                    blue: '#007acc',
                    magenta: '#c678dd',
                    cyan: '#56b6c2',
                    white: '#d4d4d4',
                    brightBlack: '#969696',
                    brightRed: '#f48771',
                    brightGreen: '#4ec9b0',
                    brightYellow: '#dcdcaa',
                    brightBlue: '#007acc',
                    brightMagenta: '#c678dd',
                    brightCyan: '#56b6c2',
                    brightWhite: '#ffffff'
                },
                scrollback: 10000,
                allowProposedApi: true
            });

            // Add fit addon
            fitAddon = new FitAddon.FitAddon();
            term.loadAddon(fitAddon);

            // Open terminal
            term.open(terminalEl.value);
            fitAddon.fit();

            // Handle terminal input
            term.onData((data) => {
                if (!isConnected.value) return;

                const code = data.charCodeAt(0);

                // Handle Enter key
                if (code === 13) {
                    term.write('\r\n');
                    if (currentLine.length > 0) {
                        // Send the complete command
                        sendCommand(currentLine);
                        currentLine = '';
                    }
                }
                // Handle Backspace
                else if (code === 127 || code === 8) {
                    if (currentLine.length > 0) {
                        currentLine = currentLine.slice(0, -1);
                        term.write('\b \b');
                    }
                }
                // Handle Ctrl+C
                else if (code === 3) {
                    term.write('^C\r\n');
                    currentLine = '';
                }
                // Handle Ctrl+U (clear line)
                else if (code === 21) {
                    while (currentLine.length > 0) {
                        term.write('\b \b');
                        currentLine = currentLine.slice(0, -1);
                    }
                }
                // Handle printable characters
                else if (code >= 32 && code < 127) {
                    currentLine += data;
                    term.write(data);
                }
            });

            // Display welcome message
            term.writeln('\x1b[36mWelcome to WebMux\x1b[0m');
            term.writeln('\x1b[2mSelect a device and click Connect to start\x1b[0m');
            term.writeln('');

            // Handle window resize
            window.addEventListener('resize', () => {
                if (fitAddon) {
                    fitAddon.fit();
                }
            });
        };

        const log = (message, type = 'output') => {
            if (!term) return;

            const timestamp = new Date().toLocaleTimeString();
            let prefix = '';
            let suffix = '\x1b[0m'; // Reset color

            switch (type) {
                case 'system':
                    prefix = `\x1b[36m[${timestamp}]\x1b[0m \x1b[36m`; // Cyan
                    break;
                case 'error':
                    prefix = `\x1b[31m[${timestamp}]\x1b[0m \x1b[31m`; // Red
                    break;
                case 'success':
                    prefix = `\x1b[32m[${timestamp}]\x1b[0m \x1b[32m`; // Green
                    break;
                case 'input':
                    prefix = `\x1b[33m[${timestamp}]\x1b[0m \x1b[33m$ `; // Yellow
                    break;
                default:
                    prefix = `\x1b[90m[${timestamp}]\x1b[0m `; // Gray
            }

            term.writeln(prefix + message + suffix);
        };

        const loadConnections = async () => {
            try {
                const response = await fetch('/api/connections');
                connections.value = await response.json();
                log(`Found ${connections.value.length} available connections`, 'system');
            } catch (error) {
                log(`Error loading connections: ${error.message}`, 'error');
            }
        };

        const onConnectionChange = () => {
            // Reset state when connection changes
            if (isConnected.value) {
                disconnect();
            }
        };

        const toggleConnection = () => {
            if (isConnected.value) {
                disconnect();
            } else {
                connect();
            }
        };

        const connect = () => {
            if (!selectedConnection.value) return;

            log(`Connecting to ${selectedConnection.value}...`, 'system');

            // Construct WebSocket URL
            const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
            const wsUrl = `${protocol}//${window.location.host}/api/connections/${selectedConnection.value}/ws`;

            ws = new WebSocket(wsUrl);
            ws.binaryType = 'arraybuffer'; // Handle binary data

            ws.onopen = () => {
                isConnected.value = true;
                status.value = 'Connected';
                log(`Connected to ${selectedConnection.value}`, 'success');

                // Load connection info and start stats updates
                updateConnectionInfo();
                updateStats();
                statsInterval = setInterval(updateStats, 5000);

                // Focus terminal
                if (term) {
                    term.focus();
                }
            };

            ws.onmessage = async (event) => {
                await handleMessage(event.data);
            };

            ws.onerror = () => {
                log(`WebSocket error`, 'error');
                status.value = 'Error';
            };

            ws.onclose = () => {
                if (isConnected.value) {
                    log(`Disconnected from ${selectedConnection.value}`, 'system');
                }
                disconnect();
            };
        };

        const disconnect = () => {
            if (ws) {
                ws.close();
                ws = null;
            }

            if (statsInterval) {
                clearInterval(statsInterval);
                statsInterval = null;
            }

            isConnected.value = false;
            status.value = 'Disconnected';
            connectionInfo.value = null;
            stats.value = null;
        };

        const sendCommand = async (command) => {
            if (!isConnected.value || !selectedConnection.value) return;

            try {
                // Add newline to command if not present
                const commandWithNewline = command.endsWith('\n') ? command : command + '\n';

                const response = await fetch(`/api/connections/${selectedConnection.value}/send`, {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({
                        data: commandWithNewline,
                        format: 'text'
                    })
                });

                if (!response.ok) {
                    throw new Error(`HTTP ${response.status}`);
                }
            } catch (error) {
                log(`Error sending command: ${error.message}`, 'error');
            }
        };

        const sendQuickCommand = (cmd) => {
            if (!isConnected.value) return;

            // Write the command to terminal
            term.writeln(`\x1b[33m$ ${cmd}\x1b[0m`);

            // Send it
            sendCommand(cmd);
        };

        const handleMessage = async (data) => {
            if (!term) return;

            try {
                // Convert ArrayBuffer to string if needed
                let text;
                if (data instanceof ArrayBuffer) {
                    text = new TextDecoder().decode(data);
                } else if (data instanceof Blob) {
                    text = await data.text();
                } else {
                    text = data;
                }

                // Try to parse as JSON first
                try {
                    const message = JSON.parse(text);
                    if (message.data) {
                        let displayData = message.data;

                        if (message.format === 'hex') {
                            displayData = hexToAscii(message.data);
                        } else if (message.format === 'base64') {
                            displayData = atob(message.data);
                        }

                        term.write(displayData);
                    }
                } catch (e) {
                    // If not JSON, display as-is
                    term.write(text);
                }
            } catch (error) {
                log(`Error handling message: ${error.message}`, 'error');
            }
        };

        const updateConnectionInfo = async () => {
            if (!selectedConnection.value) return;

            try {
                const response = await fetch(`/api/connections/${selectedConnection.value}`);
                connectionInfo.value = await response.json();
            } catch (error) {
                console.error('Error updating connection info:', error);
            }
        };

        const updateStats = async () => {
            if (!selectedConnection.value) return;

            try {
                const response = await fetch(`/api/connections/${selectedConnection.value}/stats`);
                stats.value = await response.json();
            } catch (error) {
                console.error('Error updating stats:', error);
            }
        };

        const clearTerminal = () => {
            if (term) {
                term.clear();
                log('Terminal cleared', 'system');
            }
        };

        const formatNumber = (num) => {
            return num?.toLocaleString() || '0';
        };

        const formatUptime = (seconds) => {
            const minutes = Math.floor(seconds / 60);
            const secs = seconds % 60;
            return `${minutes}m ${secs}s`;
        };

        const hexToAscii = (hex) => {
            let str = '';
            for (let i = 0; i < hex.length; i += 2) {
                str += String.fromCharCode(parseInt(hex.substr(i, 2), 16));
            }
            return str;
        };

        // Lifecycle
        onMounted(() => {
            initTerminal();
            loadConnections();
        });

        onUnmounted(() => {
            disconnect();
            if (term) {
                term.dispose();
            }
        });

        // Return reactive data and methods
        return {
            // State
            connections,
            selectedConnection,
            isConnected,
            status,
            connectionInfo,
            stats,
            quickCommands,

            // Computed
            statusClass,
            terminalTitle,

            // Refs
            terminalEl,

            // Methods
            onConnectionChange,
            toggleConnection,
            sendQuickCommand,
            clearTerminal,
            updateStats,
            formatNumber,
            formatUptime
        };
    }
}).mount('#app');
