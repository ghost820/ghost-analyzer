import type { LiveFrame } from "../engine/client";
import { formatAxisTime, formatTime } from "../lib/format";
import type { CanvasSurface } from "../lib/useCanvasLoop";
import { CachedLayer, COLORS, drawCornerBrackets, FONT_MONO, pill, plotRect, type PlotRect } from "./common";

const CYCLES_SHOWN = 4;
const MIN_WINDOW = 48;
const MAX_WINDOW = 1024;
const Y_RANGE = 1.2;
const X_DIVISIONS = 8;
const Y_TICKS = [-1, -0.5, 0, 0.5, 1];

interface Trace {
  start: number;
  window: number;
  triggered: boolean;
}

export function createScope() {
  const gridLayer = new CachedLayer();

  return function drawScope(surface: CanvasSurface, frame: LiveFrame): void {
    const { ctx, width, height, dpr } = surface;
    const rect = plotRect(width, height);
    const { sampleRate, frequency } = frame.params;
    const trace = locateTrace(frame.iq, sampleRate, frequency);
    const span = trace.window / sampleRate;

    ctx.fillStyle = COLORS.bg;
    ctx.fillRect(0, 0, width, height);

    gridLayer.render(ctx, `${span}`, width, height, dpr, (g) => paintGrid(g, width, height, rect, span));

    ctx.save();
    ctx.beginPath();
    ctx.rect(rect.left, rect.top, rect.width, rect.height);
    ctx.clip();
    const sampleWidth = rect.width / trace.window;
    ctx.lineJoin = "round";
    ctx.lineCap = "round";
    plotChannel(ctx, frame.iq, 0, trace, rect, COLORS.cyan);
    plotChannel(ctx, frame.iq, 1, trace, rect, COLORS.green);
    if (sampleWidth > 7) {
      markSamples(ctx, frame.iq, 0, trace, rect, COLORS.cyan);
      markSamples(ctx, frame.iq, 1, trace, rect, COLORS.green);
    }
    ctx.restore();

    drawCornerBrackets(ctx, rect);
    drawReadouts(ctx, rect, frame, trace, span);
  };
}

function locateTrace(iq: Float32Array, sampleRate: number, frequency: number): Trace {
  const blockSize = iq.length / 2;
  const samplesPerCycle = frequency === 0 ? Infinity : sampleRate / Math.abs(frequency);
  const window = Math.round(
    Math.min(MAX_WINDOW, Math.max(MIN_WINDOW, Number.isFinite(samplesPerCycle) ? samplesPerCycle * CYCLES_SHOWN : MAX_WINDOW)),
  );
  const limit = blockSize - window - 2;
  for (let i = 1; i < limit; i += 1) {
    const prev = iq[2 * (i - 1)];
    const next = iq[2 * i];
    if (prev < 0 && next >= 0) {
      return { start: i - 1 + -prev / (next - prev), window, triggered: true };
    }
  }
  return { start: 0, window, triggered: false };
}

function plotChannel(
  ctx: CanvasRenderingContext2D,
  iq: Float32Array,
  channel: 0 | 1,
  trace: Trace,
  rect: PlotRect,
  color: string,
): void {
  const first = Math.max(0, Math.floor(trace.start));
  const last = Math.min(iq.length / 2 - 1, Math.ceil(trace.start + trace.window) + 1);
  const xScale = rect.width / trace.window;
  const yMid = rect.top + rect.height / 2;
  const yScale = rect.height / (2 * Y_RANGE);

  ctx.beginPath();
  for (let k = first; k <= last; k += 1) {
    const x = rect.left + (k - trace.start) * xScale;
    const y = yMid - iq[2 * k + channel] * yScale;
    if (k === first) ctx.moveTo(x, y);
    else ctx.lineTo(x, y);
  }
  ctx.strokeStyle = color;
  ctx.globalAlpha = 0.14;
  ctx.lineWidth = 6;
  ctx.stroke();
  ctx.globalAlpha = 0.28;
  ctx.lineWidth = 3;
  ctx.stroke();
  ctx.globalAlpha = 1;
  ctx.lineWidth = 1.4;
  ctx.stroke();
}

function markSamples(
  ctx: CanvasRenderingContext2D,
  iq: Float32Array,
  channel: 0 | 1,
  trace: Trace,
  rect: PlotRect,
  color: string,
): void {
  const first = Math.max(0, Math.ceil(trace.start));
  const last = Math.min(iq.length / 2 - 1, Math.floor(trace.start + trace.window));
  const xScale = rect.width / trace.window;
  const yMid = rect.top + rect.height / 2;
  const yScale = rect.height / (2 * Y_RANGE);
  ctx.fillStyle = color;
  for (let k = first; k <= last; k += 1) {
    const x = rect.left + (k - trace.start) * xScale;
    const y = yMid - iq[2 * k + channel] * yScale;
    ctx.fillRect(x - 1.5, y - 1.5, 3, 3);
  }
}

function paintGrid(ctx: CanvasRenderingContext2D, width: number, height: number, rect: PlotRect, span: number): void {
  ctx.fillStyle = COLORS.bg;
  ctx.fillRect(0, 0, width, height);
  const bg = ctx.createLinearGradient(0, rect.top, 0, rect.top + rect.height);
  bg.addColorStop(0, "#071410");
  bg.addColorStop(0.5, "#06100c");
  bg.addColorStop(1, "#050d09");
  ctx.fillStyle = bg;
  ctx.fillRect(rect.left, rect.top, rect.width, rect.height);

  const yMid = rect.top + rect.height / 2;
  const yScale = rect.height / (2 * Y_RANGE);

  ctx.lineWidth = 1;
  for (let i = 0; i <= X_DIVISIONS; i += 1) {
    const x = Math.round(rect.left + (rect.width * i) / X_DIVISIONS) + 0.5;
    ctx.strokeStyle = i === 0 || i === X_DIVISIONS ? COLORS.gridMajor : COLORS.grid;
    ctx.beginPath();
    ctx.moveTo(x, rect.top);
    ctx.lineTo(x, rect.top + rect.height);
    ctx.stroke();
    for (let m = 1; m < 5 && i < X_DIVISIONS; m += 1) {
      const mx = Math.round(rect.left + (rect.width * (i + m / 5)) / X_DIVISIONS) + 0.5;
      ctx.strokeStyle = "rgba(61, 255, 139, 0.9)";
      ctx.globalAlpha = 0.18;
      ctx.beginPath();
      ctx.moveTo(mx, yMid - 3);
      ctx.lineTo(mx, yMid + 3);
      ctx.stroke();
      ctx.globalAlpha = 1;
    }
  }

  for (const tick of Y_TICKS) {
    const y = Math.round(yMid - tick * yScale) + 0.5;
    ctx.strokeStyle = tick === 0 ? COLORS.gridMajor : COLORS.grid;
    ctx.beginPath();
    ctx.moveTo(rect.left, y);
    ctx.lineTo(rect.left + rect.width, y);
    ctx.stroke();
    ctx.fillStyle = tick === 0 ? COLORS.labelStrong : COLORS.label;
    ctx.font = `500 10.5px ${FONT_MONO}`;
    ctx.textAlign = "right";
    ctx.textBaseline = "middle";
    ctx.fillText(tick > 0 ? `+${tick.toFixed(1)}` : tick.toFixed(1), rect.left - 10, y);
  }

  ctx.textAlign = "center";
  ctx.textBaseline = "top";
  ctx.font = `500 10.5px ${FONT_MONO}`;
  ctx.fillStyle = COLORS.label;
  for (let i = 0; i <= X_DIVISIONS; i += 1) {
    const x = rect.left + (rect.width * i) / X_DIVISIONS;
    ctx.textAlign = i === X_DIVISIONS ? "right" : "center";
    ctx.fillText(formatAxisTime((span * i) / X_DIVISIONS, span), x, rect.top + rect.height + 10);
  }

  ctx.save();
  ctx.translate(14, rect.top + rect.height / 2);
  ctx.rotate(-Math.PI / 2);
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillStyle = COLORS.label;
  ctx.font = `600 9.5px ${FONT_MONO}`;
  ctx.fillText("AMPLITUDE · FS", 0, 0);
  ctx.restore();

  ctx.strokeStyle = COLORS.axis;
  ctx.lineWidth = 1;
  ctx.strokeRect(rect.left - 0.5, rect.top - 0.5, rect.width + 1, rect.height + 1);
}

function drawReadouts(
  ctx: CanvasRenderingContext2D,
  rect: PlotRect,
  frame: LiveFrame,
  trace: Trace,
  span: number,
): void {
  const { frequency } = frame.params;
  const y = rect.top + 8;
  let x = rect.left + 12;
  x += pill(ctx, x, y, "I  REAL", COLORS.cyan) + 6;
  x += pill(ctx, x, y, "Q  IMAG", COLORS.green) + 6;
  if (!trace.triggered) pill(ctx, x, y, "FREE RUN", COLORS.amber);

  const right = rect.left + rect.width - 12;
  const period = frequency === 0 ? "∞" : formatTime(1 / Math.abs(frequency), 3);
  const w = pill(ctx, right, y, `PERIOD ${period}`, COLORS.labelStrong, "right");
  pill(ctx, right - w - 6, y, `${formatTime(span / X_DIVISIONS, 3)}/DIV`, COLORS.labelStrong, "right");
}
