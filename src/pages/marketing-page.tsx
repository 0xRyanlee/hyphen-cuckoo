import { useState, useCallback, useEffect } from "react";
import { call as invoke } from "@/lib/transport";
import DOMPurify from "dompurify";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Switch } from "@/components/ui/switch";
import { Label } from "@/components/ui/label";
import { Save, Eye, Smartphone, Printer, Sparkles, Zap, ClipboardCheck, Search, BarChart3 } from "lucide-react";
import { Input } from "@/components/ui/input";
import { toast } from "sonner";
import { ELEMENT_CATEGORIES, getElementLabel, getElementBadgeColor, getElementSummary, type PrintElement } from "./print-templates-page";
import { CampaignManager } from "./campaign-manager";
import { Megaphone } from "lucide-react";

// ── Types ──────────────────────────────────────────────────────────────────

// ── Default element presets ────────────────────────────────────────────────

const DEFAULT_POPUP_ELEMENTS: PrintElement[] = [
  { type: "fortune", seed_strategy: "per_order" },
  { type: "character_collect", game_name: "集字兑奖", characters: ["恭", "喜", "發", "財"], prize: "集齐四字兑换免费饮品", seed_strategy: "per_order", style: "box" },
  { type: "quote", language: "multilingual" },
];

const DEFAULT_RECEIPT_ELEMENTS: PrintElement[] = [
  { type: "art", variant: "random" },
  { type: "fortune", seed_strategy: "per_order" },
  { type: "quote", language: "multilingual" },
];

// ── Element toggle card ────────────────────────────────────────────────────

function ElementToggleCard({
  elem, onRemove
}: {
  elem: PrintElement;
  onRemove: () => void;
}) {
  return (
    <div className="flex items-center gap-2 border rounded-lg px-3 py-2 bg-background text-xs">
      <span className={`px-1.5 py-0.5 rounded text-[10px] font-medium shrink-0 ${getElementBadgeColor(elem.type)}`}>
        {getElementLabel(elem.type)}
      </span>
      <span className="flex-1 text-muted-foreground truncate">{getElementSummary(elem)}</span>
      <Button variant="ghost" size="sm" className="h-5 w-5 p-0 text-muted-foreground hover:text-destructive"
        onClick={onRemove}>✕</Button>
    </div>
  );
}

// ── Surface config panel ───────────────────────────────────────────────────

function SurfacePanel({
  title, icon, description, elements, enabled, onToggleEnabled, onAddElement, onRemoveElement, previewHtml
}: {
  title: string;
  icon: React.ReactNode;
  description: string;
  elements: PrintElement[];
  enabled: boolean;
  onToggleEnabled: (v: boolean) => void;
  onAddElement: (elem: PrintElement) => void;
  onRemoveElement: (idx: number) => void;
  previewHtml: string;
}) {
  const [pickerOpen, setPickerOpen] = useState(false);

  return (
    <div className="space-y-4">
      <div className="flex items-start justify-between">
        <div className="flex items-center gap-2">
          {icon}
          <div>
            <p className="font-medium text-sm">{title}</p>
            <p className="text-xs text-muted-foreground">{description}</p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <Label className="text-xs text-muted-foreground">{enabled ? "启用" : "停用"}</Label>
          <Switch checked={enabled} onCheckedChange={onToggleEnabled} />
        </div>
      </div>

      {enabled && (
        <>
          <div className="space-y-1.5">
            {elements.length === 0 && (
              <p className="text-xs text-muted-foreground py-2 text-center border rounded-md">
                暂无行销元件 — 点击下方添加
              </p>
            )}
            {elements.map((elem, idx) => (
              <ElementToggleCard key={idx} elem={elem} onRemove={() => onRemoveElement(idx)} />
            ))}
            <Button variant="outline" size="sm" className="w-full h-7 text-xs gap-1 mt-1"
              onClick={() => setPickerOpen(!pickerOpen)}>
              <Sparkles className="h-3 w-3" />添加行销元件
            </Button>
          </div>

          {pickerOpen && (
            <div className="border rounded-lg p-3 bg-muted/20 space-y-2">
              {ELEMENT_CATEGORIES.filter(c => c.label !== "基本").map(cat => (
                <div key={cat.label}>
                  <p className="text-[10px] font-medium text-muted-foreground mb-1.5">{cat.label}</p>
                  <div className="flex flex-wrap gap-1">
                    {cat.items.map(it => (
                      <Button key={it.type} variant="secondary" size="sm"
                        className="h-6 text-xs px-2"
                        onClick={() => { onAddElement({ ...it.defaultConfig }); setPickerOpen(false); }}>
                        {it.label}
                      </Button>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          )}

          {previewHtml && (
            <div className="border rounded-lg overflow-hidden">
              <div className="flex items-center gap-1.5 px-3 py-1.5 bg-muted/30 border-b">
                <Eye className="h-3 w-3 text-muted-foreground" />
                <span className="text-xs text-muted-foreground">预览</span>
              </div>
              <div className="p-3 max-h-64 overflow-y-auto"
                dangerouslySetInnerHTML={{ __html: DOMPurify.sanitize(previewHtml) }} />
            </div>
          )}
        </>
      )}
    </div>
  );
}

// ── Types ─────────────────────────────────────────────────────────────────

interface RedemptionRecord {
  id: number;
  order_id: number;
  order_no: string;
  component_type: string;
  note: string | null;
  staff_name: string | null;
  redeemed_at: string;
}

interface VerifyResult {
  order_no: string;
  popup_html: string;
  already_redeemed: boolean;
}

interface MarketingFunnel {
  days: number;
  scans: number;
  self_orders: number;
  redemptions: number;
  scan_to_order: number;
  by_component: { component: string; count: number }[];
}

const COMPONENT_LABELS: Record<string, string> = {
  fortune: "运势卡",
  character_collect: "集字兑奖",
  quote: "每日语录",
  art: "艺术图块",
  discount_coupon: "折价券",
  qr_code: "二维码",
  riddle: "谜语挑战",
  solar_term: "节气主题",
  chef_message: "厨师寄语",
};

// ── Main page ──────────────────────────────────────────────────────────────

export function MarketingPage() {
  const [popupEnabled, setPopupEnabled] = useState(true);
  const [receiptEnabled, setReceiptEnabled] = useState(true);
  const [popupElements, setPopupElements] = useState<PrintElement[]>(DEFAULT_POPUP_ELEMENTS);
  const [receiptElements, setReceiptElements] = useState<PrintElement[]>(DEFAULT_RECEIPT_ELEMENTS);
  const [popupPreview, setPopupPreview] = useState("");
  const [receiptPreview, setReceiptPreview] = useState("");
  const [saving, setSaving] = useState(false);
  const [popupTemplateId, setPopupTemplateId] = useState<number | null>(null);
  const [redemptions, setRedemptions] = useState<RedemptionRecord[]>([]);
  const [verifyOrderNo, setVerifyOrderNo] = useState("");
  const [verifyResult, setVerifyResult] = useState<VerifyResult | null>(null);
  const [verifying, setVerifying] = useState(false);
  const [redeemCode, setRedeemCode] = useState("");
  const [redeeming, setRedeeming] = useState(false);

  // D3: redeem directly by a scanned code — accepts a raw token or a full
  // .../#/redeem/<token> URL pasted from a scanner gun, no order-no lookup.
  async function handleRedeemByCode() {
    const raw = redeemCode.trim();
    if (!raw) return;
    const token = raw.includes("/redeem/") ? raw.split("/redeem/")[1].split(/[?#]/)[0] : raw;
    setRedeeming(true);
    try {
      const r = await invoke<{ ok: boolean; already?: boolean; reason?: string; order_no?: string; component?: string }>(
        "redeem_marketing_qr_token", { token }
      );
      if (r.ok) {
        toast.success(`核销成功 · ${COMPONENT_LABELS[r.component ?? ""] ?? r.component ?? ""} · ${r.order_no ?? ""}`);
        setRedeemCode("");
        invoke<RedemptionRecord[]>("get_marketing_redemptions", {}).then(setRedemptions).catch(() => {});
      } else if (r.already) {
        toast.error(`该码已核销 · ${r.order_no ?? ""}`);
      } else if (r.reason === "pin_required") {
        toast.error("需要店员密码，请在收银设备上核销");
      } else {
        toast.error("核销码无效");
      }
    } catch (e) {
      toast.error("核销失败", { description: String(e) });
    } finally {
      setRedeeming(false);
    }
  }

  // Load existing marketing_popup template on mount
  useEffect(() => {
    invoke<{ templates: { id: number; template_type: string; content: string; is_active: boolean }[] }>(
      "get_print_templates", { templateType: "marketing_popup" }
    ).then(result => {
      const templates = Array.isArray(result) ? result : [];
      const tpl = templates[0];
      if (tpl) {
        setPopupTemplateId(tpl.id);
        setPopupEnabled(tpl.is_active);
        try {
          const parsed = JSON.parse(tpl.content);
          if (parsed.elements) setPopupElements(parsed.elements);
        } catch { /* ignore */ }
      }
    }).catch(console.error);
  }, []);

  const updatePreview = useCallback(async (elements: PrintElement[], surface: "popup" | "receipt") => {
    if (elements.length === 0) {
      if (surface === "popup") setPopupPreview("");
      else setReceiptPreview("");
      return;
    }
    try {
      const content = JSON.stringify({ elements });
      const result = await invoke<{ html: string }>("render_template_content_preview", {
        content,
        paperSize: "58mm",
        theme: "classic",
        restaurantName: "",
        tagline: "",
        logoData: null,
        data: { order_id: 1001, table_no: "3", items: [] },
      });
      if (surface === "popup") setPopupPreview(result.html);
      else setReceiptPreview(result.html);
    } catch { /* ignore */ }
  }, []);

  useEffect(() => { updatePreview(popupElements, "popup"); }, [popupElements, updatePreview]);
  useEffect(() => { updatePreview(receiptElements, "receipt"); }, [receiptElements, updatePreview]);

  useEffect(() => {
    invoke<RedemptionRecord[]>("get_marketing_redemptions", {}).then(setRedemptions).catch(console.error);
  }, []);

  const [funnel, setFunnel] = useState<MarketingFunnel | null>(null);
  useEffect(() => {
    invoke<MarketingFunnel>("get_marketing_funnel", { days: 7 }).then(setFunnel).catch(console.error);
  }, []);

  async function handleVerify() {
    if (!verifyOrderNo.trim()) return;
    setVerifying(true);
    setVerifyResult(null);
    try {
      // Find order by order_no
      const orders = await invoke<{ id: number; order_no: string }[]>("get_orders", { limit: 500, offset: 0 });
      const order = orders.find((o) => o.order_no === verifyOrderNo.trim() || String(o.id) === verifyOrderNo.trim());
      if (!order) { toast.error("找不到该订单号，请检查后重试"); return; }
      const popup = await invoke<{ order_no: string; template_content: string }>("get_marketing_popup", {
        orderId: order.id, tableNo: "",
      });
      const previewResult = await invoke<{ html: string }>("render_template_content_preview", {
        content: popup.template_content, paperSize: "58mm", theme: "classic",
        restaurantName: "", tagline: "", logoData: null,
        data: { order_id: order.id, table_no: "", items: [] },
      });
      const redemptions_for_order = redemptions.filter(r => r.order_id === order.id);
      setVerifyResult({
        order_no: order.order_no,
        popup_html: previewResult.html,
        already_redeemed: redemptions_for_order.length > 0,
      });
    } catch (e) {
      toast.error("查询失败", { description: String(e) });
    } finally {
      setVerifying(false);
    }
  }

  async function handleRedeem(componentType: string) {
    if (!verifyResult) return;
    const order = await invoke<{ id: number; order_no: string }[]>("get_orders", { limit: 500, offset: 0 })
      .then(os => os.find(o => o.order_no === verifyResult.order_no));
    if (!order) return;
    try {
      await invoke("record_marketing_redemption", {
        orderId: order.id, componentType, note: null, staffName: null,
      });
      const updated = await invoke<RedemptionRecord[]>("get_marketing_redemptions", {});
      setRedemptions(updated);
      setVerifyResult(prev => prev ? { ...prev, already_redeemed: true } : null);
      toast.success(`已标记「${COMPONENT_LABELS[componentType] ?? componentType}」兑奖完成`);
    } catch (e) {
      toast.error("记录失败", { description: String(e) });
    }
  }

  async function savePopupTemplate() {
    setSaving(true);
    try {
      const content = JSON.stringify({ elements: popupElements });
      if (popupTemplateId) {
        await invoke("update_print_template", {
          id: popupTemplateId, content, isActive: popupEnabled,
        });
      } else {
        await invoke("create_print_template", {
          req: {
            name: "自助点单行销弹窗",
            template_type: "marketing_popup",
            paper_size: "58mm",
            content,
            is_active: popupEnabled,
          }
        });
      }
      toast.success("行銷彈窗设定已保存");
    } catch (e) {
      toast.error("保存失敗", { description: String(e) });
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="flex flex-col h-full">
      <div className="border-b px-6 py-4 flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold flex items-center gap-2">
            <Zap className="h-5 w-5 text-amber-500" />行銷中心
          </h1>
          <p className="text-sm text-muted-foreground">
            統一管理收据、厨房单、自助点单确认页的行销元件
          </p>
        </div>
        <Button onClick={savePopupTemplate} disabled={saving} size="sm">
          <Save className="h-4 w-4 mr-1" />{saving ? "保存中..." : "保存设定"}
        </Button>
      </div>

      <div className="flex-1 overflow-y-auto p-6">
        <Tabs defaultValue="popup">
          <TabsList className="mb-6">
            <TabsTrigger value="popup" className="gap-1.5">
              <Smartphone className="h-3.5 w-3.5" />自助点单弹窗
            </TabsTrigger>
            <TabsTrigger value="receipt" className="gap-1.5">
              <Printer className="h-3.5 w-3.5" />收据 / 厨房单
            </TabsTrigger>
            <TabsTrigger value="guide" className="gap-1.5">
              <Sparkles className="h-3.5 w-3.5" />元件说明
            </TabsTrigger>
            <TabsTrigger value="redeem" className="gap-1.5">
              <ClipboardCheck className="h-3.5 w-3.5" />兑奖核销
              {redemptions.length > 0 && (
                <span className="ml-1 bg-green-100 text-green-700 text-[10px] px-1.5 rounded-full">{redemptions.length}</span>
              )}
            </TabsTrigger>
            <TabsTrigger value="analytics" className="gap-1.5">
              <BarChart3 className="h-3.5 w-3.5" />数据分析
            </TabsTrigger>
            <TabsTrigger value="campaign" className="gap-1.5">
              <Megaphone className="h-3.5 w-3.5" />扫码活动
            </TabsTrigger>
          </TabsList>

          <TabsContent value="popup">
            <Card>
              <CardHeader>
                <CardTitle className="text-base">自助點單确认頁彈窗</CardTitle>
                <CardDescription>
                  顧客下單後全屏彈出，截圖可見订单號與時間，集字/运势一單一次，截圖後廢棄兑奖
                </CardDescription>
              </CardHeader>
              <CardContent>
                <SurfacePanel
                  title="自助点单行销弹窗"
                  icon={<Smartphone className="h-4 w-4 text-blue-500" />}
                  description="顧客下單成功後彈出，截圖友善，支援 Web Share API 分享"
                  elements={popupElements}
                  enabled={popupEnabled}
                  onToggleEnabled={setPopupEnabled}
                  onAddElement={e => setPopupElements(prev => [...prev, e])}
                  onRemoveElement={idx => setPopupElements(prev => prev.filter((_, i) => i !== idx))}
                  previewHtml={popupPreview}
                />
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="receipt">
            <Card>
              <CardHeader>
                <CardTitle className="text-base">收据 / 厨房单行銷元件</CardTitle>
                <CardDescription>
                  在「打印中心」的模板中新增行銷元件；此頁提供快速预览與常用配置
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex items-center gap-2 p-3 rounded-lg bg-amber-50 border border-amber-200 text-sm text-amber-800">
                  <Sparkles className="h-4 w-4 shrink-0" />
                  <span>收据行銷元件在「打印中心 → 模板 → 元素列表」中管理。此處僅预览效果。</span>
                </div>
                <SurfacePanel
                  title="收据行销预览"
                  icon={<Printer className="h-4 w-4 text-gray-500" />}
                  description="预览組合效果，實際修改请至打印中心"
                  elements={receiptElements}
                  enabled={receiptEnabled}
                  onToggleEnabled={setReceiptEnabled}
                  onAddElement={e => setReceiptElements(prev => [...prev, e])}
                  onRemoveElement={idx => setReceiptElements(prev => prev.filter((_, i) => i !== idx))}
                  previewHtml={receiptPreview}
                />
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="guide">
            <div className="space-y-4">
              {[
                { type: "fortune", title: "今日运势", desc: "大吉 / 中吉 / 小吉，每单唯一（per_order）或全桌同运（per_table）。心理学正强化，让顾客以好心情结束用餐。" },
                { type: "character_collect", title: "集字兑奖", desc: "每張收据/确认頁抽一個字，顧客截圖保存，集齊指定字組可兑换。截圖即憑據，一單一次，天然防偽。" },
                { type: "quote", title: "今日语录 / 诗句", desc: "中文古典詩詞、日文俳句、英文 Instagram 感短句輪替，增加收据可拍照性。" },
                { type: "art", title: "颜文字 / ASCII 艺术", desc: "复古 BBS 点阵感图块，( ˘◡˘ )♪ ʕ•ᴥ•ʔ ✿ 等，每天随机选一组，增加趣味性。" },
                { type: "discount_coupon", title: "折價券", desc: "動態生成订单唯一碼（12 hex），印在收据上，下次消費出示。適合做回頭客促銷。" },
                { type: "product_spotlight", title: "新品介绍", desc: "在收据尾部展示本周新品，引导顾客下次尝试。" },
                { type: "qr_code", title: "QR 碼", desc: "扫码加群、加好友、关注公众号，ESC/POS 原生 QR 指令，无需额外硬件。" },
                { type: "rich_text", title: "富文本", desc: "Markdown 格式：## 標題、- 清單、> 引用。可做活動說明、集字規則等。" },
              ].map(item => (
                <Card key={item.type}>
                  <CardContent className="pt-4 pb-4 flex gap-3">
                    <span className={`mt-0.5 px-2 py-0.5 rounded text-[10px] font-medium h-fit shrink-0 ${getElementBadgeColor(item.type)}`}>
                      {item.type}
                    </span>
                    <div>
                      <p className="text-sm font-medium">{item.title}</p>
                      <p className="text-xs text-muted-foreground mt-0.5 leading-relaxed">{item.desc}</p>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          </TabsContent>

          {/* 兑奖核销 Tab */}
          <TabsContent value="redeem" className="space-y-4">
            {/* 验码区 */}
            <Card>
              <CardHeader>
                <CardTitle className="text-base">验证订单行销内容</CardTitle>
                <CardDescription>
                  顾客出示截图时，输入订单号或ID，系统重新渲染该单的行销内容供核对
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-3">
                {/* D3: 扫码枪/粘贴核销码 直接核销 */}
                <div className="flex gap-2">
                  <Input
                    placeholder="扫码核销：粘贴核销码或扫码链接"
                    value={redeemCode}
                    onChange={e => setRedeemCode(e.target.value)}
                    onKeyDown={e => e.key === "Enter" && handleRedeemByCode()}
                    className="flex-1"
                  />
                  <Button variant="secondary" onClick={handleRedeemByCode} disabled={redeeming || !redeemCode.trim()}>
                    <ClipboardCheck className="h-4 w-4 mr-1" />{redeeming ? "核销中..." : "核销"}
                  </Button>
                </div>
                <div className="relative text-center"><span className="text-[10px] text-muted-foreground bg-background px-2">或按订单号查询</span><div className="absolute inset-x-0 top-1/2 -z-10 border-t" /></div>
                <div className="flex gap-2">
                  <Input
                    placeholder="输入订单号（如 SO20260601123456）或订单ID"
                    value={verifyOrderNo}
                    onChange={e => setVerifyOrderNo(e.target.value)}
                    onKeyDown={e => e.key === "Enter" && handleVerify()}
                    className="flex-1"
                  />
                  <Button onClick={handleVerify} disabled={verifying || !verifyOrderNo.trim()}>
                    <Search className="h-4 w-4 mr-1" />{verifying ? "查询中..." : "查询"}
                  </Button>
                </div>
                {verifyResult && (
                  <div className="border rounded-lg overflow-hidden">
                    <div className={`flex items-center justify-between px-4 py-2 text-sm font-medium ${verifyResult.already_redeemed ? "bg-green-50 text-green-800" : "bg-amber-50 text-amber-800"}`}>
                      <span>订单：{verifyResult.order_no}</span>
                      <span>{verifyResult.already_redeemed ? "✅ 已兑奖" : "⏳ 未兑奖"}</span>
                    </div>
                    <div className="p-3 max-h-64 overflow-y-auto"
                      dangerouslySetInnerHTML={{ __html: DOMPurify.sanitize(verifyResult.popup_html ?? "") }} />
                    {!verifyResult.already_redeemed && (
                      <div className="px-4 pb-4 pt-2 border-t bg-muted/20">
                        <p className="text-xs text-muted-foreground mb-2">与顾客截图核对无误后，选择要标记为已兑奖的组件：</p>
                        <div className="flex flex-wrap gap-2">
                          {["character_collect", "fortune", "discount_coupon", "riddle"].map(ct => (
                            <Button key={ct} variant="outline" size="sm" className="text-xs"
                              onClick={() => handleRedeem(ct)}>
                              标记「{COMPONENT_LABELS[ct] ?? ct}」已兑奖
                            </Button>
                          ))}
                        </div>
                      </div>
                    )}
                  </div>
                )}
              </CardContent>
            </Card>

            {/* 兑奖记录 */}
            <Card>
              <CardHeader>
                <CardTitle className="text-base">兑奖记录</CardTitle>
                <CardDescription>最近 100 条兑奖记录</CardDescription>
              </CardHeader>
              <CardContent>
                {redemptions.length === 0 ? (
                  <p className="text-sm text-muted-foreground text-center py-4">暂无兑奖记录</p>
                ) : (
                  <div className="space-y-1">
                    {redemptions.map(r => (
                      <div key={r.id} className="flex items-center gap-3 text-xs border rounded px-3 py-2">
                        <span className="font-mono text-muted-foreground shrink-0">{r.order_no || `#${r.order_id}`}</span>
                        <span className="bg-green-100 text-green-800 px-1.5 py-0.5 rounded shrink-0">
                          {COMPONENT_LABELS[r.component_type] ?? r.component_type}
                        </span>
                        <span className="flex-1 text-muted-foreground truncate">{r.note ?? ""}</span>
                        <span className="text-muted-foreground shrink-0">{r.redeemed_at.slice(0, 16)}</span>
                      </div>
                    ))}
                  </div>
                )}
              </CardContent>
            </Card>
          </TabsContent>

          {/* 数据分析 Tab */}
          <TabsContent value="analytics" className="space-y-4">
            <Card>
              <CardHeader>
                <CardTitle className="text-base">扫码营销漏斗（近 7 天）</CardTitle>
                <CardDescription>扫码进店 → 自助下单 → 营销核销的转化情况</CardDescription>
              </CardHeader>
              <CardContent>
                {!funnel ? (
                  <p className="text-sm text-muted-foreground text-center py-4">加载中…</p>
                ) : (
                  <div className="space-y-4">
                    <div className="grid grid-cols-3 gap-3">
                      <div className="rounded-lg border p-3 text-center">
                        <div className="text-2xl font-bold text-blue-600">{funnel.scans}</div>
                        <div className="text-xs text-muted-foreground mt-1">扫码进店</div>
                      </div>
                      <div className="rounded-lg border p-3 text-center">
                        <div className="text-2xl font-bold text-orange-600">{funnel.self_orders}</div>
                        <div className="text-xs text-muted-foreground mt-1">自助下单</div>
                      </div>
                      <div className="rounded-lg border p-3 text-center">
                        <div className="text-2xl font-bold text-green-600">{funnel.redemptions}</div>
                        <div className="text-xs text-muted-foreground mt-1">营销核销</div>
                      </div>
                    </div>
                    <div className="flex items-center justify-between text-sm px-1">
                      <span className="text-muted-foreground">扫码 → 下单转化率</span>
                      <span className="font-bold text-gray-900">{(funnel.scan_to_order * 100).toFixed(0)}%</span>
                    </div>
                    <div>
                      <p className="text-xs font-medium text-muted-foreground mb-2">各组件核销分布</p>
                      {funnel.by_component.length === 0 ? (
                        <p className="text-xs text-muted-foreground text-center py-3">暂无核销数据</p>
                      ) : (
                        <div className="space-y-1.5">
                          {funnel.by_component.map((c) => {
                            const max = Math.max(...funnel.by_component.map((x) => x.count), 1);
                            return (
                              <div key={c.component} className="flex items-center gap-2">
                                <span className="text-xs w-20 shrink-0 text-muted-foreground">
                                  {COMPONENT_LABELS[c.component] ?? c.component}
                                </span>
                                <div className="flex-1 bg-gray-100 rounded-full h-4 overflow-hidden">
                                  <div className="bg-green-400 h-full rounded-full" style={{ width: `${(c.count / max) * 100}%` }} />
                                </div>
                                <span className="text-xs font-medium w-8 text-right">{c.count}</span>
                              </div>
                            );
                          })}
                        </div>
                      )}
                    </div>
                  </div>
                )}
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="campaign">
            <CampaignManager />
          </TabsContent>
        </Tabs>
      </div>
    </div>
  );
}
