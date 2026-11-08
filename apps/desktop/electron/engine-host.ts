import type { GhostEngine as GhostEngineType } from "@ghost/bridge";
import type { HostCommand, HostMessage, SignalParams } from "./protocol";

const { GhostEngine } = require("@ghost/bridge") as typeof import("@ghost/bridge");

const BLOCK_SIZE = 2048;
const FRAME_INTERVAL_MS = 1000 / 60;
const MAX_IN_FLIGHT = 2;

const params: SignalParams = { sampleRate: 48_000, frequency: 3_000, amplitude: 0.8 };

let engine: GhostEngineType;
let port: Electron.MessagePortMain | null = null;
let timer: NodeJS.Timeout | null = null;
let sentSequence = 0;
let ackedSequence = 0;

function send(message: HostMessage): void {
  port?.postMessage(message);
}

function applyParams(patch: Partial<SignalParams>): void {
  const next = { ...params, ...patch };
  if (patch.sampleRate !== undefined) {
    engine.setSampleRate(next.sampleRate);
    params.sampleRate = next.sampleRate;
  }
  if (patch.frequency !== undefined) {
    engine.setFrequency(next.frequency);
    params.frequency = next.frequency;
  }
  if (patch.amplitude !== undefined) {
    engine.setAmplitude(next.amplitude);
    params.amplitude = next.amplitude;
  }
}

function tick(): void {
  if (sentSequence - ackedSequence >= MAX_IN_FLIGHT) return;
  const iq = new Float32Array(BLOCK_SIZE * 2);
  const spectrum = new Float32Array(BLOCK_SIZE);
  try {
    const info = engine.processInto(iq, spectrum);
    sentSequence = info.sequence;
    send({
      type: "frame",
      sequence: info.sequence,
      params: { ...params },
      iq,
      spectrum,
      processMicros: info.processMicros,
    });
  } catch (error) {
    send({ type: "error", message: error instanceof Error ? error.message : String(error) });
  }
}

function handle(command: HostCommand): void {
  if (command.type === "ack") {
    ackedSequence = Math.max(ackedSequence, command.sequence);
    return;
  }
  try {
    applyParams(command.params);
  } catch (error) {
    send({ type: "error", message: error instanceof Error ? error.message : String(error) });
  }
}

function attach(newPort: Electron.MessagePortMain): void {
  if (timer) clearInterval(timer);
  port?.close();
  port = newPort;
  sentSequence = 0;
  ackedSequence = 0;
  port.on("message", (event) => handle(event.data as HostCommand));
  port.start();
  send({ type: "ready", params: { ...params }, blockSize: BLOCK_SIZE });
  timer = setInterval(tick, FRAME_INTERVAL_MS);
}

engine = new GhostEngine({ ...params, blockSize: BLOCK_SIZE });

process.parentPort.on("message", (event) => {
  const [newPort] = event.ports;
  if (newPort) attach(newPort);
});
