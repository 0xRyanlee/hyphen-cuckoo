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
  owner: "bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-300",
  cashier: "bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-300",
  chef: "bg-orange-100 text-orange-700 dark:bg-orange-900/30 dark:text-orange-300",
  warehouse: "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-300",
};

// '*' = unrestricted; array = allowed page IDs
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

const pinKey = (role: Role) => `cuckoo_pin_${role}`;

export function getRolePin(role: Role): string | null {
  return localStorage.getItem(pinKey(role));
}

export function setRolePin(role: Role, pin: string | null) {
  if (pin) localStorage.setItem(pinKey(role), pin);
  else localStorage.removeItem(pinKey(role));
}

export function checkAccess(role: Role, page: string): boolean {
  const allowed = ROLE_ALLOWED_PAGES[role];
  if (allowed === "*") return true;
  return allowed.includes(page);
}
