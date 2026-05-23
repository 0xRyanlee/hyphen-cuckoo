import { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Badge } from "@/components/ui/badge";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Checkbox } from "@/components/ui/checkbox";
import { call as invoke } from "@/lib/transport";
import DOMPurify from "dompurify";
import { Printer, Copy, ExternalLink, Plus, Pencil, Trash2, Wifi, Search, XCircle, Loader2 } from "lucide-react";
import { toast } from "sonner";
import { PrintTemplatesPage } from "./print-templates-page";

interface PrintTemplate {
  id: number;
  name: string;
  template_type: string;
  paper_size: string;
  content: string;
  is_default: boolean;
  is_active: boolean;
}

interface DebugPrintResult {
  file_path: string;
  html_preview: string;
  byte_count: number;
}

interface PrinterConfig {
  id: number;
  name: string;
  printer_type: string;
  connection_type: string;
  feie_user: string | null;
  feie_ukey: string | null;
  feie_sn: string | null;
  feie_key: string | null;
  lan_ip: string | null;
  lan_port: number;
  paper_width: string;
  is_default: boolean;
  is_active: boolean;
  created_at: string;
}

interface LanPrinter {
  ip: string;
  port: number;
  sn: string | null;
}

// ── Printers Section ─────────────────────────────────────────────────────────

function PrintersSection() {
  const [printers, setPrinters] = useState<PrinterConfig[]>([]);
  const [editDialogOpen, setEditDialogOpen] = useState(false);
  const [editingPrinter, setEditingPrinter] = useState<PrinterConfig | null>(null);
  const [deleteConfirm, setDeleteConfirm] = useState<PrinterConfig | null>(null);
  const [scanDialogOpen, setScanDialogOpen] = useState(false);
  const [subnet, setSubnet] = useState("192.168.1");
  const [scanning, setScanning] = useState(false);
  const [scanResults, setScanResults] = useState<LanPrinter[]>([]);
  const [testingId, setTestingId] = useState<number | null>(null);

  // Form state
  const [formName, setFormName] = useState("");
  const [formConnectionType, setFormConnectionType] = useState("lan");
  const [formLanIp, setFormLanIp] = useState("");
  const [formLanPort, setFormLanPort] = useState("9100");
  const [formPaperWidth, setFormPaperWidth] = useState("80mm");
  const [formIsDefault, setFormIsDefault] = useState(false);
  const [formFeieUser, setFormFeieUser] = useState("");
  const [formFeieUkey, setFormFeieUkey] = useState("");
  const [formFeieSn, setFormFeieSn] = useState("");

  useEffect(() => {
    loadPrinters();
  }, []);

  async function loadPrinters() {
    try {
      const data = await invoke<PrinterConfig[]>("get_printers");
      setPrinters(data.filter(p => p.is_active));
    } catch (e) {
      toast.error("加载打印机失败", { description: String(e) });
    }
  }

  function openNew(prefill?: Partial<{ ip: string; port: number }>) {
    setEditingPrinter(null);
    setFormName("");
    setFormConnectionType("lan");
    setFormLanIp(prefill?.ip ?? "");
    setFormLanPort(String(prefill?.port ?? 9100));
    setFormPaperWidth("80mm");
    setFormIsDefault(printers.length === 0);
    setFormFeieUser("");
    setFormFeieUkey("");
    setFormFeieSn("");
    setEditDialogOpen(true);
  }

  function openEdit(p: PrinterConfig) {
    setEditingPrinter(p);
    setFormName(p.name);
    setFormConnectionType(p.connection_type);
    setFormLanIp(p.lan_ip ?? "");
    setFormLanPort(String(p.lan_port));
    setFormPaperWidth(p.paper_width);
    setFormIsDefault(p.is_default);
    setFormFeieUser(p.feie_user ?? "");
    setFormFeieUkey(p.feie_ukey ?? "");
    setFormFeieSn(p.feie_sn ?? "");
    setEditDialogOpen(true);
  }

  async function savePrinter() {
    if (!formName.trim()) { toast.error("请填写打印机名称"); return; }
    if (formConnectionType === "lan" && !formLanIp.trim()) { toast.error("请填写 IP 地址"); return; }
    if (formConnectionType === "feie" && (!formFeieUser.trim() || !formFeieSn.trim())) {
      toast.error("请填写飞鹅账户和 SN 号"); return;
    }

    const base = {
      name: formName.trim(),
      printer_type: "thermal",
      connection_type: formConnectionType,
      lan_ip: formConnectionType === "lan" ? formLanIp.trim() : null,
      lan_port: formConnectionType === "lan" ? parseInt(formLanPort) : null,
      feie_user: formConnectionType === "feie" ? formFeieUser.trim() : null,
      feie_ukey: formConnectionType === "feie" ? formFeieUkey.trim() : null,
      feie_sn: formConnectionType === "feie" ? formFeieSn.trim() : null,
      feie_key: null,
      paper_width: formPaperWidth,
      is_default: formIsDefault,
    };

    try {
      if (editingPrinter) {
        await invoke("update_printer", {
          id: editingPrinter.id,
          name: base.name,
          printerType: base.printer_type,
          connectionType: base.connection_type,
          feieUser: base.feie_user,
          feieUkey: base.feie_ukey,
          feieSn: base.feie_sn,
          feieKey: null,
          lanIp: base.lan_ip,
          lanPort: base.lan_port,
          paperWidth: base.paper_width,
          isDefault: base.is_default,
        });
        toast.success("打印机已更新");
      } else {
        await invoke("create_printer", { req: base });
        toast.success("打印机已添加");
      }
      setEditDialogOpen(false);
      loadPrinters();
    } catch (e) {
      toast.error("保存失败", { description: String(e) });
    }
  }

  async function deletePrinter() {
    if (!deleteConfirm) return;
    try {
      await invoke("delete_printer", { id: deleteConfirm.id });
      toast.success("打印机已删除");
      setDeleteConfirm(null);
      loadPrinters();
    } catch (e) {
      toast.error("删除失败", { description: String(e) });
    }
  }

  async function testPrinter(p: PrinterConfig) {
    setTestingId(p.id);
    try {
      if (p.connection_type === "lan") {
        const msg = await invoke<string>("test_lan_printer", {
          ip: p.lan_ip,
          port: p.lan_port,
        });
        toast.success(`测试成功: ${msg}`);
      } else {
        const msg = await invoke<string>("test_feie_printer", {
          user: p.feie_user,
          ukey: p.feie_ukey,
          sn: p.feie_sn,
        });
        toast.success(`测试成功: ${msg}`);
      }
    } catch (e) {
      toast.error("测试失败", { description: String(e) });
    } finally {
      setTestingId(null);
    }
  }

  async function startScan() {
    if (!subnet.trim()) { toast.error("请填写子网段"); return; }
    setScanning(true);
    setScanResults([]);
    try {
      const results = await invoke<LanPrinter[]>("scan_lan_printers", {
        subnet: subnet.trim(),
        timeoutMs: 800,
      });
      setScanResults(results);
      if (results.length === 0) toast.info("未发现局域网打印机");
      else toast.success(`发现 ${results.length} 台打印机`);
    } catch (e) {
      toast.error("扫描失败", { description: String(e) });
    } finally {
      setScanning(false);
    }
  }

  const connLabel = (p: PrinterConfig) =>
    p.connection_type === "lan" ? `${p.lan_ip}:${p.lan_port}` : `飞鹅 ${p.feie_sn ?? ""}`;

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold">打印机管理</h2>
          <p className="text-sm text-muted-foreground">配置热敏打印机连接参数</p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" onClick={() => { setScanDialogOpen(true); setScanResults([]); }}>
            <Search className="h-4 w-4 mr-2" />扫描局域网
          </Button>
          <Button onClick={() => openNew()}>
            <Plus className="h-4 w-4 mr-2" />添加打印机
          </Button>
        </div>
      </div>

      <Card>
        <CardContent className="p-0">
          {printers.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-16 text-muted-foreground gap-3">
              <Printer className="h-10 w-10 opacity-30" />
              <p className="text-sm">暂无打印机，点击"扫描局域网"自动发现或手动添加</p>
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>名称</TableHead>
                  <TableHead>连接方式</TableHead>
                  <TableHead>地址</TableHead>
                  <TableHead>纸宽</TableHead>
                  <TableHead>状态</TableHead>
                  <TableHead className="text-right">操作</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {printers.map(p => (
                  <TableRow key={p.id}>
                    <TableCell className="font-medium">
                      {p.name}
                      {p.is_default && <Badge variant="secondary" className="ml-2 text-xs">默认</Badge>}
                    </TableCell>
                    <TableCell>
                      <Badge variant="outline" className="text-xs">
                        {p.connection_type === "lan" ? "局域网" : "飞鹅云"}
                      </Badge>
                    </TableCell>
                    <TableCell className="font-mono text-xs text-muted-foreground">{connLabel(p)}</TableCell>
                    <TableCell className="text-muted-foreground text-sm">{p.paper_width}</TableCell>
                    <TableCell>
                      {testingId === p.id
                        ? <Loader2 className="h-4 w-4 animate-spin text-muted-foreground" />
                        : <Wifi className="h-4 w-4 text-muted-foreground" />
                      }
                    </TableCell>
                    <TableCell className="text-right">
                      <div className="flex justify-end gap-1">
                        <Button variant="ghost" size="sm" className="h-8 text-xs"
                          disabled={testingId === p.id}
                          onClick={() => testPrinter(p)}>
                          测试
                        </Button>
                        <Button variant="ghost" size="icon" className="h-8 w-8" onClick={() => openEdit(p)}>
                          <Pencil className="h-4 w-4" />
                        </Button>
                        <Button variant="ghost" size="icon" className="h-8 w-8 text-destructive" onClick={() => setDeleteConfirm(p)}>
                          <Trash2 className="h-4 w-4" />
                        </Button>
                      </div>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      {/* LAN Scan Dialog */}
      <Dialog open={scanDialogOpen} onOpenChange={setScanDialogOpen}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle>扫描局域网打印机</DialogTitle>
          </DialogHeader>
          <div className="space-y-4 py-2">
            <div className="space-y-2">
              <Label>子网段</Label>
              <div className="flex gap-2">
                <Input
                  value={subnet}
                  onChange={e => setSubnet(e.target.value)}
                  placeholder="192.168.1"
                  className="font-mono"
                />
                <span className="flex items-center text-sm text-muted-foreground">.0/24</span>
              </div>
              <p className="text-xs text-muted-foreground">扫描该子网内所有 9100 端口设备（约需数秒）</p>
            </div>
            <Button onClick={startScan} disabled={scanning} className="w-full">
              {scanning ? <><Loader2 className="h-4 w-4 mr-2 animate-spin" />扫描中...</> : <><Search className="h-4 w-4 mr-2" />开始扫描</>}
            </Button>

            {scanResults.length > 0 && (
              <div className="space-y-2">
                <p className="text-sm font-medium">发现 {scanResults.length} 台设备</p>
                <div className="divide-y rounded-md border">
                  {scanResults.map((r, i) => (
                    <div key={i} className="flex items-center justify-between px-3 py-2">
                      <div>
                        <p className="font-mono text-sm">{r.ip}:{r.port}</p>
                        {r.sn && <p className="text-xs text-muted-foreground">SN: {r.sn}</p>}
                      </div>
                      <Button size="sm" variant="outline" onClick={() => {
                        setScanDialogOpen(false);
                        openNew({ ip: r.ip, port: r.port });
                      }}>
                        <Plus className="h-3 w-3 mr-1" />添加
                      </Button>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {!scanning && scanResults.length === 0 && (
              <div className="flex items-center justify-center py-6 text-sm text-muted-foreground gap-2">
                <XCircle className="h-4 w-4" />
                未发现打印机，请确认打印机已开机并在同一网络
              </div>
            )}
          </div>
        </DialogContent>
      </Dialog>

      {/* Add/Edit Dialog */}
      <Dialog open={editDialogOpen} onOpenChange={setEditDialogOpen}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle>{editingPrinter ? "编辑打印机" : "添加打印机"}</DialogTitle>
          </DialogHeader>
          <div className="space-y-4 py-2">
            <div className="space-y-2">
              <Label>打印机名称</Label>
              <Input value={formName} onChange={e => setFormName(e.target.value)} placeholder="如：厨房打印机" />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label>连接方式</Label>
                <Select value={formConnectionType} onValueChange={setFormConnectionType}>
                  <SelectTrigger><SelectValue /></SelectTrigger>
                  <SelectContent>
                    <SelectItem value="lan">局域网 (LAN)</SelectItem>
                    <SelectItem value="feie">飞鹅云打印</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <Label>纸张宽度</Label>
                <Select value={formPaperWidth} onValueChange={setFormPaperWidth}>
                  <SelectTrigger><SelectValue /></SelectTrigger>
                  <SelectContent>
                    <SelectItem value="58mm">58mm</SelectItem>
                    <SelectItem value="80mm">80mm</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>

            {formConnectionType === "lan" ? (
              <div className="grid grid-cols-3 gap-4">
                <div className="col-span-2 space-y-2">
                  <Label>IP 地址</Label>
                  <Input value={formLanIp} onChange={e => setFormLanIp(e.target.value)} placeholder="192.168.1.100" className="font-mono" />
                </div>
                <div className="space-y-2">
                  <Label>端口</Label>
                  <Input value={formLanPort} onChange={e => setFormLanPort(e.target.value)} className="font-mono" />
                </div>
              </div>
            ) : (
              <div className="space-y-3">
                <div className="space-y-2">
                  <Label>飞鹅账户</Label>
                  <Input value={formFeieUser} onChange={e => setFormFeieUser(e.target.value)} placeholder="注册手机号" />
                </div>
                <div className="space-y-2">
                  <Label>UKEY</Label>
                  <Input value={formFeieUkey} onChange={e => setFormFeieUkey(e.target.value)} placeholder="飞鹅开放平台 UKEY" />
                </div>
                <div className="space-y-2">
                  <Label>打印机 SN</Label>
                  <Input value={formFeieSn} onChange={e => setFormFeieSn(e.target.value)} placeholder="打印机机身号" className="font-mono" />
                </div>
              </div>
            )}

            <div className="flex items-center gap-3 pt-1">
              <Checkbox id="is-default" checked={formIsDefault} onCheckedChange={v => setFormIsDefault(!!v)} />
              <Label htmlFor="is-default" className="cursor-pointer">设为默认打印机</Label>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setEditDialogOpen(false)}>取消</Button>
            <Button onClick={savePrinter}>保存</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Delete Confirm */}
      <Dialog open={!!deleteConfirm} onOpenChange={() => setDeleteConfirm(null)}>
        <DialogContent>
          <DialogHeader><DialogTitle>确认删除</DialogTitle></DialogHeader>
          <p className="py-4 text-sm text-muted-foreground">
            确定要删除打印机「{deleteConfirm?.name}」吗？此操作不可撤销。
          </p>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteConfirm(null)}>取消</Button>
            <Button variant="destructive" onClick={deletePrinter}>删除</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}

// ── Main Print Page ───────────────────────────────────────────────────────────

export function PrintPage() {
  const [activeTab, setActiveTab] = useState("preview");

  const [templates, setTemplates] = useState<PrintTemplate[]>([]);
  const [selectedTemplateId, setSelectedTemplateId] = useState<number | null>(null);
  const [result, setResult] = useState<DebugPrintResult | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    loadTemplates();
  }, []);

  async function loadTemplates() {
    try {
      const data = await invoke<PrintTemplate[]>("get_print_templates");
      const activeTemplates = data.filter(t => t.is_active);
      setTemplates(activeTemplates);
      const defaultTpl = activeTemplates.find(t => t.is_default);
      if (defaultTpl) setSelectedTemplateId(defaultTpl.id);
    } catch (e) {
      toast.error("加载模板失败", { description: String(e) });
    }
  }

  const selectedTemplate = templates.find(t => t.id === selectedTemplateId);

  const [orderNo, setOrderNo] = useState("ORD20260426001");
  const [dineType, setDineType] = useState("堂食");
  const [ticketNote, setTicketNote] = useState("");
  const [itemsJson, setItemsJson] = useState(`[
  ["宫保鸡丁", 2, "少辣"],
  ["麻婆豆腐", 1, null],
  ["酸菜鱼", 1, "加辣"]
]`);

  const [lotNo, setLotNo] = useState("LOT20260426001");
  const [materialName, setMaterialName] = useState("鸡胸肉");
  const [quantity, setQuantity] = useState("10.5");
  const [unit, setUnit] = useState("kg");
  const [expiryDate, setExpiryDate] = useState("2026-05-01");
  const [supplierName, setSupplierName] = useState("新鲜食材供应商");

  const [rawContent, setRawContent] = useState("测试打印内容\\n第二行");
  const [filename, setFilename] = useState("");

  const parseItems = () => {
    try { return JSON.parse(itemsJson); } catch { return []; }
  };

  const renderTicketMockup = () => {
    const t = selectedTemplate;
    const items = parseItems();
    return (
      <div className="thermal-paper font-mono text-xs leading-tight"
        style={{ width: t?.paper_size === "80mm" ? "300px" : "260px" }}>
        <div className="text-center border-b-2 border-dashed border-gray-400 pb-2 mb-2">
          <div className="text-lg font-bold">{t?.name || "厨房单"}</div>
          <div className="text-sm">ORDER: {orderNo}</div>
          <div className="text-xs">桌号: A01</div>
          <div className="text-xs bg-black text-white inline-block px-2 py-0.5 mt-1 rounded">{dineType}</div>
        </div>
        <div className="space-y-1">
          {items.map((item: any[], idx: number) => (
            <div key={idx} className="flex justify-between items-start">
              <span className="flex-1">
                <span className="font-bold">{item[1]}x </span>
                {item[0]}
                {item[2] && <span className="text-orange-600 text-xs"> ({item[2]})</span>}
              </span>
            </div>
          ))}
        </div>
        {ticketNote && (
          <div className="mt-2 pt-2 border-t border-dashed border-gray-400 text-orange-600 text-xs">
            备注: {ticketNote}
          </div>
        )}
        <div className="mt-4 pt-2 border-t border-dashed border-gray-400 text-center text-xs text-gray-500">
          <div>{new Date().toLocaleString()}</div>
        </div>
      </div>
    );
  };

  const renderLabelMockup = () => (
    <div className="thermal-paper font-mono text-xs leading-tight" style={{ width: "180px" }}>
      <div className="text-center border-b-2 border-gray-800 pb-2 mb-2">
        <div className="text-lg font-bold">{materialName}</div>
      </div>
      <div className="space-y-2">
        <div className="flex justify-between"><span className="text-gray-600">批次:</span><span className="font-bold">{lotNo}</span></div>
        <div className="flex justify-between text-lg">
          <span className="font-bold">{quantity}</span><span className="font-bold">{unit}</span>
        </div>
        {expiryDate && <div className="flex justify-between"><span className="text-gray-600">效期:</span><span className="text-red-600 font-bold">{expiryDate}</span></div>}
        {supplierName && <div className="text-xs text-gray-600 mt-1 pt-1 border-t border-dashed">{supplierName}</div>}
      </div>
      <div className="mt-4 pt-2 border-t border-dashed border-gray-400 text-center">
        <div className="text-xs text-gray-400">▮▮▮▮▮▮▮▮▮▮</div>
      </div>
    </div>
  );

  const renderRawMockup = () => {
    const lines = rawContent.split("\\n").map(l => l.trim()).filter(Boolean);
    return (
      <div className="thermal-paper font-mono text-xs leading-normal whitespace-pre-wrap" style={{ width: "260px" }}>
        {lines.map((line, idx) => <div key={idx}>{line}</div>)}
      </div>
    );
  };

  const renderMockup = () => {
    if (!selectedTemplate) return null;
    const t = selectedTemplate.template_type;
    if (t === "batch_label") return renderLabelMockup();
    if (t === "cup_label") return renderRawMockup();
    return renderTicketMockup();
  };

  async function handlePrint() {
    setLoading(true);
    try {
      let res: DebugPrintResult;
      const templateType = selectedTemplate?.template_type || "kitchen_ticket";
      if (templateType === "batch_label") {
        res = await invoke<DebugPrintResult>("debug_print_batch_label", {
          req: { lot_no: lotNo, material_name: materialName, quantity: parseFloat(quantity), unit, expiry_date: expiryDate || null, supplier_name: supplierName || null, filename: filename || null },
        });
      } else if (templateType === "cup_label") {
        res = await invoke<DebugPrintResult>("debug_print_escpos", {
          content: rawContent.replace(/\\n/g, "\n"),
          filename: filename || null,
        });
      } else {
        const items = JSON.parse(itemsJson);
        res = await invoke<DebugPrintResult>("debug_print_kitchen_ticket", {
          req: { order_no: orderNo, dine_type: dineType, items, note: ticketNote || null, filename: filename || null },
        });
      }
      setResult(res);
      toast.success(`已生成 (${res.byte_count} 字节)`);
    } catch (e: any) {
      toast.error(`打印失败: ${e}`);
    } finally {
      setLoading(false);
    }
  }

  const copyHtml = () => {
    if (result?.html_preview) {
      navigator.clipboard.writeText(result.html_preview);
      toast.success("HTML 已复制");
    }
  };

  const safePreviewHtml = result ? DOMPurify.sanitize(result.html_preview) : "";

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">打印中心</h1>
        <p className="text-muted-foreground mt-1">打印机管理、模板设计与打印预览</p>
      </div>

      <Tabs value={activeTab} onValueChange={setActiveTab} className="gap-4">
        <TabsList>
          <TabsTrigger value="printers">打印机</TabsTrigger>
          <TabsTrigger value="preview">预览与测试</TabsTrigger>
          <TabsTrigger value="templates">模板管理</TabsTrigger>
        </TabsList>

        <TabsContent value="printers">
          <PrintersSection />
        </TabsContent>

        <TabsContent value="preview">
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
            <Card className="lg:col-span-2">
              <CardHeader>
                <CardTitle>参数输入</CardTitle>
                <CardDescription>选择打印模板并输入打印参数</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex gap-2">
                  <Select value={String(selectedTemplateId)} onValueChange={(v) => setSelectedTemplateId(Number(v))}>
                    <SelectTrigger className="flex-1"><SelectValue placeholder="选择打印模板" /></SelectTrigger>
                    <SelectContent>
                      {templates.map(t => (
                        <SelectItem key={t.id} value={String(t.id)}>{t.name} {t.is_default && "(默认)"}</SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                  <Button variant="outline" size="icon" onClick={() => setActiveTab("templates")}><ExternalLink className="h-4 w-4" /></Button>
                </div>

                {selectedTemplate?.template_type === "batch_label" ? (
                  <>
                    <div className="grid grid-cols-2 gap-4">
                      <div className="space-y-2"><Label>批次号</Label><Input value={lotNo} onChange={e => setLotNo(e.target.value)} /></div>
                      <div className="space-y-2"><Label>材料名称</Label><Input value={materialName} onChange={e => setMaterialName(e.target.value)} /></div>
                    </div>
                    <div className="grid grid-cols-2 gap-4">
                      <div className="space-y-2"><Label>数量</Label><Input type="number" value={quantity} onChange={e => setQuantity(e.target.value)} /></div>
                      <div className="space-y-2"><Label>单位</Label><Input value={unit} onChange={e => setUnit(e.target.value)} /></div>
                    </div>
                    <div className="grid grid-cols-2 gap-4">
                      <div className="space-y-2"><Label>到期日期</Label><Input type="date" value={expiryDate} onChange={e => setExpiryDate(e.target.value)} /></div>
                      <div className="space-y-2"><Label>供应商</Label><Input value={supplierName} onChange={e => setSupplierName(e.target.value)} placeholder="可选" /></div>
                    </div>
                  </>
                ) : selectedTemplate?.template_type === "cup_label" ? (
                  <div className="space-y-2">
                    <Label>打印内容</Label>
                    <Textarea value={rawContent} onChange={e => setRawContent(e.target.value)} rows={6} className="font-mono text-sm" />
                    <p className="text-xs text-muted-foreground">使用 \n 表示换行</p>
                  </div>
                ) : (
                  <>
                    <div className="grid grid-cols-2 gap-4">
                      <div className="space-y-2"><Label>订单号</Label><Input value={orderNo} onChange={e => setOrderNo(e.target.value)} /></div>
                      <div className="space-y-2">
                        <Label>用餐类型</Label>
                        <Select value={dineType} onValueChange={setDineType}>
                          <SelectTrigger><SelectValue /></SelectTrigger>
                          <SelectContent>
                            <SelectItem value="堂食">堂食</SelectItem>
                            <SelectItem value="外卖">外卖</SelectItem>
                            <SelectItem value="自取">自取</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>
                    </div>
                    <div className="space-y-2">
                      <Label>菜品列表 (JSON)</Label>
                      <Textarea value={itemsJson} onChange={e => setItemsJson(e.target.value)} rows={5} className="font-mono text-sm" />
                      <p className="text-xs text-muted-foreground">格式: [["菜名", 数量, "备注"], ...]</p>
                    </div>
                    <div className="space-y-2"><Label>订单备注</Label><Input value={ticketNote} onChange={e => setTicketNote(e.target.value)} placeholder="可选" /></div>
                  </>
                )}

                <div className="space-y-2"><Label>文件名 (可选)</Label><Input value={filename} onChange={e => setFilename(e.target.value)} placeholder="debug" /></div>

                <Button onClick={handlePrint} disabled={loading || !selectedTemplate} className="w-full">
                  <Printer className="h-4 w-4 mr-2" />{loading ? "生成中..." : "生成打印"}
                </Button>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>即时预览</CardTitle>
                <CardDescription>热敏纸效果</CardDescription>
              </CardHeader>
              <CardContent className="flex justify-center bg-gray-100 p-4 rounded-lg min-h-[300px]">
                {selectedTemplate ? renderMockup() : <div className="text-muted-foreground">请选择打印模板</div>}
              </CardContent>
            </Card>
          </div>

          {result && (
            <Card className="mt-4">
              <CardHeader>
                <CardTitle className="flex items-center justify-between">
                  <span>打印结果 ({result.byte_count} 字节)</span>
                  <Button variant="outline" size="sm" onClick={copyHtml}><Copy className="h-4 w-4 mr-2" />复制</Button>
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div dangerouslySetInnerHTML={{ __html: safePreviewHtml }} />
              </CardContent>
            </Card>
          )}
        </TabsContent>

        <TabsContent value="templates">
          <PrintTemplatesPage />
        </TabsContent>
      </Tabs>
    </div>
  );
}
