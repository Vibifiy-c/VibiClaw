const { contextBridge, ipcRenderer } = require('electron');

// Detect if we're running inside the Electron desktop app
const isDesktopApp = true; // This file only runs in Electron, so always true here
const appVersion = require('./package.json').version || '1.0.0';

// Expose safe API to the frontend
contextBridge.exposeInMainWorld('electronAPI', {
  // Desktop detection flag
  isDesktopApp: isDesktopApp,
  appVersion: appVersion,
  platform: process.platform,

  // VibiCore engine controls
  vibicore: {
    open: (url, tabId) => ipcRenderer.invoke('vibicore-open', url, tabId),
    screenshot: (tabId) => ipcRenderer.invoke('vibicore-screenshot', tabId),
    execute: (tabId, script) => ipcRenderer.invoke('vibicore-execute', tabId, script),
    memory: () => ipcRenderer.invoke('vibicore-memory'),
    close: (tabId) => ipcRenderer.invoke('vibicore-close', tabId),
  },

  // Notebook BrowserView controls
  notebook: {
    show: (url) => ipcRenderer.invoke('notebook-show', url),
    hide: () => ipcRenderer.invoke('notebook-hide'),
  },
});