/**
 * Transport abstraction layer — routes IPC calls to either Tauri invoke() or
 * REST fetch() depending on the runtime environment.
 *
 * Usage (drop-in replacement for invoke):
 *   import { call } from "@/lib/transport";
 *   const data = await call<MenuItem[]>("get_menu_items");
 *
 * Migration path for hooks:
 *   Change: import { invoke } from "@tauri-apps/api/core";
 *   To:     import { call as invoke } from "@/lib/transport";
 *   Everything else stays the same.
 */

const SESSION_KEY = "cuckoo_session";

function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI__" in window;
}

/**
 * In browser mode the React app is served by the Rust HTTP server at the same
 * origin, so window.location.origin is always the correct base URL.
 */
function getBaseUrl(): string {
  return typeof window !== "undefined" ? window.location.origin : "";
}

function getSession(): string | null {
  try {
    return sessionStorage.getItem(SESSION_KEY);
  } catch {
    return null;
  }
}

export function setSession(token: string): void {
  try {
    sessionStorage.setItem(SESSION_KEY, token);
  } catch {
    // sessionStorage unavailable (e.g. private mode edge cases)
  }
}

export function clearSession(): void {
  try {
    sessionStorage.removeItem(SESSION_KEY);
  } catch {
    // ignore
  }
}

export async function call<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<T> {
  if (isTauri()) {
    // Dynamic import keeps the Tauri SDK out of the browser bundle entirely.
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<T>(command, args);
  }

  const token = getSession();
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), 8000);
  let response: Response;
  try {
    response = await fetch(`${getBaseUrl()}/api/${command}`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...(token ? { Authorization: `Bearer ${token}` } : {}),
      },
      body: JSON.stringify(args ?? {}),
      signal: controller.signal,
    });
  } finally {
    clearTimeout(timeoutId);
  }

  if (!response.ok) {
    const err = await response
      .json()
      .catch(() => ({ error: response.statusText }));
    throw new Error(
      typeof err === "object" && err !== null && "error" in err
        ? String(err.error)
        : response.statusText
    );
  }

  return response.json() as Promise<T>;
}

export { isTauri, getBaseUrl };
