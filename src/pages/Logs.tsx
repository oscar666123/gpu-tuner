import { useEffect, useState } from "react";
import { api } from "../lib/api";
import type { LogEntry } from "../lib/types";
import type { ToastState } from "../components/Toast";

interface LogsProps {
  onNotify: (toast: ToastState) => void;
}

export default function Logs({ onNotify }: LogsProps) {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [loading, setLoading] = useState(false);

  const refresh = async () => {
    setLoading(true);
    try {
      const items = await api.getLogs();
      setLogs(items);
    } catch (err) {
      onNotify({ tone: "error", message: String(err) });
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    refresh();
  }, []);

  const clear = async () => {
    setLoading(true);
    try {
      await api.clearLogs();
      setLogs([]);
      onNotify({ tone: "success", message: "Logs cleared" });
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
          <p className="eyebrow">API Records</p>
          <h2>Logs</h2>
        </div>
        <div className="actions compact">
          <button className="button subtle" type="button" onClick={refresh} disabled={loading}>
            Refresh
          </button>
          <button className="button danger" type="button" onClick={clear} disabled={loading}>
            Clear
          </button>
        </div>
      </header>

      <section className="log-table">
        <div className="log-head">
          <span>Time</span>
          <span>Action</span>
          <span>Status</span>
          <span>Code</span>
          <span>Message</span>
        </div>
        {logs.length === 0 ? <div className="empty">No log entries</div> : null}
        {logs.map((log, index) => (
          <div className="log-row" key={`${log.timestamp}-${index}`}>
            <span>{new Date(log.timestamp).toLocaleString()}</span>
            <span>{log.action}</span>
            <span className={log.success ? "ok" : "fail"}>{log.success ? "success" : "failed"}</span>
            <span>{log.code ?? "--"}</span>
            <span>{log.message}</span>
          </div>
        ))}
      </section>
    </div>
  );
}
