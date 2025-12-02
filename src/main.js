// VopecsPrinter - Main JavaScript (TableTrack Clone)
const { invoke } = window.__TAURI__.core;

// State
let config = {
  domainUrl: '',
  key: '',
  printerMappings: {},
  openDrawerAfterPrint: false,
  drawerPin: 0,
  pollingInterval: 3000
};

let isServiceRunning = false;
let pollingTimer = null;
let systemPrinters = [];
let apiPrinters = [];

// ============ Initialization ============

document.addEventListener('DOMContentLoaded', async () => {
  log('Starting VopecsPrinter...');
  await loadConfig();
  await loadSystemPrinters();
  await checkAutostart();
  setupEventListeners();

  // Auto-start service if configured
  if (config.domainUrl && config.key) {
    startService();
  }
});

// ============ Event Listeners ============

function setupEventListeners() {
  // Configuration
  document.getElementById('configure-btn').addEventListener('click', openConfigModal);
  document.getElementById('close-modal').addEventListener('click', closeConfigModal);
  document.getElementById('config-form').addEventListener('submit', saveConfig);
  document.getElementById('test-connection').addEventListener('click', testConnection);

  // Service controls
  document.getElementById('toggle-service').addEventListener('click', toggleService);
  document.getElementById('minimize-tray').addEventListener('click', minimizeToTray);

  // Printers
  document.getElementById('refresh-printers').addEventListener('click', refreshPrinters);

  // Autostart
  document.getElementById('toggle-autostart').addEventListener('click', toggleAutostart);

  // Drawer settings
  document.getElementById('open-drawer-toggle').addEventListener('change', updateDrawerSettings);
  document.querySelectorAll('input[name="drawer-pin"]').forEach(radio => {
    radio.addEventListener('change', updateDrawerSettings);
  });

  // Updates
  document.getElementById('check-updates').addEventListener('click', checkUpdates);

  // Close modal on outside click
  document.getElementById('config-modal').addEventListener('click', (e) => {
    if (e.target.id === 'config-modal') closeConfigModal();
  });
}

// ============ Configuration ============

async function loadConfig() {
  try {
    config = await invoke('get_config');
    updateUI();
    log('Configuration loaded');
  } catch (error) {
    log('Failed to load config: ' + error, 'error');
  }
}

function updateUI() {
  // Update connection display
  document.getElementById('server-url').textContent = config.domainUrl || 'Not configured';
  document.getElementById('server-url').className = config.domainUrl ? 'text-primary' : 'text-danger';

  if (config.key) {
    const maskedKey = config.key.substring(0, 12) + '***';
    document.getElementById('api-key-display').textContent = maskedKey;
    document.getElementById('api-key-display').className = 'text-danger';
  } else {
    document.getElementById('api-key-display').textContent = 'Not configured';
  }

  // Update form fields
  document.getElementById('domain-url').value = config.domainUrl || '';
  document.getElementById('api-key').value = config.key || '';
  document.getElementById('polling-interval').value = (config.pollingInterval || 3000) / 1000;

  // Update drawer settings
  document.getElementById('open-drawer-toggle').checked = config.openDrawerAfterPrint;
  document.querySelector(`input[name="drawer-pin"][value="${config.drawerPin}"]`).checked = true;
  updateDrawerInfo();

  // Update polling interval display
  document.getElementById('poll-interval-display').textContent = (config.pollingInterval || 3000) / 1000;
}

function openConfigModal() {
  document.getElementById('config-modal').classList.remove('hidden');
}

function closeConfigModal() {
  document.getElementById('config-modal').classList.add('hidden');
}

async function saveConfig(e) {
  e.preventDefault();

  const newConfig = {
    domainUrl: document.getElementById('domain-url').value,
    key: document.getElementById('api-key').value,
    printerName: config.printerName,
    printerMappings: config.printerMappings,
    openDrawerAfterPrint: config.openDrawerAfterPrint,
    drawerPin: config.drawerPin,
    pollingInterval: parseInt(document.getElementById('polling-interval').value) * 1000,
    autoStart: config.autoStart
  };

  try {
    await invoke('save_config', { configData: newConfig });
    config = newConfig;
    updateUI();
    closeConfigModal();
    log('Configuration saved successfully');

    // Restart service with new config
    if (isServiceRunning) {
      stopService();
      startService();
    }
  } catch (error) {
    log('Failed to save config: ' + error, 'error');
  }
}

async function testConnection() {
  const domainUrl = document.getElementById('domain-url').value;
  const key = document.getElementById('api-key').value;

  if (!domainUrl || !key) {
    log('Please enter domain URL and API key', 'warning');
    return;
  }

  log('Testing connection to ' + domainUrl + '...');

  try {
    const result = await invoke('test_connection', { domainUrl, key });
    if (result) {
      log('Connection successful!');
      updateConnectionStatus(true);
    } else {
      log('Connection failed', 'error');
      updateConnectionStatus(false);
    }
  } catch (error) {
    log('Connection failed: ' + error, 'error');
    updateConnectionStatus(false);
  }
}

function updateConnectionStatus(connected) {
  const dot = document.querySelector('.status-dot');
  const text = document.querySelector('.status-text');

  if (connected && isServiceRunning) {
    dot.className = 'status-dot connected';
    text.textContent = 'Connected & Polling';
  } else if (connected) {
    dot.className = 'status-dot connected';
    text.textContent = 'Connected';
  } else {
    dot.className = 'status-dot disconnected';
    text.textContent = 'Disconnected';
  }
}

// ============ Service Control ============

function toggleService() {
  if (isServiceRunning) {
    stopService();
  } else {
    startService();
  }
}

function startService() {
  if (!config.domainUrl || !config.key) {
    log('Please configure API settings first', 'warning');
    openConfigModal();
    return;
  }

  isServiceRunning = true;
  updateServiceUI();
  log('Print service started - Polling every ' + (config.pollingInterval / 1000) + ' seconds');

  // Start polling
  pollJobs();
  pollingTimer = setInterval(pollJobs, config.pollingInterval || 3000);
}

function stopService() {
  isServiceRunning = false;
  if (pollingTimer) {
    clearInterval(pollingTimer);
    pollingTimer = null;
  }
  updateServiceUI();
  log('Print service stopped');
}

function updateServiceUI() {
  const btn = document.getElementById('toggle-service');
  const statusIndicator = document.getElementById('service-status');
  const serviceCheck = document.getElementById('service-active-check');
  const pollingCheck = document.getElementById('polling-active-check');

  if (isServiceRunning) {
    btn.innerHTML = '<span class="icon">&#x25A0;</span> Stop Service';
    btn.className = 'btn btn-service btn-danger';
    statusIndicator.className = 'status-indicator active';
    statusIndicator.innerHTML = '<span class="dot"></span> Print Service Active';
    serviceCheck.checked = true;
    pollingCheck.checked = true;
    updateConnectionStatus(true);
  } else {
    btn.innerHTML = '<span class="icon">&#x25B6;</span> Start Service';
    btn.className = 'btn btn-service btn-success';
    statusIndicator.className = 'status-indicator';
    statusIndicator.innerHTML = '<span class="dot" style="background:#ef4444"></span> Service Stopped';
    serviceCheck.checked = false;
    pollingCheck.checked = false;
    updateConnectionStatus(false);
  }
}

// ============ Polling ============

async function pollJobs() {
  if (!isServiceRunning) return;

  const now = new Date();
  document.getElementById('last-check-time').textContent = now.toLocaleTimeString();

  try {
    log('Polling print jobs from: ' + config.domainUrl + '/api/print-jobs/pull-multiple');
    const jobs = await invoke('poll_print_jobs');
    log('Poll response status: 200, body length: ' + JSON.stringify(jobs).length);
    log('Processing ' + jobs.length + ' print jobs');

    for (const job of jobs) {
      if (job.status === 'pending') {
        await processJob(job);
      }
    }
  } catch (error) {
    log('Poll failed: ' + error, 'error');
  }
}

async function processJob(job) {
  const printerApiName = job.printer_name || job.printerName || job.printer;
  const localPrinter = config.printerMappings[printerApiName];

  if (!localPrinter) {
    log('No printer mapping for: ' + printerApiName, 'warning');
    try {
      await invoke('update_job_status', {
        jobId: job.id,
        status: 'failed',
        reason: 'No printer mapping configured'
      });
    } catch (e) {}
    return;
  }

  const jobType = getJobType(job);
  log('Processing job #' + job.id + ' (type: ' + jobType + ') for printer: ' + localPrinter);

  try {
    // Handle different job types
    if (job.image) {
      // Base64 image printing
      await invoke('print_base64_to_thermal', {
        printerName: localPrinter,
        base64Image: job.image
      });
    } else if (job.pdf) {
      // PDF printing (URL or base64)
      if (job.pdf.startsWith('http')) {
        await invoke('print_pdf_to_thermal', {
          printerName: localPrinter,
          pdfUrl: job.pdf
        });
      } else {
        // Base64 PDF - treat as text for now
        log('Base64 PDF received, printing as document info', 'info');
        await invoke('print_text_content', {
          printerName: localPrinter,
          content: 'PDF Document Received\nJob ID: ' + job.id
        });
      }
    } else if (job.html) {
      // HTML content printing
      await invoke('print_html_content', {
        printerName: localPrinter,
        html: job.html
      });
    } else if (job.url) {
      // Image from URL
      await invoke('print_image_from_url', {
        printerName: localPrinter,
        url: job.url
      });
    } else if (job.content) {
      // Plain text content
      await invoke('print_text_content', {
        printerName: localPrinter,
        content: job.content
      });
    } else {
      log('Unknown job type for job #' + job.id, 'warning');
      throw new Error('Unknown job type');
    }

    // Print copies if specified
    const copies = job.copies || 1;
    for (let i = 1; i < copies; i++) {
      log('Printing copy ' + (i + 1) + ' of ' + copies);
      if (job.image) {
        await invoke('print_base64_to_thermal', {
          printerName: localPrinter,
          base64Image: job.image
        });
      }
      // Add other copy printing as needed
    }

    await invoke('update_job_status', {
      jobId: job.id,
      status: 'done',
      reason: null
    });

    log('Job #' + job.id + ' completed successfully');

    if (config.openDrawerAfterPrint) {
      log('Opening cash drawer after successful print (Pin ' + (config.drawerPin === 0 ? '1' : '2') + ')');
      await invoke('open_drawer', {
        printerName: localPrinter,
        pin: config.drawerPin
      });
    }
  } catch (error) {
    log('Failed to process job #' + job.id + ': ' + error, 'error');
    try {
      await invoke('update_job_status', {
        jobId: job.id,
        status: 'failed',
        reason: error.toString()
      });
    } catch (e) {}
  }
}

// Determine job type from job data
function getJobType(job) {
  if (job.image) return 'image';
  if (job.pdf) return 'pdf';
  if (job.html) return 'html';
  if (job.url) return 'url';
  if (job.content) return 'content';
  return job.job_type || job.type || 'unknown';
}

// ============ Printers ============

async function loadSystemPrinters() {
  try {
    systemPrinters = await invoke('get_system_printers');
    log('Found ' + systemPrinters.length + ' system printers');
  } catch (error) {
    log('Failed to load system printers: ' + error, 'error');
  }
}

async function refreshPrinters() {
  log('Refreshing printers from API...');

  try {
    await loadSystemPrinters();
    apiPrinters = await invoke('fetch_printers');
    log('Found ' + apiPrinters.length + ' printers from API');
    renderPrinterMappings();
  } catch (error) {
    log('Failed to fetch printers: ' + error, 'error');
  }
}

function renderPrinterMappings() {
  const tbody = document.getElementById('mappings-body');
  const count = document.getElementById('mappings-count');

  if (apiPrinters.length === 0) {
    tbody.innerHTML = '<tr><td colspan="5" class="empty-row">No printers found. Click "Refresh Printers" to fetch from API.</td></tr>';
    count.textContent = '0';
    return;
  }

  count.textContent = apiPrinters.length;

  tbody.innerHTML = apiPrinters.map((printer, index) => {
    // Get the display name - use name, printer_name, or station name
    const stationName = printer.name || printer.printer_name || printer.kitchen_name || `POS Station ${index + 1}`;
    const printerAlias = printer.alias || printer.printer_name || stationName;
    const currentMapping = config.printerMappings[stationName] || '';
    const isThermal = true; // Default to thermal

    return `
      <tr>
        <td><strong>${stationName}</strong></td>
        <td>${printerAlias}</td>
        <td>
          <select onchange="updatePrinterMapping('${stationName}', this.value)">
            <option value="">-- Select Printer --</option>
            ${systemPrinters.map(sp =>
              `<option value="${sp}" ${sp === currentMapping ? 'selected' : ''}>${sp}</option>`
            ).join('')}
          </select>
        </td>
        <td>
          <label class="thermal-check">
            <input type="checkbox" checked>
            Thermal
          </label>
        </td>
        <td>
          <div class="test-buttons">
            <button class="btn btn-success" onclick="testPrint('${stationName}')">
              <span class="icon">&#x1F5A8;</span> Test Print
            </button>
            <button class="btn btn-primary" onclick="testDrawer('${stationName}')">
              <span class="icon">&#x1F4B0;</span> Test Drawer
            </button>
          </div>
        </td>
      </tr>
    `;
  }).join('');
}

window.updatePrinterMapping = async function(apiName, localPrinter) {
  config.printerMappings[apiName] = localPrinter;

  try {
    await invoke('save_config', { configData: config });
    log('Printer mapping updated: ' + apiName + ' -> ' + localPrinter);
  } catch (error) {
    log('Failed to save printer mapping: ' + error, 'error');
  }
};

window.testPrint = async function(apiName) {
  const localPrinter = config.printerMappings[apiName];

  if (!localPrinter) {
    log('Please select a local printer first', 'warning');
    return;
  }

  log('Sending test print to: ' + localPrinter);

  try {
    await invoke('test_print', { printerName: localPrinter });
    log('Test print sent successfully');
  } catch (error) {
    log('Test print failed: ' + error, 'error');
  }
};

window.testDrawer = async function(apiName) {
  const localPrinter = config.printerMappings[apiName];

  if (!localPrinter) {
    log('Please select a local printer first', 'warning');
    return;
  }

  log('Opening drawer on: ' + localPrinter);

  try {
    await invoke('open_drawer', { printerName: localPrinter, pin: config.drawerPin });
    log('Drawer opened successfully');
  } catch (error) {
    log('Failed to open drawer: ' + error, 'error');
  }
};

// ============ Drawer Settings ============

async function updateDrawerSettings() {
  config.openDrawerAfterPrint = document.getElementById('open-drawer-toggle').checked;
  config.drawerPin = parseInt(document.querySelector('input[name="drawer-pin"]:checked').value);

  updateDrawerInfo();

  try {
    await invoke('save_config', { configData: config });
    log('Drawer settings updated');
  } catch (error) {
    log('Failed to save drawer settings: ' + error, 'error');
  }
}

function updateDrawerInfo() {
  const infoBox = document.getElementById('drawer-info');
  const pinDisplay = document.getElementById('selected-pin');

  if (config.openDrawerAfterPrint) {
    infoBox.classList.remove('hidden');
    pinDisplay.textContent = config.drawerPin === 0 ? '1' : '2';
  } else {
    infoBox.classList.add('hidden');
  }
}

// ============ Autostart ============

async function checkAutostart() {
  try {
    const enabled = await invoke('is_autostart_enabled');
    updateAutostartUI(enabled);
  } catch (error) {
    log('Failed to check autostart status: ' + error, 'error');
  }
}

async function toggleAutostart() {
  try {
    const currentlyEnabled = await invoke('is_autostart_enabled');

    if (currentlyEnabled) {
      await invoke('disable_autostart');
      log('Autostart disabled');
    } else {
      await invoke('enable_autostart');
      log('Autostart enabled');
    }

    updateAutostartUI(!currentlyEnabled);
  } catch (error) {
    log('Failed to toggle autostart: ' + error, 'error');
  }
}

function updateAutostartUI(enabled) {
  const btn = document.getElementById('toggle-autostart');
  const status = document.getElementById('autostart-status');

  if (enabled) {
    btn.innerHTML = '<span class="icon">&#x23FB;</span> Disable Autostart';
    status.textContent = 'Enabled';
  } else {
    btn.innerHTML = '<span class="icon">&#x23FB;</span> Enable Autostart';
    status.textContent = 'Disabled';
  }
}

// ============ System ============

async function minimizeToTray() {
  try {
    await invoke('hide_to_tray');
    log('Minimized to tray');
  } catch (error) {
    log('Failed to minimize: ' + error, 'error');
  }
}

async function checkUpdates() {
  log('Checking for updates...');

  try {
    const { check } = window.__TAURI__.updater;
    const { relaunch } = window.__TAURI__.process;

    const update = await check();

    if (update) {
      log('Update available! Version ' + update.version, 'warning');
      log('Release notes: ' + (update.body || 'No release notes'));

      // Show update modal
      showUpdateModal(update, relaunch);
    } else {
      log('You are running the latest version');
    }
  } catch (error) {
    log('Failed to check for updates: ' + error, 'error');
  }
}

function showUpdateModal(update, relaunch) {
  // Create modal if it doesn't exist
  let modal = document.getElementById('update-modal');
  if (!modal) {
    modal = document.createElement('div');
    modal.id = 'update-modal';
    modal.className = 'modal';
    modal.innerHTML = `
      <div class="modal-content">
        <div class="modal-header">
          <h2>Update Available</h2>
          <button class="modal-close" onclick="closeUpdateModal()">&times;</button>
        </div>
        <div class="update-info">
          <p><strong>New version:</strong> <span id="update-version"></span></p>
          <p><strong>Current version:</strong> <span id="current-version"></span></p>
          <div id="update-changelog"></div>
          <div id="download-progress" class="hidden">
            <p>Downloading update... <span id="progress-percent">0%</span></p>
            <div class="progress-bar"><div class="progress-fill" id="progress-fill"></div></div>
          </div>
        </div>
        <div class="form-actions">
          <button class="btn btn-secondary" id="later-btn" onclick="closeUpdateModal()">Later</button>
          <button class="btn btn-success" id="download-update-btn">Download & Install</button>
        </div>
      </div>
    `;
    document.body.appendChild(modal);

    // Add progress bar styles
    const style = document.createElement('style');
    style.textContent = `
      .progress-bar { width: 100%; height: 20px; background: var(--gray-200); border-radius: 10px; overflow: hidden; margin-top: 10px; }
      .progress-fill { height: 100%; background: var(--success); width: 0%; transition: width 0.3s; }
      #download-progress { margin-top: 15px; }
    `;
    document.head.appendChild(style);
  }

  // Fill in update info
  document.getElementById('update-version').textContent = update.version;
  document.getElementById('current-version').textContent = update.currentVersion;

  if (update.body) {
    document.getElementById('update-changelog').innerHTML = '<p><strong>Release notes:</strong></p><p>' + update.body + '</p>';
  }

  // Set download button action
  const downloadBtn = document.getElementById('download-update-btn');
  const laterBtn = document.getElementById('later-btn');
  const progressDiv = document.getElementById('download-progress');
  const progressPercent = document.getElementById('progress-percent');
  const progressFill = document.getElementById('progress-fill');

  downloadBtn.onclick = async () => {
    downloadBtn.disabled = true;
    downloadBtn.textContent = 'Downloading...';
    laterBtn.style.display = 'none';
    progressDiv.classList.remove('hidden');

    try {
      let downloaded = 0;
      let contentLength = 0;

      await update.downloadAndInstall((event) => {
        switch (event.event) {
          case 'Started':
            contentLength = event.data.contentLength;
            log('Download started, size: ' + Math.round(contentLength / 1024 / 1024) + ' MB');
            break;
          case 'Progress':
            downloaded += event.data.chunkLength;
            const percent = Math.round((downloaded / contentLength) * 100);
            progressPercent.textContent = percent + '%';
            progressFill.style.width = percent + '%';
            break;
          case 'Finished':
            log('Download complete, installing...');
            progressPercent.textContent = '100%';
            progressFill.style.width = '100%';
            break;
        }
      });

      log('Update installed! Restarting...');
      await relaunch();
    } catch (error) {
      log('Update failed: ' + error, 'error');
      downloadBtn.disabled = false;
      downloadBtn.textContent = 'Retry';
      laterBtn.style.display = 'inline-flex';
    }
  };

  modal.classList.remove('hidden');
}

window.closeUpdateModal = function() {
  const modal = document.getElementById('update-modal');
  if (modal) {
    modal.classList.add('hidden');
  }
}

// ============ Logging ============

function log(message, type = 'info') {
  const container = document.getElementById('logs-container');
  const timestamp = new Date().toISOString();

  const entry = document.createElement('div');
  entry.className = 'log-entry' + (type !== 'info' ? ' ' + type : '');
  entry.textContent = `[${timestamp}] ${message}`;

  container.appendChild(entry);
  container.scrollTop = container.scrollHeight;

  // Keep only last 100 entries
  while (container.children.length > 100) {
    container.removeChild(container.firstChild);
  }

  console.log(`[${type.toUpperCase()}] ${message}`);
}
