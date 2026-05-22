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
import { Plus, Pencil, Trash2, Save, Receipt, TrendingUp, CalendarRange, Download } from "lucide-react";
import { EmptyState } from "@/components/ui/empty-state";
import { toast } from "sonner";
import { ResponsiveContainer, BarChart, Bar, CartesianGrid, XAxis, YAxis, Tooltip } from "recharts";
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
  function downloadCSV(filename: string, headers: string[], rows: (string | number)[][]) {
    const escape = (v: string | number) => `"${String(v).replace(/"/g, '""')}"`;
    const csv = [headers.map(escape), ...rows.map((r) => r.map(escape))].map((r) => r.join(",")).join("\n");
    const blob = new Blob(["﻿" + csv], { type: "text/csv;charset=utf-8;" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
  }

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
  const monthlyTotals = new Map<string, number>();
  const monthlyTypeTotals = new Map<string, Map<string, number>>();
  for (const expense of filteredExpenses) {
    const month = expense.expense_date.slice(0, 7);
    monthlyTotals.set(month, (monthlyTotals.get(month) || 0) + expense.amount);

    const typeBuckets = monthlyTypeTotals.get(month) || new Map<string, number>();
    typeBuckets.set(expense.expense_type, (typeBuckets.get(expense.expense_type) || 0) + expense.amount);
    monthlyTypeTotals.set(month, typeBuckets);
  }

  const monthlyTrendData = Array.from(monthlyTotals.entries())
    .sort(([a], [b]) => a.localeCompare(b))
    .slice(-6)
    .map(([month, amount]) => {
      const typeBuckets = monthlyTypeTotals.get(month) || new Map<string, number>();
      let topType = "未分类";
      let topTypeAmount = 0;

      for (const [expenseType, total] of typeBuckets.entries()) {
        if (total > topTypeAmount) {
          topType = expenseType;
          topTypeAmount = total;
        }
      }

      return {
        month,
        amount,
        topType: getTypeLabel(topType),
        topTypeAmount,
      };
    });

  const currentMonth = new Date().toISOString().slice(0, 7);
  const previousMonthDate = new Date();
  previousMonthDate.setMonth(previousMonthDate.getMonth() - 1);
  const previousMonth = previousMonthDate.toISOString().slice(0, 7);
  const currentMonthAmount = filteredExpenses
    .filter((e) => e.expense_date.startsWith(currentMonth))
    .reduce((sum, e) => sum + e.amount, 0);
  const previousMonthAmount = filteredExpenses
    .filter((e) => e.expense_date.startsWith(previousMonth))
    .reduce((sum, e) => sum + e.amount, 0);
  const averageAmount = filteredExpenses.length > 0 ? totalAmount / filteredExpenses.length : 0;
  const monthDelta = currentMonthAmount - previousMonthAmount;
  const monthDeltaLabel = previousMonthAmount > 0
    ? `${((monthDelta / previousMonthAmount) * 100).toFixed(1)}%`
    : currentMonthAmount > 0
      ? "新增支出"
      : "0.0%";

  const exportExpensesCSV = () => {
    downloadCSV(
      `支出记录_${typeFilter}.csv`,
      ["日期", "类型", "金额", "备注"],
      filteredExpenses.map((expense) => [
        expense.expense_date,
        getTypeLabel(expense.expense_type),
        expense.amount.toFixed(2),
        expense.note || "",
      ]),
    );
  };

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
        <Button variant="outline" onClick={exportExpensesCSV} disabled={filteredExpenses.length === 0}>
          <Download className="mr-2 h-4 w-4" />导出 CSV
        </Button>
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

      <div className="grid gap-4 md:grid-cols-3">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">本月支出</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-destructive">¥{currentMonthAmount.toFixed(2)}</div>
            <p className="mt-1 text-xs text-muted-foreground">
              相比上月 {monthDelta >= 0 ? "增加" : "减少"} ¥{Math.abs(monthDelta).toFixed(2)}
            </p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">环比变化</CardTitle>
          </CardHeader>
          <CardContent>
            <div className={`text-2xl font-bold ${monthDelta > 0 ? "text-amber-600" : monthDelta < 0 ? "text-emerald-600" : "text-foreground"}`}>
              {monthDelta >= 0 ? "+" : ""}{monthDeltaLabel}
            </div>
            <p className="mt-1 text-xs text-muted-foreground">基于当前筛选条件对比上月</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">平均单笔支出</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">¥{averageAmount.toFixed(2)}</div>
            <p className="mt-1 text-xs text-muted-foreground">{filteredExpenses.length} 笔记录</p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <TrendingUp className="h-4 w-4" />
            月度支出趋势
          </CardTitle>
          <CardDescription>最近 6 个月支出走势与主要费用类型</CardDescription>
        </CardHeader>
        <CardContent>
          {monthlyTrendData.length === 0 ? (
            <EmptyState icon={CalendarRange} title="暂无趋势数据" description="新增支出后，这里会显示月度走势" />
          ) : (
            <div className="space-y-4">
              <div className="h-[260px]">
                <ResponsiveContainer width="100%" height="100%">
                  <BarChart data={monthlyTrendData}>
                    <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
                    <XAxis dataKey="month" className="text-xs" tick={{ fontSize: 12 }} />
                    <YAxis className="text-xs" tick={{ fontSize: 12 }} />
                    <Tooltip
                      formatter={(value, name, entry) => {
                        const numericValue = typeof value === "number"
                          ? value
                          : typeof value === "string"
                            ? Number(value)
                            : 0;
                        if (name === "amount") {
                          return [`¥${numericValue.toFixed(2)}`, "支出"];
                        }
                        const payload = entry?.payload as { topType?: string; topTypeAmount?: number } | undefined;
                        return [payload?.topTypeAmount ? `¥${payload.topTypeAmount.toFixed(2)}` : "¥0.00", payload?.topType || "主要类型"];
                      }}
                      labelFormatter={(label) => `${label}`}
                    />
                    <Bar dataKey="amount" fill="#EF4444" radius={[6, 6, 0, 0]} name="amount" />
                  </BarChart>
                </ResponsiveContainer>
              </div>
              <div className="grid gap-3 md:grid-cols-3">
                {monthlyTrendData.slice().reverse().map((item) => (
                  <div key={item.month} className="rounded-lg border bg-muted/30 px-4 py-3">
                    <div className="text-sm font-medium">{item.month}</div>
                    <div className="mt-1 text-xl font-semibold text-destructive">¥{item.amount.toFixed(2)}</div>
                    <div className="mt-1 text-xs text-muted-foreground">
                      主要类型: {item.topType} ¥{item.topTypeAmount.toFixed(2)}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </CardContent>
      </Card>

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
                        <Badge variant="secondary">{getTypeLabel(e.expense_type)}</Badge>
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
