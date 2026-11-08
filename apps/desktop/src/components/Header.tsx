import { useEngineStatus } from "../lib/useEngineStatus";

export function Header() {
  const { connected, fps, processMicros, blockSize, frames, error } = useEngineStatus();
  const state = error ? "fault" : connected ? "live" : "idle";
  return (
    <header className="header">
      <div className="brand">
        <svg className="brand-mark" viewBox="0 0 32 32" aria-hidden="true">
          <path
            d="M6 27V14a10 10 0 0 1 20 0v13l-3.4-3-3.3 3-3.3-3-3.3 3-3.3-3z"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.8"
            strokeLinejoin="round"
          />
          <path
            d="M9.5 15.5h2l1.5-3 2 6 2-8 1.5 5h3.5"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.5"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        </svg>
        <span className="brand-name">GHOST<span className="brand-sep">/</span>ANALYZER</span>
        <span className="brand-tag">M1 · SIGNAL LAB</span>
      </div>
      <div className="status">
        <div className={`led ${state}`}>
          <span className="led-dot" />
          <span>{error ? "FAULT" : connected ? "ENGINE LIVE" : "CONNECTING"}</span>
        </div>
        <Stat label="FRAME RATE" value={fps.toFixed(0)} unit="fps" />
        <Stat label="DSP / BLOCK" value={processMicros.toFixed(0)} unit="µs" />
        <Stat label="BLOCK" value={String(blockSize)} unit="smp" />
        <Stat label="BLOCKS" value={frames.toLocaleString("en-US")} unit="" />
      </div>
      {error && <div className="fault-banner">{error}</div>}
    </header>
  );
}

function Stat({ label, value, unit }: { label: string; value: string; unit: string }) {
  return (
    <div className="stat">
      <span className="stat-label">{label}</span>
      <span className="stat-value">
        {value}
        {unit && <span className="stat-unit">{unit}</span>}
      </span>
    </div>
  );
}
