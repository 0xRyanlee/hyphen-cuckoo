import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Badge } from "@/components/ui/badge";
import { Plus, Pencil, Trash2, Save, Receipt } from "lucide-react";
import { EmptyState } from "@/components/ui/empty-state";
import { toast } from "sonner";
import type { Expense } from "@/types";

const EXPENSE_TYPES = [
  { value: "water", label: "水费" },
  { value: "electric", label: "电费" },
  { value: "gas", label: "燃气费" },
  { value: "internet", label: "网络费" },
  { value: "labor", label: "人工费" },
  { value: "rent", label: "租金" },
  { value: "repair", label: "维修费" },
  { value: "other", label: "其他" },
];

function getTypeLabel(value: string): string {
  return EXPENSE_TYPES.find((t) => t.value === value)?.label || value;
}

function getTypeColor(value: string): string {
  switch (value) {
    case "water": return "bg-blue-100 text-blue-800";
    case "electric": return "bg-yellow-100 text-yellow-800";
    case "gas": return "bg-orange-100 text-orange-800";
    case "internet": return "bg-purple-100 text-purple-800";
    case "labor": return "bg-green-100 text-green-800";
    case "rent": return "bg-pink-100 text-pink-800";
    case "repair": return "bg-red-100 text-red-800";
    default: return "bg-gray-100 text-gray-800";
  }
}

interface ExpensesPageProps {
  expenses: Expense[];
  onCreateExpense: (data: { expense_type: string; amount: number; expense_date: string; note: string }) => void;
  onUpdateExpense: (id: number, data: { expense_type?: string; amount?: number; expense_date?: string; note?: string }) => void;
  onDeleteExpense: (id: number) => void;
}

export function ExpensesPage({
  expenses,
  onCreateExpense,
  onUpdateExpense,
  onDeleteExpense,
}: ExpensesPageProps) {
  const [typeFilter, setTypeFilter] = useState<string>("all");
  const [newType, setNewType] = useState("water");
  const [newAmount, setNewAmount] = useState("");
  const [newDate, setNewDate] = useState(new Date().toISOString().split("T")[0]);
  const [newNote, setNewNote] = useState("");

  const [editExpense, setEditExpense] = useState<Expense | null>(null);
  const [editType, setEditType] = useState("");
  const [editAmount, setEditAmount] = useState("");
  const [editDate, setEditDate] = useState("");
  const [editNote, setEditNote] = useState("");

  const [deleteConfirm, setDeleteConfirm] = useState<Expense | null>(null);

  const filteredExpenses = expenses.filter((e) => {
    if (typeFilter !== "all" && e.expense_type !== typeFilter) return false;
    return true;
  });

  const totalAmount = filteredExpenses.reduce((sum, e) => sum + e.amount, 0);

  function openEdit(e: Expense) {
    setEditExpense(e);
    setEditType(e.expense_type);
    setEditAmount(e.amount.toString());
    setEditDate(e.expense_date);
    setEditNote(e.note || "");
  }

  function saveEdit() {
    if (!editExpense) return;
    const amount = parseFloat(editAmount);
    if (isNaN(amount) || amount <= 0) {
      toast.error("请输入有效金额");
      return;
    }
    onUpdateExpense(editExpense.id, {
      expense_type: editType,
      amount,
      expense_date: editDate,
      note: editNote || undefined,
    });
    setEditExpense(null);
  }

  function handleCreate() {
    const amount = parseFloat(newAmount);
    if (!newAmount || isNaN(amount) || amount <= 0) {
      toast.error("请输入有效金额");
      return;
    }
    onCreateExpense({ expense_type: newType, amount, expense_date: newDate, note: newNote });
    setNewAmount(""); setNewNote("");
    toast.success("支出已记录");
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">日常支出</h2>
          <p className="text-sm text-muted-foreground">管理日常运营费用</p>
        </div>
      </div>

      <div className="flex gap-4 flex-wrap items-center">
        <Select value={typeFilter} onValueChange={setTypeFilter}>
          <SelectTrigger className="w-40">
            <SelectValue placeholder="筛选类型" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">全部类型</SelectItem>
            {EXPENSE_TYPES.map((t) => (
              <SelectItem key={t.value} value={t.value}>{t.label}</SelectItem>
            ))}
          </SelectContent>
        </Select>
        <div className="ml-auto flex items-center gap-2 rounded-lg border px-4 py-2 bg-muted">
          <span className="text-sm text-muted-foreground">当前筛选合计</span>
          <span className="text-lg font-bold text-destructive">¥{totalAmount.toFixed(2)}</span>
        </div>
      </div>

      <div className="grid gap-6 lg:grid-cols-4">
        <Card>
          <CardHeader><CardTitle>新增支出</CardTitle></CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label>费用类型</Label>
              <Select value={newType} onValueChange={setNewType}>
                <SelectTrigger><SelectValue /></SelectTrigger>
                <SelectContent>
                  {EXPENSE_TYPES.map((t) => (
                    <SelectItem key={t.value} value={t.value}>{t.label}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label>金额（元）</Label>
              <Input type="number" value={newAmount} onChange={(e) => setNewAmount(e.target.value)} placeholder="0.00" step="0.01" min="0" />
            </div>
            <div className="space-y-2">
              <Label>日期</Label>
              <Input type="date" value={newDate} onChange={(e) => setNewDate(e.target.value)} />
            </div>
            <div className="space-y-2">
              <Label>备注</Label>
              <Input value={newNote} onChange={(e) => setNewNote(e.target.value)} placeholder="可选备注" />
            </div>
            <Button className="w-full" onClick={handleCreate} disabled={!newAmount}>
              <Plus className="mr-2 h-4 w-4" />记录支出
            </Button>
          </CardContent>
        </Card>

        <Card className="lg:col-span-3">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Receipt className="h-4 w-4" />
              支出记录
            </CardTitle>
            <CardDescription>共 {filteredExpenses.length} 笔记录</CardDescription>
          </CardHeader>
          <CardContent>
            {filteredExpenses.length === 0 ? (
              <EmptyState icon={Receipt} title="暂无支出记录" description="从左侧添加第一笔支出" />
            ) : (
              <ScrollArea className="max-h-[calc(100vh-22rem)]">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>日期</TableHead>
                    <TableHead>类型</TableHead>
                    <TableHead className="text-right">金额</TableHead>
                    <TableHead>备注</TableHead>
                    <TableHead className="text-right">操作</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {filteredExpenses.map((e) => (
                    <TableRow key={e.id}>
                      <TableCell className="text-muted-foreground text-sm">{e.expense_date}</TableCell>
                      <TableCell>
                        <Badge className={getTypeColor(e.expense_type)}>{getTypeLabel(e.expense_type)}</Badge>
                      </TableCell>
                      <TableCell className="text-right font-medium">¥{e.amount.toFixed(2)}</TableCell>
                      <TableCell className="text-muted-foreground text-xs">{e.note || "-"}</TableCell>
                      <TableCell className="text-right">
                        <div className="flex justify-end gap-1">
                          <Button variant="ghost" size="icon" className="h-8 w-8" onClick={() => openEdit(e)}><Pencil className="h-4 w-4" /></Button>
                          <Button variant="ghost" size="icon" className="h-8 w-8 text-destructive" onClick={() => setDeleteConfirm(e)}><Trash2 className="h-4 w-4" /></Button>
                        </div>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
              </ScrollArea>
            )}
          </CardContent>
        </Card>
      </div>

      <Dialog open={!!editExpense} onOpenChange={() => setEditExpense(null)}>
        <DialogContent>
          <DialogHeader><DialogTitle>编辑支出</DialogTitle></DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label>费用类型</Label>
              <Select value={editType} onValueChange={setEditType}>
                <SelectTrigger><SelectValue /></SelectTrigger>
                <SelectContent>
                  {EXPENSE_TYPES.map((t) => (
                    <SelectItem key={t.value} value={t.value}>{t.label}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label>金额（元）</Label>
              <Input type="number" value={editAmount} onChange={(e) => setEditAmount(e.target.value)} step="0.01" min="0" />
            </div>
            <div className="space-y-2">
              <Label>日期</Label>
              <Input type="date" value={editDate} onChange={(e) => setEditDate(e.target.value)} />
            </div>
            <div className="space-y-2">
              <Label>备注</Label>
              <Input value={editNote} onChange={(e) => setEditNote(e.target.value)} />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setEditExpense(null)}>取消</Button>
            <Button onClick={saveEdit}><Save className="mr-1 h-4 w-4" />保存</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={!!deleteConfirm} onOpenChange={() => setDeleteConfirm(null)}>
        <DialogContent>
          <DialogHeader><DialogTitle>确认删除</DialogTitle></DialogHeader>
          <p className="py-4 text-sm text-muted-foreground">
            确定要删除「{deleteConfirm ? getTypeLabel(deleteConfirm.expense_type) : ""} ¥{deleteConfirm?.amount}」这条记录吗？
          </p>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteConfirm(null)}>取消</Button>
            <Button variant="destructive" onClick={() => { if (deleteConfirm) onDeleteExpense(deleteConfirm.id); setDeleteConfirm(null); }}>删除</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}