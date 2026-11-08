import type { LiveFrame } from "../engine/client";
import { formatAxisFrequency, formatFrequency } from "../lib/format";
import type { CanvasSurface } from "../lib/useCanvasLoop";
import { CachedLayer, COLORS, drawCornerBrackets, FONT_MONO, niceStep, pill, plotRect, type PlotRect } from "./common";

const DB_MIN = -160;
const DB_MAX = 10;
const HANN_ENBW = 1.5;

export function createSpectrum() {
  const gridLayer = new CachedLayer();
  let fill: CanvasGradient | null = null;
  let fillKey = "";

  return function drawSpectrum(surface: CanvasSurface, frame: LiveFrame): void {
    const { ctx, width, height, dpr } = surface;
    const rect = plotRect(width, height);
    const { sampleRate } = frame.params;
    const bins = frame.spectrum;

    ctx.fillStyle = COLORS.bg;
    ctx.fillRect(0, 0, width, height);
    gridLayer.render(ctx, `${sampleRate}`, width, height, dpr, (g) => paintGrid(g, width, height, rect, sampleRate));

    const key = `${rect.top}|${rect.height}`;
    if (key !== fillKey) {
      fill = ctx.createLinearGradient(0, rect.top, 0, rect.top + rect.height);
      fill.addColorStop(0, "rgba(61, 255, 139, 0.42)");
      fill.addColorStop(0.55, "rgba(61, 255, 139, 0.12)");
      fill.addColorStop(1, "rgba(61, 255, 139, 0.01)");
      fillKey = key;
    }

    ctx.save();
    ctx.beginPath();
    ctx.rect(rect.left, rect.top, rect.width, rect.height);
    ctx.clip();
    const columns = columnMaxima(bins, Math.floor(rect.width));
    const yOf = (db: number) => rect.top + ((DB_MAX - db) / (DB_MAX - DB_MIN)) * rect.height;
    const base = rect.top + rect.height;

    ctx.beginPath();
    ctx.moveTo(rect.left, base);
    for (let i = 0; i < columns.length; i += 1) {
      ctx.lineTo(rect.left + (i / (columns.length - 1)) * rect.width, yOf(columns[i]));
    }
    ctx.lineTo(rect.left + rect.width, base);
    ctx.closePath();
    ctx.fillStyle = fill ?? COLORS.green;
    ctx.fill();

    ctx.beginPath();
    for (let i = 0; i < columns.length; i += 1) {
      const x = rect.left + (i / (columns.length - 1)) * rect.width;
      const y = yOf(columns[i]);
      if (i === 0) ctx.moveTo(x, y);
      else ctx.lineTo(x, y);
    }
    ctx.lineJoin = "round";
    ctx.strokeStyle = COLORS.green;
    ctx.globalAlpha = 0.16;
    ctx.lineWidth = 6;
    ctx.stroke();
    ctx.globalAlpha = 0.3;
    ctx.lineWidth = 3;
    ctx.stroke();
    ctx.globalAlpha = 1;
    ctx.lineWidth = 1.3;
    ctx.stroke();
    ctx.restore();

    const peak = findPeak(bins, sampleRate);
    drawPeak(ctx, rect, peak, sampleRate, yOf);
    drawCornerBrackets(ctx, rect);
    drawReadouts(ctx, rect, peak, sampleRate, bins.length);
  };
}

function columnMaxima(bins: Float32Array, columns: number): Float32Array {
  const out = new Float32Array(Math.max(2, columns));
  const n = bins.length;
  const last = out.length - 1;
  for (let c = 0; c <= last; c += 1) {
    const lo = Math.floor(((c - 0.5) / last) * (n - 1));
    const hi = Math.ceil(((c + 0.5) / last) * (n - 1));
    let max = -Infinity;
    for (let i = Math.max(0, lo); i <= Math.min(n - 1, hi); i += 1) {
      if (bins[i] > max) max = bins[i];
    }
    out[c] = max;
  }
  return out;
}

interface Peak {
  frequency: number;
  level: number;
}

function findPeak(bins: Float32Array, sampleRate: number): Peak {
  let index = 0;
  for (let i = 1; i < bins.length; i += 1) {
    if (bins[i] > bins[index]) index = i;
  }
  let offset = 0;
  let level = bins[index];
  if (index > 0 && index < bins.length - 1) {
    const l = bins[index - 1];
    const r = bins[index + 1];
    const denom = l - 2 * level + r;
    if (denom < 0) {
      offset = (0.5 * (l - r)) / denom;
      level = level - 0.25 * (l - r) * offset;
    }
  }
  const binWidth = sampleRate / bins.length;
  return { frequency: (index + offset - bins.length / 2) * binWidth, level };
}

function drawPeak(
  ctx: CanvasRenderingContext2D,
  rect: PlotRect,
  peak: Peak,
  sampleRate: number,
  yOf: (db: number) => number,
): void {
  if (peak.level <= DB_MIN) return;
  const x = rect.left + (peak.frequency / sampleRate + 0.5) * rect.width;
  const y = yOf(peak.level);

  ctx.save();
  ctx.beginPath();
  ctx.rect(rect.left, rect.top, rect.width, rect.height);
  ctx.clip();
  ctx.strokeStyle = COLORS.amber;
  ctx.globalAlpha = 0.5;
  ctx.setLineDash([3, 4]);
  ctx.beginPath();
  ctx.moveTo(Math.round(x) + 0.5, Math.max(rect.top, y + 8));
  ctx.lineTo(Math.round(x) + 0.5, rect.top + rect.height);
  ctx.stroke();
  ctx.setLineDash([]);
  ctx.globalAlpha = 1;
  ctx.fillStyle = COLORS.amber;
  ctx.beginPath();
  ctx.moveTo(x, y - 3);
  ctx.lineTo(x - 5, y - 11);
  ctx.lineTo(x + 5, y - 11);
  ctx.closePath();
  ctx.fill();
  ctx.restore();

  const text = `${formatFrequency(peak.frequency)}  ${peak.level.toFixed(1)} dBFS`;
  ctx.font = `500 11px ${FONT_MONO}`;
  const w = ctx.measureText(text).width + 16;
  const flip = x + 12 + w > rect.left + rect.width;
  const px = flip ? x - 12 : x + 12;
  const py = Math.min(rect.top + rect.height - 26, Math.max(rect.top + 36, y - 10));
  pill(ctx, px, py, text, COLORS.amber, flip ? "right" : "left");
}

function paintGrid(ctx: CanvasRenderingContext2D, width: number, height: number, rect: PlotRect, sampleRate: number): void {
  ctx.fillStyle = COLORS.bg;
  ctx.fillRect(0, 0, width, height);
  const bg = ctx.createLinearGradient(0, rect.top, 0, rect.top + rect.height);
  bg.addColorStop(0, "#071410");
  bg.addColorStop(1, "#050d09");
  ctx.fillStyle = bg;
  ctx.fillRect(rect.left, rect.top, rect.width, rect.height);

  ctx.lineWidth = 1;
  ctx.font = `500 10.5px ${FONT_MONO}`;

  for (let db = Math.ceil(DB_MIN / 20) * 20; db <= DB_MAX; db += 20) {
    const y = Math.round(rect.top + ((DB_MAX - db) / (DB_MAX - DB_MIN)) * rect.height) + 0.5;
    ctx.strokeStyle = db === 0 ? COLORS.gridMajor : COLORS.grid;
    ctx.beginPath();
    ctx.moveTo(rect.left, y);
    ctx.lineTo(rect.left + rect.width, y);
    ctx.stroke();
    ctx.fillStyle = db === 0 ? COLORS.labelStrong : COLORS.label;
    ctx.textAlign = "right";
    ctx.textBaseline = "middle";
    ctx.fillText(`${db}`, rect.left - 10, y);
  }

  const span = sampleRate;
  const step = niceStep(span, Math.max(4, Math.floor(rect.width / 90)));
  const start = Math.ceil(-span / 2 / step) * step;
  ctx.textAlign = "center";
  ctx.textBaseline = "top";
  for (let f = start; f <= span / 2 + step * 1e-6; f += step) {
    const snapped = Math.abs(f) < step * 1e-6 ? 0 : f;
    const x = Math.round(rect.left + (snapped / span + 0.5) * rect.width) + 0.5;
    ctx.strokeStyle = snapped === 0 ? COLORS.gridMajor : COLORS.grid;
    ctx.beginPath();
    ctx.moveTo(x, rect.top);
    ctx.lineTo(x, rect.top + rect.height);
    ctx.stroke();
    ctx.fillStyle = snapped === 0 ? COLORS.labelStrong : COLORS.label;
    ctx.textAlign = x > rect.left + rect.width - 14 ? "right" : "center";
    ctx.fillText(formatAxisFrequency(snapped), x, rect.top + rect.height + 10);
  }

  ctx.save();
  ctx.translate(14, rect.top + rect.height / 2);
  ctx.rotate(-Math.PI / 2);
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillStyle = COLORS.label;
  ctx.font = `600 9.5px ${FONT_MONO}`;
  ctx.fillText("POWER · dBFS", 0, 0);
  ctx.restore();

  ctx.strokeStyle = COLORS.axis;
  ctx.strokeRect(rect.left - 0.5, rect.top - 0.5, rect.width + 1, rect.height + 1);
}

function drawReadouts(
  ctx: CanvasRenderingContext2D,
  rect: PlotRect,
  peak: Peak,
  sampleRate: number,
  size: number,
): void {
  const y = rect.top + 8;
  const binWidth = sampleRate / size;
  let x = rect.left + 12;
  x += pill(ctx, x, y, `FFT ${size}`, COLORS.labelStrong) + 6;
  x += pill(ctx, x, y, "HANN", COLORS.labelStrong) + 6;
  pill(ctx, x, y, `RBW ${formatFrequency(binWidth * HANN_ENBW, 2)}`, COLORS.labelStrong);
  const right = rect.left + rect.width - 12;
  const w = pill(ctx, right, y, `SPAN ${formatFrequency(sampleRate, 2)}`, COLORS.labelStrong, "right");
  pill(ctx, right - w - 6, y, `PK ${formatFrequency(peak.frequency)}`, COLORS.amber, "right");
}
