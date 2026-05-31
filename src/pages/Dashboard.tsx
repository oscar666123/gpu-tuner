import { useEffect, useMemo, useRef, useState } from "react";
import { Area, AreaChart, CartesianGrid, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import StatCard from "../components/StatCard";
import { api } from "../lib/api";
import type { GPUInfo, GpuTelemetry } from "../lib/types";
import type { ToastState } from "../components/Toast";

interface DashboardProps {
  gpuIndex: number;
  gpus: GPUInfo[];
  loadingGpus: boolean;
  safetyThreshold: number;
  onGpuChange: (index: number) => void;
  onNotify: (toast: ToastState) => void;
}

export default function Dashboard({
  gpuIndex,
  gpus,
  loadingGpus,
  safetyThreshold,
  onGpuChange,
  onNotify
}: DashboardProps) {
  const [telemetry, setTelemetry] = useState<GpuTelemetry | null>(null);
  const [history, setHistory] = useState<Array<GpuTelemetry & { time: string }>>([]);
  const [error, setError] = useState<string | null>(null);
  const lastSafetyResetAt = useRef(0);

  useEffect(() => {
    let active = true;
    const refresh = async () => {
      if (gpus.length === 0) return;
      try {
        const next = await api.getTelemetry(gpuIndex);
        if (!active) return;
        const time = new Date(next.timestamp).toLocaleTimeString();
        setTelemetry(next);
        setHistory((items) => [...items.slice(-59), { ...next, time }]);
        setError(null);
        const now = Date.now();
        if (next.temperature_c >= safetyThreshold && now - lastSafetyResetAt.current > 30_000) {
          lastSafetyResetAt.current = now;
          const result = await api.resetGpuSettings(gpuIndex);
          onNotify({
            tone: result.success ? "success" : "error",
            message: result.messages.join("; ")
          });
        }
      } catch (err) {
        if (active) setError(String(err));
      }
    };

    refresh();
    const interval = window.setInterval(refresh, 1000);
    return () => {
      active = false;
      window.clearInterval(interval);
    };
  }, [gpuIndex, gpus.length, onNotify, safetyThreshold]);

  const activeGpu = useMemo(() => gpus.find((gpu) => gpu.index === gpuIndex), [gpus, gpuIndex]);

  return (
    <div className="page">
      <header className="page-header">
        <div>
          <p className="eyebrow">Realtime Monitoring</p>
          <h2>Dashboard</h2>
        </div>
        <select value={gpuIndex} onChange={(event) => onGpuChange(Number(event.target.value))} disabled={gpus.length === 0}>
          {gpus.map((gpu) => (
            <option key={gpu.index} value={gpu.index}>
              GPU {gpu.index}: {gpu.name}
            </option>
          ))}
        </select>
      </header>

      {loadingGpus ? <div className="empty">Loading GPUs...</div> : null}
      {error ? <div className="banner error">{error}</div> : null}

      <section className="info-strip">
        <div>
          <span>GPU</span>
          <strong>{activeGpu?.name ?? "Unavailable"}</strong>
        </div>
        <div>
          <span>Driver</span>
          <strong>{activeGpu?.driver_version ?? "Unknown"}</strong>
        </div>
        <div>
          <span>Memory</span>
          <strong>{activeGpu ? `${activeGpu.memory_total_mb} MB` : "Unknown"}</strong>
        </div>
      </section>

      <div className="stat-grid">
        <StatCard label="Temperature" value={telemetry ? `${telemetry.temperature_c} C` : "--"} detail={`Limit ${safetyThreshold} C`} />
        <StatCard label="GPU Usage" value={telemetry ? `${telemetry.gpu_utilization_percent}%` : "--"} />
        <StatCard label="Memory Usage" value={telemetry ? `${telemetry.memory_utilization_percent}%` : "--"} />
        <StatCard label="Core Clock" value={telemetry ? `${telemetry.core_clock_mhz} MHz` : "--"} />
        <StatCard label="Memory Clock" value={telemetry ? `${telemetry.memory_clock_mhz} MHz` : "--"} />
        <StatCard label="Power Draw" value={telemetry ? `${telemetry.power_draw_watts.toFixed(1)} W` : "--"} />
        <StatCard label="Power Limit" value={telemetry ? `${telemetry.power_limit_watts.toFixed(1)} W` : "--"} />
        <StatCard
          label="VRAM"
          value={telemetry ? `${telemetry.memory_used_mb} / ${telemetry.memory_total_mb} MB` : "--"}
        />
      </div>

      <section className="chart-panel">
        <div className="section-title">
          <h3>Telemetry Trend</h3>
          <span>1 second refresh</span>
        </div>
        <div className="chart-wrap">
          <ResponsiveContainer width="100%" height="100%">
            <AreaChart data={history}>
              <defs>
                <linearGradient id="temperature" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#e35d6a" stopOpacity={0.55} />
                  <stop offset="95%" stopColor="#e35d6a" stopOpacity={0} />
                </linearGradient>
                <linearGradient id="usage" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#4cc9a6" stopOpacity={0.45} />
                  <stop offset="95%" stopColor="#4cc9a6" stopOpacity={0} />
                </linearGradient>
              </defs>
              <CartesianGrid stroke="#243041" strokeDasharray="4 4" />
              <XAxis dataKey="time" stroke="#7d8da4" minTickGap={36} />
              <YAxis stroke="#7d8da4" />
              <Tooltip contentStyle={{ background: "#101722", border: "1px solid #2d3b4f", borderRadius: 8 }} />
              <Area type="monotone" dataKey="temperature_c" stroke="#e35d6a" fill="url(#temperature)" name="Temp C" />
              <Area type="monotone" dataKey="gpu_utilization_percent" stroke="#4cc9a6" fill="url(#usage)" name="GPU %" />
            </AreaChart>
          </ResponsiveContainer>
        </div>
      </section>
    </div>
  );
}
