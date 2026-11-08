import { ipcRenderer } from "electron";

ipcRenderer.on("engine-port", (event) => {
  window.postMessage("engine-port", "*", event.ports);
});
