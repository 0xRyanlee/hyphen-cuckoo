import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Checkbox } from "@/components/ui/checkbox";
import { Plus, Pencil, Trash2, Scan, TestTube2, History, Printer, Wand2, Cloud, Wifi,
         CheckCircle2, XCircle, Loader2, ChevronLeft, ChevronRight, Zap } from "lucide-react";
import { EmptyState } from "@/components/ui/empty-state";
import { toast } from "sonner";

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
  updated_at: string;
}

interface LanPrinter {
  ip: string;
  port: number;
  sn: string | null;
}

interface PrintTask {
  id: number;
  task_type: string;
  ref_type: string | null;
  ref_id: number | null;
  content: string;
  status: string;
  printer_id: number | null;
  printer_name: string | null;
  created_at: string;
  printed_at: string | null;
  error_msg: string | null;
}

// ==================== 快速設置嚮導 ====================

type WizardMode = "feie" | "lan" | null;
type WizardStep = 1 | 2 | 3 | 4;

function SetupWizard({ open, onClose, onDone }: {
  open: boolean;
  onClose: () => void;
  onDone: () => void;
}) {
  const [step, setStep] = useState<WizardStep>(1);
  const [mode, setMode] = useState<WizardMode>(null);

  // Feie fields
  const [feieSn, setFeieSn] = useState("");
  const [feieKey, setFeieKey] = useState("");
  const [feieUser, setFeieUser] = useState("");
  const [feieUkey, setFeieUkey] = useState("");

  // LAN fields
  const [lanIp, setLanIp] = useState("");
  const [lanPort] = useState("9100");
  const [scanSubnet, setScanSubnet] = useState("192.168.1");
  const [scanning, setScanning] = useState(false);
  const [lanPrinters, setLanPrinters] = useState<LanPrinter[]>([]);

  // Common
  const [printerName, setPrinterName] = useState("");
  const [paperWidth, setPaperWidth] = useState("58mm");
  const [isDefault, setIsDefault] = useState(true);

  // Testing / saving
  const [testing, setTesting] = useState(false);
  const [saving, setSaving] = useState(false);
  const [testResult, setTestResult] = useState<{ ok: boolean; msg: string } | null>(null);
  const [_savedPrinterId, setSavedPrinterId] = useState<number | null>(null);

  function reset() {
    setStep(1);
    setMode(null);
    setFeieSn(""); setFeieKey(""); setFeieUser(""); setFeieUkey("");
    setLanIp(""); setLanPrinters([]);
    setPrinterName(""); setPaperWidth("58mm"); setIsDefault(true);
    setTesting(false); setSaving(false); setTestResult(null); setSavedPrinterId(null);
  }

  function handleClose() { reset(); onClose(); }

  async function handleScanLan() {
    setScanning(true);
    setLanPrinters([]);
    try {
      const result = await invoke<LanPrinter[]>("scan_lan_printers", { subnet: scanSubnet, timeoutMs: 800 });
      setLanPrinters(result);
    } catch { /* ignore */ } finally { setScanning(false); }
  }

  function feieStep2Valid() { return feieSn.trim().length > 0 && feieKey.trim().length > 0; }
  function feieStep3Valid() { return feieUser.trim().length > 0 && feieUkey.trim().length > 0; }
  function lanStep2Valid() { return lanIp.trim().length > 0; }
  function nameValid() { return printerName.trim().length > 0; }

  async function handleFeieSetup() {
    if (!feieStep2Valid() || !feieStep3Valid() || !nameValid()) return;
    setSaving(true);
    setTestResult(null);
    try {
      // 1. Save to DB
      const printerId = await invoke<number>("create_printer", {
        req: {
          name: printerName.trim(),
          printer_type: "thermal",
          connection_type: "feie",
          feie_user: feieUser.trim(),
          feie_ukey: feieUkey.trim(),
          feie_sn: feieSn.trim(),
          feie_key: feieKey.trim(),
          lan_ip: null,
          lan_port: 9100,
          paper_width: paperWidth,
          is_default: isDefault,
        },
      });
      setSavedPrinterId(printerId);

      // 2. Register printer with Feie account
      setSaving(false);
      setTesting(true);
      await invoke("bind_feie_printer", { printerId, printerKey: feieKey.trim() });

      // 3. Test print
      const result = await invoke<string>("test_feie_printer", {
        user: feieUser.trim(),
        ukey: feieUkey.trim(),
        sn: feieSn.trim(),
      });
      const ok = result.toLowerCase().includes("ok") || result.includes("成功") || result.includes("\"ret\":0");
      setTestResult({ ok, msg: ok ? "打印机已成功连接，测试页已发送！请检查打印机是否出纸。" : result });
    } catch (e) {
      setTestResult({ ok: false, msg: friendlyError(String(e)) });
    } finally {
      setSaving(false);
      setTesting(false);
    }
  }

  async function handleLanSetup() {
    if (!lanStep2Valid() || !nameValid()) return;
    setSaving(true);
    setTestResult(null);
    try {
      // 1. Save to DB
      const printerId = await invoke<number>("create_printer", {
        req: {
          name: printerName.trim(),
          printer_type: "thermal",
          connection_type: "lan",
          feie_user: null,
          feie_ukey: null,
          feie_sn: null,
          feie_key: null,
          lan_ip: lanIp.trim(),
          lan_port: parseInt(lanPort) || 9100,
          paper_width: paperWidth,
          is_default: isDefault,
        },
      });
      setSavedPrinterId(printerId);

      // 2. Test connection
      setSaving(false);
      setTesting(true);
      const result = await invoke<string>("test_lan_printer", { ip: lanIp.trim(), port: parseInt(lanPort) || 9100 });
      const ok = result.includes("成功") || result.toLowerCase().includes("ok");
      setTestResult({ ok, msg: ok ? "打印机连接成功，测试页已发送！" : result });
    } catch (e) {
      setTestResult({ ok: false, msg: friendlyError(String(e)) });
    } finally {
      setSaving(false);
      setTesting(false);
    }
  }

  function friendlyError(raw: string): string {
    if (raw.includes("Connection refused") || raw.includes("拒绝连接")) return "无法连接到打印机。请确认打印机已开机，且与本设备在同一个 Wi-Fi 网络中。";
    if (raw.includes("timed out") || raw.includes("超时")) return "连接超时。打印机可能已关机，或 IP 地址不正确，请重试。";
    if (raw.includes("SN") || raw.includes("sn")) return "序列号或验证码不正确，请重新检查打印机标签上的 SN 和 KEY。";
    if (raw.includes("ukey") || raw.includes("user")) return "飞鹅账号或 UKEY 不正确，请登录 feieyun.cn 确认。";
    return raw;
  }

  function handleFinish() { reset(); onDone(); }

  const totalSteps = mode === "feie" ? 4 : mode === "lan" ? 3 : 1;
  const isWorking = saving || testing;

  return (
    <Dialog open={open} onOpenChange={(o) => { if (!o) handleClose(); }}>
      <DialogContent className="max-w-xl">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2 text-lg">
            <Wand2 className="h-5 w-5 text-primary" />
            打印机快速设置向导
          </DialogTitle>
        </DialogHeader>

        {/* Step indicator */}
        {mode && (
          <div className="flex items-center gap-1 px-1">
            {Array.from({ length: totalSteps }, (_, i) => (
              <div key={i} className="flex items-center gap-1">
                <div className={`h-2 flex-1 min-w-[32px] rounded-full transition-colors ${
                  i + 1 < step ? "bg-primary" : i + 1 === step ? "bg-primary/70" : "bg-muted"
                }`} />
              </div>
            ))}
            <span className="ml-2 text-xs text-muted-foreground whitespace-nowrap">第 {step} / {totalSteps} 步</span>
          </div>
        )}

        <div className="min-h-[280px] py-4">

          {/* ── Step 1: Choose mode ── */}
          {step === 1 && (
            <div className="space-y-4">
              <p className="text-sm text-muted-foreground">请选择打印机的连接方式：</p>
              <div className="grid grid-cols-2 gap-4">
                <button
                  onClick={() => { setMode("feie"); setStep(2); }}
                  className="group flex flex-col items-center gap-3 rounded-xl border-2 border-transparent bg-muted p-6 text-center transition-all hover:border-primary hover:bg-primary/5"
                >
                  <div className="flex h-14 w-14 items-center justify-center rounded-full bg-blue-500/10">
                    <Cloud className="h-7 w-7 text-blue-500" />
                  </div>
                  <div>
                    <p className="font-semibold">飞鹅云打印</p>
                    <p className="mt-1 text-xs text-muted-foreground">通过互联网远程打印，配置简单，推荐使用</p>
                  </div>
                  <Badge variant="default" className="text-xs">推荐</Badge>
                </button>

                <button
                  onClick={() => { setMode("lan"); setStep(2); }}
                  className="group flex flex-col items-center gap-3 rounded-xl border-2 border-transparent bg-muted p-6 text-center transition-all hover:border-primary hover:bg-primary/5"
                >
                  <div className="flex h-14 w-14 items-center justify-center rounded-full bg-emerald-500/10">
                    <Wifi className="h-7 w-7 text-emerald-500" />
                  </div>
                  <div>
                    <p className="font-semibold">局域网直连</p>
                    <p className="mt-1 text-xs text-muted-foreground">打印机与电脑在同一 Wi-Fi，打印速度更快</p>
                  </div>
                </button>
              </div>
            </div>
          )}

          {/* ── Feie Step 2: Printer label ── */}
          {step === 2 && mode === "feie" && (
            <div className="space-y-4">
              <div className="rounded-lg bg-amber-500/10 border border-amber-500/20 p-3 text-sm text-amber-700 dark:text-amber-400">
                请翻到打印机底部，找到白色标签，上面有 <strong>SN</strong>（序列号）和 <strong>KEY</strong>（验证码）两串数字。
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label>序列号 <span className="text-xs text-muted-foreground">（SN）</span></Label>
                  <Input
                    value={feieSn}
                    onChange={(e) => setFeieSn(e.target.value)}
                    placeholder="例：924754169"
                    className="font-mono"
                  />
                </div>
                <div className="space-y-2">
                  <Label>验证码 <span className="text-xs text-muted-foreground">（KEY）</span></Label>
                  <Input
                    value={feieKey}
                    onChange={(e) => setFeieKey(e.target.value)}
                    placeholder="例：8MhFyEy8"
                    className="font-mono"
                  />
                </div>
              </div>

              <Separator />

              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label>纸张宽度</Label>
                  <Select value={paperWidth} onValueChange={setPaperWidth}>
                    <SelectTrigger><SelectValue /></SelectTrigger>
                    <SelectContent>
                      <SelectItem value="58mm">58mm（小票机）</SelectItem>
                      <SelectItem value="80mm">80mm（大票机）</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-2">
                  <Label>打印机名称</Label>
                  <Input
                    value={printerName}
                    onChange={(e) => setPrinterName(e.target.value)}
                    placeholder="例：前台小票机"
                  />
                </div>
              </div>
            </div>
          )}

          {/* ── Feie Step 3: Account credentials ── */}
          {step === 3 && mode === "feie" && (
            <div className="space-y-4">
              <div className="rounded-lg bg-blue-500/10 border border-blue-500/20 p-3 text-sm text-blue-700 dark:text-blue-400">
                需要填写飞鹅云账号信息。如果还没有账号，请先在 <strong>feieyun.cn</strong> 免费注册，然后在「个人中心」→「UKEY」页面找到您的 UKEY。
              </div>

              <div className="space-y-2">
                <Label>飞鹅账号 <span className="text-xs text-muted-foreground">（手机号 / 邮箱）</span></Label>
                <Input
                  value={feieUser}
                  onChange={(e) => setFeieUser(e.target.value)}
                  placeholder="您的飞鹅账号"
                />
              </div>
              <div className="space-y-2">
                <Label>账号 UKEY <span className="text-xs text-muted-foreground">（在飞鹅网站的个人中心查看）</span></Label>
                <Input
                  value={feieUkey}
                  onChange={(e) => setFeieUkey(e.target.value)}
                  placeholder="飞鹅 UKEY"
                  className="font-mono"
                />
              </div>

              <div className="flex items-center gap-2 pt-1">
                <Checkbox
                  id="feie-default"
                  checked={isDefault}
                  onCheckedChange={(c) => setIsDefault(c === true)}
                />
                <label htmlFor="feie-default" className="text-sm cursor-pointer">设为默认打印机</label>
              </div>
            </div>
          )}

          {/* ── Feie Step 4: Connect & test ── */}
          {step === 4 && mode === "feie" && (
            <div className="flex flex-col items-center justify-center gap-6 text-center">
              {!testResult && !isWorking && (
                <div className="space-y-3">
                  <div className="flex h-16 w-16 items-center justify-center rounded-full bg-primary/10 mx-auto">
                    <Printer className="h-8 w-8 text-primary" />
                  </div>
                  <p className="text-sm text-muted-foreground">
                    准备就绪。点击下方按钮，系统将自动完成注册并发送测试页。
                    <br />请确认打印机已开机并连接到互联网。
                  </p>
                  <Button size="lg" onClick={handleFeieSetup} className="w-full">
                    <Wand2 className="mr-2 h-4 w-4" />开始连接并测试打印
                  </Button>
                </div>
              )}

              {isWorking && (
                <div className="space-y-3">
                  <Loader2 className="h-12 w-12 animate-spin text-primary mx-auto" />
                  <p className="text-sm text-muted-foreground">
                    {saving ? "正在保存配置..." : "正在注册并发送测试页，请稍候..."}
                  </p>
                </div>
              )}

              {testResult && (
                <div className="w-full space-y-4">
                  {testResult.ok ? (
                    <CheckCircle2 className="h-14 w-14 text-emerald-500 mx-auto" />
                  ) : (
                    <XCircle className="h-14 w-14 text-destructive mx-auto" />
                  )}
                  <p className={`text-sm font-medium ${testResult.ok ? "text-emerald-600" : "text-destructive"}`}>
                    {testResult.ok ? "连接成功！" : "连接失败"}
                  </p>
                  <div className={`rounded-lg p-3 text-xs text-left ${testResult.ok ? "bg-emerald-500/10 text-emerald-700" : "bg-destructive/10 text-destructive"}`}>
                    {testResult.msg}
                  </div>
                  {testResult.ok ? (
                    <Button size="lg" onClick={handleFinish} className="w-full">
                      <CheckCircle2 className="mr-2 h-4 w-4" />完成设置
                    </Button>
                  ) : (
                    <div className="flex gap-2">
                      <Button variant="outline" className="flex-1" onClick={() => { setTestResult(null); setStep(2); }}>
                        <ChevronLeft className="mr-1 h-4 w-4" />返回修改
                      </Button>
                      <Button className="flex-1" onClick={handleFeieSetup}>
                        重新尝试
                      </Button>
                    </div>
                  )}
                </div>
              )}
            </div>
          )}

          {/* ── LAN Step 2: Scan or manual ── */}
          {step === 2 && mode === "lan" && (
            <div className="space-y-4">
              <div className="rounded-lg bg-emerald-500/10 border border-emerald-500/20 p-3 text-sm text-emerald-700 dark:text-emerald-400">
                请确保打印机已通过 Wi-Fi 连接到与本设备<strong>相同的路由器</strong>，然后点击「扫描」自动查找打印机。
              </div>

              <div className="flex gap-2">
                <div className="flex items-center flex-1 gap-1">
                  <Input value={scanSubnet} onChange={(e) => setScanSubnet(e.target.value)} className="max-w-[140px] font-mono text-sm" />
                  <span className="text-sm text-muted-foreground">.1 – .254</span>
                </div>
                <Button onClick={handleScanLan} disabled={scanning}>
                  <Scan className="mr-2 h-4 w-4" />
                  {scanning ? "扫描中..." : "扫描局域网"}
                </Button>
              </div>

              {scanning && (
                <div className="flex items-center gap-2 text-sm text-muted-foreground">
                  <Loader2 className="h-4 w-4 animate-spin" />扫描中，最多需要 15 秒...
                </div>
              )}

              {!scanning && lanPrinters.length === 0 && (
                <p className="text-xs text-muted-foreground">未发现打印机。请确认打印机已开机并连接到同一 Wi-Fi，然后重新扫描。</p>
              )}

              {lanPrinters.length > 0 && (
                <div className="space-y-2">
                  <p className="text-sm font-medium">发现 {lanPrinters.length} 台设备，请选择您的打印机：</p>
                  {lanPrinters.map((lp, i) => (
                    <button
                      key={i}
                      onClick={() => setLanIp(lp.ip)}
                      className={`w-full flex items-center justify-between rounded-lg border px-3 py-2 text-sm transition-colors ${
                        lanIp === lp.ip ? "border-primary bg-primary/5" : "hover:bg-muted"
                      }`}
                    >
                      <div className="flex items-center gap-2">
                        <Printer className="h-4 w-4 text-muted-foreground" />
                        <span className="font-mono">{lp.ip}:{lp.port}</span>
                        {lp.sn && <span className="text-xs text-muted-foreground">SN: {lp.sn}</span>}
                      </div>
                      {lanIp === lp.ip && <CheckCircle2 className="h-4 w-4 text-primary" />}
                    </button>
                  ))}
                </div>
              )}

              <Separator />
              <p className="text-xs text-muted-foreground">也可以手动输入 IP 地址：</p>
              <Input value={lanIp} onChange={(e) => setLanIp(e.target.value)} placeholder="例：192.168.1.100" className="font-mono" />

              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label>打印机名称</Label>
                  <Input value={printerName} onChange={(e) => setPrinterName(e.target.value)} placeholder="例：厨房打印机" />
                </div>
                <div className="space-y-2">
                  <Label>纸张宽度</Label>
                  <Select value={paperWidth} onValueChange={setPaperWidth}>
                    <SelectTrigger><SelectValue /></SelectTrigger>
                    <SelectContent>
                      <SelectItem value="58mm">58mm</SelectItem>
                      <SelectItem value="80mm">80mm</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </div>

              <div className="flex items-center gap-2">
                <Checkbox
                  id="lan-default"
                  checked={isDefault}
                  onCheckedChange={(c) => setIsDefault(c === true)}
                />
                <label htmlFor="lan-default" className="text-sm cursor-pointer">设为默认打印机</label>
              </div>
            </div>
          )}

          {/* ── LAN Step 3: Test & save ── */}
          {step === 3 && mode === "lan" && (
            <div className="flex flex-col items-center justify-center gap-6 text-center">
              {!testResult && !isWorking && (
                <div className="space-y-3">
                  <div className="flex h-16 w-16 items-center justify-center rounded-full bg-emerald-500/10 mx-auto">
                    <Printer className="h-8 w-8 text-emerald-600" />
                  </div>
                  <p className="text-sm text-muted-foreground">
                    即将连接到 <span className="font-mono font-medium">{lanIp}:9100</span><br />
                    点击下方按钮，系统将测试连接并发送测试页。
                  </p>
                  <Button size="lg" onClick={handleLanSetup} className="w-full">
                    <TestTube2 className="mr-2 h-4 w-4" />测试连接并保存
                  </Button>
                </div>
              )}

              {isWorking && (
                <div className="space-y-3">
                  <Loader2 className="h-12 w-12 animate-spin text-primary mx-auto" />
                  <p className="text-sm text-muted-foreground">
                    {saving ? "正在保存配置..." : "正在测试连接..."}
                  </p>
                </div>
              )}

              {testResult && (
                <div className="w-full space-y-4">
                  {testResult.ok ? (
                    <CheckCircle2 className="h-14 w-14 text-emerald-500 mx-auto" />
                  ) : (
                    <XCircle className="h-14 w-14 text-destructive mx-auto" />
                  )}
                  <p className={`text-sm font-medium ${testResult.ok ? "text-emerald-600" : "text-destructive"}`}>
                    {testResult.ok ? "连接成功！" : "连接失败"}
                  </p>
                  <div className={`rounded-lg p-3 text-xs text-left ${testResult.ok ? "bg-emerald-500/10 text-emerald-700" : "bg-destructive/10 text-destructive"}`}>
                    {testResult.msg}
                  </div>
                  {testResult.ok ? (
                    <Button size="lg" onClick={handleFinish} className="w-full">
                      <CheckCircle2 className="mr-2 h-4 w-4" />完成设置
                    </Button>
                  ) : (
                    <div className="flex gap-2">
                      <Button variant="outline" className="flex-1" onClick={() => { setTestResult(null); setStep(2); }}>
                        <ChevronLeft className="mr-1 h-4 w-4" />返回修改
                      </Button>
                      <Button className="flex-1" onClick={handleLanSetup}>
                        重新尝试
                      </Button>
                    </div>
                  )}
                </div>
              )}
            </div>
          )}
        </div>

        {/* Navigation footer */}
        <DialogFooter className="gap-2">
          {step === 1 && (
            <Button variant="outline" onClick={handleClose}>取消</Button>
          )}
          {step > 1 && !isWorking && !testResult && (
            <Button variant="outline" onClick={() => setStep((s) => Math.max(1, s - 1) as WizardStep)}>
              <ChevronLeft className="mr-1 h-4 w-4" />上一步
            </Button>
          )}
          {mode === "feie" && step === 2 && (
            <Button disabled={!feieStep2Valid() || !nameValid()} onClick={() => setStep(3)}>
              下一步<ChevronRight className="ml-1 h-4 w-4" />
            </Button>
          )}
          {mode === "feie" && step === 3 && (
            <Button disabled={!feieStep3Valid()} onClick={() => setStep(4)}>
              下一步<ChevronRight className="ml-1 h-4 w-4" />
            </Button>
          )}
          {mode === "lan" && step === 2 && (
            <Button disabled={!lanStep2Valid() || !nameValid()} onClick={() => setStep(3)}>
              下一步<ChevronRight className="ml-1 h-4 w-4" />
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

// ==================== 主頁面 ====================

export function PrintSettingsPage() {
  const [printers, setPrinters] = useState<PrinterConfig[]>([]);
  const [printTasks, setPrintTasks] = useState<PrintTask[]>([]);
  const [lanPrinters, setLanPrinters] = useState<LanPrinter[]>([]);
  const [scanning, setScanning] = useState(false);
  const [scanSubnet, setScanSubnet] = useState("192.168.1");
  const [error, setError] = useState<string | null>(null);
  const [autoPrint, setAutoPrint] = useState(() => localStorage.getItem("auto_print_kitchen") === "true");
  const [autoPrintPO, setAutoPrintPO] = useState(() => localStorage.getItem("auto_print_po") === "true");
  const [autoPrintReceipt, setAutoPrintReceipt] = useState(() => localStorage.getItem("auto_print_receipt") === "true");
  const [stations, setStations] = useState<{ id: number; name: string; printer_id: number | null }[]>([]);

  useEffect(() => {
    invoke<{ id: number; name: string; printer_id: number | null }[]>("get_kitchen_stations")
      .then(setStations)
      .catch(() => {});
  }, []);

  const [wizardOpen, setWizardOpen] = useState(false);
  const [addDialogOpen, setAddDialogOpen] = useState(false);
  const [editDialogOpen, setEditDialogOpen] = useState(false);
  const [testDialogOpen, setTestDialogOpen] = useState(false);
  const [deleteConfirm, setDeleteConfirm] = useState<PrinterConfig | null>(null);

  const [newPrinterName, setNewPrinterName] = useState("");
  const [newPrinterType, setNewPrinterType] = useState("thermal");
  const [newConnectionType, setNewConnectionType] = useState("feie");
  const [newFeieUser, setNewFeieUser] = useState("");
  const [newFeieUkey, setNewFeieUkey] = useState("");
  const [newFeieSn, setNewFeieSn] = useState("");
  const [newFeieKey, setNewFeieKey] = useState("");
  const [newLanIp, setNewLanIp] = useState("");
  const [newLanPort, setNewLanPort] = useState("9100");
  const [newPaperWidth, setNewPaperWidth] = useState("80mm");
  const [newIsDefault, setNewIsDefault] = useState(false);

  const [editPrinter, setEditPrinter] = useState<PrinterConfig | null>(null);
  const [testPrinter, setTestPrinter] = useState<PrinterConfig | null>(null);
  const [testResult, setTestResult] = useState<string | null>(null);
  const [testLoading, setTestLoading] = useState(false);

  async function loadPrinters() {
    try {
      const data = await invoke<PrinterConfig[]>("get_printers");
      setPrinters(data);
    } catch (e) { toast.error("加载打印机列表失败", { description: String(e) }); }
  }

  async function loadPrintTasks() {
    try {
      const data = await invoke<PrintTask[]>("get_print_tasks", { limit: 20 });
      setPrintTasks(data);
    } catch (e) { toast.error("加载打印任务失败", { description: String(e) }); }
  }

  useEffect(() => { loadPrinters(); loadPrintTasks(); }, []);

  async function handleAddPrinter() {
    try {
      await invoke("create_printer", {
        req: {
          name: newPrinterName,
          printer_type: newPrinterType,
          connection_type: newConnectionType,
          feie_user: newFeieUser || null,
          feie_ukey: newFeieUkey || null,
          feie_sn: newFeieSn || null,
          feie_key: newFeieKey || null,
          lan_ip: newLanIp || null,
          lan_port: parseInt(newLanPort) || 9100,
          paper_width: newPaperWidth,
          is_default: newIsDefault,
        },
      });
      setAddDialogOpen(false);
      resetAddForm();
      loadPrinters();
    } catch (e) { setError(String(e)); }
  }

  async function handleEditPrinter() {
    if (!editPrinter) return;
    try {
      await invoke("update_printer", {
        id: editPrinter.id,
        name: editPrinter.name,
        printerType: editPrinter.printer_type,
        connectionType: editPrinter.connection_type,
        feieUser: editPrinter.feie_user,
        feieUkey: editPrinter.feie_ukey,
        feieSn: editPrinter.feie_sn,
        feieKey: editPrinter.feie_key,
        lanIp: editPrinter.lan_ip,
        lanPort: editPrinter.lan_port,
        paperWidth: editPrinter.paper_width,
        isDefault: editPrinter.is_default,
      });
      setEditDialogOpen(false);
      loadPrinters();
    } catch (e) { setError(String(e)); }
  }

  async function handleDeletePrinter() {
    if (!deleteConfirm) return;
    try {
      await invoke("delete_printer", { id: deleteConfirm.id });
      setDeleteConfirm(null);
      loadPrinters();
    } catch (e) { setError(String(e)); }
  }

  async function handleTestPrinter() {
    if (!testPrinter) return;
    setTestLoading(true);
    setTestResult(null);
    try {
      if (testPrinter.connection_type === "feie") {
        const result = await invoke<string>("test_feie_printer", {
          user: testPrinter.feie_user,
          ukey: testPrinter.feie_ukey,
          sn: testPrinter.feie_sn,
        });
        setTestResult(result);
      } else {
        const result = await invoke<string>("test_lan_printer", {
          ip: testPrinter.lan_ip,
          port: testPrinter.lan_port,
        });
        setTestResult(result);
      }
    } catch (e) {
      setTestResult(String(e));
    } finally {
      setTestLoading(false);
    }
  }

  async function handleScanLan() {
    setScanning(true);
    setLanPrinters([]);
    try {
      const result = await invoke<LanPrinter[]>("scan_lan_printers", {
        subnet: scanSubnet,
        timeoutMs: 500,
      });
      setLanPrinters(result);
    } catch (e) { console.error(e); } finally {
      setScanning(false);
    }
  }

  function resetAddForm() {
    setNewPrinterName("");
    setNewPrinterType("thermal");
    setNewConnectionType("feie");
    setNewFeieUser("");
    setNewFeieUkey("");
    setNewFeieSn("");
    setNewFeieKey("");
    setNewLanIp("");
    setNewLanPort("9100");
    setNewPaperWidth("80mm");
    setNewIsDefault(false);
  }

  function openAddDialog() {
    resetAddForm();
    setAddDialogOpen(true);
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-semibold tracking-tight">打印设置</h2>
        <p className="text-sm text-muted-foreground">管理打印机、连接配置和打印任务</p>
      </div>

      {/* Auto-print setting */}
      <Card>
        <CardContent className="flex items-center justify-between pt-6">
          <div className="flex items-center gap-4">
            <div className="flex h-10 w-10 items-center justify-center rounded-full bg-amber-500/10">
              <Zap className="h-5 w-5 text-amber-500" />
            </div>
            <div>
              <p className="font-semibold">POS 下单自动打印厨房工单</p>
              <p className="text-sm text-muted-foreground">提交订单后立即将工单发送到打印机，无需手动在 KDS 触发</p>
            </div>
          </div>
          <Checkbox
            checked={autoPrint}
            onCheckedChange={(v) => {
              const next = !!v;
              setAutoPrint(next);
              localStorage.setItem("auto_print_kitchen", next ? "true" : "false");
            }}
          />
        </CardContent>
      </Card>

      {/* PO receive auto-print setting */}
      <Card>
        <CardContent className="flex items-center justify-between pt-6">
          <div className="flex items-center gap-4">
            <div className="flex h-10 w-10 items-center justify-center rounded-full bg-blue-500/10">
              <Printer className="h-5 w-5 text-blue-500" />
            </div>
            <div>
              <p className="font-semibold">入库自动打印批次标签</p>
              <p className="text-sm text-muted-foreground">采购单入库后自动为每个批次打印标签，方便盘点和追溯</p>
            </div>
          </div>
          <Checkbox
            checked={autoPrintPO}
            onCheckedChange={(v) => {
              const next = !!v;
              setAutoPrintPO(next);
              localStorage.setItem("auto_print_po", next ? "true" : "false");
            }}
          />
        </CardContent>
      </Card>

      {/* Receipt auto-print setting */}
      <Card>
        <CardContent className="flex items-center justify-between pt-6">
          <div className="flex items-center gap-4">
            <div className="flex h-10 w-10 items-center justify-center rounded-full bg-green-500/10">
              <Zap className="h-5 w-5 text-green-500" />
            </div>
            <div>
              <p className="font-semibold">收款后自动打印收据</p>
              <p className="text-sm text-muted-foreground">登记收款后立即将收据发送到默认打印机</p>
            </div>
          </div>
          <Checkbox
            checked={autoPrintReceipt}
            onCheckedChange={(v) => {
              const next = !!v;
              setAutoPrintReceipt(next);
              localStorage.setItem("auto_print_receipt", next ? "true" : "false");
            }}
          />
        </CardContent>
      </Card>

      {/* Station printer assignment */}
      {stations.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Printer className="h-4 w-4" />
              廚房工作站打印機
            </CardTitle>
            <CardDescription>為每個工作站指定專屬打印機，未指定時使用默認打印機</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            {stations.map((station) => (
              <div key={station.id} className="flex items-center justify-between">
                <span className="text-sm font-medium">{station.name}</span>
                <Select
                  value={station.printer_id != null ? String(station.printer_id) : "default"}
                  onValueChange={async (val) => {
                    const newId = val === "default" ? null : Number(val);
                    try {
                      await invoke("update_station_printer", { stationId: station.id, printerId: newId });
                      setStations((prev) => prev.map((s) => s.id === station.id ? { ...s, printer_id: newId } : s));
                    } catch (e) { toast.error("更新失败", { description: String(e) }); }
                  }}
                >
                  <SelectTrigger className="w-48">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="default">使用默認打印機</SelectItem>
                    {printers.filter((p) => p.is_active).map((p) => (
                      <SelectItem key={p.id} value={String(p.id)}>{p.name}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            ))}
          </CardContent>
        </Card>
      )}

      {/* Quick setup wizard card */}
      <Card className="border-primary/30 bg-primary/5">
        <CardContent className="flex items-center justify-between pt-6">
          <div className="flex items-center gap-4">
            <div className="flex h-12 w-12 items-center justify-center rounded-full bg-primary/10">
              <Wand2 className="h-6 w-6 text-primary" />
            </div>
            <div>
              <p className="font-semibold">快速设置向导</p>
              <p className="text-sm text-muted-foreground">首次添加打印机？按步骤引导，三分钟完成配置</p>
            </div>
          </div>
          <Button onClick={() => setWizardOpen(true)}>
            <Wand2 className="mr-2 h-4 w-4" />启动向导
          </Button>
        </CardContent>
      </Card>

      <SetupWizard
        open={wizardOpen}
        onClose={() => setWizardOpen(false)}
        onDone={() => { setWizardOpen(false); loadPrinters(); }}
      />

      {error && (
        <div className="rounded-lg border border-destructive/50 bg-destructive/10 p-4 text-sm text-destructive">
          {error}
          <Button variant="link" onClick={() => setError(null)} className="ml-2">关闭</Button>
        </div>
      )}

      {/* Printers List */}
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-medium">打印机列表</h3>
          <p className="text-sm text-muted-foreground">共 {printers.length} 台打印机</p>
        </div>
        <Button variant="outline" onClick={openAddDialog}>
          <Plus className="mr-2 h-4 w-4" />手动添加
        </Button>
      </div>

      <Card>
        <CardContent className="pt-6">
          {printers.length === 0 ? (
            <EmptyState icon={Printer} title="暂无打印机" description="使用快速向导或手动添加打印机" action={<Button onClick={() => setWizardOpen(true)}><Wand2 className="mr-2 h-4 w-4" />启动向导</Button>} />
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>名称</TableHead>
                  <TableHead>类型</TableHead>
                  <TableHead>连接方式</TableHead>
                  <TableHead>纸张</TableHead>
                  <TableHead>状态</TableHead>
                  <TableHead className="text-right">操作</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {printers.map((p) => (
                  <TableRow key={p.id}>
                    <TableCell className="font-medium">
                      {p.name}
                      {p.is_default && <Badge variant="default" className="ml-2 text-xs">默认</Badge>}
                    </TableCell>
                    <TableCell>
                      <Badge variant="outline">{p.printer_type === "thermal" ? "热敏小票" : "标签"}</Badge>
                    </TableCell>
                    <TableCell className="text-xs">
                      {p.connection_type === "feie" ? (
                        <span className="flex items-center gap-1"><Cloud className="h-3 w-3" />飞鹅云 ({p.feie_sn})</span>
                      ) : (
                        <span className="flex items-center gap-1"><Wifi className="h-3 w-3" />局域网 ({p.lan_ip}:{p.lan_port})</span>
                      )}
                    </TableCell>
                    <TableCell className="text-xs">{p.paper_width}</TableCell>
                    <TableCell>
                      <Badge variant={p.is_active ? "default" : "secondary"}>
                        {p.is_active ? "启用" : "停用"}
                      </Badge>
                    </TableCell>
                    <TableCell className="text-right">
                      <div className="flex justify-end gap-1">
                        <Button variant="ghost" size="icon" className="h-8 w-8" onClick={() => { setTestPrinter(p); setTestResult(null); setTestDialogOpen(true); }}>
                          <TestTube2 className="h-4 w-4" />
                        </Button>
                        <Button variant="ghost" size="icon" className="h-8 w-8" onClick={() => { setEditPrinter(p); setEditDialogOpen(true); }}>
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

      {/* LAN Scanner */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Scan className="h-4 w-4" />
            局域网扫描
          </CardTitle>
          <CardDescription>扫描局域网内 TCP 9100 端口的打印机</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex gap-2">
            <Input value={scanSubnet} onChange={(e) => setScanSubnet(e.target.value)} placeholder="192.168.1" className="max-w-[200px]" />
            <span className="flex items-center text-sm text-muted-foreground">.1 - .254</span>
            <Button onClick={handleScanLan} disabled={scanning}>
              <Scan className="mr-2 h-4 w-4" />
              {scanning ? "扫描中..." : "开始扫描"}
            </Button>
          </div>
          {lanPrinters.length > 0 && (
            <div className="space-y-2">
              <p className="text-sm font-medium">发现 {lanPrinters.length} 台打印机</p>
              {lanPrinters.map((lp, i) => (
                <div key={i} className="flex items-center justify-between rounded-md border px-3 py-2 text-sm">
                  <span>{lp.ip}:{lp.port}</span>
                  <Button size="sm" variant="outline" onClick={() => { setNewLanIp(lp.ip); openAddDialog(); setNewConnectionType("lan"); }}>
                    <Plus className="mr-1 h-3 w-3" />添加
                  </Button>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Print Tasks History */}
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-medium">打印任务历史</h3>
          <p className="text-sm text-muted-foreground">最近 20 条打印任务</p>
        </div>
        <Button variant="outline" onClick={loadPrintTasks}>
          <History className="mr-2 h-4 w-4" />刷新
        </Button>
      </div>

      <Card>
        <CardContent className="pt-6">
          {printTasks.length === 0 ? (
            <EmptyState icon={Printer} title="暂无打印任务" description="打印任务记录将在此显示" />
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>类型</TableHead>
                  <TableHead>打印机</TableHead>
                  <TableHead>状态</TableHead>
                  <TableHead>创建时间</TableHead>
                  <TableHead>打印时间</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {printTasks.map((t) => (
                  <TableRow key={t.id}>
                    <TableCell>
                      <Badge variant="outline">
                        {t.task_type === "kitchen_ticket" ? "厨房单" : t.task_type === "batch_label" ? "批次标签" : t.task_type}
                      </Badge>
                    </TableCell>
                    <TableCell className="text-xs">{t.printer_name || "-"}</TableCell>
                    <TableCell>
                      <Badge variant={t.status === "printed" ? "default" : t.status === "failed" ? "destructive" : "secondary"}>
                        {t.status === "printed" ? "已打印" : t.status === "failed" ? "失败" : "待打印"}
                      </Badge>
                    </TableCell>
                    <TableCell className="text-xs text-muted-foreground">{t.created_at}</TableCell>
                    <TableCell className="text-xs text-muted-foreground">{t.printed_at || "-"}</TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      {/* Add Printer Dialog */}
      <Dialog open={addDialogOpen} onOpenChange={setAddDialogOpen}>
        <DialogContent className="max-w-lg">
          <DialogHeader>
            <DialogTitle>手动添加打印机</DialogTitle>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label>打印机名称</Label>
              <Input value={newPrinterName} onChange={(e) => setNewPrinterName(e.target.value)} placeholder="如：前台小票机" />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label>打印机类型</Label>
                <Select value={newPrinterType} onValueChange={setNewPrinterType}>
                  <SelectTrigger><SelectValue /></SelectTrigger>
                  <SelectContent>
                    <SelectItem value="thermal">热敏小票机</SelectItem>
                    <SelectItem value="label">标签打印机</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <Label>连接方式</Label>
                <Select value={newConnectionType} onValueChange={setNewConnectionType}>
                  <SelectTrigger><SelectValue /></SelectTrigger>
                  <SelectContent>
                    <SelectItem value="feie">飞鹅云打印</SelectItem>
                    <SelectItem value="lan">局域网 TCP</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>

            {newConnectionType === "feie" && (
              <>
                <Separator />
                <p className="text-sm font-medium">飞鹅云配置</p>
                <div className="grid grid-cols-2 gap-4">
                  <div className="space-y-2">
                    <Label>USER</Label>
                    <Input value={newFeieUser} onChange={(e) => setNewFeieUser(e.target.value)} placeholder="飞鹅用户名" />
                  </div>
                  <div className="space-y-2">
                    <Label>UKEY</Label>
                    <Input value={newFeieUkey} onChange={(e) => setNewFeieUkey(e.target.value)} placeholder="飞鹅 UKEY" />
                  </div>
                </div>
                <div className="grid grid-cols-2 gap-4">
                  <div className="space-y-2">
                    <Label>打印机 SN</Label>
                    <Input value={newFeieSn} onChange={(e) => setNewFeieSn(e.target.value)} placeholder="打印机序列号" />
                  </div>
                  <div className="space-y-2">
                    <Label>打印机 KEY</Label>
                    <Input value={newFeieKey} onChange={(e) => setNewFeieKey(e.target.value)} placeholder="打印机 KEY" />
                  </div>
                </div>
              </>
            )}

            {newConnectionType === "lan" && (
              <>
                <Separator />
                <p className="text-sm font-medium">局域网配置</p>
                <div className="grid grid-cols-3 gap-4">
                  <div className="col-span-2 space-y-2">
                    <Label>IP 地址</Label>
                    <Input value={newLanIp} onChange={(e) => setNewLanIp(e.target.value)} placeholder="192.168.1.100" />
                  </div>
                  <div className="space-y-2">
                    <Label>端口</Label>
                    <Input value={newLanPort} onChange={(e) => setNewLanPort(e.target.value)} placeholder="9100" />
                  </div>
                </div>
              </>
            )}

            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label>纸张宽度</Label>
                <Select value={newPaperWidth} onValueChange={setNewPaperWidth}>
                  <SelectTrigger><SelectValue /></SelectTrigger>
                  <SelectContent>
                    <SelectItem value="58mm">58mm</SelectItem>
                    <SelectItem value="80mm">80mm</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="flex items-end">
                <label className="flex items-center gap-2 text-sm cursor-pointer">
                  <Checkbox checked={newIsDefault} onCheckedChange={(checked) => setNewIsDefault(checked === true)} />
                  设为默认打印机
                </label>
              </div>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setAddDialogOpen(false)}>取消</Button>
            <Button onClick={handleAddPrinter}>添加</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Edit Printer Dialog */}
      <Dialog open={editDialogOpen} onOpenChange={setEditDialogOpen}>
        <DialogContent className="max-w-lg">
          <DialogHeader>
            <DialogTitle>编辑打印机</DialogTitle>
          </DialogHeader>
          {editPrinter && (
            <div className="space-y-4 py-4">
              <div className="space-y-2">
                <Label>打印机名称</Label>
                <Input value={editPrinter.name} onChange={(e) => setEditPrinter({ ...editPrinter, name: e.target.value })} />
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label>打印机类型</Label>
                  <Select value={editPrinter.printer_type} onValueChange={(v) => setEditPrinter({ ...editPrinter, printer_type: v })}>
                    <SelectTrigger><SelectValue /></SelectTrigger>
                    <SelectContent>
                      <SelectItem value="thermal">热敏小票机</SelectItem>
                      <SelectItem value="label">标签打印机</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-2">
                  <Label>连接方式</Label>
                  <Select value={editPrinter.connection_type} onValueChange={(v) => setEditPrinter({ ...editPrinter, connection_type: v })}>
                    <SelectTrigger><SelectValue /></SelectTrigger>
                    <SelectContent>
                      <SelectItem value="feie">飞鹅云打印</SelectItem>
                      <SelectItem value="lan">局域网 TCP</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </div>

              {editPrinter.connection_type === "feie" && (
                <>
                  <Separator />
                  <p className="text-sm font-medium">飞鹅云配置</p>
                  <div className="grid grid-cols-2 gap-4">
                    <div className="space-y-2">
                      <Label>USER</Label>
                      <Input value={editPrinter.feie_user || ""} onChange={(e) => setEditPrinter({ ...editPrinter, feie_user: e.target.value })} />
                    </div>
                    <div className="space-y-2">
                      <Label>UKEY</Label>
                      <Input value={editPrinter.feie_ukey || ""} onChange={(e) => setEditPrinter({ ...editPrinter, feie_ukey: e.target.value })} />
                    </div>
                  </div>
                  <div className="grid grid-cols-2 gap-4">
                    <div className="space-y-2">
                      <Label>打印机 SN</Label>
                      <Input value={editPrinter.feie_sn || ""} onChange={(e) => setEditPrinter({ ...editPrinter, feie_sn: e.target.value })} />
                    </div>
                    <div className="space-y-2">
                      <Label>打印机 KEY</Label>
                      <Input value={editPrinter.feie_key || ""} onChange={(e) => setEditPrinter({ ...editPrinter, feie_key: e.target.value })} />
                    </div>
                  </div>
                </>
              )}

              {editPrinter.connection_type === "lan" && (
                <>
                  <Separator />
                  <p className="text-sm font-medium">局域网配置</p>
                  <div className="grid grid-cols-3 gap-4">
                    <div className="col-span-2 space-y-2">
                      <Label>IP 地址</Label>
                      <Input value={editPrinter.lan_ip || ""} onChange={(e) => setEditPrinter({ ...editPrinter, lan_ip: e.target.value })} />
                    </div>
                    <div className="space-y-2">
                      <Label>端口</Label>
                      <Input value={editPrinter.lan_port.toString()} onChange={(e) => setEditPrinter({ ...editPrinter, lan_port: parseInt(e.target.value) || 9100 })} />
                    </div>
                  </div>
                </>
              )}

              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label>纸张宽度</Label>
                  <Select value={editPrinter.paper_width} onValueChange={(v) => setEditPrinter({ ...editPrinter, paper_width: v })}>
                    <SelectTrigger><SelectValue /></SelectTrigger>
                    <SelectContent>
                      <SelectItem value="58mm">58mm</SelectItem>
                      <SelectItem value="80mm">80mm</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div className="flex items-end">
                  <label className="flex items-center gap-2 text-sm cursor-pointer">
                    <Checkbox checked={editPrinter.is_default} onCheckedChange={(checked) => setEditPrinter({ ...editPrinter, is_default: checked === true })} />
                    设为默认打印机
                  </label>
                </div>
              </div>
            </div>
          )}
          <DialogFooter>
            <Button variant="outline" onClick={() => setEditDialogOpen(false)}>取消</Button>
            <Button onClick={handleEditPrinter}>保存</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Test Printer Dialog */}
      <Dialog open={testDialogOpen} onOpenChange={setTestDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>测试打印 - {testPrinter?.name}</DialogTitle>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <p className="text-sm text-muted-foreground">
              将发送测试页到 {testPrinter?.connection_type === "feie" ? `飞鹅云 (${testPrinter?.feie_sn})` : `局域网 (${testPrinter?.lan_ip}:${testPrinter?.lan_port})`}
            </p>
            {testResult && (
              <div className={`rounded-md p-3 text-sm ${testResult.includes("ok") || testResult.includes("成功") ? "bg-emerald-500/10 text-emerald-500" : "bg-destructive/10 text-destructive"}`}>
                {testResult}
              </div>
            )}
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setTestDialogOpen(false)}>关闭</Button>
            <Button onClick={handleTestPrinter} disabled={testLoading}>
              <TestTube2 className="mr-2 h-4 w-4" />
              {testLoading ? "发送中..." : "发送测试页"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Delete Confirm Dialog */}
      <Dialog open={!!deleteConfirm} onOpenChange={() => setDeleteConfirm(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>确认删除</DialogTitle>
          </DialogHeader>
          <p className="py-4 text-sm text-muted-foreground">
            确定要删除打印机「{deleteConfirm?.name}」吗？
          </p>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteConfirm(null)}>取消</Button>
            <Button variant="destructive" onClick={handleDeletePrinter}>删除</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
