import { useState, useEffect, useRef, useCallback } from "react";
import { useParams } from "react-router-dom";
import { call } from "@/lib/transport";
import { StyledQR } from "@/components/styled-qr";
import type { PublicMenuCategory, PublicMenuItem, PublicMenuItemSpec, TableOrderSummary } from "@/types";

// ── Types ──────────────────────────────────────────────────────────────────

interface MarketingPopupData {
  order_id: number;
  order_no: string;
  table_no: string;
  created_at: string;
  amount_total: number;
  template_content: string;  // JSON elements[]
}

interface CartModifier {
  modifier_type: string;
  price_delta: number;
}

interface CartItem {
  menu_item_id: number;
  name: string;
  spec_code: string | null;
  spec_name: string | null;
  unit_price: number;  // base + sum(modifier price_delta), per single item
  qty: number;
  note: string;
  modifiers: CartModifier[];
}

// ── Helpers ────────────────────────────────────────────────────────────────

function modSig(mods: CartModifier[]) {
  return mods.map((m) => m.modifier_type).sort().join(",");
}

function cartKey(item: CartItem) {
  return `${item.menu_item_id}__${item.spec_code ?? ""}__${modSig(item.modifiers)}`;
}

function priceOf(item: PublicMenuItem, specCode: string | null): number {
  if (!specCode) return item.sales_price;
  const spec = item.specs.find((s) => s.spec_code === specCode);
  return item.sales_price + (spec?.price_delta ?? 0);
}

function statusLabel(status: string) {
  switch (status) {
    case "pending": return { text: "等待确认", cls: "bg-blue-100 text-blue-700" };
    case "submitted": return { text: "备餐中", cls: "bg-amber-100 text-amber-700" };
    case "ready": return { text: "完成", cls: "bg-green-100 text-green-700" };
    case "cancelled": return { text: "已取消", cls: "bg-gray-100 text-gray-500" };
    default: return { text: status, cls: "bg-gray-100 text-gray-500" };
  }
}

function fmt(n: number) {
  return n % 1 === 0 ? String(n) : n.toFixed(2);
}

// ── Category color palette (for photo-less items) ──────────────────────────

const CAT_COLORS = [
  "from-orange-400 to-red-400",
  "from-blue-400 to-cyan-400",
  "from-green-400 to-emerald-400",
  "from-purple-400 to-pink-400",
  "from-yellow-400 to-amber-400",
  "from-teal-400 to-green-400",
];

// ── Sub-components ─────────────────────────────────────────────────────────

const MODIFIER_OPTIONS: { label: string; price: number }[] = [
  { label: "少冰", price: 0 },
  { label: "去冰", price: 0 },
  { label: "少糖", price: 0 },
  { label: "半糖", price: 0 },
  { label: "不要辣", price: 0 },
  { label: "微辣", price: 0 },
  { label: "加珍珠", price: 3 },
  { label: "加椰果", price: 3 },
  { label: "加布丁", price: 4 },
  { label: "加蛋", price: 5 },
  { label: "加大份", price: 8 },
];

function ItemCard({
  item,
  catIndex,
  cartQty,
  onAdd,
  onRemove,
}: {
  item: PublicMenuItem;
  catIndex: number;
  cartQty: number;
  onAdd: (item: PublicMenuItem, spec: PublicMenuItemSpec | null, qty: number, modifiers: CartModifier[]) => void;
  onRemove: (item: PublicMenuItem, spec: PublicMenuItemSpec | null) => void;
}) {
  const [specOpen, setSpecOpen] = useState(false);
  const [selSpec, setSelSpec] = useState<PublicMenuItemSpec | null>(null);
  const [qty, setQty] = useState(1);
  const [mods, setMods] = useState<string[]>([]);
  const hasSpecs = item.specs.length > 0;
  const colorClass = CAT_COLORS[catIndex % CAT_COLORS.length];

  function openSheet() {
    setSelSpec(hasSpecs ? item.specs[0] : null);
    setQty(1);
    setMods([]);
    setSpecOpen(true);
  }

  function toggleMod(label: string) {
    setMods((prev) => prev.includes(label) ? prev.filter((x) => x !== label) : [...prev, label]);
  }

  const modsExtra = MODIFIER_OPTIONS.filter((o) => mods.includes(o.label)).reduce((s, o) => s + o.price, 0);

  function confirmAdd() {
    const selected: CartModifier[] = MODIFIER_OPTIONS
      .filter((o) => mods.includes(o.label))
      .map((o) => ({ modifier_type: o.label, price_delta: o.price }));
    onAdd(item, selSpec, qty, selected);
    setSpecOpen(false);
  }

  return (
    <>
      <div className="flex gap-3 py-3 border-b border-gray-100 last:border-0">
        {/* Thumbnail */}
        <div className="flex-shrink-0 w-20 h-20 rounded-xl overflow-hidden">
          {item.image_path ? (
            <img
              src={item.image_path}
              alt={item.name}
              className="w-full h-full object-cover"
              onError={(e) => {
                const t = e.currentTarget;
                t.style.display = "none";
                const fb = t.nextElementSibling as HTMLElement | null;
                if (fb) fb.style.display = "flex";
              }}
            />
          ) : null}
          <div
            className={`w-full h-full bg-gradient-to-br ${colorClass} items-center justify-center`}
            style={{ display: item.image_path ? "none" : "flex" }}
          >
            <span className="text-white text-xl font-bold">{item.name[0]}</span>
          </div>
        </div>
        {/* Info */}
        <div className="flex-1 min-w-0 flex flex-col justify-between">
          <div>
            <div className="flex items-center gap-1.5">
              <p className="font-semibold text-gray-900 text-sm leading-tight">{item.name}</p>
              {item.is_hot && (
                <span className="text-[9px] bg-red-100 text-red-500 px-1 py-0.5 rounded font-bold shrink-0">热销</span>
              )}
            </div>
            {item.description && (
              <p className="text-xs text-gray-500 mt-0.5 line-clamp-2">{item.description}</p>
            )}
          </div>
          <div className="flex items-center justify-between mt-1">
            <span className="text-orange-500 font-bold text-base">¥{fmt(item.sales_price)}</span>
            <div className="flex items-center gap-2">
              {cartQty > 0 && (
                <>
                  <button
                    onClick={() => onRemove(item, null)}
                    className="w-11 h-11 rounded-full border-2 border-orange-400 text-orange-400 flex items-center justify-center font-bold text-lg leading-none"
                  >−</button>
                  <span className="text-sm font-semibold w-4 text-center">{cartQty}</span>
                </>
              )}
              <button
                onClick={openSheet}
                className="w-11 h-11 rounded-full bg-orange-400 text-white flex items-center justify-center font-bold text-lg leading-none shadow-sm"
              >+</button>
            </div>
          </div>
        </div>
      </div>

      {/* Add sheet: spec + quantity + flavors */}
      {specOpen && (
        <div className="fixed inset-0 z-50 flex items-end bg-black/20" onClick={() => setSpecOpen(false)}>
          <div className="w-full bg-white rounded-t-2xl p-5 shadow-2xl max-h-[80vh] overflow-y-auto" onClick={(e) => e.stopPropagation()}>
            <p className="text-base font-bold mb-3">{item.name}</p>

            {hasSpecs && (
              <>
                <p className="text-xs text-gray-400 mb-1.5">规格</p>
                <div className="grid grid-cols-2 gap-2 mb-4">
                  {item.specs.map((spec) => (
                    <button
                      key={spec.spec_code}
                      onClick={() => setSelSpec(spec)}
                      className={`flex justify-between items-center px-3 py-2.5 rounded-xl border text-sm ${
                        selSpec?.spec_code === spec.spec_code ? "border-orange-400 bg-orange-50" : "border-gray-200"
                      }`}
                    >
                      <span>{spec.spec_name}</span>
                      <span className="text-orange-500 font-semibold text-xs">¥{fmt(item.sales_price + spec.price_delta)}</span>
                    </button>
                  ))}
                </div>
              </>
            )}

            <p className="text-xs text-gray-400 mb-1.5">口味 / 加料（可选）</p>
            <div className="flex flex-wrap gap-2 mb-4">
              {MODIFIER_OPTIONS.map((o) => (
                <button
                  key={o.label}
                  onClick={() => toggleMod(o.label)}
                  className={`px-4 py-3 rounded-full text-xs border ${
                    mods.includes(o.label) ? "border-orange-400 bg-orange-50 text-orange-600" : "border-gray-200 text-gray-600"
                  }`}
                >{o.label}{o.price > 0 && <span className="ml-0.5 text-orange-400">+{o.price}</span>}</button>
              ))}
            </div>

            <div className="flex items-center justify-between mb-4">
              <span className="text-sm text-gray-500">数量</span>
              <div className="flex items-center gap-3">
                <button onClick={() => setQty((q) => Math.max(1, q - 1))}
                  className="w-11 h-11 rounded-full border-2 border-orange-400 text-orange-400 font-bold text-lg flex items-center justify-center">−</button>
                <span className="w-6 text-center font-semibold">{qty}</span>
                <button onClick={() => setQty((q) => q + 1)}
                  className="w-11 h-11 rounded-full bg-orange-400 text-white font-bold text-lg flex items-center justify-center">+</button>
              </div>
            </div>

            <button onClick={confirmAdd} className="w-full py-3.5 rounded-2xl bg-orange-500 text-white font-bold text-base">
              加入购物车 · ¥{fmt((item.sales_price + (selSpec?.price_delta ?? 0) + modsExtra) * qty)}
            </button>
            <button onClick={() => setSpecOpen(false)} className="mt-2 w-full py-2 text-gray-400 text-sm">取消</button>
          </div>
        </div>
      )}
    </>
  );
}

function CartSheet({
  cart,
  onQtyChange,
  onNoteChange,
  onClose,
  onSubmit,
  submitting,
}: {
  cart: CartItem[];
  onQtyChange: (key: string, delta: number) => void;
  onNoteChange: (key: string, note: string) => void;
  onClose: () => void;
  onSubmit: () => void;
  submitting: boolean;
}) {
  const [confirmOpen, setConfirmOpen] = useState(false);
  const total = cart.reduce((s, i) => s + i.unit_price * i.qty, 0);

  return (
    <div className="fixed inset-0 z-50 flex items-end" onClick={onClose}>
      <div
        className="w-full bg-white rounded-t-2xl shadow-2xl max-h-[80vh] flex flex-col"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between px-5 pt-5 pb-3 border-b">
          <span className="font-bold text-base">购物车</span>
          <button onClick={onClose} className="text-gray-400 text-2xl leading-none">×</button>
        </div>
        <div className="overflow-y-auto flex-1 px-5 py-3 space-y-4">
          {cart.map((item) => {
            const key = cartKey(item);
            return (
              <div key={key}>
                <div className="flex items-start justify-between gap-3">
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-semibold">{item.name}</p>
                    {item.spec_name && <p className="text-xs text-gray-500">{item.spec_name}</p>}
                    {item.modifiers.length > 0 && (
                      <p className="text-xs text-gray-400">{item.modifiers.map((m) => m.modifier_type).join("、")}</p>
                    )}
                    <p className="text-orange-500 text-sm font-bold mt-0.5">¥{fmt(item.unit_price)}</p>
                  </div>
                  <div className="flex items-center gap-2 flex-shrink-0">
                    <button onClick={() => onQtyChange(key, -1)}
                      className="w-11 h-11 rounded-full border-2 border-orange-400 text-orange-400 font-bold text-lg flex items-center justify-center">−</button>
                    <span className="w-4 text-center text-sm font-semibold">{item.qty}</span>
                    <button onClick={() => onQtyChange(key, 1)}
                      className="w-11 h-11 rounded-full bg-orange-400 text-white font-bold text-lg flex items-center justify-center">+</button>
                  </div>
                </div>
                <input
                  className="mt-1.5 w-full text-xs border border-gray-200 rounded-lg px-3 py-1.5 focus:outline-none focus:border-orange-400"
                  placeholder="备註（去辣、少鹽...）"
                  value={item.note}
                  onChange={(e) => onNoteChange(key, e.target.value)}
                />
              </div>
            );
          })}
        </div>
        <div className="px-5 pb-6 pt-3 border-t">
          <div className="flex justify-between text-sm mb-3">
            <span className="text-gray-500">合計</span>
            <span className="font-bold text-base text-gray-900">¥{fmt(total)}</span>
          </div>
          <button
            onClick={() => setConfirmOpen(true)}
            disabled={submitting}
            className="w-full py-4 rounded-2xl bg-orange-400 text-white font-bold text-base disabled:opacity-50"
          >
            {submitting ? "提交中..." : "确认下单"}
          </button>
        </div>
      </div>
      {confirmOpen && (
        <div className="fixed inset-0 z-[60] flex items-center justify-center bg-black/40" onClick={() => setConfirmOpen(false)}>
          <div className="bg-white rounded-2xl p-6 mx-5 w-full max-w-sm shadow-2xl" onClick={(e) => e.stopPropagation()}>
            <p className="text-base font-bold text-center mb-1">確認下單？</p>
            <p className="text-center text-orange-500 font-bold text-xl mb-5">¥{fmt(total)}</p>
            <div className="flex gap-3">
              <button onClick={() => setConfirmOpen(false)} className="flex-1 py-3 rounded-xl border border-gray-200 text-gray-600 font-medium">
                返回
              </button>
              <button
                onClick={() => { setConfirmOpen(false); onSubmit(); }}
                disabled={submitting}
                className="flex-1 py-3 rounded-xl bg-orange-400 text-white font-bold disabled:opacity-50"
              >
                {submitting ? "提交中..." : "確認下單"}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

// ── Main Page ──────────────────────────────────────────────────────────────

// ── MarketingCard: renders the marketing popup elements ───────────────────

type MarketingElement = { type: string; [key: string]: unknown };

const FORTUNE_TEXTS: Record<string, { level: string; stars: string; texts: string[] }> = {
  "大吉": { level: "大 吉", stars: "★ ★ ★", texts: ["今日万事俱备，美食开路，好运随行。","口福即天福，饱食者心宽，心宽者天下大吉。","今日吉星高照，用餐愉快，好事接连而至。"] },
  "中吉": { level: "中 吉", stars: "★ ★",   texts: ["菜香心静，凡事不急，好事自来。","今日宜慢食慢行，细品生活每一味。","饭吃七分饱，事做三分稳，中吉福报至。"] },
  "小吉": { level: "小 吉", stars: "★",     texts: ["小吉已是福，知足者常乐，今日享受当下。","凡事稍作等候，如等上菜，值得的都值得等。","平稳是福，今日安步当车，无惊无险皆好事。"] },
};
const QUOTES_ZH = ["人间有味是清欢 — 苏轼","此刻此味，是最好的时刻","食之以诚，暖之以心"];
const QUOTES_EN = ["Good food is the foundation of genuine happiness.","Life is short. Eat the good stuff first.","Every meal is a love letter to your body."];
const QUOTES_JA = ["食べることは生きること、愛すること","一碗の温もり、心に満ちる幸せ"];
const ART_BLOCKS = [
  "╔══════════════════╗\n║ ( ˘◡˘ )♪ 用心料理 ║\n╚══════════════════╝",
  "☆ ☆ LUCKY RECEIPT ☆ ☆\n  ／￣＼\n （°▽°）感谢光临！",
  "/ᐠ｡ꞈ｡ᐟ\\  感谢惠顾！\n♪ 布谷！布谷！ ♪",
  "ʕ•ᴥ•ʔ  吃飽了嗎？",
  "✿ ✿ ✿  用心烹飪  ✿ ✿ ✿",
];

function djb2(s: string): number {
  let h = 5381;
  for (let i = 0; i < s.length; i++) h = (Math.imul(h, 31) + s.charCodeAt(i)) | 0;
  return Math.abs(h);
}

function getSeed(strategy: string, orderId: number, tableNo: string): number {
  const date = new Date().toISOString().slice(0, 10);
  if (strategy === "per_order") return djb2(`${orderId}`);
  if (strategy === "per_table") return djb2(`${tableNo}${date}`);
  return djb2(date);
}

function MarketingCard({ popup }: { popup: MarketingPopupData }) {
  let elements: MarketingElement[] = [];
  try { elements = JSON.parse(popup.template_content).elements ?? []; } catch { /* ignore */ }

  return (
    <div className="w-full space-y-3">
      {elements.map((elem, i) => {
        const seed = getSeed((elem.seed_strategy as string) ?? "per_order", popup.order_id, popup.table_no);

        if (elem.type === "fortune") {
          const customTexts = (elem.custom_texts as string[] | undefined)?.filter(Boolean) ?? [];
          if (customTexts.length > 0) {
            const text = customTexts[seed % customTexts.length];
            return (
              <div key={i} className="w-full rounded-2xl bg-white border-2 border-amber-300 p-5 text-center shadow-sm">
                <div className="text-xs text-amber-500 mb-1 tracking-widest">今日运势</div>
                <div className="text-sm text-amber-700 leading-relaxed">{text}</div>
              </div>
            );
          }
          const pct = seed % 100;
          const key = pct < 20 ? "小吉" : pct < 70 ? "中吉" : "大吉";
          const f = FORTUNE_TEXTS[key];
          const text = f.texts[(seed >> 2) % f.texts.length];
          return (
            <div key={i} className="w-full rounded-2xl bg-white border-2 border-amber-300 p-5 text-center shadow-sm">
              <div className="text-xs text-amber-500 mb-1 tracking-widest">今日运势</div>
              <div className="text-3xl font-bold text-amber-800 mb-1">{f.stars} {f.level} {f.stars}</div>
              <div className="text-sm text-amber-700 leading-relaxed">{text}</div>
            </div>
          );
        }

        if (elem.type === "character_collect") {
          const chars = (elem.characters as string[] | undefined) ?? ["恭","喜","發","財"];
          const prize = (elem.prize as string) ?? "集齐兑换免费饮品";
          // Prefer the backend-computed char (single source of truth, matches receipt);
          // fall back to local seed only if absent.
          const pickedChar = elem.picked_char as string | undefined;
          const idx = pickedChar ? Math.max(0, chars.indexOf(pickedChar)) : seed % chars.length;
          const qrToken = elem.qr_token as string | undefined;
          return (
            <div key={i} className="w-full rounded-2xl bg-white border-2 border-orange-200 p-5 text-center shadow-sm">
              <div className="text-xs text-orange-500 mb-2 tracking-widest">{(elem.game_name as string) ?? "集字兑奖"}</div>
              <div className="text-4xl font-bold text-orange-700 mb-3">
                【{chars[idx]}】
              </div>
              <div className="flex justify-center gap-2 text-lg mb-2">
                {chars.map((ch, ci) => (
                  <span key={ci} className={ci === idx ? "text-orange-600 font-bold border-b-2 border-orange-400" : "text-gray-300"}>
                    {ci === idx ? ch : "□"}
                  </span>
                ))}
              </div>
              <div className="text-xs text-orange-600 mb-3">{prize}</div>
              {qrToken && (
                <div className="flex flex-col items-center gap-1 pt-3 border-t border-orange-100">
                  <StyledQR value={`${window.location.origin}/#/redeem/${qrToken}`} size={120} dotColor="#ea580c" />
                  <div className="text-[10px] text-orange-400">店员扫码核销 · 一码一次</div>
                  <div className="text-[10px] text-orange-400 font-mono">序号 {popup.order_no}</div>
                </div>
              )}
            </div>
          );
        }

        if (elem.type === "quote") {
          const customTexts = (elem.custom_texts as string[] | undefined)?.filter(Boolean) ?? [];
          let quote: string;
          if (customTexts.length > 0) {
            quote = customTexts[seed % customTexts.length];
          } else {
            const lang = (elem.language as string) ?? "multilingual";
            const pool = lang === "en" ? QUOTES_EN : lang === "ja" ? QUOTES_JA :
              lang === "zh" ? QUOTES_ZH : [QUOTES_ZH, QUOTES_EN, QUOTES_JA][seed % 3];
            const quotes = Array.isArray(pool) ? pool : QUOTES_ZH;
            quote = quotes[seed % quotes.length];
          }
          return (
            <div key={i} className="w-full rounded-2xl bg-white border border-gray-200 p-4 text-center shadow-sm">
              <div className="text-xs text-gray-400 mb-1">今日语录</div>
              <div className="text-sm text-gray-600 italic leading-relaxed">&ldquo;{quote}&rdquo;</div>
            </div>
          );
        }

        if (elem.type === "marketing_image") {
          const src = (elem.image_data as string | undefined) || (elem.url as string | undefined);
          const alt = (elem.alt as string | undefined) ?? "";
          if (!src) return null;
          return (
            <div key={i} className="w-full rounded-2xl overflow-hidden shadow-sm">
              <img src={src} alt={alt} className="w-full object-contain max-h-64" />
            </div>
          );
        }

        if (elem.type === "art") {
          const block = ART_BLOCKS[seed % ART_BLOCKS.length];
          return (
            <div key={i} className="w-full rounded-2xl bg-white border border-gray-200 p-4 text-center shadow-sm">
              <pre className="text-xs text-gray-600 font-mono whitespace-pre leading-relaxed inline-block text-left">{block}</pre>
            </div>
          );
        }

        if (elem.type === "solar_term") {
          // Show solar term theme if we have one for today — simplistic frontend version
          const now = new Date();
          const m = now.getMonth() + 1, d = now.getDate();
          const terms: [number, number, number, string, string][] = [
            [1, 5, 8, "小寒", "小寒已至，寒气渐深，暖身御寒。"],
            [1, 19, 22, "大寒", "大寒岁末，滋补靓汤，暖身御寒。"],
            [2, 3, 6, "立春", "立春一至，万象更新，时令鲜蔬。"],
            [6, 21, 24, "夏至", "夏至阳极，冰饮甜品，清热解暑。"],
            [7, 22, 25, "大暑", "大暑酷热，绿豆冰饮，消暑开胃。"],
            [12, 21, 24, "冬至", "冬至阳生，饺子汤圆，阖家团圆。"],
          ];
          const term = terms.find(([tm, ds, de]) => tm === m && d >= ds && d <= de);
          if (!term) return null;
          return (
            <div key={i} className="w-full rounded-2xl bg-gradient-to-br from-green-50 to-emerald-50 border border-green-200 p-4 text-center shadow-sm">
              <div className="text-xs text-green-600 mb-1">✦ 节气</div>
              <div className="text-2xl font-bold text-green-800 mb-1">{term[3]}</div>
              <div className="text-sm text-green-700">{term[4]}</div>
            </div>
          );
        }

        if (elem.type === "chef_message") {
          const msgs = (elem.messages as string[] | undefined) ?? [];
          const defaultMsgs = ["今天的食材格外新鲜，用心为您烹饪。","感谢光临，愿每一口都让您满意。","好食材，慢火候，是我们的承诺。"];
          const weekday = new Date().getDay();
          const msg = msgs.length > 0 ? msgs[weekday % msgs.length] : defaultMsgs[weekday % defaultMsgs.length];
          return (
            <div key={i} className="w-full rounded-2xl bg-white border border-gray-200 p-4 shadow-sm">
              <div className="text-xs text-gray-400 mb-1">👨‍🍳 {String(elem.title ?? "厨师寄语")}</div>
              <div className="text-sm text-gray-700 leading-relaxed">{msg}</div>
              <div className="text-xs text-gray-400 mt-1 text-right">— {String(elem.author ?? "本店厨师")}</div>
            </div>
          );
        }

        if (elem.type === "riddle") {
          return (
            <div key={i} className="w-full rounded-2xl bg-amber-50 border border-amber-200 p-4 shadow-sm">
              <div className="text-xs text-amber-600 mb-2">🤔 今日谜语</div>
              <div className="text-sm text-amber-800 font-medium">谜题已印在收据上，回店说出答案即可兑奖！</div>
              <div className="text-xs text-amber-600 mt-1">{String(elem.prize ?? "下次来店说出答案，赢取小惊喜！")}</div>
            </div>
          );
        }

        if (elem.type === "qr_code") {
          const url = (elem.url as string) ?? "";
          if (!url) return null;
          return (
            <div key={i} className="w-full rounded-2xl bg-white border border-gray-200 p-4 flex flex-col items-center gap-2 shadow-sm">
              <div className="text-xs text-gray-400">{(elem.label as string) ?? "扫码关注"}</div>
              <StyledQR value={url} size={120} dotColor="#1f2937" />
            </div>
          );
        }

        if (elem.type === "discount_coupon") {
          const dt = (elem.discount_type as string) ?? "percent";
          const val = (elem.value as number) ?? 0;
          const cond = (elem.condition as string) ?? "";
          const validDays = (elem.valid_days as number) ?? 30;
          const discountText = dt === "percent" ? `${val}% OFF` : dt === "amount" ? `减 ¥${val}` : "免费赠品";
          return (
            <div key={i} className="w-full rounded-2xl border-2 border-dashed border-red-300 bg-red-50 p-5 text-center shadow-sm">
              <div className="text-xs text-red-400 mb-1">{(elem.label as string) ?? "下次消费优惠"}</div>
              <div className="text-3xl font-extrabold text-red-600 mb-1">{discountText}</div>
              {cond && <div className="text-xs text-red-500">{cond}</div>}
              <div className="text-[10px] text-red-400 mt-2">{validDays}天内有效 · 凭本页核销</div>
            </div>
          );
        }

        if (elem.type === "product_spotlight") {
          const name = (elem.name as string) ?? "";
          if (!name) return null;
          const price = elem.price as number | undefined;
          const badge = (elem.badge as string) ?? "";
          return (
            <div key={i} className="w-full rounded-2xl bg-white border border-gray-200 p-4 shadow-sm">
              <div className="flex items-center gap-2 mb-1">
                <span className="text-xs text-gray-400">{(elem.title as string) ?? "本店推荐"}</span>
                {badge && <span className="text-[10px] bg-orange-100 text-orange-600 px-1.5 py-0.5 rounded-full font-bold">{badge}</span>}
              </div>
              <div className="text-base font-bold text-gray-900">{name}</div>
              {(elem.description as string) && <div className="text-xs text-gray-500 mt-0.5">{elem.description as string}</div>}
              {price !== undefined && <div className="text-orange-500 font-bold mt-1">¥{fmt(price)}</div>}
            </div>
          );
        }

        if (elem.type === "rich_text") {
          const content = (elem.content as string) ?? "";
          if (!content) return null;
          return (
            <div key={i} className="w-full rounded-2xl bg-white border border-gray-200 p-4 shadow-sm">
              <pre className="text-sm text-gray-700 whitespace-pre-wrap font-sans leading-relaxed">{content.replace(/^[#>\-*]+\s?/gm, "").trim()}</pre>
            </div>
          );
        }

        if (elem.type === "dish_easter_egg") {
          return (
            <div key={i} className="w-full rounded-2xl bg-gradient-to-br from-purple-50 to-pink-50 border border-purple-200 p-4 text-center shadow-sm">
              <div className="text-xs text-purple-400 mb-1">🥚 隐藏彩蛋</div>
              <div className="text-sm text-purple-700">{(elem.message as string) ?? "今日彩蛋：愿你好运连连！"}</div>
            </div>
          );
        }

        return null;
      })}
    </div>
  );
}

// ── Main page ─────────────────────────────────────────────────────────────

export function SelfOrderPage() {
  const params = useParams<{ tableNo?: string; token?: string }>();
  const token = params.token ?? null;
  const [tableNo, setTableNo] = useState<string | undefined>(params.tableNo);
  const [tokenInvalid, setTokenInvalid] = useState(false);
  const [menu, setMenu] = useState<PublicMenuCategory[]>([]);
  const [search, setSearch] = useState("");
  const [loading, setLoading] = useState(true);
  const [activecat, setActiveCat] = useState<number | null>(null);
  const [cart, setCart] = useState<CartItem[]>([]);
  const [cartOpen, setCartOpen] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [successOrder, setSuccessOrder] = useState<{ id: number; order_no: string } | null>(null);
  const [marketingPopup, setMarketingPopup] = useState<MarketingPopupData | null>(null);
  const [pastOrders, setPastOrders] = useState<TableOrderSummary[]>([]);
  const [paymentOverlay, setPaymentOverlay] = useState<{ qr: string; total: number } | null>(null);
  const sectionRefs = useRef<Record<number, HTMLDivElement | null>>({});
  const scrollRef = useRef<HTMLDivElement>(null);
  const pollRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    call<PublicMenuCategory[]>("get_public_menu").then((data) => {
      setMenu(data);
      if (data.length > 0) setActiveCat(data[0].id);
    }).catch(console.error).finally(() => setLoading(false));
  }, []);

  // Resolve signed table token → real table_no (logs the scan server-side).
  useEffect(() => {
    if (!token) return;
    call<{ valid: boolean; table_no?: string }>("resolve_table_token", { token })
      .then((r) => {
        if (r.valid && r.table_no) setTableNo(r.table_no);
        else setTokenInvalid(true);
      })
      .catch(() => setTokenInvalid(true));
  }, [token]);

  const fetchPastOrders = useCallback(() => {
    if (!tableNo) return;
    call<TableOrderSummary[]>("get_table_orders_today", { table_no: tableNo })
      .then(setPastOrders).catch(console.error);
  }, [tableNo]);

  useEffect(() => {
    fetchPastOrders();
    pollRef.current = setInterval(fetchPastOrders, 4000);

    function onVisibility() {
      if (document.hidden) {
        if (pollRef.current) { clearInterval(pollRef.current); pollRef.current = null; }
      } else {
        fetchPastOrders();
        pollRef.current = setInterval(fetchPastOrders, 4000);
      }
    }
    document.addEventListener("visibilitychange", onVisibility);

    return () => {
      if (pollRef.current) clearInterval(pollRef.current);
      document.removeEventListener("visibilitychange", onVisibility);
    };
  }, [fetchPastOrders]);

  function scrollToCategory(catId: number) {
    setActiveCat(catId);
    sectionRefs.current[catId]?.scrollIntoView({ behavior: "smooth", block: "start" });
  }

  function addToCart(item: PublicMenuItem, spec: PublicMenuItemSpec | null, qty = 1, modifiers: CartModifier[] = []) {
    const modsExtra = modifiers.reduce((s, m) => s + m.price_delta, 0);
    const key = `${item.id}__${spec?.spec_code ?? ""}__${modSig(modifiers)}`;
    setCart((prev) => {
      const existing = prev.find((c) => cartKey(c) === key);
      if (existing) return prev.map((c) => cartKey(c) === key ? { ...c, qty: c.qty + qty } : c);
      return [...prev, {
        menu_item_id: item.id, name: item.name,
        spec_code: spec?.spec_code ?? null, spec_name: spec?.spec_name ?? null,
        unit_price: priceOf(item, spec?.spec_code ?? null) + modsExtra, qty, note: "", modifiers,
      }];
    });
  }

  function removeFromCart(item: PublicMenuItem, spec: PublicMenuItemSpec | null) {
    const key = `${item.id}__${spec?.spec_code ?? ""}`;
    setCart((prev) => {
      const match = prev.find((c) => cartKey(c) === key);
      if (!match) return prev;
      if (match.qty <= 1) return prev.filter((c) => cartKey(c) !== key);
      return prev.map((c) => cartKey(c) === key ? { ...c, qty: c.qty - 1 } : c);
    });
  }

  function changeQty(key: string, delta: number) {
    setCart((prev) => {
      const item = prev.find((c) => cartKey(c) === key);
      if (!item) return prev;
      if (item.qty + delta <= 0) return prev.filter((c) => cartKey(c) !== key);
      return prev.map((c) => cartKey(c) === key ? { ...c, qty: c.qty + delta } : c);
    });
  }

  function changeNote(key: string, note: string) {
    setCart((prev) => prev.map((c) => cartKey(c) === key ? { ...c, note } : c));
  }

  async function fetchAndShowMarketingPopup(orderId: number, orderNo: string) {
    const DEFAULT_POPUP_CONTENT = JSON.stringify({ elements: [
      { type: "fortune", seed_strategy: "per_order" },
      { type: "character_collect", game_name: "集字兑奖", characters: ["恭","喜","发","财"], prize: "集齐四字兑换免费饮品", seed_strategy: "per_order", style: "box" },
      { type: "quote", language: "multilingual" },
    ]});
    const fallbackPopup: MarketingPopupData = {
      order_id: orderId, order_no: orderNo,
      table_no: tableNo ?? "", created_at: new Date().toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit" }),
      amount_total: 0, template_content: DEFAULT_POPUP_CONTENT,
    };
    try {
      const timeout = new Promise<never>((_, reject) =>
        setTimeout(() => reject(new Error("timeout")), 3000)
      );
      const popup = await Promise.race([
        call<MarketingPopupData>("get_marketing_popup", { orderId, tableNo: tableNo }),
        timeout,
      ]);
      setMarketingPopup(popup);
    } catch {
      // Show local fallback immediately, then retry once in the background so
      // the authoritative popup (with real qr_token + picked_char) replaces it.
      setMarketingPopup(fallbackPopup);
      call<MarketingPopupData>("get_marketing_popup", { orderId, tableNo: tableNo })
        .then((p) => setMarketingPopup(p))
        .catch(() => {});
    }
  }

  async function submitOrder() {
    if (!tableNo || cart.length === 0) return;
    setSubmitting(true);
    try {
      const result = await call<{ id: number; order_no: string }>("create_self_order", {
        table_no: tableNo,
        token,
        items: cart.map((c) => ({
          menu_item_id: c.menu_item_id,
          spec_code: c.spec_code ?? null,
          qty: c.qty,
          note: c.note || null,
          modifiers: c.modifiers.map((m) => ({
            modifier_type: m.modifier_type,
            price_delta: m.price_delta,
            qty: c.qty,
          })),
        })),
      });
      const cartTotal = cart.reduce((s, i) => s + i.unit_price * i.qty, 0);
      setSuccessOrder(result);
      setCart([]);
      setCartOpen(false);
      fetchPastOrders();
      // Check for payment QR; if present, show overlay and defer marketing popup
      try {
        const qrResp = await fetch("/api/payment_qr");
        if (qrResp.ok) {
          const qrData = await qrResp.json();
          if (qrData.data) {
            setPaymentOverlay({ qr: qrData.data, total: cartTotal });
            return;
          }
        }
      } catch {
        // silent — proceed to marketing popup normally
      }
      await fetchAndShowMarketingPopup(result.id, result.order_no);
    } catch (e) {
      alert(`下单失败：${e instanceof Error ? e.message : "请重试"}`);
      console.error(e);
    } finally {
      setSubmitting(false);
    }
  }

  const q = search.trim().toLowerCase();
  const filteredMenu = q
    ? menu
        .map((cat) => ({
          ...cat,
          items: cat.items.filter(
            (it) => it.name.toLowerCase().includes(q) || (it.description ?? "").toLowerCase().includes(q)
          ),
        }))
        .filter((cat) => cat.items.length > 0)
    : menu;

  const totalQty = cart.reduce((s, c) => s + c.qty, 0);
  const totalPrice = cart.reduce((s, c) => s + c.unit_price * c.qty, 0);

  if (tokenInvalid) {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center p-6">
        <div className="text-center text-gray-500">
          <p className="text-4xl mb-3">📷</p>
          <p className="font-semibold text-gray-700">二维码已失效</p>
          <p className="text-sm mt-1">请重新扫描桌上的最新二维码</p>
        </div>
      </div>
    );
  }

  if (loading) {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center">
        <div className="text-gray-400 text-sm">加载中...</div>
      </div>
    );
  }

  if (menu.length === 0) {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center">
        <div className="text-gray-400 text-sm text-center">
          <p className="text-4xl mb-3">🍽</p>
          <p>菜单尚未设定，请稍后再试</p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50 pb-24 font-sans">
      {/* Header */}
      <div className="sticky top-0 z-30 bg-white shadow-sm">
        <div className="flex items-center justify-between px-4 py-3">
          <span className="font-bold text-gray-900 text-base">自助点餐</span>
          {tableNo && (
            <span className="text-xs font-semibold bg-orange-100 text-orange-600 px-3 py-1 rounded-full">
              桌 {tableNo}
            </span>
          )}
        </div>
        {/* Search */}
        <div className="px-4 pb-2">
          <input
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="搜索菜品…"
            className="w-full text-sm bg-gray-100 rounded-full px-4 py-2 focus:outline-none focus:ring-2 focus:ring-orange-300"
          />
        </div>
        {/* Category tabs — hidden while searching */}
        {!q && (
          <div className="flex gap-2 px-4 pb-3 overflow-x-auto scrollbar-hide">
            {menu.map((cat) => (
              <button
                key={cat.id}
                onClick={() => scrollToCategory(cat.id)}
                className={`flex-shrink-0 text-xs px-4 py-1.5 rounded-full font-medium transition-colors ${
                  activecat === cat.id
                    ? "bg-orange-400 text-white"
                    : "bg-gray-100 text-gray-600"
                }`}
              >
                {cat.name}
              </button>
            ))}
          </div>
        )}
      </div>

      {/* Menu sections */}
      <div ref={scrollRef} className="px-4 pt-3 space-y-4">
        {!q && (
          <div className="rounded-2xl bg-gradient-to-r from-amber-100 to-orange-100 border border-amber-200 px-4 py-3 flex items-center gap-3">
            <span className="text-2xl">🎁</span>
            <div className="min-w-0">
              <p className="text-sm font-bold text-amber-800">下单即抽今日运势</p>
              <p className="text-xs text-amber-600">集字兑好礼 · 截图保存凭码核销</p>
            </div>
          </div>
        )}
        {q && filteredMenu.length === 0 && (
          <p className="text-center text-sm text-gray-400 py-8">没有找到「{search}」相关菜品</p>
        )}
        {filteredMenu.map((cat, catIdx) => (
          <div
            key={cat.id}
            ref={(el) => { sectionRefs.current[cat.id] = el; }}
          >
            <p className="text-xs font-bold text-gray-400 uppercase tracking-widest mb-2 pt-1">
              {cat.name}
            </p>
            <div className="bg-white rounded-2xl px-4 shadow-sm">
              {cat.items.map((item) => {
                const qty = cart
                  .filter((c) => c.menu_item_id === item.id)
                  .reduce((s, c) => s + c.qty, 0);
                return (
                  <ItemCard
                    key={item.id}
                    item={item}
                    catIndex={catIdx}
                    cartQty={qty}
                    onAdd={addToCart}
                    onRemove={removeFromCart}
                  />
                );
              })}
            </div>
          </div>
        ))}

        {/* Past orders */}
        {pastOrders.length > 0 && (
          <div>
            <p className="text-xs font-bold text-gray-400 uppercase tracking-widest mb-2 pt-1">
              本桌今日订单
            </p>
            <div className="bg-white rounded-2xl px-4 shadow-sm divide-y divide-gray-100">
              {pastOrders.map((order) => {
                const badge = statusLabel(order.status);
                return (
                  <div key={order.id} className="py-3">
                    <div className="flex items-center justify-between mb-1.5">
                      <span className="text-xs text-gray-500">{order.created_at.slice(11, 16)}</span>
                      <span className={`text-xs font-semibold px-2 py-0.5 rounded-full ${badge.cls}`}>
                        {badge.text}
                      </span>
                    </div>
                    <div className="space-y-0.5">
                      {order.items.map((item, i) => (
                        <div key={i} className="flex justify-between text-xs text-gray-700">
                          <span>{item.name}{item.spec_code ? ` · ${item.spec_code}` : ""} ×{item.qty}</span>
                          <span>¥{fmt(item.unit_price * item.qty)}</span>
                        </div>
                      ))}
                    </div>
                    <div className="flex justify-between text-xs font-bold text-gray-900 mt-1.5 pt-1.5 border-t border-gray-100">
                      <span>小計</span>
                      <span>¥{fmt(order.amount_total)}</span>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        )}
      </div>

      {/* Floating cart bar */}
      {totalQty > 0 && !cartOpen && (
        <div className="fixed bottom-6 left-4 right-4 z-40">
          <button
            onClick={() => setCartOpen(true)}
            className="w-full bg-orange-400 text-white rounded-2xl px-5 py-4 flex items-center justify-between shadow-xl"
          >
            <span className="bg-orange-600 text-white text-xs font-bold w-6 h-6 rounded-full flex items-center justify-center">
              {totalQty}
            </span>
            <span className="font-bold text-sm">查看购物车</span>
            <span className="font-bold text-sm">¥{fmt(totalPrice)}</span>
          </button>
        </div>
      )}

      {/* Order success + marketing popup */}
      {successOrder && (
        <div className="fixed inset-0 z-50 bg-gradient-to-b from-amber-50 to-orange-50 flex flex-col overflow-y-auto">
          <div className="flex-1 flex flex-col items-center justify-start pt-8 px-6 pb-6 gap-4 max-w-sm mx-auto w-full">
            {/* Header */}
            <div className="text-center">
              <div className="text-5xl mb-2">🎉</div>
              <p className="text-2xl font-bold text-amber-800">订单已提交！</p>
              <p className="text-sm text-amber-600 mt-1">厨房正在准备中，请稍候</p>
            </div>

            {/* Marketing content card */}
            {marketingPopup ? (
              <MarketingCard popup={marketingPopup} />
            ) : (
              <div className="w-full rounded-2xl bg-white/80 border border-amber-200 p-5 text-center text-sm text-amber-700">
                <div className="animate-pulse mb-1">✨</div>
                <div className="text-xs">加载今日惊喜...</div>
              </div>
            )}

            {/* Order info strip — screenshot-friendly */}
            <div className="w-full rounded-xl bg-white/60 border border-amber-200 px-4 py-3 text-xs text-amber-800 flex justify-between items-center">
              <span className="font-mono font-bold">{successOrder.order_no}</span>
              <span>桌 {tableNo}</span>
              <span>今日有效</span>
            </div>

            {/* Actions */}
            <div className="flex gap-3 w-full pt-2">
              {typeof navigator !== "undefined" && "share" in navigator && (
                <button
                  className="flex-1 py-3 rounded-2xl border border-amber-300 bg-white/70 text-amber-700 font-semibold text-sm"
                  onClick={() => navigator.share?.({
                    title: `我的今日运势 ${successOrder.order_no}`,
                    text: "來看看我在美食店抽到的今日运势！",
                  }).catch(() => {})}
                >
                  📤 分享
                </button>
              )}
              <button
                className="flex-1 py-3 rounded-2xl bg-amber-400 text-white font-semibold text-sm"
                onClick={() => { setSuccessOrder(null); setMarketingPopup(null); }}
              >
                继续点餐
              </button>
            </div>
            <p className="text-xs text-amber-500/70 text-center">截图保存后即可凭图兑奖 · 一单一次</p>
          </div>
        </div>
      )}

      {/* Cart sheet */}
      {cartOpen && (
        <CartSheet
          cart={cart}
          onQtyChange={changeQty}
          onNoteChange={changeNote}
          onClose={() => setCartOpen(false)}
          onSubmit={submitOrder}
          submitting={submitting}
        />
      )}

      {/* Payment QR overlay */}
      {paymentOverlay && (
        <div className="fixed inset-0 z-[70] flex flex-col items-center justify-center bg-white">
          <div className="flex flex-col items-center gap-5 px-6 w-full max-w-sm">
            <div className="text-center">
              <p className="text-lg font-bold text-gray-800">請掃碼付款</p>
              <p className="text-3xl font-bold text-orange-500 mt-1">¥{fmt(paymentOverlay.total)}</p>
            </div>
            <img
              src={`data:image/png;base64,${paymentOverlay.qr}`}
              alt="收款碼"
              className="w-56 h-56 rounded-2xl border-2 border-orange-200 object-contain shadow-lg"
            />
            <p className="text-sm text-gray-400 text-center">微信 / 支付寶 掃描左方二維碼</p>
            <button
              onClick={() => {
                const overlay = paymentOverlay;
                setPaymentOverlay(null);
                if (successOrder) {
                  fetchAndShowMarketingPopup(successOrder.id, successOrder.order_no);
                }
                void overlay;
              }}
              className="w-full py-4 rounded-2xl bg-orange-400 text-white font-bold text-base"
            >
              我已付款 ✓
            </button>
            <button
              onClick={() => setPaymentOverlay(null)}
              className="text-gray-400 text-sm py-2"
            >
              稍後付款
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
