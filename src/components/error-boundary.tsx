import { Component, ErrorInfo, ReactNode } from "react";

declare const __APP_VERSION__: string;
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { AlertTriangle, Copy, RefreshCw } from "lucide-react";
import { appLogger, type LogEntry } from "@/lib/logger";

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
  recentLogs: LogEntry[];
}

const CATEGORY_LABEL: Record<string, string> = {
  db: "資料庫錯誤",
  not_found: "資料不存在",
  validation: "參數驗證失敗",
  ipc: "IPC 通訊錯誤",
  print: "印表機錯誤",
  render: "介面渲染崩潰",
  runtime: "JavaScript 執行期錯誤",
  logic: "業務邏輯錯誤",
};

function LogRow({ entry }: { entry: LogEntry }) {
  const label = CATEGORY_LABEL[entry.category] ?? entry.category;
  const time = new Date(entry.ts).toLocaleTimeString();
  return (
    <div className="text-xs font-mono border-b last:border-b-0 py-1.5 grid grid-cols-[64px_88px_1fr] gap-2 items-start">
      <span className="text-muted-foreground">{time}</span>
      <span className="text-amber-600 dark:text-amber-400 truncate">{label}</span>
      <span className="break-all">[{entry.operation}] {entry.message}</span>
    </div>
  );
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null, errorInfo: null, recentLogs: [] };
  }

  static getDerivedStateFromError(error: Error): Partial<State> {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    const entry = appLogger.logRenderError(error, errorInfo.componentStack ?? undefined);
    // Grab the last 10 log entries so they appear inline in the error page
    const recentLogs = appLogger.getRecent(10);
    this.setState({ errorInfo, recentLogs });
    console.error("[ErrorBoundary]", entry.id, error, errorInfo);
  }

  handleCopyReport = () => {
    const { error, errorInfo } = this.state;
    const report = [
      "=== Cuckoo 崩潰報告 ===",
      `時間: ${new Date().toISOString()}`,
      `版本: ${__APP_VERSION__ ?? "unknown"}`,
      `平台: ${navigator.platform}`,
      `URL: ${location.href}`,
      "",
      "--- 錯誤訊息 ---",
      error?.toString() ?? "Unknown error",
      "",
      "--- 組件堆疊 ---",
      errorInfo?.componentStack ?? "N/A",
      "",
      "--- 最近 10 條錯誤日誌 ---",
      appLogger.exportJson(),
      "=== 報告結束 ===",
    ].join("\n");

    navigator.clipboard.writeText(report).then(() => {
      alert("報告已複製到剪貼簿，請傳送給開發者");
    }).catch(() => {
      prompt("請複製以下報告傳送給開發者:", report);
    });
  };

  handleReload = () => {
    window.location.reload();
  };

  render() {
    if (!this.state.hasError) return this.props.children;

    const { error, recentLogs } = this.state;

    return (
      <div className="flex h-screen w-full items-center justify-center bg-background p-8">
        <Card className="max-w-2xl w-full">
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-destructive">
              <AlertTriangle className="h-5 w-5" />
              應用程式崩潰 — render 錯誤
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <p className="text-sm text-muted-foreground">
              React 元件樹渲染失敗。請複製報告傳送給開發者，或重新載入嘗試恢復。
            </p>

            {error && (
              <div className="rounded-md bg-destructive/10 p-3 space-y-1">
                <p className="text-xs text-muted-foreground">錯誤訊息</p>
                <p className="text-xs font-mono text-destructive whitespace-pre-wrap break-all">
                  {error.message}
                </p>
              </div>
            )}

            {recentLogs.length > 0 && (
              <div className="rounded-md border p-3 space-y-1">
                <p className="text-xs font-semibold text-muted-foreground mb-2">
                  崩潰前最近 {recentLogs.length} 條錯誤記錄
                </p>
                {recentLogs.map((e) => <LogRow key={e.id} entry={e} />)}
              </div>
            )}

            <div className="flex gap-2">
              <Button onClick={this.handleCopyReport} className="flex-1 gap-2">
                <Copy className="h-4 w-4" />
                複製完整報告
              </Button>
              <Button variant="outline" onClick={this.handleReload} className="gap-2">
                <RefreshCw className="h-4 w-4" />
                重新載入
              </Button>
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }
}
