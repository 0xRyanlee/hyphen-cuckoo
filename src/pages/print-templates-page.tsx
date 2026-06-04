import { useState, useEffect, useCallback, useRef, useMemo } from "react";
import { call as invoke } from "@/lib/transport";
import DOMPurify from "dompurify";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Checkbox } from "@/components/ui/checkbox";
import { Plus, Pencil, Trash2, Eye, Save, Printer, Star, FileBox, ChevronUp, ChevronDown, GripVertical } from "lucide-react";
import { EmptyState } from "@/components/ui/empty-state";
import { toast } from "sonner";

// ── Element types for visual card layer ──────────────────────────────────────

export type PrintElement = Record<string, unknown> & { type: string };

export const ELEMENT_CATEGORIES = [
  {
    label: "基本", items: [
      { type: "text", label: "文字", defaultConfig: { type: "text", content: "文字内容", align: "left", bold: false, size: "normal" } },
      { type: "separator", label: "分隔线", defaultConfig: { type: "separator" } },
      { type: "blank_lines", label: "空行", defaultConfig: { type: "blank_lines", count: 1 } },
      { type: "items", label: "菜品明细", defaultConfig: { type: "items" } },
    ]
  },
  {
    label: "创意", items: [
      { type: "fortune", label: "今日运势", defaultConfig: { type: "fortune", seed_strategy: "per_table" } },
      { type: "quote", label: "今日语录", defaultConfig: { type: "quote", language: "multilingual" } },
      { type: "art", label: "艺术图块", defaultConfig: { type: "art", variant: "random" } },
      { type: "image_block", label: "图像占位", defaultConfig: { type: "image_block" } },
    ]
  },
  {
    label: "营销", items: [
      { type: "discount_coupon", label: "折价券", defaultConfig: { type: "discount_coupon", discount_type: "percent", value: 9, condition: "", valid_days: 30, label: "下次消费享折扣" } },
      { type: "product_spotlight", label: "新品介绍", defaultConfig: { type: "product_spotlight", title: "本周新品", name: "", description: "", price: 0, badge: "NEW" } },
      { type: "qr_code", label: "QR码", defaultConfig: { type: "qr_code", url: "", label: "扫码加入VIP群", size: 5 } },
      { type: "character_collect", label: "集字兑奖", defaultConfig: { type: "character_collect", game_name: "集字兑奖", characters: ["恭", "喜", "发", "财"], prize: "集齐四字兑换免费饮品", seed_strategy: "per_order", style: "box" } },
      { type: "rich_text", label: "富文本", defaultConfig: { type: "rich_text", content: "## 今日特惠\n- 项目一\n- 项目二" } },
    ]
  },
  {
    label: "情境", items: [
      { type: "solar_term", label: "节气主题", defaultConfig: { type: "solar_term", show_all: false } },
      { type: "chef_message", label: "厨师寄语", defaultConfig: { type: "chef_message", title: "厨师寄语", author: "本店厨师", messages: [] } },
      { type: "riddle", label: "谜语挑战", defaultConfig: { type: "riddle", prize: "下次来店说出答案，赢取小惊喜！" } },
      { type: "dish_easter_egg", label: "订单彩蛋", defaultConfig: { type: "dish_easter_egg", eggs: [{ keyword: "虾", message: "解锁：海鲜达人称号！下次点虾享95折" }] } },
    ]
  },
];

export function getElementLabel(type: string): string {
  for (const cat of ELEMENT_CATEGORIES) for (const it of cat.items) if (it.type === type) return it.label;
  return type;
}

export function getElementBadgeColor(type: string): string {
  const creative = ["fortune", "quote", "art", "image_block"];
  const marketing = ["discount_coupon", "product_spotlight", "qr_code", "character_collect", "rich_text"];
  if (creative.includes(type)) return "bg-primary/10 text-primary";
  if (marketing.includes(type)) return "bg-accent text-accent-foreground";
  return "bg-muted text-muted-foreground";
}

export function getElementSummary(elem: PrintElement): string {
  switch (elem.type) {
    case "text": return String(elem.content ?? "").slice(0, 30) || "(空)";
    case "separator": return "──────";
    case "blank_lines": return `${elem.count ?? 1} 行空白`;
    case "items": return "订单菜品明细";
    case "fortune": return `运势 (${elem.seed_strategy ?? "daily"})`;
    case "quote": return `语录 (${elem.language ?? "multilingual"})`;
    case "art": return `ASCII 艺术 (${elem.variant ?? "random"})`;
    case "image_block": return "图像占位";
    case "discount_coupon": return `${elem.discount_type === "amount" ? `立減${elem.value}元` : `${elem.value}% off`}，${elem.valid_days}天有效`;
    case "product_spotlight": return String(elem.name || elem.title || "新品介绍");
    case "qr_code": return String(elem.label || elem.url || "QR Code");
    case "character_collect": return `${elem.game_name} — ${(elem.characters as string[] | undefined)?.join("") ?? ""}`;
    case "rich_text": return String(elem.content ?? "").split("\n")[0].replace(/^#+\s*/, "").slice(0, 30);
    case "solar_term": return "节气期间自动显示主题文案";
    case "dish_easter_egg": { const eggs = elem.eggs as { keyword: string; message: string }[] | undefined; return `${eggs?.length ?? 0}个触发条件（菜品关键词）`; }
    case "chef_message": return `${String(elem.title ?? "厨师寄语")} · ${(elem.messages as string[] | undefined)?.length ?? 0}条自定义`;
    case "riddle": return `今日谜语 → ${String(elem.prize ?? "赢取小惊喜")}`;
    default: return elem.type;
  }
}

interface PrintTemplate {
  id: number;
  name: string;
  template_type: string;
  paper_size: string;
  label_width_mm: number | null;
  label_height_mm: number | null;
  content: string;
  is_default: boolean;
  is_active: boolean;
  theme: string | null;
  restaurant_name: string | null;
  tagline: string | null;
  logo_data: string | null;
  show_price: boolean | null;
  show_tax: boolean | null;
  show_service_charge: boolean | null;
  item_sort: string | null;
  modifiers_color: string | null;
  created_at: string;
  updated_at: string;
}

interface PrintTemplatesPageProps {
  onPreview?: (templateId: number, data: Record<string, unknown>) => void;
}

interface PreviewResult {
  html: string;
  lines: string[];
  paper_width: string;
}

export function PrintTemplatesPage(_props: PrintTemplatesPageProps) {
  const [templates, setTemplates] = useState<PrintTemplate[]>([]);
  const [selectedType, setSelectedType] = useState<string>("all");
  const [editDialogOpen, setEditDialogOpen] = useState(false);
  const [previewDialogOpen, setPreviewDialogOpen] = useState(false);
  const [editingTemplate, setEditingTemplate] = useState<PrintTemplate | null>(null);
  const [previewHtml, setPreviewHtml] = useState("");
  const [previewLines, setPreviewLines] = useState<string[]>([]);
  const [livePreviewHtml, setLivePreviewHtml] = useState("");
  const [livePreviewError, setLivePreviewError] = useState("");

  const [formName, setFormName] = useState("");
  const [formType, setFormType] = useState("kitchen_ticket");
  const [formPaperSize, setFormPaperSize] = useState("80mm");
  const [formLabelWidth, setFormLabelWidth] = useState("");
  const [formLabelHeight, setFormLabelHeight] = useState("");
  const [formContent, setFormContent] = useState("");
  const [formTheme, setFormTheme] = useState("classic");
  const [formRestaurantName, setFormRestaurantName] = useState("");
  const [formTagline, setFormTagline] = useState("");
  const [formLogoData, setFormLogoData] = useState("");
  const [formShowPrice, setFormShowPrice] = useState(true);
  const [formShowTax, setFormShowTax] = useState(true);
  const [formShowServiceCharge, setFormShowServiceCharge] = useState(true);
  const [formItemSort, setFormItemSort] = useState("entry");
  const [formModifiersColor, setFormModifiersColor] = useState("red");
  const [formIsActive, setFormIsActive] = useState(true);

  const [elementPickerOpen, setElementPickerOpen] = useState(false);
  const [elementEditorOpen, setElementEditorOpen] = useState(false);
  const [editingIdx, setEditingIdx] = useState<number | null>(null);
  const [editingElem, setEditingElem] = useState<PrintElement | null>(null);

  // Derived: parsed elements from formContent JSON (memoized to avoid repeated parsing)
  const parsedElements = useMemo<PrintElement[]>(() => {
    try { return JSON.parse(formContent || '{"elements":[]}').elements ?? []; } catch { return []; }
  }, [formContent]);

  // Stable callback: apply updated element array back to formContent + live preview
  const applyElements = useCallback((els: PrintElement[]) => {
    try {
      const parsed = JSON.parse(formContent || '{"elements":[]}');
      parsed.elements = els;
      const updated = JSON.stringify(parsed, null, 2);
      setFormContent(updated);
      updateLivePreview(updated, formPaperSize, formTheme, formRestaurantName, formTagline, formLogoData);
    } catch { /* invalid JSON — no-op */ }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [formContent, formPaperSize, formTheme, formRestaurantName, formTagline, formLogoData]);

  const previewTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  async function loadTemplates() {
    try {
      const type = selectedType === "all" ? undefined : selectedType;
      const result = await invoke<PrintTemplate[]>("get_print_templates", { templateType: type });
      setTemplates(result);
    } catch (e) {
      console.error("载入模板失败:", e);
      toast.error("载入模板失败", { description: String(e) });
    }
  }

  useEffect(() => { loadTemplates(); }, [selectedType]);

  function openNew() {
    setEditingTemplate(null);
    setFormName("");
    setFormType("kitchen_ticket");
    setFormPaperSize("80mm");
    setFormLabelWidth("");
    setFormLabelHeight("");
    setFormContent(JSON.stringify(defaultKitchenTemplate, null, 2));
    setFormTheme("classic");
    setFormRestaurantName("");
    setFormTagline("");
    setFormLogoData("");
    setFormShowPrice(true);
    setFormShowTax(true);
    setFormShowServiceCharge(true);
    setFormItemSort("entry");
    setFormModifiersColor("red");
    setFormIsActive(true);
    setLivePreviewHtml("");
    setLivePreviewError("");
    setEditDialogOpen(true);
  }

  function openEdit(tpl: PrintTemplate) {
    setEditingTemplate(tpl);
    setFormName(tpl.name);
    setFormType(tpl.template_type);
    setFormPaperSize(tpl.paper_size);
    setFormLabelWidth(tpl.label_width_mm?.toString() || "");
    setFormLabelHeight(tpl.label_height_mm?.toString() || "");
    setFormContent(tpl.content);
    setFormTheme(tpl.theme || "classic");
    setFormRestaurantName(tpl.restaurant_name || "");
    setFormTagline(tpl.tagline || "");
    setFormLogoData(tpl.logo_data || "");
    setFormShowPrice(tpl.show_price ?? true);
    setFormShowTax(tpl.show_tax ?? true);
    setFormShowServiceCharge(tpl.show_service_charge ?? true);
    setFormItemSort(tpl.item_sort || "entry");
    setFormModifiersColor(tpl.modifiers_color || "red");
    setFormIsActive(tpl.is_active);
    setLivePreviewHtml("");
    setLivePreviewError("");
    setEditDialogOpen(true);
    // 開啟時立即触发預覽，不等用戶修改
    updateLivePreview(
      tpl.content,
      tpl.paper_size,
      tpl.theme || "classic",
      tpl.restaurant_name || "",
      tpl.tagline || "",
      tpl.logo_data || "",
    );
  }

  async function saveTemplate() {
    if (!formName.trim()) {
      toast.error("请填写模板名称");
      return;
    }
    try {
      if (editingTemplate) {
        await invoke("update_print_template", {
          id: editingTemplate.id,
          name: formName,
          content: formContent,
          paperSize: formPaperSize,
          labelWidthMm: formLabelWidth ? parseFloat(formLabelWidth) : null,
          labelHeightMm: formLabelHeight ? parseFloat(formLabelHeight) : null,
          theme: formTheme,
          restaurantName: formRestaurantName || null,
          tagline: formTagline || null,
          logoData: formLogoData || null,
          showPrice: formShowPrice,
          showTax: formShowTax,
          showServiceCharge: formShowServiceCharge,
          itemSort: formItemSort,
          modifiersColor: formModifiersColor,
          isActive: formIsActive,
        });
        toast.success("模板已更新");
      } else {
        await invoke("create_print_template", {
          req: {
            name: formName,
            template_type: formType,
            paper_size: formPaperSize,
            label_width_mm: formLabelWidth ? parseFloat(formLabelWidth) : null,
            label_height_mm: formLabelHeight ? parseFloat(formLabelHeight) : null,
            content: formContent,
            theme: formTheme,
            restaurant_name: formRestaurantName || null,
            tagline: formTagline || null,
            logo_data: formLogoData || null,
            show_price: formShowPrice,
            show_tax: formShowTax,
            show_service_charge: formShowServiceCharge,
            item_sort: formItemSort,
            modifiers_color: formModifiersColor,
            is_active: formIsActive,
          },
        });
        toast.success("模板已创建");
      }
      setEditDialogOpen(false);
      loadTemplates();
    } catch (e) {
      toast.error("保存模板失败", { description: String(e) });
    }
  }

  async function deleteTemplate(id: number) {
    try {
      await invoke("delete_print_template", { id });
      loadTemplates();
    } catch (e) {
      console.error("删除模板失败:", e);
    }
  }

  async function setDefault(id: number, type: string) {
    try {
      await invoke("set_default_template", { id, templateType: type });
      loadTemplates();
    } catch (e) {
      console.error("设置默认失败:", e);
    }
  }

  async function previewTemplate(tpl: PrintTemplate) {
    const sampleData = tpl.template_type === "kitchen_ticket"
      ? { order_no: "ORD20260423001", dine_type: "堂食", time: "2026-04-23 14:30", items: [{ name: "宮保雞丁", qty: 2, note: "少辣" }, { name: "麻婆豆腐", qty: 1, note: null }], note: "加急" }
      : { lot_no: "LOT-20260423-001", material_name: "雞胸肉", quantity: 10, unit: "kg", expiry_date: "2026-05-01", supplier_name: "鲜肉供应商" };

    try {
      const result = await invoke<PreviewResult>("render_template_preview", { templateId: tpl.id, data: sampleData });
      setPreviewHtml(result.html);
      setPreviewLines(result.lines);
      setPreviewDialogOpen(true);
    } catch (e) {
      console.error("预览失败:", e);
    }
  }

  const updateLivePreview = useCallback(async (
    content: string,
    paperSize: string,
    theme: string,
    restaurantName: string,
    tagline: string,
    logoData: string,
  ) => {
    if (previewTimerRef.current) {
      clearTimeout(previewTimerRef.current);
    }

    previewTimerRef.current = setTimeout(async () => {
      try {
        const sampleData = {
          order_no: "ORD20260424001",
          dine_type: "堂食",
          time: "2026-04-24 14:30",
          items: [
            { name: "宮保雞丁", qty: 2, note: "少辣" },
            { name: "麻婆豆腐", qty: 1, note: null },
            { name: "酸菜魚", qty: 1, note: "加辣" },
          ],
          note: "加急處理",
        };

        const result = await invoke<PreviewResult>("render_template_content_preview", {
          content,
          paperSize,
          theme,
          restaurantName,
          tagline,
          logoData: logoData || null,
          data: sampleData,
        });
        setLivePreviewHtml(result.html);
        setLivePreviewError("");
      } catch (e: any) {
        setLivePreviewHtml("");
        setLivePreviewError(e.toString());
      }
    }, 400);
  }, []);

  const getTypeLabel = (type: string) => {
    switch (type) {
      case "kitchen_ticket": return "厨房单";
      case "batch_label": return "批次标签";
      case "cup_label": return "杯贴";
      case "receipt": return "收据";
      case "marketing_popup": return "营销弹窗";
      default: return type;
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">打印模板</h2>
          <p className="text-sm text-muted-foreground">管理厨房单、标签等打印模板</p>
        </div>
        <Button onClick={openNew}><Plus className="mr-2 h-4 w-4" />新增模板</Button>
      </div>

      <div className="flex items-center gap-2">
        <Select value={selectedType} onValueChange={setSelectedType}>
          <SelectTrigger className="w-40"><SelectValue placeholder="类型筛选" /></SelectTrigger>
          <SelectContent>
            <SelectItem value="all">全部类型</SelectItem>
            <SelectItem value="kitchen_ticket">厨房单</SelectItem>
            <SelectItem value="batch_label">批次标签</SelectItem>
            <SelectItem value="cup_label">杯贴</SelectItem>
            <SelectItem value="receipt">收据</SelectItem>
            <SelectItem value="marketing_popup">营销弹窗</SelectItem>
          </SelectContent>
        </Select>
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        {templates.map((tpl) => (
          <Card key={tpl.id} className="relative">
            <CardHeader>
              <div className="flex items-start justify-between">
                <div>
                  <CardTitle className="flex items-center gap-2 text-base">
                    {tpl.is_default && <Star className="h-3.5 w-3.5 text-amber-500 fill-amber-500" />}
                    {tpl.name}
                  </CardTitle>
                  <CardDescription>{getTypeLabel(tpl.template_type)} · {tpl.paper_size}</CardDescription>
                </div>
              </div>
            </CardHeader>
            <CardContent>
              <div className="flex flex-wrap gap-1 mb-3">
                <Badge variant="secondary">{tpl.paper_size}</Badge>
                {tpl.label_width_mm && <Badge variant="outline">{tpl.label_width_mm}x{tpl.label_height_mm}mm</Badge>}
              </div>
              {tpl.template_type === "marketing_popup" && (
                <p className="text-xs text-muted-foreground mb-2 flex items-center gap-1">
                  <span className="inline-block w-1.5 h-1.5 rounded-full bg-amber-400 shrink-0" />
                  由营销中心管理，此处仅预览
                </p>
              )}
              <div className="flex gap-1">
                <Button variant="outline" size="sm" className="flex-1" onClick={() => previewTemplate(tpl)}>
                  <Eye className="h-3.5 w-3.5 mr-1" />预览
                </Button>
                {tpl.template_type !== "marketing_popup" && (
                  <>
                    <Button variant="outline" size="sm" onClick={() => openEdit(tpl)}><Pencil className="h-3.5 w-3.5" /></Button>
                    {!tpl.is_default && (
                      <Button variant="outline" size="sm" onClick={() => setDefault(tpl.id, tpl.template_type)}><Star className="h-3.5 w-3.5" /></Button>
                    )}
                    <Button variant="outline" size="sm" onClick={() => deleteTemplate(tpl.id)}><Trash2 className="h-3.5 w-3.5 text-destructive" /></Button>
                  </>
                )}
              </div>
            </CardContent>
          </Card>
        ))}
        {templates.length === 0 && (
          <EmptyState icon={FileBox} title="暂无模板" description="点击新增模板创建" action={<Button onClick={openNew}><Plus className="mr-2 h-4 w-4" />新增模板</Button>} className="col-span-full" />
        )}
      </div>

      <Dialog open={editDialogOpen} onOpenChange={(open) => { setEditDialogOpen(open); if (!open) { setLivePreviewHtml(""); setLivePreviewError(""); } }}>
        <DialogContent className="max-w-7xl max-h-[95vh]">
          <DialogHeader>
            <DialogTitle>{editingTemplate ? "编辑模板" : "新增模板"}</DialogTitle>
          </DialogHeader>
          <div className="flex flex-col lg:flex-row gap-4 h-[70vh]">
            {/* 左側：表單 */}
            <div className="flex-1 overflow-y-auto space-y-4 pr-2">
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label>模板名称</Label>
                  <Input value={formName} onChange={(e) => setFormName(e.target.value)} placeholder="如：标准厨房单" />
                </div>
                <div className="space-y-2">
                  <Label>模板類型</Label>
                  <Select value={formType} onValueChange={(v) => {
                    setFormType(v);
                    if (!editingTemplate) {
                      if (v === "kitchen_ticket") {
                        setFormContent(JSON.stringify(defaultKitchenTemplate, null, 2));
                      } else if (v === "batch_label") {
                        setFormContent(JSON.stringify(defaultBatchLabelTemplate, null, 2));
                      } else {
                        setFormContent(JSON.stringify(defaultKitchenTemplate, null, 2));
                      }
                    }
                    updateLivePreview(formContent, formPaperSize, formTheme, formRestaurantName, formTagline, formLogoData);
                  }}>
                    <SelectTrigger><SelectValue /></SelectTrigger>
                    <SelectContent>
                      <SelectItem value="kitchen_ticket">厨房单</SelectItem>
                      <SelectItem value="batch_label">批次标签</SelectItem>
                      <SelectItem value="receipt">收据</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </div>
              
              <Separator />
              
              <div className="space-y-2">
                <Label>纸张尺寸</Label>
                <Select value={formPaperSize} onValueChange={(v) => { setFormPaperSize(v); updateLivePreview(formContent, v, formTheme, formRestaurantName, formTagline, formLogoData); }}>
                  <SelectTrigger><SelectValue /></SelectTrigger>
                  <SelectContent>
                    <SelectItem value="58mm">58mm 热敏</SelectItem>
                    <SelectItem value="80mm">80mm 热敏</SelectItem>
                    <SelectItem value="custom">自定义标签</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              
              <div className="space-y-2">
                <Label>标签宽度 / 高度 (mm)</Label>
                <div className="grid grid-cols-2 gap-4">
                  <Input type="number" value={formLabelWidth} onChange={(e) => setFormLabelWidth(e.target.value)} placeholder="60" disabled={formPaperSize !== "custom"} />
                  <Input type="number" value={formLabelHeight} onChange={(e) => setFormLabelHeight(e.target.value)} placeholder="40" disabled={formPaperSize !== "custom"} />
                </div>
              </div>
              
              <Separator />
              
              <div className="space-y-2">
                <Label>主题风格</Label>
                <Select value={formTheme} onValueChange={(v) => { setFormTheme(v); updateLivePreview(formContent, formPaperSize, v, formRestaurantName, formTagline, formLogoData); }}>
                  <SelectTrigger><SelectValue /></SelectTrigger>
                  <SelectContent>
                    <SelectItem value="classic">经典 (Classic)</SelectItem>
                    <SelectItem value="minimal">简约 (Minimal)</SelectItem>
                    <SelectItem value="modern">现代 (Modern)</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label>餐厅名称</Label>
                  <Input value={formRestaurantName} onChange={(e) => { setFormRestaurantName(e.target.value); updateLivePreview(formContent, formPaperSize, formTheme, e.target.value, formTagline, formLogoData); }} placeholder="Cuckoo 餐厅" />
                </div>
                <div className="space-y-2">
                  <Label>标语 (Tagline)</Label>
                  <Input value={formTagline} onChange={(e) => { setFormTagline(e.target.value); updateLivePreview(formContent, formPaperSize, formTheme, formRestaurantName, e.target.value, formLogoData); }} placeholder="用心做好每一道菜" />
                </div>
              </div>
              
              <Separator />
              
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <Label>模板元素</Label>
                  <Button size="sm" variant="outline" className="h-7 text-xs gap-1" onClick={() => setElementPickerOpen(true)}>
                    <Plus className="h-3 w-3" />新增元件
                  </Button>
                </div>

                {/* Visual element card list — uses memoized parsedElements + stable applyElements */}
                <div className="space-y-1 min-h-[40px] border rounded-md p-2 bg-muted/20">
                  {parsedElements.length === 0 && (
                    <p className="text-xs text-muted-foreground text-center py-3">尚無元素 — 点击「新增元件」开始</p>
                  )}
                  {parsedElements.map((elem, idx) => (
                    <div key={idx} className="flex items-center gap-1.5 bg-background border rounded px-2 py-1 text-xs group">
                      <GripVertical className="h-3 w-3 text-muted-foreground/40 shrink-0" aria-hidden />
                      <span className={`px-1.5 py-0.5 rounded text-[10px] font-medium shrink-0 ${getElementBadgeColor(elem.type)}`}>{getElementLabel(elem.type)}</span>
                      <span className="flex-1 text-muted-foreground truncate">{getElementSummary(elem)}</span>
                      <div className="flex gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
                        <Button variant="ghost" size="icon" className="h-5 w-5" disabled={idx === 0} aria-label="上移"
                          onClick={() => { const els = [...parsedElements]; [els[idx-1], els[idx]] = [els[idx], els[idx-1]]; applyElements(els); }}>
                          <ChevronUp className="h-3 w-3" />
                        </Button>
                        <Button variant="ghost" size="icon" className="h-5 w-5" disabled={idx === parsedElements.length - 1} aria-label="下移"
                          onClick={() => { const els = [...parsedElements]; [els[idx], els[idx+1]] = [els[idx+1], els[idx]]; applyElements(els); }}>
                          <ChevronDown className="h-3 w-3" />
                        </Button>
                        <Button variant="ghost" size="icon" className="h-5 w-5" aria-label="編輯元件"
                          onClick={() => { setEditingIdx(idx); setEditingElem({ ...elem }); setElementEditorOpen(true); }}>
                          <Pencil className="h-3 w-3" />
                        </Button>
                        <Button variant="ghost" size="icon" className="h-5 w-5 text-destructive" aria-label="删除元件"
                          onClick={() => applyElements(parsedElements.filter((_, i) => i !== idx))}>
                          <Trash2 className="h-3 w-3" />
                        </Button>
                      </div>
                    </div>
                  ))}
                </div>

                <details className="mt-1">
                  <summary className="text-xs text-muted-foreground cursor-pointer select-none">進階：直接編輯 JSON</summary>
                  <Textarea value={formContent} onChange={(e) => { setFormContent(e.target.value); updateLivePreview(e.target.value, formPaperSize, formTheme, formRestaurantName, formTagline, formLogoData); }} rows={8} className="font-mono text-xs mt-1" />
                </details>
              </div>

              <Separator />

              <div className="flex items-center gap-3">
                <Checkbox id="formIsActive" checked={formIsActive} onCheckedChange={(v) => setFormIsActive(!!v)} />
                <Label htmlFor="formIsActive" className="text-sm font-normal">启用此模板（停用后不再出现在打印选项中）</Label>
              </div>
            </div>
            
            {/* 右側：預覽 */}
            <div className="w-full lg:w-1/2 flex flex-col">
              <Label className="text-base font-semibold mb-2 flex items-center gap-2">
                <Eye className="h-4 w-4" />实时预览
              </Label>
              <div className="flex-1 border rounded-lg p-4 bg-muted/30 overflow-auto">
                {livePreviewError ? (
                  <div className="text-sm text-destructive p-4">
                    <p className="font-medium mb-1">预览错误</p>
                    <pre className="text-xs whitespace-pre-wrap">{livePreviewError}</pre>
                  </div>
                ) : livePreviewHtml ? (
                  <div dangerouslySetInnerHTML={{ __html: DOMPurify.sanitize(livePreviewHtml) }} />
                ) : (
                  <div className="flex items-center justify-center h-full text-muted-foreground text-sm">
                    编辑以查看预览
                  </div>
                )}
              </div>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setEditDialogOpen(false)}>取消</Button>
            <Button onClick={saveTemplate}><Save className="mr-2 h-4 w-4" />保存</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Element Picker Dialog */}
      <Dialog open={elementPickerOpen} onOpenChange={setElementPickerOpen}>
        <DialogContent className="max-w-md">
          <DialogHeader><DialogTitle>选择元件類型</DialogTitle></DialogHeader>
          <div className="space-y-4 py-2">
            {ELEMENT_CATEGORIES.map(cat => (
              <div key={cat.label}>
                <p className="text-xs font-medium text-muted-foreground mb-2">{cat.label}</p>
                <div className="grid grid-cols-2 gap-1.5">
                  {cat.items.map(it => (
                    <Button key={it.type} variant="outline" size="sm" className="justify-start text-xs h-8 gap-2"
                      onClick={() => {
                        applyElements([...parsedElements, { ...it.defaultConfig }]);
                        setElementPickerOpen(false);
                      }}>
                      <span className={`px-1 py-0.5 rounded text-[9px] font-medium ${getElementBadgeColor(it.type)}`}>{cat.label[0]}</span>
                      {it.label}
                    </Button>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </DialogContent>
      </Dialog>

      {/* Element Editor Dialog */}
      <Dialog open={elementEditorOpen} onOpenChange={(o) => { if (!o) { setElementEditorOpen(false); setEditingElem(null); setEditingIdx(null); } }}>
        <DialogContent className="max-w-lg">
          <DialogHeader><DialogTitle>編輯元件 — {editingElem ? getElementLabel(editingElem.type) : ""}</DialogTitle></DialogHeader>
          {editingElem && (
            <div className="space-y-3 py-2 max-h-[60vh] overflow-y-auto pr-1">
              {editingElem.type === "text" && (<>
                <div><Label className="text-xs">文字内容</Label><Textarea value={String(editingElem.content ?? "")} onChange={e => setEditingElem({ ...editingElem, content: e.target.value })} rows={3} className="font-mono text-xs mt-1" /></div>
                <div className="grid grid-cols-2 gap-2">
                  <div><Label className="text-xs">對齊</Label>
                    <Select value={String(editingElem.align ?? "left")} onValueChange={v => setEditingElem({ ...editingElem, align: v })}>
                      <SelectTrigger className="h-8 text-xs mt-1"><SelectValue /></SelectTrigger>
                      <SelectContent><SelectItem value="left">左對齊</SelectItem><SelectItem value="center">置中</SelectItem><SelectItem value="right">右對齊</SelectItem></SelectContent>
                    </Select>
                  </div>
                  <div><Label className="text-xs">大小</Label>
                    <Select value={String(editingElem.size ?? "normal")} onValueChange={v => setEditingElem({ ...editingElem, size: v })}>
                      <SelectTrigger className="h-8 text-xs mt-1"><SelectValue /></SelectTrigger>
                      <SelectContent><SelectItem value="small">小</SelectItem><SelectItem value="normal">正常</SelectItem><SelectItem value="large">大</SelectItem></SelectContent>
                    </Select>
                  </div>
                </div>
                <div className="flex items-center gap-2"><Checkbox id="elem-bold" checked={!!editingElem.bold} onCheckedChange={v => setEditingElem({ ...editingElem, bold: !!v })} /><Label htmlFor="elem-bold" className="text-xs">粗體</Label></div>
              </>)}
              {editingElem.type === "blank_lines" && (
                <div><Label className="text-xs">空行數量</Label><Input type="number" min={1} max={10} value={Number(editingElem.count ?? 1)} onChange={e => setEditingElem({ ...editingElem, count: parseInt(e.target.value) || 1 })} className="h-8 text-xs mt-1 w-24" /></div>
              )}
              {editingElem.type === "fortune" && (
                <div><Label className="text-xs">种子策略</Label>
                  <Select value={String(editingElem.seed_strategy ?? "daily")} onValueChange={v => setEditingElem({ ...editingElem, seed_strategy: v })}>
                    <SelectTrigger className="h-8 text-xs mt-1"><SelectValue /></SelectTrigger>
                    <SelectContent><SelectItem value="daily">全店同日</SelectItem><SelectItem value="per_table">每桌不同</SelectItem><SelectItem value="per_order">每單唯一</SelectItem></SelectContent>
                  </Select>
                </div>
              )}
              {editingElem.type === "quote" && (
                <div><Label className="text-xs">语言</Label>
                  <Select value={String(editingElem.language ?? "multilingual")} onValueChange={v => setEditingElem({ ...editingElem, language: v })}>
                    <SelectTrigger className="h-8 text-xs mt-1"><SelectValue /></SelectTrigger>
                    <SelectContent><SelectItem value="multilingual">多語輪替</SelectItem><SelectItem value="zh">中文</SelectItem><SelectItem value="en">英文</SelectItem><SelectItem value="ja">日文</SelectItem></SelectContent>
                  </Select>
                </div>
              )}
              {editingElem.type === "discount_coupon" && (<>
                <div className="grid grid-cols-2 gap-2">
                  <div><Label className="text-xs">折扣類型</Label>
                    <Select value={String(editingElem.discount_type ?? "percent")} onValueChange={v => setEditingElem({ ...editingElem, discount_type: v })}>
                      <SelectTrigger className="h-8 text-xs mt-1"><SelectValue /></SelectTrigger>
                      <SelectContent><SelectItem value="percent">百分比折扣</SelectItem><SelectItem value="amount">固定金額</SelectItem><SelectItem value="free_item">指定免費</SelectItem></SelectContent>
                    </Select>
                  </div>
                  <div><Label className="text-xs">折扣值 (%或元)</Label><Input type="number" value={Number(editingElem.value ?? 0)} onChange={e => setEditingElem({ ...editingElem, value: parseFloat(e.target.value) || 0 })} className="h-8 text-xs mt-1" /></div>
                </div>
                <div><Label className="text-xs">使用条件</Label><Input value={String(editingElem.condition ?? "")} onChange={e => setEditingElem({ ...editingElem, condition: e.target.value })} className="h-8 text-xs mt-1" placeholder="消費滿100元" /></div>
                <div className="grid grid-cols-2 gap-2">
                  <div><Label className="text-xs">有效天數</Label><Input type="number" value={Number(editingElem.valid_days ?? 30)} onChange={e => setEditingElem({ ...editingElem, valid_days: parseInt(e.target.value) || 30 })} className="h-8 text-xs mt-1" /></div>
                  <div><Label className="text-xs">標題文字</Label><Input value={String(editingElem.label ?? "")} onChange={e => setEditingElem({ ...editingElem, label: e.target.value })} className="h-8 text-xs mt-1" /></div>
                </div>
              </>)}
              {editingElem.type === "product_spotlight" && (<>
                <div className="grid grid-cols-2 gap-2">
                  <div><Label className="text-xs">標題</Label><Input value={String(editingElem.title ?? "")} onChange={e => setEditingElem({ ...editingElem, title: e.target.value })} className="h-8 text-xs mt-1" /></div>
                  <div><Label className="text-xs">徽章</Label><Input value={String(editingElem.badge ?? "NEW")} onChange={e => setEditingElem({ ...editingElem, badge: e.target.value })} className="h-8 text-xs mt-1" /></div>
                </div>
                <div><Label className="text-xs">商品名稱</Label><Input value={String(editingElem.name ?? "")} onChange={e => setEditingElem({ ...editingElem, name: e.target.value })} className="h-8 text-xs mt-1" /></div>
                <div><Label className="text-xs">描述</Label><Textarea value={String(editingElem.description ?? "")} onChange={e => setEditingElem({ ...editingElem, description: e.target.value })} rows={2} className="text-xs mt-1" /></div>
                <div><Label className="text-xs">定價 (0=不显示)</Label><Input type="number" value={Number(editingElem.price ?? 0)} onChange={e => setEditingElem({ ...editingElem, price: parseFloat(e.target.value) || 0 })} className="h-8 text-xs mt-1 w-32" /></div>
              </>)}
              {editingElem.type === "qr_code" && (<>
                <div><Label className="text-xs">URL</Label><Input value={String(editingElem.url ?? "")} onChange={e => setEditingElem({ ...editingElem, url: e.target.value })} className="h-8 text-xs mt-1" placeholder="https://..." /></div>
                <div><Label className="text-xs">說明文字</Label><Input value={String(editingElem.label ?? "")} onChange={e => setEditingElem({ ...editingElem, label: e.target.value })} className="h-8 text-xs mt-1" /></div>
                <div><Label className="text-xs">尺寸 (1-8)</Label><Input type="number" min={1} max={8} value={Number(editingElem.size ?? 5)} onChange={e => setEditingElem({ ...editingElem, size: parseInt(e.target.value) || 5 })} className="h-8 text-xs mt-1 w-24" /></div>
              </>)}
              {editingElem.type === "character_collect" && (<>
                <div className="grid grid-cols-2 gap-2">
                  <div><Label className="text-xs">遊戲名稱</Label><Input value={String(editingElem.game_name ?? "")} onChange={e => setEditingElem({ ...editingElem, game_name: e.target.value })} className="h-8 text-xs mt-1" /></div>
                  <div><Label className="text-xs">樣式</Label>
                    <Select value={String(editingElem.style ?? "box")} onValueChange={v => setEditingElem({ ...editingElem, style: v })}>
                      <SelectTrigger className="h-8 text-xs mt-1"><SelectValue /></SelectTrigger>
                      <SelectContent><SelectItem value="box">方框</SelectItem><SelectItem value="mahjong">麻將</SelectItem></SelectContent>
                    </Select>
                  </div>
                </div>
                <div>
                  <Label className="text-xs">集字/集章组合（逗号分隔，支持汉字、emoji、符号）</Label>
                  <Input value={(editingElem.characters as string[] | undefined)?.join(",") ?? ""} onChange={e => setEditingElem({ ...editingElem, characters: e.target.value.split(",").map(s => s.trim()).filter(Boolean) })} className="h-8 text-xs mt-1" placeholder="恭,喜,发,财" />
                  <div className="flex flex-wrap gap-1 mt-1.5">
                    {[
                      { label: "🀄 麻将", val: "🀇,🀈,🀉,🀊" },
                      { label: "🍤 海鲜", val: "🍤,🦐,🦞,🦀" },
                      { label: "🌸 四季", val: "🌸,☀️,🍂,❄️" },
                      { label: "福禄寿喜", val: "福,禄,寿,喜" },
                    ].map(p => (
                      <button key={p.label} type="button" className="text-[10px] px-1.5 py-0.5 rounded border border-muted-foreground/30 text-muted-foreground hover:bg-muted"
                        onClick={() => setEditingElem({ ...editingElem, characters: p.val.split(",") })}>
                        {p.label}
                      </button>
                    ))}
                  </div>
                  <p className="text-[10px] text-muted-foreground mt-1">每张小票抽一个，顾客集齐所有才可兑奖</p>
                </div>
                <div><Label className="text-xs">兑奖说明</Label><Input value={String(editingElem.prize ?? "")} onChange={e => setEditingElem({ ...editingElem, prize: e.target.value })} className="h-8 text-xs mt-1" /></div>
                <div><Label className="text-xs">种子策略</Label>
                  <Select value={String(editingElem.seed_strategy ?? "per_order")} onValueChange={v => setEditingElem({ ...editingElem, seed_strategy: v })}>
                    <SelectTrigger className="h-8 text-xs mt-1"><SelectValue /></SelectTrigger>
                    <SelectContent><SelectItem value="per_order">每單唯一</SelectItem><SelectItem value="per_table">每桌不同</SelectItem><SelectItem value="daily">全店同日</SelectItem></SelectContent>
                  </Select>
                </div>
              </>)}
              {editingElem.type === "rich_text" && (
                <div><Label className="text-xs">Markdown内容</Label><Textarea value={String(editingElem.content ?? "")} onChange={e => setEditingElem({ ...editingElem, content: e.target.value })} rows={8} className="font-mono text-xs mt-1" placeholder={"## 标题\n- 项目一\n- 项目二\n> 引用文字"} /></div>
              )}
              {editingElem.type === "solar_term" && (<>
                <p className="text-xs text-muted-foreground">节气期间（前后约7天）自动显示对应主题文案，不在节气期间时不显示。</p>
                <div className="flex items-center gap-2"><input type="checkbox" checked={!!editingElem.show_all} onChange={e => setEditingElem({ ...editingElem, show_all: e.target.checked })} id="solar-show-all" /><Label htmlFor="solar-show-all" className="text-xs">不在节气期间时也显示"下一个节气"提示</Label></div>
              </>)}
              {editingElem.type === "chef_message" && (<>
                <div><Label className="text-xs">标题</Label><Input value={String(editingElem.title ?? "厨师寄语")} onChange={e => setEditingElem({ ...editingElem, title: e.target.value })} className="h-8 text-xs mt-1" /></div>
                <div><Label className="text-xs">署名（显示在消息末尾）</Label><Input value={String(editingElem.author ?? "本店厨师")} onChange={e => setEditingElem({ ...editingElem, author: e.target.value })} className="h-8 text-xs mt-1" placeholder="例：张师傅" /></div>
                <div><Label className="text-xs">每日消息（每行一条，留空使用内置默认，按星期循环最多7条）</Label>
                  <Textarea value={(editingElem.messages as string[] | undefined)?.join("\n") ?? ""} onChange={e => setEditingElem({ ...editingElem, messages: e.target.value.split("\n").map(s => s.trim()).filter(Boolean) })} rows={7} className="text-xs mt-1" placeholder={"周一：今天的食材格外新鲜...\n周二：感谢光临，用心烹饪...\n（最多7条，按星期轮播）"} /></div>
              </>)}
              {editingElem.type === "riddle" && (<>
                <p className="text-xs text-muted-foreground">留空使用内置谜语库（每日随机），或自定义谜题和答案。</p>
                <div><Label className="text-xs">自定义谜题（选填，留空用内置库）</Label><Textarea value={String(editingElem.question ?? "")} onChange={e => setEditingElem({ ...editingElem, question: e.target.value || undefined })} rows={2} className="text-xs mt-1" placeholder="输入谜题..." /></div>
                <div><Label className="text-xs">答案（收据上不显示，仅供核对）</Label><Input value={String(editingElem.answer ?? "")} onChange={e => setEditingElem({ ...editingElem, answer: e.target.value || undefined })} className="h-8 text-xs mt-1" placeholder="谜底..." /></div>
                <div><Label className="text-xs">兑奖说明（显示在谜题下方）</Label><Input value={String(editingElem.prize ?? "")} onChange={e => setEditingElem({ ...editingElem, prize: e.target.value })} className="h-8 text-xs mt-1" placeholder="下次来店说出答案，赢取小惊喜！" /></div>
              </>)}
              {editingElem.type === "dish_easter_egg" && (<>
                <p className="text-xs text-muted-foreground">当订单中包含指定关键词菜品时，显示隐藏彩蛋消息。每条规则一个关键词。</p>
                {((editingElem.eggs as { keyword: string; message: string }[] | undefined) ?? []).map((egg, idx) => (
                  <div key={idx} className="flex gap-1 items-start">
                    <div className="flex-1 space-y-1">
                      <Input value={egg.keyword} placeholder="菜品关键词（如：虾）" className="h-7 text-xs"
                        onChange={e => { const eggs = [...((editingElem.eggs as typeof egg[]) ?? [])]; eggs[idx] = { ...egg, keyword: e.target.value }; setEditingElem({ ...editingElem, eggs }); }} />
                      <Input value={egg.message} placeholder="彩蛋消息（如：解锁：海鲜达人！享95折）" className="h-7 text-xs"
                        onChange={e => { const eggs = [...((editingElem.eggs as typeof egg[]) ?? [])]; eggs[idx] = { ...egg, message: e.target.value }; setEditingElem({ ...editingElem, eggs }); }} />
                    </div>
                    <button type="button" className="text-xs text-destructive px-1 pt-1" onClick={() => { const eggs = ((editingElem.eggs as typeof egg[]) ?? []).filter((_, i) => i !== idx); setEditingElem({ ...editingElem, eggs }); }}>✕</button>
                  </div>
                ))}
                <Button type="button" variant="outline" size="sm" className="w-full h-7 text-xs"
                  onClick={() => { const eggs = [...((editingElem.eggs as { keyword: string; message: string }[]) ?? []), { keyword: "", message: "" }]; setEditingElem({ ...editingElem, eggs }); }}>
                  + 添加触发规则
                </Button>
              </>)}
              {["separator", "items", "art", "image_block"].includes(editingElem.type) && (
                <p className="text-sm text-muted-foreground py-2">此元件无需额外配置。</p>
              )}
            </div>
          )}
          <DialogFooter>
            <Button variant="outline" onClick={() => { setElementEditorOpen(false); setEditingElem(null); setEditingIdx(null); }}>取消</Button>
            <Button onClick={() => {
              if (editingElem === null || editingIdx === null) return;
              const els = [...parsedElements];
              els[editingIdx] = editingElem;
              applyElements(els);
              setElementEditorOpen(false); setEditingElem(null); setEditingIdx(null);
            }}><Save className="h-3.5 w-3.5 mr-1" />套用</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={previewDialogOpen} onOpenChange={setPreviewDialogOpen}>
        <DialogContent className="max-w-lg">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2"><Printer className="h-4 w-4" />打印预览</DialogTitle>
          </DialogHeader>
          <div className="max-h-[65vh] overflow-y-auto space-y-4 pr-1">
            <div className="border rounded-lg p-4 bg-white text-black font-mono text-sm" dangerouslySetInnerHTML={{ __html: DOMPurify.sanitize(previewHtml) }} />
            <Separator />
            <div>
              <h4 className="text-sm font-medium mb-2 text-muted-foreground">原始文本</h4>
              <pre className="text-xs bg-muted p-3 rounded overflow-x-auto whitespace-pre-wrap break-all">
                {previewLines.join("\n")}
              </pre>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setPreviewDialogOpen(false)}>关闭</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}

const defaultKitchenTemplate = {
  elements: [
    { type: "text", content: "Cuckoo 厨房单", align: "center", bold: true, size: "large" },
    { type: "separator" },
    { type: "text", content: "单号: {{order_no}}" },
    { type: "text", content: "类型: {{dine_type}}" },
    { type: "text", content: "时间: {{time}}" },
    { type: "separator" },
    { type: "text", content: "菜品明细", bold: true },
    { type: "items" },
    { type: "separator" },
    { type: "text", content: "订单备注: {{note}}", bold: true },
    { type: "blank_lines", count: 3 },
  ],
};

const defaultBatchLabelTemplate = {
  elements: [
    { type: "text", content: "{{material_name}}", align: "center", bold: true, size: "large" },
    { type: "separator" },
    { type: "text", content: "批次: {{lot_no}}" },
    { type: "text", content: "数量: {{quantity}} {{unit}}" },
    { type: "text", content: "到期: {{expiry_date}}" },
    { type: "text", content: "供应商: {{supplier_name}}" },
    { type: "blank_lines", count: 2 },
  ],
};
