import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { ChefHat, QrCode, Printer, Sparkles, ArrowRight, X } from "lucide-react";

const SETUP_DONE_KEY = "cuckoo_setup_done";

export function useSetupWizard(menuItemCount: number) {
  const isDone = localStorage.getItem(SETUP_DONE_KEY) === "true";
  const [dismissed, setDismissed] = useState(isDone);

  function dismiss() {
    localStorage.setItem(SETUP_DONE_KEY, "true");
    setDismissed(true);
  }

  const show = !dismissed && menuItemCount === 0;
  return { show, dismiss };
}

interface SetupWizardProps {
  onDismiss: () => void;
}

const STEPS = [
  {
    icon: ChefHat,
    title: "添加菜品",
    description: "先创建菜单分类和商品，顾客才能在自助点单页浏览下单。",
    action: "前往菜单管理",
    route: "/menu",
  },
  {
    icon: QrCode,
    title: "设置桌号二维码",
    description: "为每张桌子生成专属二维码，顾客扫码后直接进入点单页面。",
    action: "前往桌号设置",
    route: "/tables",
  },
  {
    icon: Printer,
    title: "配置打印机（可选）",
    description: "配置厨房打印机，下单后自动打印工单，支持飞鹅云和局域网直连。",
    action: "前往打印设置",
    route: "/print-settings",
  },
];

export function SetupWizard({ onDismiss }: SetupWizardProps) {
  const navigate = useNavigate();

  return (
    <Card className="border-primary/30 bg-gradient-to-br from-primary/5 to-background">
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <CardTitle className="flex items-center gap-2 text-base">
            <Sparkles className="h-4 w-4 text-primary" />
            欢迎使用 Cuckoo — 完成初始设置
          </CardTitle>
          <Button variant="ghost" size="icon" className="h-7 w-7 text-muted-foreground" onClick={onDismiss}>
            <X className="h-4 w-4" />
          </Button>
        </div>
        <p className="text-xs text-muted-foreground mt-0.5">完成以下 3 步即可开始营业，随时可跳过</p>
      </CardHeader>
      <CardContent className="space-y-2">
        {STEPS.map(({ icon: Icon, title, description, action, route }) => (
          <div
            key={route}
            className="flex items-center gap-3 rounded-lg border bg-background p-3 hover:bg-muted/40 transition-colors"
          >
            <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-primary/10">
              <Icon className="h-4 w-4 text-primary" />
            </div>
            <div className="flex-1 min-w-0">
              <p className="text-sm font-medium">{title}</p>
              <p className="text-xs text-muted-foreground leading-relaxed">{description}</p>
            </div>
            <Button
              size="sm"
              variant="outline"
              className="shrink-0 gap-1"
              onClick={() => { onDismiss(); navigate(route); }}
            >
              {action}
              <ArrowRight className="h-3.5 w-3.5" />
            </Button>
          </div>
        ))}
        <div className="flex justify-end pt-1">
          <Button size="sm" variant="ghost" className="text-muted-foreground text-xs" onClick={onDismiss}>
            稍后再说，不再显示
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
