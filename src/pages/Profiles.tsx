import { useEffect, useMemo, useState } from "react";
import ConfirmDialog from "../components/ConfirmDialog";
import { api } from "../lib/api";
import type { GPUInfo, Profile } from "../lib/types";
import type { ToastState } from "../components/Toast";

interface ProfilesProps {
  gpuIndex: number;
  gpus: GPUInfo[];
  onGpuChange: (index: number) => void;
  onNotify: (toast: ToastState) => void;
}

export default function Profiles({ gpuIndex, gpus, onGpuChange, onNotify }: ProfilesProps) {
  const [profiles, setProfiles] = useState<Profile[]>([]);
  const [name, setName] = useState("");
  const [powerLimit, setPowerLimit] = useState<number | null>(null);
  const [coreClock, setCoreClock] = useState<number | null>(null);
  const [memoryClock, setMemoryClock] = useState<number | null>(null);
  const [coreOffset, setCoreOffset] = useState(0);
  const [memoryOffset, setMemoryOffset] = useState(0);
  const [loading, setLoading] = useState(false);
  const [pendingProfile, setPendingProfile] = useState<Profile | null>(null);

  const refresh = async () => {
    const items = await api.listProfiles();
    setProfiles(items);
  };

  useEffect(() => {
    refresh().catch((err) => onNotify({ tone: "error", message: String(err) }));
  }, [onNotify]);

  useEffect(() => {
    setPowerLimit(null);
    setCoreClock(null);
    setMemoryClock(null);
  }, [gpuIndex]);

  const activeGpu = useMemo(() => gpus.find((gpu) => gpu.index === gpuIndex), [gpus, gpuIndex]);

  const save = async () => {
    const now = new Date().toISOString();
    const profile: Profile = {
      name: name.trim(),
      gpu_index: gpuIndex,
      power_limit_watts: powerLimit,
      core_clock_mhz: coreClock,
      memory_clock_mhz: memoryClock,
      core_clock_offset_mhz: coreOffset,
      memory_clock_offset_mhz: memoryOffset,
      created_at: now,
      updated_at: now
    };
    if (!profile.name) {
      onNotify({ tone: "error", message: "Profile name is required" });
      return;
    }
    setLoading(true);
    try {
      await api.saveProfile(profile);
      await refresh();
      onNotify({ tone: "success", message: "Profile saved" });
    } catch (err) {
      onNotify({ tone: "error", message: String(err) });
    } finally {
      setLoading(false);
    }
  };

  const loadForApply = async (profile: Profile) => {
    setName(profile.name);
    setPowerLimit(profile.power_limit_watts);
    setCoreClock(profile.core_clock_mhz ?? null);
    setMemoryClock(profile.memory_clock_mhz ?? null);
    setCoreOffset(profile.core_clock_offset_mhz ?? 0);
    setMemoryOffset(profile.memory_clock_offset_mhz ?? 0);
    setPendingProfile(profile);
  };

  const applyProfile = async () => {
    if (!pendingProfile) return;
    setLoading(true);
    try {
      const result = await api.applyTuning({
        gpu_index: pendingProfile.gpu_index,
        power_limit_watts: pendingProfile.power_limit_watts,
        core_clock_mhz: pendingProfile.core_clock_mhz,
        memory_clock_mhz: pendingProfile.memory_clock_mhz,
        core_clock_offset_mhz: pendingProfile.core_clock_offset_mhz,
        memory_clock_offset_mhz: pendingProfile.memory_clock_offset_mhz
      });
      onNotify({ tone: result.success ? "success" : "error", message: result.messages.join("; ") });
    } catch (err) {
      onNotify({ tone: "error", message: String(err) });
    } finally {
      setLoading(false);
      setPendingProfile(null);
    }
  };

  const remove = async (profileName: string) => {
    setLoading(true);
    try {
      await api.deleteProfile(profileName);
      await refresh();
      onNotify({ tone: "success", message: "Profile deleted" });
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
          <p className="eyebrow">Local JSON Profiles</p>
          <h2>Profiles</h2>
        </div>
        <select value={gpuIndex} onChange={(event) => onGpuChange(Number(event.target.value))} disabled={gpus.length === 0}>
          {gpus.map((gpu) => (
            <option key={gpu.index} value={gpu.index}>
              GPU {gpu.index}: {gpu.name}
            </option>
          ))}
        </select>
      </header>

      <section className="panel two-column">
        <div className="profile-editor">
          <h3>Save Current Settings</h3>
          <p>{activeGpu?.name ?? "Select a GPU"}</p>
          <label>
            Name
            <input value={name} onChange={(event) => setName(event.target.value)} placeholder="Daily profile" />
          </label>
          <label>
            Power Limit W
            <input
              type="number"
              value={powerLimit ?? ""}
              onChange={(event) => setPowerLimit(event.target.value === "" ? null : Number(event.target.value))}
            />
          </label>
          <label>
            Core Clock MHz
            <input
              type="number"
              value={coreClock ?? ""}
              onChange={(event) => setCoreClock(event.target.value === "" ? null : Number(event.target.value))}
            />
          </label>
          <label>
            Memory Clock MHz
            <input
              type="number"
              value={memoryClock ?? ""}
              onChange={(event) => setMemoryClock(event.target.value === "" ? null : Number(event.target.value))}
            />
          </label>
          <label>
            Core Clock Offset MHz
            <input type="number" value={coreOffset} min={-200} max={300} onChange={(event) => setCoreOffset(Number(event.target.value))} />
          </label>
          <label>
            Memory Clock Offset MHz
            <input
              type="number"
              value={memoryOffset}
              min={-500}
              max={1500}
              onChange={(event) => setMemoryOffset(Number(event.target.value))}
            />
          </label>
          <button className="button primary" type="button" onClick={save} disabled={loading}>
            Save Profile
          </button>
        </div>

        <div className="profile-list">
          <h3>Saved Profiles</h3>
          {profiles.length === 0 ? <div className="empty">No profiles saved</div> : null}
          {profiles.map((profile) => (
            <article className="profile-item" key={profile.name}>
              <div>
                <strong>{profile.name}</strong>
                <span>
                  GPU {profile.gpu_index} | {profile.power_limit_watts ?? "--"} W | Core Lock {profile.core_clock_mhz ?? "--"} MHz |
                  Mem Lock {profile.memory_clock_mhz ?? "--"} MHz | Core Offset {profile.core_clock_offset_mhz ?? 0} MHz | Mem Offset{" "}
                  {profile.memory_clock_offset_mhz ?? 0} MHz
                </span>
              </div>
              <div className="mini-actions">
                <button className="button subtle" type="button" onClick={() => loadForApply(profile)} disabled={loading}>
                  Load
                </button>
                <button className="button danger" type="button" onClick={() => remove(profile.name)} disabled={loading}>
                  Delete
                </button>
              </div>
            </article>
          ))}
        </div>
      </section>

      <ConfirmDialog
        open={Boolean(pendingProfile)}
        title="Apply profile"
        message={`Apply ${pendingProfile?.name ?? "profile"} to GPU ${pendingProfile?.gpu_index ?? gpuIndex}?`}
        busy={loading}
        onCancel={() => setPendingProfile(null)}
        onConfirm={applyProfile}
      />
    </div>
  );
}
