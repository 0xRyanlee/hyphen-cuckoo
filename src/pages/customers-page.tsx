import { useState } from "react";
import { call as invoke } from "@/lib/transport";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Badge } from "@/components/ui/badge";
import { toast } from "sonner";
import { UserPlus, Search, Star, Phone, Clock, Gift, ChevronRight, X } from "lucide-react";
import type { Customer, LoyaltyTxn } from "@/types";

interface CustomersPageProps {
  customers: Customer[];
  onCreateCustomer: (name: string, phone: string | null) => Promise<void>;
  onUpdateCustomer: (id: number, name: string | null, phone: string | null, clearPhone: boolean) => Promise<void>;
  onDeleteCustomer: (id: number) => Promise<void>;
  onAddLoyaltyPoints: (customerId: number, orderId: number | null, delta: number, reason: string) => Promise<number>;
}

export function CustomersPage({
  customers,
  onCreateCustomer,
  onUpdateCustomer,
  onDeleteCustomer,
  onAddLoyaltyPoints,
}: CustomersPageProps) {
  const [search, setSearch] = useState("");
  const [createOpen, setCreateOpen] = useState(false);
  const [editTarget, setEditTarget] = useState<Customer | null>(null);
  const [detailTarget, setDetailTarget] = useState<Customer | null>(null);
  const [adjustTarget, setAdjustTarget] = useState<Customer | null>(null);
  const [txns, setTxns] = useState<LoyaltyTxn[]>([]);
  const [txnsLoading, setTxnsLoading] = useState(false);

  const [createName, setCreateName] = useState("");
  const [createPhone, setCreatePhone] = useState("");

  const [editName, setEditName] = useState("");
  const [editPhone, setEditPhone] = useState("");
  const [editClearPhone, setEditClearPhone] = useState(false);

  const [adjustDelta, setAdjustDelta] = useState("");
  const [adjustReason, setAdjustReason] = useState("");

  const filtered = customers.filter(c =>
    !search ||
    c.name.toLowerCase().includes(search.toLowerCase()) ||
    (c.phone && c.phone.includes(search))
  );

  const openDetail = async (c: Customer) => {
    setDetailTarget(c);
    setTxnsLoading(true);
    try {
      const result = await invoke<LoyaltyTxn[]>("get_loyalty_txns", { customerId: c.id });
      setTxns(result);
    } catch (e) {
      toast.error(String(e));
    } finally {
      setTxnsLoading(false);
    }
  };

  const openEdit = (c: Customer) => {
    setEditTarget(c);
    setEditName(c.name);
    setEditPhone(c.phone ?? "");
    setEditClearPhone(false);
  };

  const handleCreate = async () => {
    if (!createName.trim()) { toast.error("请输入顾客姓名"); return; }
    await onCreateCustomer(createName.trim(), createPhone.trim() || null);
    setCreateOpen(false);
    setCreateName("");
    setCreatePhone("");
  };

  const handleEdit = async () => {
    if (!editTarget) return;
    if (!editName.trim()) { toast.error("请输入顾客姓名"); return; }
    await onUpdateCustomer(
      editTarget.id,
      editName.trim(),
      editClearPhone ? null : (editPhone.trim() || null),
      editClearPhone,
    );
    setEditTarget(null);
  };

  const handleAdjust = async () => {
    if (!adjustTarget) return;
    const delta = parseInt(adjustDelta, 10);
    if (isNaN(delta) || delta === 0) { toast.error("请输入有效的积分数（非零整数）"); return; }
    if (!adjustReason.trim()) { toast.error("请输入调整原因"); return; }
    try {
      const newPoints = await onAddLoyaltyPoints(adjustTarget.id, null, delta, adjustReason.trim());
      toast.success(`积分已调整，当前积分：${newPoints}`);
      setAdjustTarget(null);
      setAdjustDelta("");
      setAdjustReason("");
    } catch (e) {
      toast.error(String(e));
    }
  };

  return (
    <div className="flex flex-col h-full p-6 gap-4">
      <div className="flex items-center justify-between">
        <h1 className="text-xl font-semibold">顾客管理</h1>
        <Button onClick={() => setCreateOpen(true)} size="sm">
          <UserPlus className="w-4 h-4 mr-1" />
          添加顾客
        </Button>
      </div>

      <div className="relative max-w-sm">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
        <Input
          className="pl-9"
          placeholder="搜索姓名或手机号…"
          value={search}
          onChange={e => setSearch(e.target.value)}
        />
      </div>

      <div className="flex-1 overflow-auto">
        {filtered.length === 0 ? (
          <p className="text-muted-foreground text-sm mt-8 text-center">暂无顾客记录</p>
        ) : (
          <div className="grid gap-2">
            {filtered.map(c => (
              <div
                key={c.id}
                className="flex items-center gap-3 p-3 rounded-lg border bg-card hover:bg-accent/30 cursor-pointer"
                onClick={() => openDetail(c)}
              >
                <div className="w-9 h-9 rounded-full bg-primary/10 flex items-center justify-center text-primary font-semibold text-sm flex-shrink-0">
                  {c.name.charAt(0).toUpperCase()}
                </div>
                <div className="flex-1 min-w-0">
                  <p className="font-medium truncate">{c.name}</p>
                  <p className="text-xs text-muted-foreground">{c.phone ?? "无手机号"}</p>
                </div>
                <div className="flex items-center gap-3 text-sm">
                  <span className="flex items-center gap-1 text-amber-600">
                    <Star className="w-3.5 h-3.5" />
                    {c.points}
                  </span>
                  <span className="text-muted-foreground">¥{c.total_spent.toFixed(2)}</span>
                </div>
                <ChevronRight className="w-4 h-4 text-muted-foreground flex-shrink-0" />
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Create Dialog */}
      <Dialog open={createOpen} onOpenChange={setCreateOpen}>
        <DialogContent>
          <DialogHeader><DialogTitle>添加顾客</DialogTitle></DialogHeader>
          <div className="space-y-4 py-2">
            <div className="space-y-1.5">
              <Label>姓名 *</Label>
              <Input value={createName} onChange={e => setCreateName(e.target.value)} placeholder="顾客姓名" />
            </div>
            <div className="space-y-1.5">
              <Label>手机号</Label>
              <Input value={createPhone} onChange={e => setCreatePhone(e.target.value)} placeholder="（选填）" />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setCreateOpen(false)}>取消</Button>
            <Button onClick={handleCreate}>添加</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Edit Dialog */}
      <Dialog open={!!editTarget} onOpenChange={v => !v && setEditTarget(null)}>
        <DialogContent>
          <DialogHeader><DialogTitle>编辑顾客</DialogTitle></DialogHeader>
          <div className="space-y-4 py-2">
            <div className="space-y-1.5">
              <Label>姓名 *</Label>
              <Input value={editName} onChange={e => setEditName(e.target.value)} />
            </div>
            <div className="space-y-1.5">
              <Label>手机号</Label>
              {editClearPhone ? (
                <div className="flex items-center gap-2">
                  <span className="text-sm text-muted-foreground flex-1">（将清除手机号）</span>
                  <Button size="sm" variant="ghost" onClick={() => setEditClearPhone(false)}>
                    <X className="w-3.5 h-3.5" />
                  </Button>
                </div>
              ) : (
                <div className="flex gap-2">
                  <Input value={editPhone} onChange={e => setEditPhone(e.target.value)} placeholder="（选填）" />
                  {editPhone && (
                    <Button size="sm" variant="ghost" onClick={() => setEditClearPhone(true)} title="清除手机号">
                      <X className="w-3.5 h-3.5" />
                    </Button>
                  )}
                </div>
              )}
            </div>
          </div>
          <DialogFooter>
            <Button variant="destructive" size="sm" onClick={async () => {
              if (!editTarget) return;
              await onDeleteCustomer(editTarget.id);
              setEditTarget(null);
            }}>删除</Button>
            <div className="flex-1" />
            <Button variant="outline" onClick={() => setEditTarget(null)}>取消</Button>
            <Button onClick={handleEdit}>保存</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Detail Dialog */}
      <Dialog open={!!detailTarget} onOpenChange={v => !v && setDetailTarget(null)}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle>{detailTarget?.name}</DialogTitle>
          </DialogHeader>
          {detailTarget && (
            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-3">
                <div className="rounded-lg bg-amber-50 dark:bg-amber-900/20 p-3 flex flex-col gap-1">
                  <span className="text-xs text-muted-foreground flex items-center gap-1"><Star className="w-3 h-3" />积分余额</span>
                  <span className="text-2xl font-bold text-amber-600">{detailTarget.points}</span>
                </div>
                <div className="rounded-lg bg-blue-50 dark:bg-blue-900/20 p-3 flex flex-col gap-1">
                  <span className="text-xs text-muted-foreground">累计消费</span>
                  <span className="text-2xl font-bold text-blue-600">¥{detailTarget.total_spent.toFixed(0)}</span>
                </div>
              </div>
              {detailTarget.phone && (
                <p className="text-sm flex items-center gap-1.5 text-muted-foreground">
                  <Phone className="w-3.5 h-3.5" />
                  {detailTarget.phone}
                </p>
              )}
              <div className="flex gap-2">
                <Button size="sm" variant="outline" onClick={() => { setDetailTarget(null); openEdit(detailTarget); }}>
                  编辑信息
                </Button>
                <Button size="sm" variant="outline" onClick={() => { setDetailTarget(null); setAdjustTarget(detailTarget); }}>
                  <Gift className="w-3.5 h-3.5 mr-1" />
                  手动调整积分
                </Button>
              </div>
              <div>
                <p className="text-sm font-medium mb-2 flex items-center gap-1.5">
                  <Clock className="w-3.5 h-3.5" />
                  积分记录
                </p>
                {txnsLoading ? (
                  <p className="text-xs text-muted-foreground">加载中…</p>
                ) : txns.length === 0 ? (
                  <p className="text-xs text-muted-foreground">暂无记录</p>
                ) : (
                  <div className="space-y-1 max-h-48 overflow-auto">
                    {txns.map(t => (
                      <div key={t.id} className="flex items-center justify-between text-xs py-1 border-b last:border-0">
                        <span className="text-muted-foreground truncate flex-1">{t.reason}</span>
                        <span className={t.delta > 0 ? "text-green-600 font-medium ml-2" : "text-red-500 font-medium ml-2"}>
                          {t.delta > 0 ? `+${t.delta}` : t.delta}
                        </span>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </div>
          )}
        </DialogContent>
      </Dialog>

      {/* Manual Adjust Dialog */}
      <Dialog open={!!adjustTarget} onOpenChange={v => !v && setAdjustTarget(null)}>
        <DialogContent>
          <DialogHeader><DialogTitle>手动调整积分 — {adjustTarget?.name}</DialogTitle></DialogHeader>
          <div className="space-y-4 py-2">
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <Badge variant="outline">当前积分：{adjustTarget?.points ?? 0}</Badge>
            </div>
            <div className="space-y-1.5">
              <Label>调整数量（正数增加，负数扣减）</Label>
              <Input
                type="number"
                value={adjustDelta}
                onChange={e => setAdjustDelta(e.target.value)}
                placeholder="例：+100 或 -50"
              />
            </div>
            <div className="space-y-1.5">
              <Label>原因 *</Label>
              <Input value={adjustReason} onChange={e => setAdjustReason(e.target.value)} placeholder="例：活动赠送" />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setAdjustTarget(null)}>取消</Button>
            <Button onClick={handleAdjust}>确认调整</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
