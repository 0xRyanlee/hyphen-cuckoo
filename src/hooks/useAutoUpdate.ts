import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { UpdateInfo } from "@/components/UpdateDialog";

const STORAGE_KEY = "cuckoo_auto_update";
const SKIP_KEY = "cuckoo_skipped_version";
export const PENDING_KEY = "cuckoo_pending_update";

export function useAutoUpdate() {
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);

  useEffect(() => {
    const enabled = localStorage.getItem(STORAGE_KEY) !== "false";
    if (!enabled) return;

    const timer = setTimeout(async () => {
      try {
        const info = await invoke<UpdateInfo | null>("check_for_update");
        if (!info) return;
        const skipped = localStorage.getItem(SKIP_KEY);
        if (skipped === info.new_version) return;
        setUpdateInfo(info);
      } catch (e) {
        console.warn("[useAutoUpdate] update check failed:", e);
      }
    }, 3000);

    return () => clearTimeout(timer);
  }, []);

  function dismiss() {
    if (updateInfo) {
      localStorage.setItem(PENDING_KEY, JSON.stringify(updateInfo));
    }
    setUpdateInfo(null);
  }

  function skip() {
    if (updateInfo) {
      localStorage.setItem(SKIP_KEY, updateInfo.new_version);
      localStorage.removeItem(PENDING_KEY);
    }
    setUpdateInfo(null);
  }

  return { updateInfo, dismiss, skip };
}
