const { app, BrowserWindow, ipcMain, BrowserView } = require('electron');
const { spawn } = require('child_process');
const path = require('path');
const https = require('https');

let mainWindow;
let proxyProcess;
let isQuitting = false;
let notebookView = null;  // BrowserView for Kaggle/Colab notebooks

const KAGGLE_USERNAME = 'spooky8823';
const KAGGLE_API_TOKEN = 'KGAT_844574b0b2c91c629c0e6e247c618316';
const KAGGLE_KERNEL_SLUG = 'fork-of-notebook87be8ebf87-a2ff4f';

function startProxy() {
  let scriptPath = path.join(__dirname, 'app.py');
  if (scriptPath.includes('app.asar')) {
    scriptPath = scriptPath.replace('app.asar', 'app.asar.unpacked');
  }
  console.log('🔍 app.py path:', scriptPath);
  console.log('🔍 exists:', require('fs').existsSync(scriptPath));

  proxyProcess = spawn('python3', [scriptPath], {
    env: { ...process.env, PORT: '5050' },
    stdio: 'inherit'
  });
  proxyProcess.on('error', (err) => console.error('Proxy error:', err));
  console.log('✅ Local proxy started on port 5050');
}

function stopKaggle() {
  return new Promise((resolve) => {
    const options = {
      hostname: 'www.kaggle.com',
      path: `/api/v1/kernels/${KAGGLE_USERNAME}/${KAGGLE_KERNEL_SLUG}/stop`,
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${KAGGLE_API_TOKEN}`,
        'Content-Type': 'application/json'
      }
    };
    const req = https.request(options, (res) => {
      console.log(`✅ Kaggle stop signal sent: ${res.statusCode}`);
      resolve();
    });
    req.on('error', (e) => {
      console.error('Could not stop Kaggle:', e);
      resolve();
    });
    req.end();
  });
}

function cleanupAndQuit() {
  if (isQuitting) return;
  isQuitting = true;

  console.log('🛑 Shutting down everything...');

  // Shutdown VibiCore engine
  shutdownVibiCore();

  stopKaggle().finally(() => {
    if (proxyProcess) {
      console.log('🛑 Killing local proxy...');
      proxyProcess.kill('SIGKILL');
    }
    app.quit();
    process.exit(0);
  });
}

// ===================== VIBICORE INTEGRATION =====================
// IPC handlers for the HTML frontend to communicate with VibiCore

ipcMain.handle('vibicore-open', async (event, url, tabId) => {
  try {
    const res = await fetch('http://localhost:5050/vibicore/open', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ url, tab_id: tabId })
    });
    return await res.json();
  } catch (e) {
    return { error: e.message };
  }
});

ipcMain.handle('vibicore-screenshot', async (event, tabId) => {
  try {
    const res = await fetch(`http://localhost:5050/vibicore/screenshot/${tabId}`);
    return await res.json();
  } catch (e) {
    return { error: e.message };
  }
});

ipcMain.handle('vibicore-execute', async (event, tabId, script) => {
  try {
    const res = await fetch(`http://localhost:5050/vibicore/execute/${tabId}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ script })
    });
    return await res.json();
  } catch (e) {
    return { error: e.message };
  }
});

ipcMain.handle('vibicore-memory', async () => {
  try {
    const res = await fetch('http://localhost:5050/vibicore/memory');
    return await res.json();
  } catch (e) {
    return { error: e.message };
  }
});

ipcMain.handle('vibicore-close', async (event, tabId) => {
  try {
    const res = await fetch(`http://localhost:5050/vibicore/close/${tabId}`, {
      method: 'POST'
    });
    return await res.json();
  } catch (e) {
    return { error: e.message };
  }
});

// Create a BrowserView to embed VibiCore-rendered content
function createNotebookView() {
  if (notebookView) return notebookView;

  notebookView = new BrowserView({
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      sandbox: true,
    }
  });

  // Set bounds to fill the main window content area
  const bounds = mainWindow.getBounds();
  const sidebarWidth = 260; // Match your sidebar width
  const topbarHeight = 70;  // Match your topbar height

  notebookView.setBounds({
    x: sidebarWidth,
    y: topbarHeight,
    width: bounds.width - sidebarWidth,
    height: bounds.height - topbarHeight
  });

  notebookView.setAutoResize({ width: true, height: true });

  // Hide initially
  mainWindow.removeBrowserView(notebookView);

  return notebookView;
}

// Show/hide notebook view
ipcMain.handle('notebook-show', async (event, url) => {
  if (!notebookView) {
    createNotebookView();
  }

  // Load the URL directly in the BrowserView
  // VibiCore engine handles the actual rendering
  await notebookView.webContents.loadURL(url);

  mainWindow.addBrowserView(notebookView);

  // Inject anti-detection scripts
  await notebookView.webContents.executeJavaScript(`
    Object.defineProperty(navigator, 'webdriver', {get: () => undefined});
    Object.defineProperty(navigator, 'plugins', {get: () => [1, 2, 3, 4, 5]});
    window.chrome = { runtime: {} };
  `);

  return { status: 'shown' };
});

ipcMain.handle('notebook-hide', async () => {
  if (notebookView) {
    mainWindow.removeBrowserView(notebookView);
  }
  return { status: 'hidden' };
});

function shutdownVibiCore() {
  if (notebookView) {
    notebookView.webContents.destroy();
    notebookView = null;
  }
  // Tell backend to shutdown engine
  fetch('http://localhost:5050/vibicore/shutdown', { method: 'POST' }).catch(() => {});
}

// ===================== WINDOW MANAGEMENT =====================

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1200,
    height: 800,
    minWidth: 800,
    minHeight: 600,
    title: 'Vibi AI',
    backgroundColor: '#212121',
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      preload: path.join(__dirname, 'preload.js'), // Add preload for IPC
      // Set a custom user agent so the web version can detect if desktop app is installed
      // The web version checks for this user agent substring
    },
    frame: true,
  });

  setTimeout(() => {
    // Set a custom user agent that the web version can detect
  // This allows the web app to know if the desktop app is "installed" (running)
  const customUserAgent = 'VibiAI-Desktop/1.0.0 Electron/' + process.versions.electron;
  mainWindow.webContents.setUserAgent(customUserAgent);
  
  mainWindow.loadFile('front_end_ui.html');
  }, 2000);

  mainWindow.on('close', (e) => {
    if (!isQuitting) {
      e.preventDefault();
      cleanupAndQuit();
    }
  });

  // Handle window resize for BrowserView
  mainWindow.on('resize', () => {
    if (notebookView && mainWindow.getBrowserViews().includes(notebookView)) {
      const bounds = mainWindow.getBounds();
      const sidebarWidth = 260;
      const topbarHeight = 70;
      notebookView.setBounds({
        x: sidebarWidth,
        y: topbarHeight,
        width: bounds.width - sidebarWidth,
        height: bounds.height - topbarHeight
      });
    }
  });
}

app.whenReady().then(() => {
  startProxy();
  createWindow();
});

app.on('window-all-closed', () => {
  cleanupAndQuit();
});

app.on('before-quit', () => {
  if (!isQuitting) {
    cleanupAndQuit();
  }
});