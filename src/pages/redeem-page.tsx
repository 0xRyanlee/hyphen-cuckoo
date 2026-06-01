import { useState } from "react";
import { useParams } from "react-router-dom";
import { call } from "@/lib/transport";

interface RedeemResult {
  ok: boolean;
  already?: boolean;
  reason?: string;
  order_no?: string;
  component?: string;
  ch?: string;
}

const COMPONENT_LABELS: Record<string, string> = {
  character_collect: "集字兑奖",
  fortune: "运势卡",
  discount_coupon: "折价券",
  riddle: "谜语挑战",
};

/** Staff scans the marketing QR on a customer's phone → opens this page → confirms redeem. */
export function RedeemPage() {
  const { token } = useParams<{ token: string }>();
  const [result, setResult] = useState<RedeemResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [done, setDone] = useState(false);

  async function doRedeem() {
    if (!token) return;
    setLoading(true);
    try {
      const r = await call<RedeemResult>("redeem_marketing_qr_token", { token });
      setResult(r);
      setDone(true);
    } catch (e) {
      setResult({ ok: false, reason: e instanceof Error ? e.message : "error" });
      setDone(true);
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="min-h-screen bg-gray-50 flex items-center justify-center p-6">
      <div className="w-full max-w-sm bg-white rounded-2xl shadow-lg p-6 text-center">
        {!done ? (
          <>
            <div className="text-4xl mb-3">🎟️</div>
            <p className="text-lg font-bold text-gray-900 mb-1">营销核销</p>
            <p className="text-sm text-gray-500 mb-5">店员确认与顾客出示一致后点击核销</p>
            <button
              onClick={doRedeem}
              disabled={loading}
              className="w-full py-4 rounded-2xl bg-orange-500 text-white font-bold text-base disabled:opacity-50"
            >
              {loading ? "核销中..." : "确认核销"}
            </button>
          </>
        ) : result?.ok ? (
          <>
            <div className="text-5xl mb-3">✅</div>
            <p className="text-xl font-bold text-green-700 mb-1">核销成功</p>
            <p className="text-sm text-gray-600">
              {COMPONENT_LABELS[result.component ?? ""] ?? result.component}
              {result.ch ? ` · 【${result.ch}】` : ""}
            </p>
            <p className="text-xs text-gray-400 mt-1 font-mono">{result.order_no}</p>
          </>
        ) : result?.already ? (
          <>
            <div className="text-5xl mb-3">⚠️</div>
            <p className="text-xl font-bold text-amber-600 mb-1">该码已核销</p>
            <p className="text-sm text-gray-600">此营销码此前已被核销，不能重复使用</p>
            <p className="text-xs text-gray-400 mt-1 font-mono">{result.order_no}</p>
          </>
        ) : (
          <>
            <div className="text-5xl mb-3">❌</div>
            <p className="text-xl font-bold text-red-600 mb-1">核销失败</p>
            <p className="text-sm text-gray-600">
              {result?.reason === "invalid_signature" ? "二维码无效或被篡改" : "无法识别此码"}
            </p>
          </>
        )}
      </div>
    </div>
  );
}
