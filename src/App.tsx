import { useCallback, useEffect, useMemo, useState } from "react";
import { Activity, Gauge, ListChecks, Settings2, ShieldCheck } from "lucide-react";
import Layout, { type NavItem } from "./components/Layout";
import Toast, { type ToastState } from "./components/Toast";
import Dashboard from "./pages/Dashboard";
import Tuning from "./pages/Tuning";
import Profiles from "./pages/Profiles";
import Safety from "./pages/Safety";
import Logs from "./pages/Logs";
import { api } from "./lib/api";
import type { GPUInfo } from "./lib/types";

type PageKey = "dashboard" | "tuning" | "profiles" | "safety" | "logs";

export default function App() {
  const [page, setPage] = useState<PageKey>("dashboard");
  const [gpus, setGpus] = useState<GPUInfo[]>([]);
  const [selectedGpu, setSelectedGpu] = useState(0);
  const [loadingGpus, setLoadingGpus] = useState(true);
  const [toast, setToast] = useState<ToastState | null>(null);
  const [safetyThreshold, setSafetyThreshold] = useState(85);

  const navItems: NavItem<PageKey>[] = useMemo(
    () => [
      { key: "dashboard", label: "Dashboard", icon: Activity },
      { key: "tuning", label: "Tuning", icon: Gauge },
      { key: "profiles", label: "Profiles", icon: ListChecks },
      { key: "safety", label: "Safety", icon: ShieldCheck },
      { key: "logs", label: "Logs", icon: Settings2 }
    ],
    []
  );

  useEffect(() => {
    let mounted = true;
    api
      .getGpuList()
      .then((items) => {
        if (!mounted) return;
        setGpus(items);
        if (items.length > 0) setSelectedGpu(items[0].index);
      })
      .catch((error) => {
        if (mounted) setToast({ tone: "error", message: String(error) });
      })
      .finally(() => {
        if (mounted) setLoadingGpus(false);
      });
    return () => {
      mounted = false;
    };
  }, []);

  const notify = useCallback((nextToast: ToastState) => setToast(nextToast), []);
  const closeToast = useCallback(() => setToast(null), []);

  const pageContent = {
    dashboard: (
      <Dashboard
        gpuIndex={selectedGpu}
        gpus={gpus}
        loadingGpus={loadingGpus}
        safetyThreshold={safetyThreshold}
        onGpuChange={setSelectedGpu}
        onNotify={notify}
      />
    ),
    tuning: <Tuning gpuIndex={selectedGpu} gpus={gpus} onGpuChange={setSelectedGpu} onNotify={notify} />,
    profiles: <Profiles gpuIndex={selectedGpu} gpus={gpus} onGpuChange={setSelectedGpu} onNotify={notify} />,
    safety: (
      <Safety
        gpuIndex={selectedGpu}
        threshold={safetyThreshold}
        onThresholdChange={setSafetyThreshold}
        onNotify={notify}
      />
    ),
    logs: <Logs onNotify={notify} />
  } satisfies Record<PageKey, JSX.Element>;

  return (
    <>
      <Layout navItems={navItems} activeKey={page} onNavigate={setPage} gpus={gpus}>
        {pageContent[page]}
      </Layout>
      <Toast toast={toast} onClose={closeToast} />
    </>
  );
}
