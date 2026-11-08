export interface SignalParams {
  sampleRate: number;
  frequency: number;
  amplitude: number;
}

export type HostCommand =
  | { type: "set"; params: Partial<SignalParams> }
  | { type: "ack"; sequence: number };

export type HostMessage =
  | {
      type: "frame";
      sequence: number;
      params: SignalParams;
      iq: Float32Array;
      spectrum: Float32Array;
      processMicros: number;
    }
  | { type: "ready"; params: SignalParams; blockSize: number }
  | { type: "error"; message: string };
