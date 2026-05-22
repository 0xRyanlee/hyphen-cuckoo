import { useEffect, useState } from "react";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ShieldAlert } from "lucide-react";
import {
  type Role,
  ROLE_LABELS,
  ROLE_DESCRIPTIONS,
  ROLE_COLORS,
  getRolePinStatuses,
  switchRole,
} from "@/lib/roles";
import { toast } from "sonner";

const ROLES: Role[] = ["owner", "cashier", "chef", "warehouse"];

interface RoleSwitchDialogProps {
  open: boolean;
  currentRole: Role;
  onSwitch: (role: Role) => void;
  onClose: () => void;
}

export function RoleSwitchDialog({ open, currentRole, onSwitch, onClose }: RoleSwitchDialogProps) {
  const [pendingRole, setPendingRole] = useState<Role | null>(null);
  const [pin, setPin] = useState("");
  const [error, setError] = useState("");
  const [pinStatuses, setPinStatuses] = useState<Record<Role, boolean>>({
    owner: false,
    cashier: false,
    chef: false,
    warehouse: false,
  });

  useEffect(() => {
    if (!open) return;
    getRolePinStatuses().then(setPinStatuses).catch(() => {});
  }, [open]);

  const handleRoleClick = async (role: Role) => {
    if (role === currentRole) { onClose(); return; }
    if (pinStatuses[role]) {
      setPendingRole(role);
      setPin("");
      setError("");
      return;
    }
    try {
      const nextRole = await switchRole(role, null);
      onSwitch(nextRole);
      onClose();
    } catch (e) {
      toast.error("切换角色失败", { description: String(e) });
    }
  };

  const handleConfirmPin = async () => {
    if (!pendingRole) return;
    try {
      const nextRole = await switchRole(pendingRole, pin);
      onSwitch(nextRole);
      setPendingRole(null);
      setPin("");
      setError("");
      onClose();
    } catch (e) {
      setError(String(e));
      setPin("");
    }
  };

  const handleClose = () => {
    setPendingRole(null);
    setPin("");
    setError("");
    onClose();
  };

  return (
    <Dialog open={open} onOpenChange={(v) => !v && handleClose()}>
      <DialogContent className="max-w-sm">
        <DialogHeader>
          <DialogTitle>
            {pendingRole ? `切换到 ${ROLE_LABELS[pendingRole]}` : "切换角色"}
          </DialogTitle>
        </DialogHeader>

        {pendingRole ? (
          <div className="space-y-4 py-2">
            <p className="text-sm text-muted-foreground">
              请输入 <span className="font-medium text-foreground">{ROLE_LABELS[pendingRole]}</span> 的 PIN 码
            </p>
            <div className="space-y-1.5">
              <Label>PIN</Label>
              <Input
                type="password"
                inputMode="numeric"
                maxLength={8}
                value={pin}
                onChange={(e) => { setPin(e.target.value); setError(""); }}
                onKeyDown={(e) => e.key === "Enter" && handleConfirmPin()}
                autoFocus
                placeholder="输入 PIN"
              />
              {error && (
                <p className="text-xs text-destructive flex items-center gap-1">
                  <ShieldAlert className="w-3 h-3" />
                  {error}
                </p>
              )}
            </div>
            <div className="flex justify-end gap-2">
              <Button variant="outline" onClick={() => { setPendingRole(null); setPin(""); setError(""); }}>
                返回
              </Button>
              <Button onClick={handleConfirmPin} disabled={!pin}>确认</Button>
            </div>
          </div>
        ) : (
          <div className="grid grid-cols-2 gap-3 py-2">
            {ROLES.map((role) => {
              const isActive = role === currentRole;
              const hasPin = pinStatuses[role];
              return (
                <button
                  key={role}
                  onClick={() => handleRoleClick(role)}
                  className={`flex flex-col items-start gap-1 rounded-xl border p-3 text-left transition-all hover:shadow-sm ${
                    isActive
                      ? "border-primary bg-primary/5 ring-1 ring-primary"
                      : "border-border hover:border-primary/50"
                  }`}
                >
                  <span className={`rounded-md px-1.5 py-0.5 text-xs font-semibold ${ROLE_COLORS[role]}`}>
                    {ROLE_LABELS[role]}
                  </span>
                  <span className="text-xs text-muted-foreground">{ROLE_DESCRIPTIONS[role]}</span>
                  {hasPin && (
                    <span className="text-[10px] text-muted-foreground flex items-center gap-0.5">
                      <ShieldAlert className="w-2.5 h-2.5" />需要 PIN
                    </span>
                  )}
                  {isActive && (
                    <span className="text-[10px] text-primary font-medium">当前角色</span>
                  )}
                </button>
              );
            })}
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
