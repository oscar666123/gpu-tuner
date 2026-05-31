import { useEffect, useMemo, useState } from "react";
import ConfirmDialog from "../components/ConfirmDialog";
import { api } from "../lib/api";
import type { GPUInfo, GpuCapabilities, TuningCommand } from "../lib/types";
import type { ToastState } from "../components/Toast";

interface TuningProps {
  gpuIndex: number;
  gpus: GPUInfo[];
  onGpuChange: (index: number) => void;
  onNotify: (toast: ToastState) => void;
}

export default function Tuning({ gpuIndex, gpus, onGpuChange, onNotify }: TuningProps) {
  const [capabilities, setCapabilities] = useState<GpuCapabilities | null>(null);
  const [powerLimit, setPowerLimit] = useState(0);
  const [coreClock, setCoreClock] = useState(0);
  const [memoryClock, setMemoryClock] = useState(0);
  const [coreOffset, setCoreOffset] = useState(0);
  const [memoryOffset, setMemoryOffset] = useState(0);
  const [powerLimitEnabled, setPowerLimitEnabled] = useState(false);
  const [coreClockEnabled, setCoreClockEnabled] = useState(false);
  const [memoryClockEnabled, setMemoryClockEnabled] = useState(false);
  const [coreOffsetEnabled, setCoreOffsetEnabled] = useState(false);
  const [memoryOffsetEnabled, setMemoryOffsetEnabled] = useState(false);
  const [loading, setLoading] = useState(false);
  const [confirmOpen, setConfirmOpen] = useState(false);

  useEffect(() => {
    let mounted = true;
    if (gpus.length === 0) return;
    setLoading(true);
    Promise.all([api.getCapabilities(gpuIndex), api.getTelemetry(gpuIndex)])
      .then(([caps, telemetry]) => {
        if (!mounted) return;
        setCapabilities(caps);
        setPowerLimit(Math.round(telemetry.power_limit_watts));
        setCoreClock(telemetry.core_clock_mhz);
        setMemoryClock(telemetry.memory_clock_mhz);
        setCoreOffset(0);
        setMemoryOffset(0);
        setPowerLimitEnabled(false);
        setCoreClockEnabled(false);
        setMemoryClockEnabled(false);
        setCoreOffsetEnabled(false);
        setMemoryOffsetEnabled(false);
      })
      .catch((err) => {
        if (mounted) onNotify({ tone: "error", message: String(err) });
      })
      .finally(() => {
        if (mounted) setLoading(false);
      });
    return () => {
      mounted = false;
    };
  }, [gpuIndex, gpus.length, onNotify]);

  const limits = useMemo(
    () => ({
      powerMin: capabilities?.power_limit_min_watts ?? 0,
      powerMax: capabilities?.power_limit_max_watts ?? 0,
      coreClockMin: capabilities?.core_clock_min_mhz ?? 0,
      coreClockMax: capabilities?.core_clock_max_mhz ?? 0,
      memoryClockMin: capabilities?.memory_clock_min_mhz ?? 0,
      memoryClockMax: capabilities?.memory_clock_max_mhz ?? 0,
      coreMin: capabilities?.core_clock_offset_min_mhz ?? -200,
      coreMax: capabilities?.core_clock_offset_max_mhz ?? 300,
      memMin: capabilities?.memory_clock_offset_min_mhz ?? -500,
      memMax: capabilities?.memory_clock_offset_max_mhz ?? 1500
    }),
    [capabilities]
  );

  const clamp = (value: number, min: number, max: number) => Math.min(max, Math.max(min, value));

  const apply = async () => {
    setLoading(true);
    const command: TuningCommand = {
      gpu_index: gpuIndex,
      power_limit_watts: powerLimitEnabled && capabilities?.can_set_power_limit === "supported" ? powerLimit : null,
      core_clock_mhz: coreClockEnabled && capabilities?.can_set_core_clock === "supported" ? coreClock : null,
      memory_clock_mhz: memoryClockEnabled && capabilities?.can_set_memory_clock === "supported" ? memoryClock : null,
      core_clock_offset_mhz:
        coreOffsetEnabled && capabilities?.can_set_core_clock_offset === "supported" ? coreOffset : null,
      memory_clock_offset_mhz:
        memoryOffsetEnabled && capabilities?.can_set_memory_clock_offset === "supported" ? memoryOffset : null
    };
    try {
      const result = await api.applyTuning(command);
      onNotify({
        tone: result.success ? "success" : "error",
        message: result.messages.join("; ")
      });
    } catch (err) {
      onNotify({ tone: "error", message: String(err) });
    } finally {
      setLoading(false);
      setConfirmOpen(false);
    }
  };

  const reset = async () => {
    setLoading(true);
    try {
      const result = await api.resetGpuSettings(gpuIndex);
      const telemetry = await api.getTelemetry(gpuIndex);
      setCoreClock(telemetry.core_clock_mhz);
      setMemoryClock(telemetry.memory_clock_mhz);
      setCoreOffset(0);
      setMemoryOffset(0);
      setPowerLimitEnabled(false);
      setCoreClockEnabled(false);
      setMemoryClockEnabled(false);
      setCoreOffsetEnabled(false);
      setMemoryOffsetEnabled(false);
      onNotify({ tone: result.success ? "success" : "error", message: result.messages.join("; ") });
    } catch (err) {
      onNotify({ tone: "error", message: String(err) });
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="page">
      <header className="page-header">
        <div>
          <p className="eyebrow">Guarded Writes</p>
          <h2>Tuning</h2>
        </div>
        <select value={gpuIndex} onChange={(event) => onGpuChange(Number(event.target.value))} disabled={gpus.length === 0}>
          {gpus.map((gpu) => (
            <option key={gpu.index} value={gpu.index}>
              GPU {gpu.index}: {gpu.name}
            </option>
          ))}
        </select>
      </header>

      <section className="capability-grid">
        <Capability label="Telemetry" value={capabilities?.can_read_telemetry} />
        <Capability label="Power Limit" value={capabilities?.can_set_power_limit} />
        <Capability label="Core Clock" value={capabilities?.can_set_core_clock} />
        <Capability label="Memory Clock" value={capabilities?.can_set_memory_clock} />
        <Capability label="Core Offset" value={capabilities?.can_set_core_clock_offset} />
        <Capability label="Memory Offset" value={capabilities?.can_set_memory_clock_offset} />
      </section>

      <section className="panel">
        <RangeControl
          label="Power Limit"
          unit="W"
          min={limits.powerMin}
          max={limits.powerMax}
          value={powerLimit}
          enabled={powerLimitEnabled}
          disabled={loading || capabilities?.can_set_power_limit !== "supported"}
          onEnabledChange={setPowerLimitEnabled}
          onChange={(value) => setPowerLimit(clamp(value, limits.powerMin, limits.powerMax))}
        />
        <RangeControl
          label="Core Clock"
          unit="MHz"
          min={limits.coreClockMin}
          max={limits.coreClockMax}
          value={coreClock}
          enabled={coreClockEnabled}
          disabled={loading || capabilities?.can_set_core_clock !== "supported"}
          onEnabledChange={setCoreClockEnabled}
          onChange={setCoreClock}
          options={capabilities?.supported_core_clocks_mhz}
        />
        <RangeControl
          label="Memory Clock"
          unit="MHz"
          min={limits.memoryClockMin}
          max={limits.memoryClockMax}
          value={memoryClock}
          enabled={memoryClockEnabled}
          disabled={loading || capabilities?.can_set_memory_clock !== "supported"}
          onEnabledChange={setMemoryClockEnabled}
          onChange={setMemoryClock}
          options={capabilities?.supported_memory_clocks_mhz}
        />
        <RangeControl
          label="Core Clock Offset"
          unit="MHz"
          min={limits.coreMin}
          max={limits.coreMax}
          value={coreOffset}
          enabled={coreOffsetEnabled}
          disabled={loading || capabilities?.can_set_core_clock_offset !== "supported"}
          onEnabledChange={setCoreOffsetEnabled}
          onChange={(value) => setCoreOffset(clamp(value, limits.coreMin, limits.coreMax))}
        />
        <RangeControl
          label="Memory Clock Offset"
          unit="MHz"
          min={limits.memMin}
          max={limits.memMax}
          value={memoryOffset}
          enabled={memoryOffsetEnabled}
          disabled={loading || capabilities?.can_set_memory_clock_offset !== "supported"}
          onEnabledChange={setMemoryOffsetEnabled}
          onChange={(value) => setMemoryOffset(clamp(value, limits.memMin, limits.memMax))}
        />
        <div className="actions">
          <button className="button subtle" type="button" onClick={reset} disabled={loading}>
            Reset
          </button>
          <button className="button primary" type="button" onClick={() => setConfirmOpen(true)} disabled={loading}>
            Apply
          </button>
        </div>
      </section>

      {capabilities?.notes.length ? (
        <section className="notes">
          {capabilities.notes.map((note) => (
            <p key={note}>{note}</p>
          ))}
        </section>
      ) : null}

      <ConfirmDialog
        open={confirmOpen}
        title="Apply GPU tuning"
        message="Power and clock changes will be sent to the NVIDIA driver for the selected GPU."
        busy={loading}
        onCancel={() => setConfirmOpen(false)}
        onConfirm={apply}
      />
    </div>
  );
}

function Capability({ label, value }: { label: string; value?: string }) {
  const text = value ?? "loading";
  return (
    <div className={`capability ${text.split(" ").join("-")}`}>
      <span>{label}</span>
      <strong>{text}</strong>
    </div>
  );
}

function RangeControl({
  label,
  unit,
  min,
  max,
  value,
  enabled,
  disabled,
  onEnabledChange,
  onChange,
  options
}: {
  label: string;
  unit: string;
  min: number;
  max: number;
  value: number;
  enabled: boolean;
  disabled: boolean;
  onEnabledChange: (enabled: boolean) => void;
  onChange: (value: number) => void;
  options?: number[];
}) {
  const effectiveMin = Number.isFinite(min) ? min : 0;
  const effectiveMax = Number.isFinite(max) && max > effectiveMin ? max : effectiveMin + 1;
  const listId = `${label.toLowerCase().replace(/\s+/g, "-")}-values`;
  const [inputValue, setInputValue] = useState(String(value));

  useEffect(() => {
    setInputValue(String(value));
  }, [value]);

  const commitInput = () => {
    if (inputValue.trim() === "" || inputValue.trim() === "-") {
      setInputValue(String(value));
      return;
    }
    const parsed = Number(inputValue);
    if (!Number.isFinite(parsed)) {
      setInputValue(String(value));
      return;
    }
    const nextValue = Math.min(effectiveMax, Math.max(effectiveMin, parsed));
    onChange(nextValue);
    setInputValue(String(nextValue));
  };

  return (
    <div className="range-row">
      <div className="range-label">
        <label>
          <input
            type="checkbox"
            checked={enabled}
            disabled={disabled}
            onChange={(event) => onEnabledChange(event.target.checked)}
          />
          <span>{label}</span>
        </label>
        <span>
          {effectiveMin} to {effectiveMax} {unit}
        </span>
      </div>
      <input
        type="range"
        min={effectiveMin}
        max={effectiveMax}
        step={1}
        value={Math.min(effectiveMax, Math.max(effectiveMin, value))}
        disabled={disabled || !enabled}
        onChange={(event) => onChange(Number(event.target.value))}
      />
      <div className="number-input">
        <input
          type="number"
          min={effectiveMin}
          max={effectiveMax}
          list={options?.length ? listId : undefined}
          value={inputValue}
          disabled={disabled || !enabled}
          onChange={(event) => {
            const nextValue = event.target.value;
            setInputValue(nextValue);
            if (nextValue.trim() === "" || nextValue.trim() === "-") {
              return;
            }
            const parsed = Number(nextValue);
            if (Number.isFinite(parsed)) {
              onChange(parsed);
            }
          }}
          onBlur={commitInput}
          onKeyDown={(event) => {
            if (event.key === "Enter") {
              event.currentTarget.blur();
            }
          }}
        />
        {options?.length ? (
          <datalist id={listId}>
            {options.map((option) => (
              <option key={option} value={option} />
            ))}
          </datalist>
        ) : null}
        <span>{unit}</span>
      </div>
    </div>
  );
}
