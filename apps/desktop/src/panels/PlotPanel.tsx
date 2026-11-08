import { useMemo } from "react";
import { engine } from "../engine/client";
import { useCanvasLoop, type DrawFn } from "../lib/useCanvasLoop";
import { createScope } from "../plots/scope";
import { createSpectrum } from "../plots/spectrum";
import { useEngineStatus } from "../lib/useEngineStatus";

function Plot({ draw }: { draw: DrawFn }) {
  const { canvasRef, containerRef } = useCanvasLoop(draw);
  const { connected } = useEngineStatus();
  return (
    <div className="plot" ref={containerRef}>
      <canvas ref={canvasRef} />
      {!connected && !engine.latest && <div className="plot-wait">AWAITING ENGINE LINK…</div>}
    </div>
  );
}

export function ScopePanel() {
  const draw = useMemo(() => createScope(), []);
  return <Plot draw={draw} />;
}

export function SpectrumPanel() {
  const draw = useMemo(() => createSpectrum(), []);
  return <Plot draw={draw} />;
}
