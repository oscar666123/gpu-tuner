import { useEffect } from "react";

export interface ToastState {
  tone: "success" | "error" | "info";
  message: string;
}

interface ToastProps {
  toast: ToastState | null;
  onClose: () => void;
}

export default function Toast({ toast, onClose }: ToastProps) {
  useEffect(() => {
    if (!toast) return;
    const timer = window.setTimeout(onClose, 4200);
    return () => window.clearTimeout(timer);
  }, [toast, onClose]);

  if (!toast) return null;

  return (
    <div className={`toast ${toast.tone}`} role="status">
      {toast.message}
    </div>
  );
}
