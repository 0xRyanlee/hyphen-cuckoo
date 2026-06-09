import { call as invoke } from "@/lib/transport";

export type Role = "owner" | "cashier" | "chef" | "warehouse";

export const ROLE_LABELS: Record<Role, string> = {
  owner: "老板",
  cashier: "收银",
  chef: "厨师",
  warehouse: "仓库",
};

export const ROLE_DESCRIPTIONS: Record<Role, string> = {
  owner: "完整权限",
  cashier: "前台收银",
  chef: "厨房生产",
  warehouse: "仓储进货",
};

export const ROLE_COLORS: Record<Role, string> = {
  owner: "bg-secondary text-secondary-foreground border-secondary",
  cashier: "bg-blue-100 text-blue-700 border-blue-200 dark:bg-blue-900/30 dark:text-blue-300 dark:border-blue-800",
  chef: "bg-orange-100 text-orange-700 border-orange-200 dark:bg-orange-900/30 dark:text-orange-300 dark:border-orange-800",
  warehouse: "bg-emerald-100 text-emerald-700 border-emerald-200 dark:bg-emerald-900/30 dark:text-emerald-300 dark:border-emerald-800",
};

export const ROLE_ALLOWED_PAGES: Record<Role, string[] | "*"> = {
  owner: "*",
  cashier: ["dashboard", "pos", "orders", "customers", "kds"],
  chef: ["dashboard", "kds", "production-orders"],
  warehouse: [
    "dashboard", "inventory", "materials", "stocktakes",
    "purchase-orders", "production-orders", "suppliers",
    "material-states", "expenses",
  ],
};

export interface RolePinStatus {
  role: Role;
  has_pin: boolean;
}

export function checkAccess(role: Role, page: string): boolean {
  const allowed = ROLE_ALLOWED_PAGES[role];
  if (allowed === "*") return true;
  return allowed.includes(page);
}

export async function getCurrentRole(): Promise<Role> {
  return invoke<Role>("get_current_role");
}

export async function getRolePinStatuses(): Promise<Record<Role, boolean>> {
  const statuses = await invoke<RolePinStatus[]>("get_role_pin_statuses");
  return statuses.reduce<Record<Role, boolean>>((acc, status) => {
    acc[status.role] = status.has_pin;
    return acc;
  }, {
    owner: false,
    cashier: false,
    chef: false,
    warehouse: false,
  });
}

export async function saveRolePin(role: Role, pin: string | null): Promise<void> {
  await invoke("set_role_pin", { role, pin });
}

export async function switchRole(role: Role, pin: string | null): Promise<Role> {
  return invoke<Role>("switch_role", { role, pin });
}
