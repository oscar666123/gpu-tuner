import { useState } from "react";
import { api } from "../lib/api";
import type { ToastState } from "../components/Toast";

interface SafetyProps {
  gpuIndex: number;
  threshold: number;
  onThresholdChange: (value: number) => void;
  onNotify: (toast: ToastState) => void;
}

export default function Safety({ gpuIndex, threshold, onThresholdChange, onNotify }: SafetyProps) {
  const [checking, setChecking] = useState(false);

  const checkNow = async () => {
    setChecking(true);
    try {
      const telemetry = await api.getTelemetry(gpuIndex);
      if (telemetry.temperature_c >= threshold) {
        const result = await api.resetGpuSettings(gpuIndex);
        onNotify({ tone: result.success ? "success" : "error", message: result.messages.join("; ") });
        return;
      }
      onNotify({ tone: "success", message: `Temperature ${telemetry.temperature_c} C is below threshold` });
    } catch (err) {
      onNotify({ tone: "error", message: String(err) });
    } finally {
      setChecking(false);
    }
  };

  return (
    <div className="page">
      <header className="page-header">
        <div>
          <p className="eyebrow">Thermal Protection</p>
          <h2>Safety</h2>
        </div>
      </header>

      <section className="panel">
        <div className="range-row">
          <div>
            <label>Temperature Protection Threshold</label>
            <span>Default 85 C</span>
          </div>
          <input
            type="range"
            min={60}
            max={95}
            step={1}
            value={threshold}
            onChange={(event) => onThresholdChange(Number(event.target.value))}
          />
          <div className="number-input">
            <input
              type="number"
              min={60}
              max={95}
              value={threshold}
              onChange={(event) => onThresholdChange(Number(event.target.value))}
            />
            <span>C</span>
          </div>
        </div>
        <div className="banner">
          Dashboard checks this threshold every second. When the selected GPU reaches the threshold, reset is attempted and the event is logged.
        </div>
        <div className="actions">
          <button className="button primary" type="button" onClick={checkNow} disabled={checking}>
            {checking ? "Checking..." : "Check Now"}
          </button>
        </div>
      </section>
    </div>
  );
}
