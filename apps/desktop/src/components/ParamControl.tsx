import { useEffect, useRef, useState, type CSSProperties } from "react";
import { parseFrequency } from "../lib/format";

interface ParamControlProps {
  label: string;
  value: number;
  min: number;
  max: number;
  step: number;
  readout: string;
  inputText: (value: number) => string;
  unit: string;
  onChange: (value: number) => void;
  toSlider?: (value: number) => number;
  fromSlider?: (position: number) => number;
  parse?: (text: string) => number | null;
  hint?: string;
}

export function ParamControl({
  label,
  value,
  min,
  max,
  step,
  readout,
  inputText,
  unit,
  onChange,
  toSlider = (v) => v,
  fromSlider = (p) => p,
  parse = parseFrequency,
  hint,
}: ParamControlProps) {
  const [draft, setDraft] = useState<string | null>(null);
  const [invalid, setInvalid] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    setInvalid(false);
  }, [value]);

  const lo = toSlider(min);
  const hi = toSlider(max);
  const position = toSlider(value);
  const pct = hi === lo ? 0 : Math.min(100, Math.max(0, ((position - lo) / (hi - lo)) * 100));

  const commit = () => {
    if (draft === null) return;
    const parsed = parse(draft);
    if (parsed === null || !Number.isFinite(parsed)) {
      setInvalid(true);
      setDraft(null);
      return;
    }
    setInvalid(false);
    setDraft(null);
    onChange(Math.min(max, Math.max(min, parsed)));
  };

  return (
    <div className="param">
      <div className="param-head">
        <span className="param-label">{label}</span>
        <span className="param-readout">{readout}</span>
      </div>
      <input
        className="slider"
        type="range"
        min={lo}
        max={hi}
        step={step}
        value={position}
        style={{ "--pct": `${pct}%` } as CSSProperties}
        onChange={(event) => onChange(fromSlider(Number(event.target.value)))}
        aria-label={label}
      />
      <div className="param-foot">
        <label className={`field${invalid ? " invalid" : ""}`}>
          <input
            ref={inputRef}
            className="field-input"
            spellCheck={false}
            value={draft ?? inputText(value)}
            onFocus={(event) => {
              setDraft(inputText(value));
              event.currentTarget.select();
            }}
            onChange={(event) => setDraft(event.target.value)}
            onBlur={commit}
            onKeyDown={(event) => {
              if (event.key === "Enter") inputRef.current?.blur();
              if (event.key === "Escape") {
                setDraft(null);
                inputRef.current?.blur();
              }
            }}
          />
          <span className="field-unit">{unit}</span>
        </label>
        {hint && <span className="param-hint">{hint}</span>}
      </div>
    </div>
  );
}
