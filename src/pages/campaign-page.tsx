import { useState, useEffect } from "react";
import { useParams } from "react-router-dom";
import { call } from "@/lib/transport";
import { StyledQR } from "@/components/styled-qr";

interface CampaignInfo {
  id: number;
  name: string;
  discount_type: string;
  discount_value: number;
  condition_text: string | null;
  valid_days: number;
}

interface ResolveResult {
  valid: boolean;
  reason?: string;
  coupon_token?: string;
  campaign?: CampaignInfo;
}

function discountText(t: string, v: number): string {
  if (t === "percent") return `${v}% OFF`;
  if (t === "amount") return `减 ¥${v}`;
  return "免费赠品";
}

/** Customer scans a campaign poster QR (/c/:token) → gets a fresh single-use coupon. */
export function CampaignPage() {
  const { token } = useParams<{ token: string }>();
  const [data, setData] = useState<ResolveResult | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!token) return;
    // MUL: same-device, same-poster, same-day claim guard (防呆，防误领/刷领).
    const key = `cuckoo_campaign_${token}_${new Date().toISOString().slice(0, 10)}`;
    if (localStorage.getItem(key)) {
      setData({ valid: false, reason: "already_claimed_today" });
      setLoading(false);
      return;
    }
    call<ResolveResult>("resolve_campaign", { token })
      .then((r) => {
        if (r.valid) localStorage.setItem(key, "1");
        setData(r);
      })
      .catch(() => setData({ valid: false }))
      .finally(() => setLoading(false));
  }, [token]);

  if (loading) {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center">
        <div className="text-gray-400 text-sm">领取中...</div>
      </div>
    );
  }

  if (!data?.valid || !data.campaign || !data.coupon_token) {
    const claimedToday = data?.reason === "already_claimed_today";
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center p-6">
        <div className="text-center text-gray-500">
          <p className="text-4xl mb-3">{claimedToday ? "✅" : "🎫"}</p>
          <p className="font-semibold text-gray-700">
            {claimedToday ? "今日已领取，明天再来吧" : "活动已结束或二维码无效"}
          </p>
        </div>
      </div>
    );
  }

  const c = data.campaign;
  return (
    <div className="min-h-screen bg-gradient-to-b from-red-50 to-orange-50 flex flex-col items-center justify-start pt-10 px-6 gap-4 pb-10">
      <div className="text-center">
        <div className="text-5xl mb-2">🎁</div>
        <p className="text-xl font-bold text-red-700">{c.name}</p>
        <p className="text-sm text-red-500 mt-1">恭喜获得专属优惠券</p>
      </div>
      <div className="w-full max-w-sm rounded-2xl border-2 border-dashed border-red-300 bg-white p-6 text-center">
        <div className="text-4xl font-extrabold text-red-600 mb-2">{discountText(c.discount_type, c.discount_value)}</div>
        {c.condition_text && <div className="text-sm text-red-500 mb-1">{c.condition_text}</div>}
        <div className="text-xs text-red-400 mb-4">{c.valid_days}天内有效</div>
        <div className="flex flex-col items-center gap-1 pt-4 border-t border-red-100">
          <StyledQR value={`${window.location.origin}/#/redeem/${data.coupon_token}`} size={140} dotColor="#dc2626" />
          <div className="text-[10px] text-red-400">店员扫码核销 · 一码一次</div>
        </div>
      </div>
      <p className="text-xs text-red-500/70 text-center max-w-sm">📷 截图保存此券 · 到店出示给店员扫码核销</p>
    </div>
  );
}
