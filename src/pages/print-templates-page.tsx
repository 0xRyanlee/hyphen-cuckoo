import { useState, useEffect, useCallback, useRef } from "react";
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
import { Plus, Pencil, Trash2, Eye, Save, Printer, Star, FileBox } from "lucide-react";
import { EmptyState } from "@/components/ui/empty-state";
import { toast } from "sonner";

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
    // 開啟時立即觸發預覽，不等用戶修改
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
              <div className="flex gap-1">
                <Button variant="outline" size="sm" className="flex-1" onClick={() => previewTemplate(tpl)}>
                  <Eye className="h-3.5 w-3.5 mr-1" />预览
                </Button>
                <Button variant="outline" size="sm" onClick={() => openEdit(tpl)}><Pencil className="h-3.5 w-3.5" /></Button>
                {!tpl.is_default && (
                  <Button variant="outline" size="sm" onClick={() => setDefault(tpl.id, tpl.template_type)}><Star className="h-3.5 w-3.5" /></Button>
                )}
                <Button variant="outline" size="sm" onClick={() => deleteTemplate(tpl.id)}><Trash2 className="h-3.5 w-3.5 text-destructive" /></Button>
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
                <Label>模板內容 (JSON)</Label>
                <Textarea value={formContent} onChange={(e) => { setFormContent(e.target.value); updateLivePreview(e.target.value, formPaperSize, formTheme, formRestaurantName, formTagline, formLogoData); }} rows={12} className="font-mono text-xs" />
                <p className="text-xs text-muted-foreground">支持元素类型：text, separator, blank_lines, items。使用 {"{{variable}}"} 作为占位符。</p>
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
