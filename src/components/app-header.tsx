import { Bell, AlertTriangle, AlertCircle, Info, CheckCircle, X, Sun, Moon, RefreshCw } from "lucide-react";
import { EmptyState } from "@/components/ui/empty-state";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { SidebarTrigger } from "@/components/ui/sidebar";
import { Separator } from "@/components/ui/separator";
import { DropdownMenu, DropdownMenuContent, DropdownMenuGroup, DropdownMenuLabel, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Notification {
  id: number;
  notification_type: string;
  title: string;
  message: string;
  severity: string;
  ref_type: string | null;
  ref_id: number | null;
  is_read: boolean;
  read_at: string | null;
  created_at: string;
}

interface AppHeaderProps {
  searchQuery: string;
  onSearchChange: (query: string) => void;
  onRefresh: () => void;
  refreshing?: boolean;
}

const severityIcons: Record<string, React.ReactNode> = {
  warning: <AlertTriangle className="h-4 w-4 text-amber-500" />,
  error: <AlertCircle className="h-4 w-4 text-destructive" />,
  info: <Info className="h-4 w-4 text-blue-500" />,
  success: <CheckCircle className="h-4 w-4 text-emerald-500" />,
};

function timeAgo(dateStr: string): string {
  const now = new Date();
  const date = new Date(dateStr.replace(" ", "T") + "Z");
  const diffMs = now.getTime() - date.getTime();
  const diffMin = Math.floor(diffMs / 60000);
  if (diffMin < 1) return "刚刚";
  if (diffMin < 60) return `${diffMin} 分钟前`;
  const diffHr = Math.floor(diffMin / 60);
  if (diffHr < 24) return `${diffHr} 小时前`;
  const diffDay = Math.floor(diffHr / 24);
  return `${diffDay} 天前`;
}

export function AppHeader({ searchQuery, onSearchChange, onRefresh, refreshing = false }: AppHeaderProps) {
  const [dropdownOpen, setDropdownOpen] = useState(false);
  const [notifications, setNotifications] = useState<Notification[]>([]);
  const [unreadCount, setUnreadCount] = useState(0);
  const [isDark, setIsDark] = useState(() => document.documentElement.classList.contains("dark"));

  function toggleTheme() {
    const html = document.documentElement;
    if (isDark) {
      html.classList.remove("dark");
    } else {
      html.classList.add("dark");
    }
    setIsDark(!isDark);
  }

  async function loadNotifications() {
    try {
      const [notifs, count] = await Promise.all([
        invoke<Notification[]>("get_notifications", { limit: 20, unreadOnly: false }),
        invoke<number>("get_unread_notification_count"),
      ]);
      setNotifications(notifs);
      setUnreadCount(count);
    } catch (e) {
      console.error("加载通知失败:", e);
    }
  }

  useEffect(() => {
    if (dropdownOpen) {
      loadNotifications();
    }
  }, [dropdownOpen]);

  async function handleMarkRead(id: number) {
    try {
      await invoke("mark_notification_read", { id });
      setNotifications((prev) => prev.map((n) => (n.id === id ? { ...n, is_read: true } : n)));
      setUnreadCount((prev) => Math.max(0, prev - 1));
    } catch (e) {
      console.error("标记已读失败:", e);
    }
  }

  async function handleMarkAllRead() {
    try {
      await invoke("mark_all_notifications_read");
      setNotifications((prev) => prev.map((n) => ({ ...n, is_read: true })));
      setUnreadCount(0);
    } catch (e) {
      console.error("全部标记已读失败:", e);
    }
  }

  async function handleDelete(id: number) {
    try {
      await invoke("delete_notification", { id });
      setNotifications((prev) => prev.filter((n) => n.id !== id));
    } catch (e) {
      console.error("删除通知失败:", e);
    }
  }

  return (
    <header className="sticky top-0 z-10 flex h-14 shrink-0 items-center gap-2 border-b border-border bg-background px-4">
      <SidebarTrigger className="-ml-1" />
      <Separator orientation="vertical" className="mr-2 h-4" />

      <div className="flex flex-1 items-center gap-2">
        <div className="relative w-full max-w-sm">
          <Search className="absolute left-2.5 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            placeholder="搜索原材料、配方、订单..."
            value={searchQuery}
            onChange={(e) => onSearchChange(e.target.value)}
            className="pl-8 bg-transparent border-border"
          />
        </div>
      </div>

      <div className="flex items-center gap-1">
        <Button
          variant="ghost"
          size="icon"
          onClick={onRefresh}
          disabled={refreshing}
          className="h-8 w-8 text-muted-foreground"
          title="刷新数据"
        >
          <RefreshCw className={`h-4 w-4 ${refreshing ? "animate-spin" : ""}`} />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          onClick={toggleTheme}
          className="h-8 w-8 text-muted-foreground"
          title={isDark ? "切换浅色模式" : "切换深色模式"}
        >
          {isDark ? <Sun className="h-4 w-4" /> : <Moon className="h-4 w-4" />}
        </Button>
        <DropdownMenu open={dropdownOpen} onOpenChange={setDropdownOpen}>
          <DropdownMenuTrigger>
            <Button variant="ghost" size="icon" className="h-8 w-8 text-muted-foreground relative">
              <Bell className="h-4 w-4" />
              {unreadCount > 0 && (
                <Badge variant="destructive" className="absolute -top-1 -right-1 h-4 w-4 flex items-center justify-center p-0 text-[10px] min-w-0 animate-in zoom-in-95 fade-in-0 duration-200">
                  {unreadCount > 99 ? "99+" : unreadCount}
                </Badge>
              )}
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end" className="w-80 p-0">
            <DropdownMenuGroup>
              <div className="flex items-center justify-between px-3 py-2 border-b">
                <DropdownMenuLabel className="p-0 m-0">消息中心</DropdownMenuLabel>
                {unreadCount > 0 && (
                  <Button variant="ghost" size="sm" className="h-6 text-xs text-muted-foreground hover:text-foreground" onClick={handleMarkAllRead}>
                    全部已读
                  </Button>
                )}
              </div>
            </DropdownMenuGroup>
            <ScrollArea className="h-[320px]">
              {notifications.length === 0 ? (
                <EmptyState icon={Bell} title="暂无消息" className="py-8" />
              ) : (
                <div className="py-1">
                  {notifications.map((n) => (
                    <div
                      key={n.id}
                      className={`group flex items-start gap-3 px-3 py-2.5 hover:bg-accent/50 cursor-pointer transition-colors duration-150 ${
                        !n.is_read ? "bg-accent/20" : ""
                      }`}
                      onClick={() => !n.is_read && handleMarkRead(n.id)}
                    >
                      <div className="mt-0.5 shrink-0">
                        {severityIcons[n.severity] || severityIcons.info}
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="flex items-start justify-between gap-2">
                          <p className={`text-sm truncate ${!n.is_read ? "font-medium" : ""}`}>
                            {n.title}
                          </p>
                          <Button
                            variant="ghost"
                            size="icon"
                            className="shrink-0 opacity-0 group-hover:opacity-100 transition-opacity h-6 w-6 p-0"
                            onClick={(e) => { e.stopPropagation(); handleDelete(n.id); }}
                          >
                            <X className="h-3 w-3 text-muted-foreground" />
                          </Button>
                        </div>
                        <p className="text-xs text-muted-foreground mt-0.5 line-clamp-2">{n.message}</p>
                        <p className="text-[10px] text-muted-foreground/60 mt-1">{timeAgo(n.created_at)}</p>
                      </div>
                      {!n.is_read && (
                        <div className="shrink-0 mt-2 h-2 w-2 rounded-full bg-blue-500" />
                      )}
                    </div>
                  ))}
                </div>
              )}
            </ScrollArea>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </header>
  );
}

function Search(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg
      {...props}
      xmlns="http://www.w3.org/2000/svg"
      width="24"
      height="24"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <circle cx="11" cy="11" r="8" />
      <path d="m21 21-4.3-4.3" />
    </svg>
  );
}
