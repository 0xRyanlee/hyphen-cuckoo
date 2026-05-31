import { useState, useCallback, useEffect } from "react";
import { call as invoke } from "@/lib/transport";
import DOMPurify from "dompurify";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Switch } from "@/components/ui/switch";
import { Label } from "@/components/ui/label";
import { Save, Eye, Smartphone, Printer, Sparkles, Zap } from "lucide-react";
import { toast } from "sonner";
import { ELEMENT_CATEGORIES, getElementLabel, getElementBadgeColor, getElementSummary, type PrintElement } from "./print-templates-page";

// ── Types ──────────────────────────────────────────────────────────────────

// ── Default element presets ────────────────────────────────────────────────

const DEFAULT_POPUP_ELEMENTS: PrintElement[] = [
  { type: "fortune", seed_strategy: "per_order" },
  { type: "character_collect", game_name: "集字兌獎", characters: ["恭", "喜", "發", "財"], prize: "集齊四字兌換免費飲品", seed_strategy: "per_order", style: "box" },
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
          <Label className="text-xs text-muted-foreground">{enabled ? "啟用" : "停用"}</Label>
          <Switch checked={enabled} onCheckedChange={onToggleEnabled} />
        </div>
      </div>

      {enabled && (
        <>
          <div className="space-y-1.5">
            {elements.length === 0 && (
              <p className="text-xs text-muted-foreground py-2 text-center border rounded-md">
                尚無行銷元件 — 點擊下方添加
              </p>
            )}
            {elements.map((elem, idx) => (
              <ElementToggleCard key={idx} elem={elem} onRemove={() => onRemoveElement(idx)} />
            ))}
            <Button variant="outline" size="sm" className="w-full h-7 text-xs gap-1 mt-1"
              onClick={() => setPickerOpen(!pickerOpen)}>
              <Sparkles className="h-3 w-3" />添加行銷元件
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
                <span className="text-xs text-muted-foreground">預覽</span>
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
            name: "自助點單行銷彈窗",
            template_type: "marketing_popup",
            paper_size: "58mm",
            content,
            is_active: popupEnabled,
          }
        });
      }
      toast.success("行銷彈窗設定已保存");
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
            統一管理收據、廚房單、自助點單確認頁的行銷元件
          </p>
        </div>
        <Button onClick={savePopupTemplate} disabled={saving} size="sm">
          <Save className="h-4 w-4 mr-1" />{saving ? "保存中..." : "保存設定"}
        </Button>
      </div>

      <div className="flex-1 overflow-y-auto p-6">
        <Tabs defaultValue="popup">
          <TabsList className="mb-6">
            <TabsTrigger value="popup" className="gap-1.5">
              <Smartphone className="h-3.5 w-3.5" />自助點單彈窗
            </TabsTrigger>
            <TabsTrigger value="receipt" className="gap-1.5">
              <Printer className="h-3.5 w-3.5" />收據 / 廚房單
            </TabsTrigger>
            <TabsTrigger value="guide" className="gap-1.5">
              <Sparkles className="h-3.5 w-3.5" />元件說明
            </TabsTrigger>
          </TabsList>

          <TabsContent value="popup">
            <Card>
              <CardHeader>
                <CardTitle className="text-base">自助點單確認頁彈窗</CardTitle>
                <CardDescription>
                  顧客下單後全屏彈出，截圖可見訂單號與時間，集字/運勢一單一次，截圖後廢棄兌獎
                </CardDescription>
              </CardHeader>
              <CardContent>
                <SurfacePanel
                  title="自助點單行銷彈窗"
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
                <CardTitle className="text-base">收據 / 廚房單行銷元件</CardTitle>
                <CardDescription>
                  在「打印中心」的模板中新增行銷元件；此頁提供快速預覽與常用配置
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex items-center gap-2 p-3 rounded-lg bg-amber-50 border border-amber-200 text-sm text-amber-800">
                  <Sparkles className="h-4 w-4 shrink-0" />
                  <span>收據行銷元件在「打印中心 → 模板 → 元素列表」中管理。此處僅預覽效果。</span>
                </div>
                <SurfacePanel
                  title="收據行銷預覽"
                  icon={<Printer className="h-4 w-4 text-gray-500" />}
                  description="預覽組合效果，實際修改請至打印中心"
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
                { type: "fortune", title: "今日運勢", desc: "大吉 / 中吉 / 小吉，每單唯一（per_order）或全桌同運（per_table）。心理學正強化，讓顧客以好心情結束用餐。" },
                { type: "character_collect", title: "集字兌獎", desc: "每張收據/確認頁抽一個字，顧客截圖保存，集齊指定字組可兌換。截圖即憑據，一單一次，天然防偽。" },
                { type: "quote", title: "今日語錄 / 詩句", desc: "中文古典詩詞、日文俳句、英文 Instagram 感短句輪替，增加收據可拍照性。" },
                { type: "art", title: "顏文字 / ASCII 藝術", desc: "復古 BBS 點陣感圖塊，( ˘◡˘ )♪ ʕ•ᴥ•ʔ ✿ 等，每天隨機選一組，增加趣味性。" },
                { type: "discount_coupon", title: "折價券", desc: "動態生成訂單唯一碼（12 hex），印在收據上，下次消費出示。適合做回頭客促銷。" },
                { type: "product_spotlight", title: "新品介紹", desc: "在收據尾部展示本週新品，引導顧客下次嘗試。" },
                { type: "qr_code", title: "QR 碼", desc: "掃碼加群、加好友、關注公眾號，ESC/POS 原生 QR 指令，無需額外硬件。" },
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
        </Tabs>
      </div>
    </div>
  );
}
