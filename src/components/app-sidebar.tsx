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
import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  type DragEndEvent,
} from "@dnd-kit/core";
import {
  SortableContext,
  sortableKeyboardCoordinates,
  useSortable,
  verticalListSortingStrategy,
  arrayMove,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { MoreHorizontal, Package, ChefHat, Warehouse, FileText, ShoppingCart, Settings, Home, LogOut, CreditCard, Factory, FileBox, BarChart3, Printer, Monitor, SlidersHorizontal, Receipt, Users, ShieldCheck, LayoutGrid, GripVertical } from "lucide-react";
import { type Role, ROLE_LABELS, checkAccess } from "@/lib/roles";
import { useState, useCallback } from "react";

const NAV_GROUPS_DEFAULT = [
  {
    id: "frontend",
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
    id: "backend",
    label: "后台管理",
    items: [
      { id: "dashboard", label: "仪表板", icon: Home },
      { id: "menu", label: "菜单", icon: FileText },
      { id: "materials", label: "材料管理", icon: Package },
      { id: "recipes", label: "配方", icon: ChefHat },
      { id: "inventory", label: "库存", icon: Warehouse },
      { id: "reports", label: "数据报表", icon: BarChart3 },
    ],
  },
  {
    id: "supply",
    label: "进货 / 生产",
    items: [
      { id: "purchase-orders", label: "进货管理", icon: FileBox },
      { id: "production-orders", label: "生产单", icon: Factory },
    ],
  },
  {
    id: "marketing",
    label: "营销",
    items: [
      { id: "marketing", label: "营销中心", icon: Monitor },
    ],
  },
  {
    id: "settings",
    label: "设置",
    items: [
      { id: "attributes", label: "属性模板", icon: SlidersHorizontal },
      { id: "expenses", label: "日常支出", icon: Receipt },
      { id: "print", label: "打印中心", icon: Printer },
      { id: "settings", label: "系统设置", icon: Settings },
    ],
  },
];

const NAV_ORDER_KEY = "cuckoo_sidebar_group_order";

function loadGroupOrder(): string[] {
  try {
    const raw = localStorage.getItem(NAV_ORDER_KEY);
    if (raw) {
      const order = JSON.parse(raw) as string[];
      const defaultIds = NAV_GROUPS_DEFAULT.map((g) => g.id);
      if (order.length === defaultIds.length && defaultIds.every((id) => order.includes(id))) {
        return order;
      }
    }
  } catch { /* ignore */ }
  return NAV_GROUPS_DEFAULT.map((g) => g.id);
}

interface SortableGroupProps {
  group: typeof NAV_GROUPS_DEFAULT[0];
  activeTab: string;
  onTabChange: (tab: string) => void;
  isCollapsed: boolean;
  errorCount: number;
  notificationCount: number;
  isEditMode: boolean;
}

function SortableNavGroup({
  group, activeTab, onTabChange, isCollapsed, errorCount, notificationCount, isEditMode,
  currentRole,
}: SortableGroupProps & { currentRole: Role }) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({ id: group.id });
  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  const visibleItems = group.items.filter((item) => checkAccess(currentRole, item.id));
  if (visibleItems.length === 0) return null;

  return (
    <div ref={setNodeRef} style={style}>
      <SidebarGroup>
        {!isCollapsed && (
          <SidebarGroupLabel className="text-xs text-muted-foreground flex items-center gap-1">
            {isEditMode && (
              <span
                {...attributes}
                {...listeners}
                className="cursor-grab active:cursor-grabbing text-muted-foreground/50 hover:text-muted-foreground"
              >
                <GripVertical className="h-3 w-3" />
              </span>
            )}
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
                    onClick={() => !isEditMode && onTabChange(item.id)}
                    isActive={activeTab === item.id}
                    tooltip={item.label}
                    className={isEditMode ? "cursor-default" : ""}
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
    </div>
  );
}

interface AppSidebarProps {
  activeTab: string;
  onTabChange: (tab: string) => void;
  connected: boolean;
  errorCount?: number;
  notificationCount?: number;
  currentRole?: Role;
  onOpenRoleSwitch?: () => void;
  onLogout?: () => void;
}

export function AppSidebar({ activeTab, onTabChange, errorCount = 0, notificationCount = 0, currentRole = "owner", onOpenRoleSwitch, onLogout }: AppSidebarProps) {
  const { state } = useSidebar();
  const isCollapsed = state === "collapsed";
  const [isEditMode, setIsEditMode] = useState(false);
  const [groupOrder, setGroupOrder] = useState<string[]>(loadGroupOrder);

  const sensors = useSensors(
    useSensor(PointerSensor),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates }),
  );

  const orderedGroups = groupOrder
    .map((id) => NAV_GROUPS_DEFAULT.find((g) => g.id === id))
    .filter(Boolean) as typeof NAV_GROUPS_DEFAULT;

  const handleDragEnd = useCallback((event: DragEndEvent) => {
    const { active, over } = event;
    if (!over || active.id === over.id) return;
    setGroupOrder((prev) => {
      const oldIdx = prev.indexOf(String(active.id));
      const newIdx = prev.indexOf(String(over.id));
      const next = arrayMove(prev, oldIdx, newIdx);
      localStorage.setItem(NAV_ORDER_KEY, JSON.stringify(next));
      return next;
    });
  }, []);

  function toggleEditMode() {
    setIsEditMode((v) => !v);
  }

  return (
    <Sidebar collapsible="icon" className="border-r border-border">
      <SidebarHeader className="h-14 border-b border-border px-4">
        <div className="flex items-center gap-3">
          <div className="flex h-7 w-7 shrink-0 items-center justify-center rounded-lg bg-primary text-primary-foreground">
            <span className="text-xs font-bold">C</span>
          </div>
          {!isCollapsed && (
            <div className="flex flex-col gap-0.5 leading-none flex-1 min-w-0">
              <span className="font-semibold text-sm">Cuckoo</span>
              <span className="text-xs text-muted-foreground">餐饮作业系统</span>
            </div>
          )}
          {!isCollapsed && (
            <button
              onClick={toggleEditMode}
              className={`text-xs px-1.5 py-0.5 rounded transition-colors shrink-0 ${isEditMode ? "bg-primary text-primary-foreground" : "text-muted-foreground hover:text-foreground"}`}
              title="自定义排序"
            >
              <GripVertical className="h-3.5 w-3.5" />
            </button>
          )}
        </div>
      </SidebarHeader>

      <SidebarContent>
        <DndContext
          sensors={sensors}
          collisionDetection={closestCenter}
          onDragEnd={handleDragEnd}
        >
          <SortableContext items={groupOrder} strategy={verticalListSortingStrategy}>
            {orderedGroups.map((group) => (
              <SortableNavGroup
                key={group.id}
                group={group}
                activeTab={activeTab}
                onTabChange={onTabChange}
                isCollapsed={isCollapsed}
                errorCount={errorCount}
                notificationCount={notificationCount}
                isEditMode={isEditMode}
                currentRole={currentRole}
              />
            ))}
          </SortableContext>
        </DndContext>
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
              <DropdownMenuItem className="text-destructive" onClick={onLogout}>
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
