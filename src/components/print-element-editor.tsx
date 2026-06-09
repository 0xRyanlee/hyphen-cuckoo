import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Checkbox } from "@/components/ui/checkbox";
import { open } from "@tauri-apps/plugin-dialog";
import { call as invoke } from "@/lib/transport";
import type { PrintElement } from "@/pages/print-templates-page";

export function ElementEditorFields({
  elem,
  onChange,
}: {
  elem: PrintElement;
  onChange: (e: PrintElement) => void;
}) {
  return (
    <div className="space-y-3 pt-2">
      {elem.type === "text" && (<>
        <div><Label className="text-xs">文字内容</Label><Textarea value={String(elem.content ?? "")} onChange={e => onChange({ ...elem, content: e.target.value })} rows={3} className="font-mono text-xs mt-1" /></div>
        <div className="grid grid-cols-2 gap-2">
          <div><Label className="text-xs">对齐</Label>
            <Select value={String(elem.align ?? "left")} onValueChange={v => onChange({ ...elem, align: v })}>
              <SelectTrigger className="h-8 text-xs mt-1"><SelectValue /></SelectTrigger>
              <SelectContent><SelectItem value="left">左对齐</SelectItem><SelectItem value="center">居中</SelectItem><SelectItem value="right">右对齐</SelectItem></SelectContent>
            </Select>
          </div>
          <div><Label className="text-xs">大小</Label>
            <Select value={String(elem.size ?? "normal")} onValueChange={v => onChange({ ...elem, size: v })}>
              <SelectTrigger className="h-8 text-xs mt-1"><SelectValue /></SelectTrigger>
              <SelectContent><SelectItem value="small">小</SelectItem><SelectItem value="normal">正常</SelectItem><SelectItem value="large">大</SelectItem></SelectContent>
            </Select>
          </div>
        </div>
        <div className="flex items-center gap-2"><Checkbox id="ef-bold" checked={!!elem.bold} onCheckedChange={v => onChange({ ...elem, bold: !!v })} /><Label htmlFor="ef-bold" className="text-xs">粗体</Label></div>
      </>)}
      {elem.type === "blank_lines" && (
        <div><Label className="text-xs">空行数量</Label><Input type="number" min={1} max={10} value={Number(elem.count ?? 1)} onChange={e => onChange({ ...elem, count: parseInt(e.target.value) || 1 })} className="h-8 text-xs mt-1 w-24" /></div>
      )}
      {elem.type === "fortune" && (<>
        <div><Label className="text-xs">种子策略</Label>
          <Select value={String(elem.seed_strategy ?? "daily")} onValueChange={v => onChange({ ...elem, seed_strategy: v })}>
            <SelectTrigger className="h-8 text-xs mt-1"><SelectValue /></SelectTrigger>
            <SelectContent><SelectItem value="daily">全店同日</SelectItem><SelectItem value="per_table">每桌不同</SelectItem><SelectItem value="per_order">每单唯一</SelectItem></SelectContent>
          </Select>
        </div>
        <div>
          <Label className="text-xs">自定义运势库（每行一条，留空用系统默认 15 条）</Label>
          <Textarea
            value={((elem.custom_texts as string[] | undefined) ?? []).join("\n")}
            onChange={e => onChange({ ...elem, custom_texts: e.target.value.split("\n").map(s => s.trim()).filter(Boolean) })}
            rows={4}
            className="text-xs mt-1 font-mono"
            placeholder={"今日吉星高照，美食带来好运。\n凡事不急，好事自来。\n小吉亦是福，平稳是福。"}
          />
          <p className="text-[10px] text-muted-foreground mt-1">有内容时优先使用，忽略大/中/小吉分级，直接随机</p>
        </div>
      </>)}
      {elem.type === "quote" && (<>
        <div><Label className="text-xs">语言</Label>
          <Select value={String(elem.language ?? "multilingual")} onValueChange={v => onChange({ ...elem, language: v })}>
            <SelectTrigger className="h-8 text-xs mt-1"><SelectValue /></SelectTrigger>
            <SelectContent><SelectItem value="multilingual">多语轮替</SelectItem><SelectItem value="zh">中文</SelectItem><SelectItem value="en">英文</SelectItem><SelectItem value="ja">日文</SelectItem></SelectContent>
          </Select>
        </div>
        <div>
          <Label className="text-xs">自定义语录库（每行一条，留空用系统默认）</Label>
          <Textarea
            value={((elem.custom_texts as string[] | undefined) ?? []).join("\n")}
            onChange={e => onChange({ ...elem, custom_texts: e.target.value.split("\n").map(s => s.trim()).filter(Boolean) })}
            rows={4}
            className="text-xs mt-1 font-mono"
            placeholder={"人间有味是清欢 — 苏轼\nEvery meal is a love letter.\n食べることは生きること"}
          />
          <p className="text-[10px] text-muted-foreground mt-1">有内容时忽略语言设置，直接从此池随机取</p>
        </div>
      </>)}
      {elem.type === "discount_coupon" && (<>
        <div className="grid grid-cols-2 gap-2">
          <div><Label className="text-xs">折扣类型</Label>
            <Select value={String(elem.discount_type ?? "percent")} onValueChange={v => onChange({ ...elem, discount_type: v })}>
              <SelectTrigger className="h-8 text-xs mt-1"><SelectValue /></SelectTrigger>
              <SelectContent><SelectItem value="percent">百分比折扣</SelectItem><SelectItem value="amount">固定金额</SelectItem><SelectItem value="free_item">指定免费</SelectItem></SelectContent>
            </Select>
          </div>
          <div><Label className="text-xs">折扣值 (%或元)</Label><Input type="number" value={Number(elem.value ?? 0)} onChange={e => onChange({ ...elem, value: parseFloat(e.target.value) || 0 })} className="h-8 text-xs mt-1" /></div>
        </div>
        <div><Label className="text-xs">使用条件</Label><Input value={String(elem.condition ?? "")} onChange={e => onChange({ ...elem, condition: e.target.value })} className="h-8 text-xs mt-1" placeholder="消费满100元" /></div>
        <div className="grid grid-cols-2 gap-2">
          <div><Label className="text-xs">有效天数</Label><Input type="number" value={Number(elem.valid_days ?? 30)} onChange={e => onChange({ ...elem, valid_days: parseInt(e.target.value) || 30 })} className="h-8 text-xs mt-1" /></div>
          <div><Label className="text-xs">标题文字</Label><Input value={String(elem.label ?? "")} onChange={e => onChange({ ...elem, label: e.target.value })} className="h-8 text-xs mt-1" /></div>
        </div>
      </>)}
      {elem.type === "product_spotlight" && (<>
        <div className="grid grid-cols-2 gap-2">
          <div><Label className="text-xs">标题</Label><Input value={String(elem.title ?? "")} onChange={e => onChange({ ...elem, title: e.target.value })} className="h-8 text-xs mt-1" /></div>
          <div><Label className="text-xs">徽章</Label><Input value={String(elem.badge ?? "NEW")} onChange={e => onChange({ ...elem, badge: e.target.value })} className="h-8 text-xs mt-1" /></div>
        </div>
        <div><Label className="text-xs">商品名称</Label><Input value={String(elem.name ?? "")} onChange={e => onChange({ ...elem, name: e.target.value })} className="h-8 text-xs mt-1" /></div>
        <div><Label className="text-xs">描述</Label><Textarea value={String(elem.description ?? "")} onChange={e => onChange({ ...elem, description: e.target.value })} rows={2} className="text-xs mt-1" /></div>
        <div><Label className="text-xs">定价 (0=不显示)</Label><Input type="number" value={Number(elem.price ?? 0)} onChange={e => onChange({ ...elem, price: parseFloat(e.target.value) || 0 })} className="h-8 text-xs mt-1 w-32" /></div>
      </>)}
      {elem.type === "qr_code" && (<>
        <div><Label className="text-xs">URL</Label><Input value={String(elem.url ?? "")} onChange={e => onChange({ ...elem, url: e.target.value })} className="h-8 text-xs mt-1" placeholder="https://..." /></div>
        <div><Label className="text-xs">说明文字</Label><Input value={String(elem.label ?? "")} onChange={e => onChange({ ...elem, label: e.target.value })} className="h-8 text-xs mt-1" /></div>
        <div><Label className="text-xs">尺寸 (1-8)</Label><Input type="number" min={1} max={8} value={Number(elem.size ?? 5)} onChange={e => onChange({ ...elem, size: parseInt(e.target.value) || 5 })} className="h-8 text-xs mt-1 w-24" /></div>
      </>)}
      {elem.type === "character_collect" && (<>
        <div className="grid grid-cols-2 gap-2">
          <div><Label className="text-xs">游戏名称</Label><Input value={String(elem.game_name ?? "")} onChange={e => onChange({ ...elem, game_name: e.target.value })} className="h-8 text-xs mt-1" /></div>
          <div><Label className="text-xs">样式</Label>
            <Select value={String(elem.style ?? "box")} onValueChange={v => onChange({ ...elem, style: v })}>
              <SelectTrigger className="h-8 text-xs mt-1"><SelectValue /></SelectTrigger>
              <SelectContent><SelectItem value="box">方框</SelectItem><SelectItem value="mahjong">麻将</SelectItem></SelectContent>
            </Select>
          </div>
        </div>
        <div>
          <Label className="text-xs">集字组合（逗号分隔）</Label>
          <Input value={(elem.characters as string[] | undefined)?.join(",") ?? ""} onChange={e => onChange({ ...elem, characters: e.target.value.split(",").map(s => s.trim()).filter(Boolean) })} className="h-8 text-xs mt-1" placeholder="恭,喜,发,财" />
          <div className="flex flex-wrap gap-1 mt-1.5">
            {[
              { label: "🀄 麻将", val: "🀇,🀈,🀉,🀊" },
              { label: "🍤 海鲜", val: "🍤,🦐,🦞,🦀" },
              { label: "🌸 四季", val: "🌸,☀️,🍂,❄️" },
              { label: "福禄寿喜", val: "福,禄,寿,喜" },
            ].map(p => (
              <button key={p.label} type="button" className="text-[10px] px-1.5 py-0.5 rounded border border-muted-foreground/30 text-muted-foreground hover:bg-muted"
                onClick={() => onChange({ ...elem, characters: p.val.split(",") })}>
                {p.label}
              </button>
            ))}
          </div>
          <p className="text-[10px] text-muted-foreground mt-1">每张小票抽一个，顾客集齐所有才可兑奖</p>
        </div>
        <div><Label className="text-xs">兑奖说明</Label><Input value={String(elem.prize ?? "")} onChange={e => onChange({ ...elem, prize: e.target.value })} className="h-8 text-xs mt-1" /></div>
        <div><Label className="text-xs">种子策略</Label>
          <Select value={String(elem.seed_strategy ?? "per_order")} onValueChange={v => onChange({ ...elem, seed_strategy: v })}>
            <SelectTrigger className="h-8 text-xs mt-1"><SelectValue /></SelectTrigger>
            <SelectContent><SelectItem value="per_order">每单唯一</SelectItem><SelectItem value="per_table">每桌不同</SelectItem><SelectItem value="daily">全店同日</SelectItem></SelectContent>
          </Select>
        </div>
      </>)}
      {elem.type === "rich_text" && (
        <div><Label className="text-xs">Markdown 内容</Label><Textarea value={String(elem.content ?? "")} onChange={e => onChange({ ...elem, content: e.target.value })} rows={6} className="font-mono text-xs mt-1" placeholder={"## 标题\n- 项目一\n- 项目二\n> 引用文字"} /></div>
      )}
      {elem.type === "solar_term" && (<>
        <p className="text-xs text-muted-foreground">节气期间（前后约7天）自动显示对应主题文案。</p>
        <div className="flex items-center gap-2"><input type="checkbox" checked={!!elem.show_all} onChange={e => onChange({ ...elem, show_all: e.target.checked })} id="ef-solar-show-all" /><Label htmlFor="ef-solar-show-all" className="text-xs">不在节气期间时也显示"下一个节气"提示</Label></div>
      </>)}
      {elem.type === "chef_message" && (<>
        <div><Label className="text-xs">标题</Label><Input value={String(elem.title ?? "厨师寄语")} onChange={e => onChange({ ...elem, title: e.target.value })} className="h-8 text-xs mt-1" /></div>
        <div><Label className="text-xs">署名</Label><Input value={String(elem.author ?? "本店厨师")} onChange={e => onChange({ ...elem, author: e.target.value })} className="h-8 text-xs mt-1" placeholder="例：张师傅" /></div>
        <div><Label className="text-xs">每日消息（每行一条，最多7条）</Label>
          <Textarea value={(elem.messages as string[] | undefined)?.join("\n") ?? ""} onChange={e => onChange({ ...elem, messages: e.target.value.split("\n").map(s => s.trim()).filter(Boolean) })} rows={5} className="text-xs mt-1" placeholder={"周一：今天的食材格外新鲜...\n周二：感谢光临，用心烹饪..."} /></div>
      </>)}
      {elem.type === "riddle" && (<>
        <p className="text-xs text-muted-foreground">留空使用内置谜语库（每日随机），或自定义谜题。</p>
        <div><Label className="text-xs">自定义谜题（选填）</Label><Textarea value={String(elem.question ?? "")} onChange={e => onChange({ ...elem, question: e.target.value || undefined })} rows={2} className="text-xs mt-1" placeholder="输入谜题..." /></div>
        <div><Label className="text-xs">答案（收据上不显示）</Label><Input value={String(elem.answer ?? "")} onChange={e => onChange({ ...elem, answer: e.target.value || undefined })} className="h-8 text-xs mt-1" placeholder="谜底..." /></div>
        <div><Label className="text-xs">兑奖说明</Label><Input value={String(elem.prize ?? "")} onChange={e => onChange({ ...elem, prize: e.target.value })} className="h-8 text-xs mt-1" /></div>
      </>)}
      {elem.type === "dish_easter_egg" && (<>
        <p className="text-xs text-muted-foreground">当订单中包含指定关键词菜品时，显示隐藏彩蛋消息。</p>
        {((elem.eggs as { keyword: string; message: string }[] | undefined) ?? []).map((egg, idx) => (
          <div key={idx} className="flex gap-1 items-start">
            <div className="flex-1 space-y-1">
              <Input value={egg.keyword} placeholder="菜品关键词（如：虾）" className="h-7 text-xs"
                onChange={e => { const eggs = [...((elem.eggs as typeof egg[]) ?? [])]; eggs[idx] = { ...egg, keyword: e.target.value }; onChange({ ...elem, eggs }); }} />
              <Input value={egg.message} placeholder="彩蛋消息（如：解锁：海鲜达人！）" className="h-7 text-xs"
                onChange={e => { const eggs = [...((elem.eggs as typeof egg[]) ?? [])]; eggs[idx] = { ...egg, message: e.target.value }; onChange({ ...elem, eggs }); }} />
            </div>
            <button type="button" className="text-xs text-destructive px-1 pt-1"
              onClick={() => { const eggs = ((elem.eggs as typeof egg[]) ?? []).filter((_, i) => i !== idx); onChange({ ...elem, eggs }); }}>✕</button>
          </div>
        ))}
        <Button type="button" variant="outline" size="sm" className="w-full h-7 text-xs"
          onClick={() => { const eggs = [...((elem.eggs as { keyword: string; message: string }[]) ?? []), { keyword: "", message: "" }]; onChange({ ...elem, eggs }); }}>
          + 添加触发规则
        </Button>
      </>)}
      {elem.type === "marketing_image" && (<>
        <div>
          <Label className="text-xs">图片 URL（直接引用）</Label>
          <div className="flex gap-2 mt-1">
            <Input
              value={String(elem.url ?? "")}
              onChange={e => onChange({ ...elem, url: e.target.value })}
              className="h-8 text-xs flex-1"
              placeholder="https://example.com/image.jpg"
            />
            <Button
              type="button"
              size="sm"
              variant="outline"
              className="h-8 text-xs shrink-0"
              disabled={!elem.url}
              onClick={async () => {
                try {
                  const dataUrl = await invoke<string>("load_image_as_data_url", { source: String(elem.url) });
                  onChange({ ...elem, image_data: dataUrl });
                } catch (e) {
                  console.error("下载失败", e);
                }
              }}
            >
              下载到本地
            </Button>
          </div>
        </div>
        <div>
          <Label className="text-xs">或从本地文件选择</Label>
          <Button
            type="button"
            variant="outline"
            size="sm"
            className="h-8 text-xs mt-1 w-full"
            onClick={async () => {
              const selected = await open({
                multiple: false,
                filters: [{ name: "图片", extensions: ["png", "jpg", "jpeg", "gif", "webp"] }],
              });
              if (selected && typeof selected === "string") {
                try {
                  const dataUrl = await invoke<string>("load_image_as_data_url", { source: selected });
                  onChange({ ...elem, image_data: dataUrl });
                } catch (e) {
                  console.error("读取失败", e);
                }
              }
            }}
          >
            选择本地图片
          </Button>
          {!!elem.image_data && (
            <div className="mt-2 rounded border overflow-hidden">
              <img src={String(elem.image_data)} alt="预览" className="w-full object-cover max-h-32" />
            </div>
          )}
        </div>
        <div>
          <Label className="text-xs">说明文字（选填）</Label>
          <Input value={String(elem.alt ?? "")} onChange={e => onChange({ ...elem, alt: e.target.value })} className="h-8 text-xs mt-1" placeholder="店家宣传图" />
        </div>
      </>)}
      {["separator", "items", "art", "image_block"].includes(elem.type) && (
        <p className="text-xs text-muted-foreground py-1">此元件无需额外配置。</p>
      )}
    </div>
  );
}
