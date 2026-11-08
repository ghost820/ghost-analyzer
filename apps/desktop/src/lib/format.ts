export function formatFrequency(hz: number, digits = 3): string {
  const abs = Math.abs(hz);
  if (abs >= 1e9) return `${(hz / 1e9).toFixed(digits)} GHz`;
  if (abs >= 1e6) return `${(hz / 1e6).toFixed(digits)} MHz`;
  if (abs >= 1e3) return `${(hz / 1e3).toFixed(digits)} kHz`;
  return `${hz.toFixed(digits > 1 ? 2 : digits)} Hz`;
}

export function formatAxisFrequency(hz: number): string {
  const abs = Math.abs(hz);
  if (abs >= 1e6) return `${trim(hz / 1e6)}M`;
  if (abs >= 1e3) return `${trim(hz / 1e3)}k`;
  return trim(hz);
}

export function formatTime(seconds: number, digits = 2): string {
  const abs = Math.abs(seconds);
  if (abs === 0) return "0";
  if (abs >= 1) return `${seconds.toFixed(digits)} s`;
  if (abs >= 1e-3) return `${(seconds * 1e3).toFixed(digits)} ms`;
  if (abs >= 1e-6) return `${(seconds * 1e6).toFixed(digits)} µs`;
  return `${(seconds * 1e9).toFixed(digits)} ns`;
}

export function formatAxisTime(seconds: number, span: number): string {
  if (seconds === 0) return "0";
  if (span >= 1) return `${trim(seconds)}s`;
  if (span >= 1e-3) return `${trim(seconds * 1e3)}ms`;
  if (span >= 1e-6) return `${trim(seconds * 1e6)}µs`;
  return `${trim(seconds * 1e9)}ns`;
}

export function formatDb(db: number, digits = 1): string {
  return `${db.toFixed(digits)} dB`;
}

export function amplitudeToDb(amplitude: number): number {
  return amplitude <= 1e-9 ? -Infinity : 20 * Math.log10(amplitude);
}

export function parseFrequency(text: string): number | null {
  const match = /^\s*([+-]?(?:\d+\.?\d*|\.\d+)(?:e[+-]?\d+)?)\s*([kmg]?)\s*(?:hz|sps|s\/s)?\s*$/i.exec(text);
  if (!match) return null;
  const base = Number(match[1]);
  if (!Number.isFinite(base)) return null;
  const scale = { "": 1, k: 1e3, m: 1e6, g: 1e9 }[match[2].toLowerCase() as "" | "k" | "m" | "g"];
  return base * scale;
}

function trim(value: number): string {
  return Number(value.toPrecision(4)).toString();
}
