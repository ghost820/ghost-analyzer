import { useEffect, useRef } from "react";
import { engine, type LiveFrame } from "../engine/client";

export interface CanvasSurface {
  ctx: CanvasRenderingContext2D;
  width: number;
  height: number;
  dpr: number;
  resized: boolean;
}

export type DrawFn = (surface: CanvasSurface, frame: LiveFrame) => void;

export function useCanvasLoop(draw: DrawFn) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const containerRef = useRef<HTMLDivElement | null>(null);
  const drawRef = useRef(draw);
  drawRef.current = draw;

  useEffect(() => {
    const canvas = canvasRef.current;
    const container = containerRef.current;
    if (!canvas || !container) return;
    const ctx = canvas.getContext("2d", { alpha: false });
    if (!ctx) return;

    let width = 0;
    let height = 0;
    let dpr = window.devicePixelRatio || 1;
    let dirty = true;
    let resized = true;
    let lastSequence = -1;
    let raf = 0;

    const observer = new ResizeObserver((entries) => {
      const box = entries[0].contentRect;
      width = Math.max(1, Math.floor(box.width));
      height = Math.max(1, Math.floor(box.height));
      dpr = window.devicePixelRatio || 1;
      canvas.width = Math.round(width * dpr);
      canvas.height = Math.round(height * dpr);
      canvas.style.width = `${width}px`;
      canvas.style.height = `${height}px`;
      dirty = true;
      resized = true;
    });
    observer.observe(container);

    const tick = () => {
      raf = requestAnimationFrame(tick);
      const frame = engine.latest;
      if (!frame || width === 0) return;
      if (!dirty && frame.sequence === lastSequence) return;
      lastSequence = frame.sequence;
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      drawRef.current({ ctx, width, height, dpr, resized }, frame);
      dirty = false;
      resized = false;
    };
    raf = requestAnimationFrame(tick);

    return () => {
      cancelAnimationFrame(raf);
      observer.disconnect();
    };
  }, []);

  return { canvasRef, containerRef };
}
