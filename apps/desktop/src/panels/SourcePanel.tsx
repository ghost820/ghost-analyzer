import { engine, SAMPLE_RATE_MAX, SAMPLE_RATE_MIN } from "../engine/client";
import { ParamControl } from "../components/ParamControl";
import { amplitudeToDb, formatFrequency } from "../lib/format";
import { useEngineStatus } from "../lib/useEngineStatus";

const SAMPLE_RATE_PRESETS = [8_000, 44_100, 48_000, 96_000, 192_000, 1_000_000, 2_400_000, 10_000_000];

const logMin = Math.log10(SAMPLE_RATE_MIN);
const logMax = Math.log10(SAMPLE_RATE_MAX);

function presetLabel(hz: number): string {
  return hz >= 1e6 ? `${hz / 1e6}M` : `${hz / 1e3}k`;
}

export function SourcePanel() {
  const { params, blockSize } = useEngineStatus();
  const { sampleRate, frequency, amplitude } = params;
  const nyquist = sampleRate / 2;
  const db = amplitudeToDb(amplitude);
  const samplesPerCycle = frequency === 0 ? Infinity : sampleRate / Math.abs(frequency);
  const phaseStep = (2 * Math.PI * frequency) / sampleRate;

  return (
    <div className="source">
      <div className="source-title">
        <span className="glyph">∿</span>
        <span>COMPLEX TONE · e<sup>jωn</sup></span>
      </div>

      <ParamControl
        label="FREQUENCY"
        value={frequency}
        min={-nyquist}
        max={nyquist}
        step={nyquist / 5000}
        readout={formatFrequency(frequency)}
        inputText={(v) => String(Number(v.toFixed(3)))}
        unit="Hz"
        onChange={(v) => engine.setParams({ frequency: Math.round(v * 100) / 100 })}
        hint={`±${formatFrequency(nyquist, 1)}`}
      />

      <ParamControl
        label="AMPLITUDE"
        value={amplitude}
        min={0}
        max={1}
        step={0.001}
        readout={amplitude.toFixed(3)}
        inputText={(v) => v.toFixed(3)}
        unit="FS"
        onChange={(v) => engine.setParams({ amplitude: v })}
        parse={(text) => {
          const n = Number(text);
          return text.trim() !== "" && Number.isFinite(n) ? n : null;
        }}
        hint={Number.isFinite(db) ? `${db.toFixed(2)} dBFS` : "−∞ dBFS"}
      />

      <ParamControl
        label="SAMPLE RATE"
        value={sampleRate}
        min={SAMPLE_RATE_MIN}
        max={SAMPLE_RATE_MAX}
        step={0.001}
        toSlider={(v) => Math.log10(v)}
        fromSlider={(p) => Math.round(Math.pow(10, Math.min(logMax, Math.max(logMin, p))))}
        readout={formatFrequency(sampleRate, sampleRate % 1000 === 0 ? 1 : 3).replace("Hz", "S/s")}
        inputText={(v) => String(Math.round(v))}
        unit="S/s"
        onChange={(v) => engine.setParams({ sampleRate: Math.round(v) })}
        hint="log scale"
      />

      <div className="chips" role="group" aria-label="Sample rate presets">
        {SAMPLE_RATE_PRESETS.map((preset) => (
          <button
            key={preset}
            className={`chip${preset === sampleRate ? " active" : ""}`}
            onClick={() => engine.setParams({ sampleRate: preset })}
          >
            {presetLabel(preset)}
          </button>
        ))}
      </div>

      <dl className="telemetry">
        <div>
          <dt>NYQUIST</dt>
          <dd>{formatFrequency(nyquist, 2)}</dd>
        </div>
        <div>
          <dt>BIN WIDTH</dt>
          <dd>{formatFrequency(sampleRate / blockSize, 2)}</dd>
        </div>
        <div>
          <dt>Δφ / SAMPLE</dt>
          <dd>{phaseStep.toFixed(4)} rad</dd>
        </div>
        <div>
          <dt>SAMPLES / CYCLE</dt>
          <dd>{Number.isFinite(samplesPerCycle) ? samplesPerCycle.toFixed(2) : "∞"}</dd>
        </div>
      </dl>
    </div>
  );
}
