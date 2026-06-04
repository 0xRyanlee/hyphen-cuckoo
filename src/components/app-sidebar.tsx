import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarRail,
  useSidebar,
} from "@/components/ui/sidebar";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { MoreHorizontal, Package, ChefHat, Warehouse, FileText, ShoppingCart, Settings, Home, LogOut, CreditCard, Truck, ClipboardList, Factory, FileBox, BarChart3, Printer, Monitor, Layers, SlidersHorizontal, Receipt, Users, ShieldCheck, LayoutGrid } from "lucide-react";
import { type Role, ROLE_LABELS, checkAccess } from "@/lib/roles";

const navGroups = [
  {
    label: "前台操作",
    items: [
      { id: "pos", label: "POS 点餐", icon: CreditCard },
      { id: "orders", label: "订单", icon: ShoppingCart },
      { id: "kds", label: "KDS 厨房", icon: Monitor },
      { id: "tables", label: "餐桌 / QR", icon: LayoutGrid },
      { id: "customers", label: "顾客管理", icon: Users },
    ],
  },
  {
    label: "后台管理",
    items: [
      { id: "dashboard", label: "仪表板", icon: Home },
      { id: "menu", label: "菜单", icon: FileText },
      { id: "materials", label: "材料管理", icon: Package },
      { id: "recipes", label: "配方", icon: ChefHat },
      { id: "inventory", label: "库存", icon: Warehouse },
      { id: "stocktakes", label: "库存盘点", icon: ClipboardList },
      { id: "reports", label: "数据报表", icon: BarChart3 },
    ],
  },
  {
    label: "进货 / 生产",
    items: [
      { id: "suppliers", label: "供应商", icon: Truck },
      { id: "purchase-orders", label: "采购单", icon: FileBox },
      { id: "production-orders", label: "生产单", icon: Factory },
      { id: "material-states", label: "材料状态", icon: Layers },
    ],
  },
  {
    label: "营销",
    items: [
      { id: "marketing", label: "营销中心", icon: Monitor },
    ],
  },
  {
    label: "设置",
    items: [
      { id: "attributes", label: "属性模板", icon: SlidersHorizontal },
      { id: "expenses", label: "日常支出", icon: Receipt },
      { id: "print", label: "打印中心", icon: Printer },
      { id: "settings", label: "系统设置", icon: Settings },
    ],
  },
];

interface AppSidebarProps {
  activeTab: string;
  onTabChange: (tab: string) => void;
  connected: boolean;
  errorCount?: number;
  notificationCount?: number;
  currentRole?: Role;
  onOpenRoleSwitch?: () => void;
}

export function AppSidebar({ activeTab, onTabChange, errorCount = 0, notificationCount = 0, currentRole = "owner", onOpenRoleSwitch }: AppSidebarProps) {
  const { state } = useSidebar();
  const isCollapsed = state === "collapsed";

  return (
    <Sidebar collapsible="icon" className="border-r border-border">
      <SidebarHeader className="h-14 border-b border-border px-4">
        <div className="flex items-center gap-3">
          <div className="flex h-7 w-7 shrink-0 items-center justify-center rounded-lg bg-primary text-primary-foreground">
            <span className="text-xs font-bold">C</span>
          </div>
          {!isCollapsed && (
            <div className="flex flex-col gap-0.5 leading-none">
              <span className="font-semibold text-sm">Cuckoo</span>
              <span className="text-xs text-muted-foreground">餐饮作业系统</span>
            </div>
          )}
        </div>
      </SidebarHeader>

      <SidebarContent>
        {navGroups.map((group) => {
          const visibleItems = group.items.filter(item => checkAccess(currentRole, item.id));
          if (visibleItems.length === 0) return null;
          return (
            <SidebarGroup key={group.label}>
              {!isCollapsed && (
                <SidebarGroupLabel className="text-xs text-muted-foreground">
                  {group.label}
                </SidebarGroupLabel>
              )}
              <SidebarGroupContent>
                <SidebarMenu>
                  {visibleItems.map((item) => {
                    const showErrorBadge = item.id === "settings" && errorCount > 0;
                    const showNotificationBadge = item.id === "dashboard" && notificationCount > 0;
                    return (
                      <SidebarMenuItem key={item.id}>
                        <SidebarMenuButton
                          onClick={() => onTabChange(item.id)}
                          isActive={activeTab === item.id}
                          tooltip={item.label}
                        >
                          {item.icon && <item.icon className="h-4 w-4" />}
                          <span className="flex-1">{item.label}</span>
                          {showErrorBadge && (
                            <span className="ml-auto flex h-4 min-w-4 items-center justify-center rounded-full bg-destructive px-1 text-[10px] font-medium text-destructive-foreground">
                              {errorCount > 99 ? "99+" : errorCount}
                            </span>
                          )}
                          {showNotificationBadge && (
                            <span className="ml-auto flex h-4 min-w-4 items-center justify-center rounded-full bg-primary px-1 text-[10px] font-medium text-primary-foreground">
                              {notificationCount > 99 ? "99+" : notificationCount}
                            </span>
                          )}
                        </SidebarMenuButton>
                      </SidebarMenuItem>
                    );
                  })}
                </SidebarMenu>
              </SidebarGroupContent>
            </SidebarGroup>
          );
        })}
      </SidebarContent>

      <SidebarFooter className="border-t border-border p-4">
        <DropdownMenu>
          <DropdownMenuTrigger>
            <div className="flex items-center gap-3 cursor-pointer hover:bg-accent rounded-lg p-2 -mx-2 transition-colors">
              <Avatar className="h-8 w-8">
                <AvatarFallback className="bg-primary text-primary-foreground text-xs font-medium">
                  {ROLE_LABELS[currentRole].charAt(0)}
                </AvatarFallback>
              </Avatar>
              {!isCollapsed && (
                <div className="flex flex-1 flex-col gap-0.5 leading-none min-w-0">
                  <span className="text-sm font-medium truncate">{ROLE_LABELS[currentRole]}</span>
                  <span className="text-xs text-muted-foreground">
                    {currentRole === "owner" ? "完整权限" : "受限模式"}
                  </span>
                </div>
              )}
              {!isCollapsed && <MoreHorizontal className="ml-auto h-4 w-4 text-muted-foreground" />}
            </div>
          </DropdownMenuTrigger>
          <DropdownMenuContent
            className="w-56"
            align="end"
            side={isCollapsed ? "right" : "top"}
          >
            <DropdownMenuGroup>
              <DropdownMenuLabel>当前角色：{ROLE_LABELS[currentRole]}</DropdownMenuLabel>
              <DropdownMenuSeparator />
              <DropdownMenuItem onClick={onOpenRoleSwitch}>
                <ShieldCheck className="mr-2 h-4 w-4" />
                切换角色
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem className="text-destructive">
                <LogOut className="mr-2 h-4 w-4" />
                退出登录
              </DropdownMenuItem>
            </DropdownMenuGroup>
          </DropdownMenuContent>
        </DropdownMenu>
      </SidebarFooter>

      <SidebarRail />
    </Sidebar>
  );
}
