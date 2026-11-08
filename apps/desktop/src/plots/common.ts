export const COLORS = {
  bg: "#040a07",
  plotBg: "#050d09",
  grid: "rgba(61, 255, 139, 0.075)",
  gridMajor: "rgba(61, 255, 139, 0.17)",
  axis: "rgba(61, 255, 139, 0.45)",
  label: "#5f8f76",
  labelStrong: "#9fd9b6",
  green: "#3dff8b",
  cyan: "#4de3ff",
  amber: "#ffc857",
} as const;

export const FONT_MONO = '"JetBrains Mono Variable", ui-monospace, monospace';

export interface PlotRect {
  left: number;
  top: number;
  width: number;
  height: number;
}

export function plotRect(width: number, height: number): PlotRect {
  const left = 58;
  const right = 18;
  const top = 16;
  const bottom = 32;
  return {
    left,
    top,
    width: Math.max(10, width - left - right),
    height: Math.max(10, height - top - bottom),
  };
}

export class CachedLayer {
  private canvas = document.createElement("canvas");
  private key = "";

  render(
    target: CanvasRenderingContext2D,
    key: string,
    width: number,
    height: number,
    dpr: number,
    paint: (ctx: CanvasRenderingContext2D) => void,
  ): void {
    const pw = Math.round(width * dpr);
    const ph = Math.round(height * dpr);
    const fullKey = `${key}|${pw}x${ph}`;
    if (fullKey !== this.key) {
      this.canvas.width = pw;
      this.canvas.height = ph;
      const ctx = this.canvas.getContext("2d");
      if (!ctx) return;
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      paint(ctx);
      this.key = fullKey;
    }
    target.save();
    target.setTransform(1, 0, 0, 1, 0, 0);
    target.drawImage(this.canvas, 0, 0);
    target.restore();
  }
}

export function niceStep(range: number, targetTicks: number): number {
  const rough = range / targetTicks;
  const pow = Math.pow(10, Math.floor(Math.log10(rough)));
  const norm = rough / pow;
  const nice = norm < 1.5 ? 1 : norm < 3.5 ? 2 : norm < 7.5 ? 5 : 10;
  return nice * pow;
}

export function drawCornerBrackets(ctx: CanvasRenderingContext2D, r: PlotRect): void {
  const len = 10;
  ctx.save();
  ctx.strokeStyle = COLORS.green;
  ctx.lineWidth = 1.5;
  ctx.globalAlpha = 0.9;
  ctx.beginPath();
  const x0 = r.left - 0.5;
  const y0 = r.top - 0.5;
  const x1 = r.left + r.width + 0.5;
  const y1 = r.top + r.height + 0.5;
  ctx.moveTo(x0, y0 + len);
  ctx.lineTo(x0, y0);
  ctx.lineTo(x0 + len, y0);
  ctx.moveTo(x1 - len, y0);
  ctx.lineTo(x1, y0);
  ctx.lineTo(x1, y0 + len);
  ctx.moveTo(x1, y1 - len);
  ctx.lineTo(x1, y1);
  ctx.lineTo(x1 - len, y1);
  ctx.moveTo(x0 + len, y1);
  ctx.lineTo(x0, y1);
  ctx.lineTo(x0, y1 - len);
  ctx.stroke();
  ctx.restore();
}

export function pill(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  text: string,
  color: string,
  align: "left" | "right" = "left",
): number {
  ctx.font = `500 11px ${FONT_MONO}`;
  const padX = 8;
  const w = ctx.measureText(text).width + padX * 2;
  const h = 20;
  const left = align === "left" ? x : x - w;
  ctx.fillStyle = "rgba(4, 12, 8, 0.82)";
  ctx.strokeStyle = "rgba(61, 255, 139, 0.28)";
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.roundRect(left + 0.5, y + 0.5, w, h, 3);
  ctx.fill();
  ctx.stroke();
  ctx.fillStyle = color;
  ctx.textAlign = "left";
  ctx.textBaseline = "middle";
  ctx.fillText(text, left + padX, y + h / 2 + 0.5);
  return w;
}
