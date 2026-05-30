/// <reference types="vite/client" />
// Structured error logger for Cuckoo
// All invoke errors are categorized and persisted to localStorage so
// the debug pipeline (docs/debug-pipeline.md) can query them without
// needing a running backend or native logs.

export type ErrorCategory =
  | "db"        // DB constraint / SQL error from Rust
  | "not_found" // Entity not found
  | "validation"// Input validation failure
  | "ipc"       // Tauri IPC / channel failure
  | "print"     // Printer / label errors (non-fatal)
  | "render"    // React component crash (ErrorBoundary)
  | "runtime"   // Unhandled JS error / promise rejection
  | "logic";    // Business logic error (catch-all)

export interface LogEntry {
  id: string;
  ts: string;
  category: ErrorCategory;
  operation: string;
  message: string;
  stack?: string;
  context?: Record<string, unknown>;
}

const STORAGE_KEY = "cuckoo_error_log_v2";
const MAX_ENTRIES = 60;
const SENSITIVE_KEY_RE = /(phone|tel|mobile|url|webhook|endpoint|token|secret|key|password|pin|sn|ukey|order_no|orderno|client_id|email)/i;
const URL_RE = /^https?:\/\/\S+/i;
const LONG_DIGITS_RE = /\b\d{7,}\b/g;

function categorize(msg: string): ErrorCategory {
  const s = msg.toLowerCase();
  if (s.includes("unique constraint") || s.includes("database error") || s.includes("sqlite") || s.includes("foreign key")) return "db";
  if (s.includes("no rows") || s.includes("not found") || s.includes("does not exist")) return "not_found";
  if (s.includes("invalid") || s.includes("validation") || s.includes("required") || s.includes("cannot be null")) return "validation";
  if (s.includes("invoke") || s.includes("channel") || s.includes("ipc") || s.includes("webview")) return "ipc";
  if (s.includes("print") || s.includes("printer") || s.includes("serial")) return "print";
  return "logic";
}

/** Strip Tauri/JS error prefixes and return a clean message. */
export function formatError(raw: unknown): string {
  const s = raw instanceof Error ? raw.message : String(raw);
  return s.replace(/^Error:\s*/i, "").replace(/^Tauri error:\s*/i, "").trim();
}

function persist(entry: LogEntry): void {
  try {
    const existing: LogEntry[] = JSON.parse(localStorage.getItem(STORAGE_KEY) || "[]");
    existing.unshift(entry);
    if (existing.length > MAX_ENTRIES) existing.length = MAX_ENTRIES;
    localStorage.setItem(STORAGE_KEY, JSON.stringify(existing));
  } catch { /* storage quota or parse error — silently skip */ }
}

export function sanitizeSensitiveValue(value: unknown, key?: string): unknown {
  if (value == null) return value;
  if (typeof value === "string") {
    if (key && SENSITIVE_KEY_RE.test(key)) return "[REDACTED]";
    if (URL_RE.test(value)) return "[REDACTED_URL]";
    return value.replace(LONG_DIGITS_RE, (match) => `${match.slice(0, 2)}***${match.slice(-2)}`);
  }
  if (Array.isArray(value)) {
    return value.map((item) => sanitizeSensitiveValue(item));
  }
  if (typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value as Record<string, unknown>).map(([entryKey, entryValue]) => {
        if (SENSITIVE_KEY_RE.test(entryKey)) {
          return [entryKey, "[REDACTED]"];
        }
        return [entryKey, sanitizeSensitiveValue(entryValue, entryKey)];
      }),
    );
  }
  return value;
}

export const appLogger = {
  log(entry: Omit<LogEntry, "id" | "ts">): LogEntry {
    const full: LogEntry = {
      id: `${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
      ts: new Date().toISOString(),
      ...entry,
      message: sanitizeSensitiveValue(entry.message) as string,
      stack: entry.stack ? (sanitizeSensitiveValue(entry.stack) as string) : undefined,
      context: entry.context ? (sanitizeSensitiveValue(entry.context) as Record<string, unknown>) : undefined,
    };
    persist(full);
    // Notify UI components so they can show badges/alerts without polling
    window.dispatchEvent(new CustomEvent("cuckoo:logged", { detail: full }));
    // In dev, emit a structured console.error so DevTools shows file:line
    if (import.meta.env.DEV) {
      const label = `[CUCKOO:${full.category.toUpperCase()}] ${full.operation}`;
      if (full.context) {
        console.error(label, full.message, full.context);
      } else {
        console.error(label, full.message);
      }
    }
    return full;
  },

  logInvokeError(operation: string, raw: unknown, context?: Record<string, unknown>): LogEntry {
    const message = formatError(raw);
    return this.log({
      category: categorize(message),
      operation,
      message: sanitizeSensitiveValue(message) as string,
      stack: raw instanceof Error ? raw.stack : undefined,
      context,
    });
  },

  logRenderError(error: Error, componentStack?: string): LogEntry {
    return this.log({
      category: "render",
      operation: "react_render",
      message: error.message,
      stack: componentStack
        ? `${error.stack ?? ""}\n--- component stack ---\n${componentStack}`
        : error.stack,
    });
  },

  logRuntimeError(event: ErrorEvent | PromiseRejectionEvent): LogEntry {
    const isRejection = "reason" in event;
    const message = isRejection ? formatError(event.reason) : event.message;
    return this.log({
      category: "runtime",
      operation: isRejection ? "unhandled_rejection" : "window_error",
      message,
      stack: isRejection
        ? (event.reason instanceof Error ? event.reason.stack : undefined)
        : undefined,
      context: isRejection ? undefined : { filename: event.filename, lineno: event.lineno, colno: event.colno },
    });
  },

  getRecent(n = 20): LogEntry[] {
    try {
      const entries: LogEntry[] = JSON.parse(localStorage.getItem(STORAGE_KEY) || "[]");
      return entries.slice(0, n);
    } catch { return []; }
  },

  clear(): void {
    try { localStorage.removeItem(STORAGE_KEY); } catch { /* ignore */ }
  },

  exportJson(): string {
    return JSON.stringify(this.getRecent(MAX_ENTRIES), null, 2);
  },
};
