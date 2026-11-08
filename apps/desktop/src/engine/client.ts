import type { HostCommand, HostMessage, SignalParams } from "../../electron/protocol";

export type { SignalParams };

export interface LiveFrame {
  sequence: number;
  params: SignalParams;
  iq: Float32Array;
  spectrum: Float32Array;
  processMicros: number;
}

export interface EngineStatus {
  connected: boolean;
  params: SignalParams;
  blockSize: number;
  fps: number;
  processMicros: number;
  frames: number;
  error: string | null;
}

export const SAMPLE_RATE_MIN = 1_000;
export const SAMPLE_RATE_MAX = 20_000_000;

const INITIAL_PARAMS: SignalParams = { sampleRate: 48_000, frequency: 3_000, amplitude: 0.8 };

class EngineClient {
  latest: LiveFrame | null = null;

  private port: MessagePort | null = null;
  private listeners = new Set<() => void>();
  private status: EngineStatus = {
    connected: false,
    params: INITIAL_PARAMS,
    blockSize: 2048,
    fps: 0,
    processMicros: 0,
    frames: 0,
    error: null,
  };
  private framesSinceTick = 0;
  private lastTick = performance.now();
  private smoothedMicros = 0;
  private totalFrames = 0;

  constructor() {
    window.addEventListener("message", (event) => {
      if (event.source === window && event.data === "engine-port" && event.ports[0]) {
        this.attach(event.ports[0]);
      }
    });
    setInterval(() => this.publishStats(), 500);
  }

  subscribe = (listener: () => void): (() => void) => {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  };

  getStatus = (): EngineStatus => this.status;

  setParams(patch: Partial<SignalParams>): void {
    const current = this.status.params;
    const sampleRate = clamp(patch.sampleRate ?? current.sampleRate, SAMPLE_RATE_MIN, SAMPLE_RATE_MAX);
    const nyquist = sampleRate / 2;
    const frequency = clamp(patch.frequency ?? current.frequency, -nyquist, nyquist);
    const amplitude = clamp(patch.amplitude ?? current.amplitude, 0, 1);
    const next: SignalParams = { sampleRate, frequency, amplitude };

    const diff: Partial<SignalParams> = {};
    if (next.sampleRate !== current.sampleRate) diff.sampleRate = next.sampleRate;
    if (next.frequency !== current.frequency) diff.frequency = next.frequency;
    if (next.amplitude !== current.amplitude) diff.amplitude = next.amplitude;
    if (Object.keys(diff).length === 0) return;

    this.update({ params: next });
    this.send({ type: "set", params: diff });
  }

  private attach(port: MessagePort): void {
    this.port?.close();
    this.port = port;
    this.latest = null;
    port.onmessage = (event: MessageEvent<HostMessage>) => this.receive(event.data);
    port.start();
  }

  private receive(message: HostMessage): void {
    switch (message.type) {
      case "ready":
        this.update({ connected: true, params: message.params, blockSize: message.blockSize, error: null });
        this.send({ type: "set", params: message.params });
        break;
      case "frame":
        this.latest = message;
        this.framesSinceTick += 1;
        this.totalFrames += 1;
        this.smoothedMicros =
          this.smoothedMicros === 0
            ? message.processMicros
            : this.smoothedMicros * 0.9 + message.processMicros * 0.1;
        this.send({ type: "ack", sequence: message.sequence });
        break;
      case "error":
        this.update({ error: message.message });
        break;
    }
  }

  private send(command: HostCommand): void {
    this.port?.postMessage(command);
  }

  private publishStats(): void {
    const now = performance.now();
    const fps = (this.framesSinceTick * 1000) / (now - this.lastTick);
    this.framesSinceTick = 0;
    this.lastTick = now;
    this.update({ fps, processMicros: this.smoothedMicros, frames: this.totalFrames });
  }

  private update(patch: Partial<EngineStatus>): void {
    this.status = { ...this.status, ...patch };
    this.listeners.forEach((listener) => listener());
  }
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

export const engine = new EngineClient();
