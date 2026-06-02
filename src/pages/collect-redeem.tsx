import { useState } from "react";
import { call as invoke } from "@/lib/transport";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { toast } from "sonner";
import { Plus, Check, X } from "lucide-react";

interface Collected {
  token: string;
  ch: string;
}

interface PeekResult {
  valid: boolean;
  reason?: string;
  token?: string;
  ch?: string;
  already_void?: boolean;
}

interface FindResult {
  valid: boolean;
  token?: string;
  ch?: string;
  already_void?: boolean;
}

function extractToken(raw: string): string {
  const s = raw.trim();
  if (s.includes("/redeem/")) return s.split("/redeem/")[1].split(/[?#]/)[0];
  return s;
}

/** Staff 集字兑换: accumulate scanned/typed coupon codes, show collected chars,
 *  then redeem the whole set at once (batch void). Order-no fallback supported. */
export function CollectRedeem() {
  const [input, setInput] = useState("");
  const [collected, setCollected] = useState<Collected[]>([]);
  const [busy, setBusy] = useState(false);

  async function addOne() {
    const raw = input.trim();
    if (!raw) return;
    setBusy(true);
    try {
      // A code (contains '.') or /redeem/ URL → peek by token; else treat as order_no.
      const looksLikeToken = raw.includes(".") || raw.includes("/redeem/");
      let token: string | undefined;
      let ch: string | undefined;
      let alreadyVoid = false;

      if (looksLikeToken) {
        const r = await invoke<PeekResult>("peek_marketing_qr_token", { token: extractToken(raw) });
        if (!r.valid) { toast.error("核销码无效"); return; }
        token = r.token; ch = r.ch; alreadyVoid = !!r.already_void;
      } else {
        const r = await invoke<FindResult>("find_collect_token_by_order_no", { orderNo: raw });
        if (!r.valid || !r.token) { toast.error(`序号 ${raw} 未找到集字券`); return; }
        token = r.token; ch = r.ch; alreadyVoid = !!r.already_void;
      }

      if (alreadyVoid) { toast.error(`【${ch ?? "?"}】该码已核销`); return; }
      if (!token) { toast.error("解析失败"); return; }
      if (collected.some((c) => c.token === token)) { toast("该券已在列表中", { icon: "ℹ️" }); return; }

      setCollected((prev) => [...prev, { token: token!, ch: ch ?? "?" }]);
      setInput("");
    } catch (e) {
      toast.error("读取失败", { description: String(e) });
    } finally {
      setBusy(false);
    }
  }

  function removeAt(idx: number) {
    setCollected((prev) => prev.filter((_, i) => i !== idx));
  }

  async function redeemSet() {
    if (collected.length === 0) return;
    setBusy(true);
    try {
      const r = await invoke<{ ok: boolean; reason?: string; chars?: string[]; count?: number; ch?: string }>(
        "collect_redeem_set", { tokens: collected.map((c) => c.token) }
      );
      if (r.ok) {
        toast.success(`集字兑换完成：${(r.chars ?? []).join("")}（${r.count} 张）`);
        setCollected([]);
      } else if (r.reason === "already_void") {
        toast.error(`【${r.ch ?? "?"}】已被核销，请移除后重试`);
      } else {
        toast.error("兑换失败，请检查券码");
      }
    } catch (e) {
      toast.error("兑换失败", { description: String(e) });
    } finally {
      setBusy(false);
    }
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">集字兑换</CardTitle>
        <CardDescription>
          顾客集齐字组后出示多张截图：逐张扫码 / 粘贴核销码 / 输入序号加入，凑齐后一次性核销
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-3">
        <div className="flex gap-2">
          <Input
            placeholder="扫码 / 粘贴核销码 / 输入订单序号"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && addOne()}
            className="flex-1"
          />
          <Button variant="secondary" onClick={addOne} disabled={busy || !input.trim()}>
            <Plus className="h-4 w-4 mr-1" />加入
          </Button>
        </div>

        {collected.length > 0 && (
          <>
            <div className="flex flex-wrap gap-2">
              {collected.map((c, i) => (
                <span key={c.token} className="inline-flex items-center gap-1 bg-orange-50 border border-orange-200 text-orange-700 rounded-full pl-3 pr-1 py-1 text-sm">
                  <span className="font-bold">{c.ch}</span>
                  <button onClick={() => removeAt(i)} className="text-orange-400 hover:text-orange-600">
                    <X className="h-3.5 w-3.5" />
                  </button>
                </span>
              ))}
            </div>
            <Button onClick={redeemSet} disabled={busy} className="w-full">
              <Check className="h-4 w-4 mr-1" />
              确认集字兑换（{collected.length} 张：{collected.map((c) => c.ch).join("")}）
            </Button>
            <p className="text-[11px] text-muted-foreground text-center">
              核销后这些券将一次性作废，不可重复使用
            </p>
          </>
        )}
      </CardContent>
    </Card>
  );
}
