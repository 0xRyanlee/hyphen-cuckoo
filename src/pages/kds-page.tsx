import { useEffect, useRef, useCallback, useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Layers, Play, Check, Clock, MessageSquare, AlertCircle, Printer } from "lucide-react";
import { EmptyState } from "@/components/ui/empty-state";

interface OrderItem {
  id: number;
  order_id: number;
  menu_item_id: number;
  spec_code: string | null;
  qty: number;
  unit_price: number;
  note: string | null;
}

interface TicketWithItems {
  id: number;
  order_id: number;
  station_id: number;
  status: string;
  priority: number;
  printed_at: string | null;
  started_at: string | null;
  finished_at: string | null;
  created_at: string;
  order_no: string;
  dine_type: string;
  table_no: string | null;
  items: OrderItem[];
}

interface KitchenStation {
  id: number;
  code: string;
  name: string;
  station_type: string;
}

interface KDSPageProps {
  allTickets: TicketWithItems[];
  stations: KitchenStation[];
  menuItemNames: Record<number, string>;
  onStartTicket: (ticketId: number) => void;
  onFinishTicket: (ticketId: number) => void;
  onReprintTicket: (ticket: TicketWithItems) => void;
  onRefresh: () => void;
}

const OVERTIME_MINUTES = 15;

function playNewTicketBeep() {
  try {
    const ctx = new AudioContext();
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.connect(gain);
    gain.connect(ctx.destination);
    osc.type = "sine";
    osc.frequency.value = 880;
    gain.gain.setValueAtTime(0.35, ctx.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.01, ctx.currentTime + 0.35);
    osc.start(ctx.currentTime);
    osc.stop(ctx.currentTime + 0.35);
    osc.onended = () => ctx.close();
  } catch {
    // AudioContext blocked (e.g., before user interaction)
  }
}

export function KDSPage({
  allTickets,
  stations,
  menuItemNames,
  onStartTicket,
  onFinishTicket,
  onReprintTicket,
  onRefresh,
}: KDSPageProps) {
  const [selectedStationId, setSelectedStationId] = useState<string>("all");
  const seenTicketIds = useRef<Set<number>>(new Set());
  const initialized = useRef(false);

  const playBeepIfNewTickets = useCallback(() => {
    let hasNew = false;
    for (const ticket of allTickets) {
      if (ticket.status !== "finished" && !seenTicketIds.current.has(ticket.id)) {
        hasNew = true;
      }
      seenTicketIds.current.add(ticket.id);
    }
    if (initialized.current && hasNew) {
      playNewTicketBeep();
    }
    initialized.current = true;
  }, [allTickets]);

  useEffect(() => {
    playBeepIfNewTickets();
  }, [playBeepIfNewTickets]);

  useEffect(() => {
    const interval = setInterval(() => {
      onRefresh();
    }, 15000);
    return () => clearInterval(interval);
  }, [onRefresh]);

  const getElapsedMinutes = (createdAt: string): number => {
    const ts = createdAt.includes("T") ? createdAt : createdAt.replace(" ", "T");
    return Math.floor((Date.now() - new Date(ts + "Z").getTime()) / 60000);
  };

  const formatElapsed = (mins: number): string => {
    if (mins < 1) return "剛剛";
    if (mins < 60) return `${mins}分鐘前`;
    return `${Math.floor(mins / 60)}小時前`;
  };

  const getStatusColor = (status: string, overtimed: boolean) => {
    if (overtimed) return "border-red-500/70 bg-red-500/5";
    switch (status) {
      case "pending": return "border-amber-500/50 bg-amber-500/5";
      case "started": return "border-blue-500/50 bg-blue-500/5";
      case "finished": return "border-emerald-500/50 bg-emerald-500/5";
      default: return "";
    }
  };

  const getStatusBadge = (status: string) => {
    switch (status) {
      case "pending": return <Badge variant="outline" className="border-amber-500 text-amber-500">待制作</Badge>;
      case "started": return <Badge className="bg-blue-600">制作中</Badge>;
      case "finished": return <Badge className="bg-emerald-600">已完成</Badge>;
      default: return <Badge variant="secondary">{status}</Badge>;
    }
  };

  const pendingTickets = allTickets
    .filter((t) => t.status !== "finished")
    .filter((t) => selectedStationId === "all" || t.station_id === Number(selectedStationId));

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">厨房显示系统</h2>
          <p className="text-sm text-muted-foreground">{pendingTickets.length} 张待处理工单</p>
        </div>
        <div className="flex items-center gap-2">
          {stations.length > 0 && (
            <Select value={selectedStationId} onValueChange={setSelectedStationId}>
              <SelectTrigger className="w-36">
                <SelectValue placeholder="全部工作站" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">全部工作站</SelectItem>
                {stations.map((s) => (
                  <SelectItem key={s.id} value={String(s.id)}>{s.name}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          )}
          <Button variant="outline" size="sm" onClick={onRefresh}>
            <Layers className="mr-2 h-4 w-4" />刷新
          </Button>
        </div>
      </div>

      {pendingTickets.length === 0 ? (
        <Card>
          <CardContent className="py-8">
            <EmptyState icon={Layers} title="暂无待处理工单" description="所有订单已完成" />
          </CardContent>
        </Card>
      ) : (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {pendingTickets.map((ticket) => {
            const elapsedMins = getElapsedMinutes(ticket.created_at);
            const overtimed = elapsedMins >= OVERTIME_MINUTES;
            return (
              <Card key={ticket.id} className={`border-2 ${getStatusColor(ticket.status, overtimed)}`}>
                <CardHeader className="pb-2">
                  <div className="flex items-center justify-between">
                    <div>
                      <CardTitle className="text-base">{ticket.order_no}</CardTitle>
                      <div className="flex items-center gap-2 mt-1">
                        <Badge variant="outline" className="text-xs">{ticket.dine_type}</Badge>
                        {ticket.table_no && <Badge variant="outline" className="text-xs">桌号 {ticket.table_no}</Badge>}
                      </div>
                    </div>
                    {getStatusBadge(ticket.status)}
                  </div>
                  <div className={`flex items-center gap-1 text-xs mt-1 ${overtimed ? "text-red-500 font-medium" : "text-muted-foreground"}`}>
                    {overtimed ? <AlertCircle className="h-3 w-3" /> : <Clock className="h-3 w-3" />}
                    <span>{formatElapsed(elapsedMins)}</span>
                    {overtimed && <span className="ml-1">⚠ 超時</span>}
                  </div>
                </CardHeader>
                <CardContent className="pt-2">
                  <ScrollArea className="h-40 mb-3">
                    <div className="space-y-2">
                      {ticket.items.map((item) => (
                        <div key={item.id} className="rounded-md bg-muted/50 p-2.5">
                          <div className="flex items-center justify-between">
                            <span className="font-medium text-sm">
                              {menuItemNames[item.menu_item_id] || `商品 #${item.menu_item_id}`}
                            </span>
                            <span className="text-sm font-bold text-primary">x{item.qty}</span>
                          </div>
                          {item.spec_code && (
                            <div className="text-xs text-muted-foreground mt-0.5">規格: {item.spec_code}</div>
                          )}
                          {item.note && (
                            <div className="flex items-center gap-1 mt-1 text-xs text-amber-500">
                              <MessageSquare className="h-3 w-3" />
                              <span>{item.note}</span>
                            </div>
                          )}
                        </div>
                      ))}
                    </div>
                  </ScrollArea>
                  <div className="flex gap-2">
                    {ticket.status === "pending" && (
                      <Button size="sm" className="flex-1" onClick={() => onStartTicket(ticket.id)}>
                        <Play className="mr-2 h-3 w-3" />开始制作
                      </Button>
                    )}
                    {ticket.status === "started" && (
                      <Button size="sm" className="flex-1 bg-emerald-600 hover:bg-emerald-700" onClick={() => onFinishTicket(ticket.id)}>
                        <Check className="mr-2 h-3 w-3" />完成出餐
                      </Button>
                    )}
                    <Button size="sm" variant="outline" onClick={() => onReprintTicket(ticket)} title="补打印">
                      <Printer className="h-3 w-3" />
                    </Button>
                  </div>
                </CardContent>
              </Card>
            );
          })}
        </div>
      )}
    </div>
  );
}
