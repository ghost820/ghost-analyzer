import { app, BrowserWindow, MessageChannelMain, utilityProcess } from "electron";
import path from "node:path";

const devUrl = process.env.GHOST_DEV_URL;

let engineProcess: Electron.UtilityProcess | null = null;

function startEngine(win: BrowserWindow): void {
  engineProcess?.kill();
  const child = utilityProcess.fork(path.join(__dirname, "engine-host.js"), [], {
    serviceName: "ghost-engine",
  });
  engineProcess = child;
  child.on("exit", (code) => {
    if (engineProcess === child) engineProcess = null;
    if (code !== 0) console.error(`ghost-engine exited with code ${code}`);
  });
  const { port1, port2 } = new MessageChannelMain();
  child.postMessage({ type: "attach" }, [port1]);
  win.webContents.postMessage("engine-port", null, [port2]);
}

function createWindow(): void {
  const win = new BrowserWindow({
    width: 1480,
    height: 920,
    minWidth: 960,
    minHeight: 600,
    backgroundColor: "#040806",
    title: "Ghost Analyzer",
    autoHideMenuBar: true,
    webPreferences: {
      preload: path.join(__dirname, "preload.js"),
      contextIsolation: true,
      sandbox: true,
      nodeIntegration: false,
    },
  });

  win.webContents.on("did-finish-load", () => startEngine(win));

  if (devUrl) {
    void win.loadURL(devUrl);
  } else {
    void win.loadFile(path.join(__dirname, "..", "dist", "index.html"));
  }
}

app.whenReady().then(() => {
  createWindow();
  app.on("activate", () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});

app.on("before-quit", () => engineProcess?.kill());

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") app.quit();
});
