import { useState, useEffect, useRef } from "react";
import { toPng } from "html-to-image";
import { call as invoke } from "@/lib/transport";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter, DialogDescription } from "@/components/ui/dialog";
import { toast } from "sonner";
import { Plus, QrCode, Trash2, Download, ImagePlus, X } from "lucide-react";
import { StyledQR } from "@/components/styled-qr";
import type { WebServerStatus } from "@/types";

interface Campaign {
  id: number;
  name: string;
  discount_type: string;
  discount_value: number;
  condition_text: string | null;
  valid_days: number;
  is_active: boolean;
  cover_image?: string | null;
  claimed?: number;
  redeemed?: number;
}

function discountText(t: string, v: number): string {
  if (t === "percent") return `${v}% OFF`;
  if (t === "amount") return `减 ¥${v}`;
  return "免费赠品";
}

function CampaignPosterDialog({ campaign, baseUrl, onClose }: { campaign: Campaign; baseUrl: string; onClose: () => void }) {
  const [token, setToken] = useState<string | null>(null);
  const [exporting, setExporting] = useState(false);
  const posterRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    invoke<string>("sign_campaign_token", { campaignId: campaign.id }).then(setToken).catch(() => setToken(null));
  }, [campaign.id]);

  const url = token ? `${baseUrl}/#/c/${token}` : "";

  async function downloadPoster() {
    if (!posterRef.current) return;
    setExporting(true);
    try {
      const dataUrl = await toPng(posterRef.current, { pixelRatio: 3, backgroundColor: "#ffffff" });
      const link = document.createElement("a");
      link.download = `活动码-${campaign.name}.png`;
      link.href = dataUrl;
      link.click();
    } catch {
      toast.error("导出失败，请重试");
    } finally {
      setExporting(false);
    }
  }

  return (
    <Dialog open onOpenChange={onClose}>
      <DialogContent className="max-w-xs">
        <DialogHeader>
          <DialogTitle>{campaign.name} 活动码</DialogTitle>
        </DialogHeader>
        <div className="flex flex-col items-center gap-4 py-2">
          <div ref={posterRef} className="w-full rounded-2xl border-2 border-red-200 bg-white p-5 flex flex-col items-center gap-3">
            {campaign.cover_image && (
              <img
                src={`data:image/jpeg;base64,${campaign.cover_image}`}
                alt="活动封面"
                className="w-full rounded-xl object-cover max-h-32"
              />
            )}
            <div className="text-center">
              <div className="text-[11px] text-gray-400 tracking-[0.3em]">扫码领券</div>
              <div className="text-xl font-extrabold text-red-700 mt-1">{campaign.name}</div>
              <div className="text-2xl font-extrabold text-red-600 mt-1">{discountText(campaign.discount_type, campaign.discount_value)}</div>
            </div>
            {url ? <StyledQR value={url} size={190} dotColor="#dc2626" /> : <div className="h-[190px] flex items-center justify-center text-xs text-gray-400">生成中…</div>}
            <div className="flex items-center gap-1.5 text-red-500 text-sm font-bold">
              <QrCode className="h-4 w-4" /> 扫一扫 领取优惠券
            </div>
          </div>
          <Button className="w-full" size="sm" onClick={downloadPoster} disabled={exporting || !url}>
            <Download className="h-3.5 w-3.5 mr-1.5" />{exporting ? "导出中…" : "下载活动海报"}
          </Button>
          <p className="text-[10px] text-muted-foreground text-center">下载高清海报可张贴或投放朋友圈</p>
        </div>
      </DialogContent>
    </Dialog>
  );
}

export function CampaignManager() {
  const [campaigns, setCampaigns] = useState<Campaign[]>([]);
  const [baseUrl, setBaseUrl] = useState("");
  const [poster, setPoster] = useState<Campaign | null>(null);
  // create form
  const [name, setName] = useState("");
  const [discountType, setDiscountType] = useState("percent");
  const [discountValue, setDiscountValue] = useState("10");
  const [condition, setCondition] = useState("");
  const [validDays, setValidDays] = useState("30");
  const [dailyLimit, setDailyLimit] = useState("0");
  const [saving, setSaving] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<Campaign | null>(null);

  const load = () => {
    invoke<Campaign[]>("list_campaigns", {}).then(setCampaigns).catch(console.error);
  };

  useEffect(() => {
    load();
    invoke<WebServerStatus>("get_web_server_status", {}).then((s) => {
      let u = s.url ?? "";
      // Reuse the manual LAN IP override set on the 餐桌管理 page, if any.
      const manualIp = localStorage.getItem("cuckoo_manual_lan_ip");
      if (manualIp && u) u = u.replace(/\/\/[^/:]+(:\d+)/, `//${manualIp}$1`);
      setBaseUrl(u);
    }).catch(() => {});
  }, []);

  async function handleCreate() {
    if (!name.trim()) { toast.error("请填写活动名称"); return; }
    setSaving(true);
    try {
      await invoke("create_campaign", {
        name: name.trim(),
        discountType,
        discountValue: parseFloat(discountValue) || 0,
        conditionText: condition.trim() || null,
        validDays: parseInt(validDays, 10) || 30,
        dailyLimit: parseInt(dailyLimit, 10) || 0,
      });
      toast.success("活动已创建");
      setName(""); setCondition("");
      load();
    } catch (e) {
      toast.error("创建失败", { description: String(e) });
    } finally {
      setSaving(false);
    }
  }

  async function toggleActive(c: Campaign) {
    try {
      await invoke("set_campaign_active", { id: c.id, active: !c.is_active });
      load();
    } catch (e) { toast.error(String(e)); }
  }

  async function confirmDelete() {
    if (!deleteTarget) return;
    const target = deleteTarget;
    setDeleteTarget(null);
    try {
      await invoke("delete_campaign", { id: target.id });
      toast.success("已删除");
      load();
    } catch (e) { toast.error(String(e)); }
  }

  async function handleCoverUpload(c: Campaign, e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0];
    if (!file) return;
    try {
      const base64 = await new Promise<string>((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = () => resolve((reader.result as string).split(",")[1]);
        reader.onerror = reject;
        reader.readAsDataURL(file);
      });
      await invoke("update_campaign_cover", { id: c.id, coverImage: base64 });
      toast.success("封面已更新");
      load();
    } catch (err) {
      toast.error("上传失败", { description: String(err) });
    } finally {
      e.target.value = "";
    }
  }

  async function handleCoverRemove(c: Campaign) {
    try {
      await invoke("update_campaign_cover", { id: c.id, coverImage: null });
      load();
    } catch (err) { toast.error(String(err)); }
  }

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader>
          <CardTitle className="text-base">创建活动</CardTitle>
          <CardDescription>顾客扫活动码即领一张专属优惠券，截图到店由店员扫码核销（一码一次）</CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="grid grid-cols-2 gap-3">
            <div className="space-y-1.5 col-span-2">
              <Label className="text-xs">活动名称</Label>
              <Input placeholder="例：开业大酬宾" value={name} onChange={(e) => setName(e.target.value)} />
            </div>
            <div className="space-y-1.5">
              <Label className="text-xs">优惠类型</Label>
              <select value={discountType} onChange={(e) => setDiscountType(e.target.value)}
                className="w-full h-9 rounded-md border bg-background px-3 text-sm">
                <option value="percent">百分比折扣 (% OFF)</option>
                <option value="amount">立减金额 (¥)</option>
                <option value="free_item">免费赠品</option>
              </select>
            </div>
            <div className="space-y-1.5">
              <Label className="text-xs">优惠值</Label>
              <Input type="number" value={discountValue} onChange={(e) => setDiscountValue(e.target.value)}
                disabled={discountType === "free_item"} />
            </div>
            <div className="space-y-1.5">
              <Label className="text-xs">使用条件（选填）</Label>
              <Input placeholder="例：满50元可用" value={condition} onChange={(e) => setCondition(e.target.value)} />
            </div>
            <div className="space-y-1.5">
              <Label className="text-xs">有效天数</Label>
              <Input type="number" value={validDays} onChange={(e) => setValidDays(e.target.value)} />
            </div>
            <div className="space-y-1.5">
              <Label className="text-xs">每日领取上限（0=不限）</Label>
              <Input type="number" value={dailyLimit} onChange={(e) => setDailyLimit(e.target.value)} />
            </div>
          </div>
          <Button onClick={handleCreate} disabled={saving} size="sm">
            <Plus className="h-4 w-4 mr-1" />{saving ? "创建中…" : "创建活动"}
          </Button>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-base">活动列表</CardTitle>
        </CardHeader>
        <CardContent>
          {campaigns.length === 0 ? (
            <p className="text-sm text-muted-foreground text-center py-4">还没有活动，创建一个开始吧</p>
          ) : (
            <div className="space-y-2">
              {campaigns.map((c) => (
                <div key={c.id} className={`flex items-center gap-3 border rounded-lg px-3 py-2 ${c.is_active ? "" : "opacity-50"}`}>
                  {/* Cover image thumbnail */}
                  <label className="relative shrink-0 cursor-pointer group">
                    {c.cover_image ? (
                      <>
                        <img src={`data:image/jpeg;base64,${c.cover_image}`} alt="封面" className="h-12 w-12 rounded-md object-cover border" />
                        <button
                          type="button"
                          className="absolute -top-1 -right-1 h-4 w-4 rounded-full bg-destructive text-white flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity"
                          onClick={(e) => { e.preventDefault(); handleCoverRemove(c); }}
                        >
                          <X className="h-2.5 w-2.5" />
                        </button>
                      </>
                    ) : (
                      <div className="h-12 w-12 rounded-md border-2 border-dashed flex items-center justify-center bg-muted/30 hover:bg-muted/60 transition-colors">
                        <ImagePlus className="h-4 w-4 text-muted-foreground" />
                      </div>
                    )}
                    <input type="file" accept="image/*" className="hidden" onChange={(e) => handleCoverUpload(c, e)} />
                  </label>
                  <div className="min-w-0 flex-1">
                    <p className="text-sm font-medium truncate">{c.name}</p>
                    <p className="text-xs text-muted-foreground">{discountText(c.discount_type, c.discount_value)} · {c.valid_days}天{c.condition_text ? ` · ${c.condition_text}` : ""}</p>
                    <p className="text-[10px] text-muted-foreground mt-0.5">
                      领取 {c.claimed ?? 0} · 核销 {c.redeemed ?? 0}
                      {(c.claimed ?? 0) > 0 && ` · 核销率 ${Math.round(((c.redeemed ?? 0) / (c.claimed ?? 1)) * 100)}%`}
                    </p>
                  </div>
                  <Switch checked={c.is_active} onCheckedChange={() => toggleActive(c)} />
                  <Button size="icon" variant="ghost" className="h-7 w-7" title="活动码" disabled={!baseUrl} onClick={() => setPoster(c)}>
                    <QrCode className="h-3.5 w-3.5" />
                  </Button>
                  <Button size="icon" variant="ghost" className="h-7 w-7 text-destructive hover:text-destructive" onClick={() => setDeleteTarget(c)}>
                    <Trash2 className="h-3.5 w-3.5" />
                  </Button>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {poster && baseUrl && (
        <CampaignPosterDialog campaign={poster} baseUrl={baseUrl} onClose={() => setPoster(null)} />
      )}

      <Dialog open={!!deleteTarget} onOpenChange={(v) => !v && setDeleteTarget(null)}>
        <DialogContent className="max-w-sm">
          <DialogHeader>
            <DialogTitle>删除活动</DialogTitle>
            <DialogDescription>
              确定删除活动「{deleteTarget?.name}」？已发出的券将无法核销，此操作不可撤销。
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteTarget(null)}>取消</Button>
            <Button variant="destructive" onClick={confirmDelete}>删除</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
