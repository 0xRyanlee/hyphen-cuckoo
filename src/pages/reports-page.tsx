import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { BarChart3, TrendingUp, Trophy, PieChart as PieChartIcon, Package, Download, Clock } from "lucide-react";
import { EmptyState } from "@/components/ui/empty-state";
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, PieChart, Pie, Cell, LineChart, Line, Legend } from "recharts";

const COLORS = ["#3B82F6", "#10B981", "#F59E0B", "#EF4444", "#8B5CF6", "#EC4899", "#06B6D4", "#84CC16"];

export function ReportsPage() {
  const today = new Date().toISOString().split("T")[0];
  const weekAgo = new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString().split("T")[0];

  const [startDate, setStartDate] = useState(weekAgo);
  const [endDate, setEndDate] = useState(today);
  const [salesData, setSalesData] = useState<[string, number, number, number][]>([]);
  const [categoryData, setCategoryData] = useState<[string, number, number][]>([]);
  const [profitData, setProfitData] = useState<[string, number, number, number, number, number][]>([]);
  const [topItems, setTopItems] = useState<[string, number, number, number][]>([]);
  const [consumptionData, setConsumptionData] = useState<[string, number, number, number][]>([]);
  const [hourData, setHourData] = useState<[number, number, number][]>([]);
  const [weekdayData, setWeekdayData] = useState<[number, number, number][]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

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

  function exportAllCSV() {
    const ds = `${startDate}_${endDate}`;
    downloadCSV(`销售报表_${ds}.csv`, ["日期", "应收", "实收", "订单数"], salesData.map(([d, amt, cnt, col]) => [d, amt.toFixed(2), col.toFixed(2), cnt]));
    downloadCSV(`分类报表_${ds}.csv`, ["分类", "销售额", "订单数"], categoryData.map(([cat, amt, cnt]) => [cat, amt.toFixed(2), cnt]));
    downloadCSV(`毛利报表_${ds}.csv`, ["日期", "收入", "食材成本", "毛利", "支出", "净利"], profitData.map(([d, rev, cost, gp, exp, net]) => [d, rev.toFixed(2), cost.toFixed(2), gp.toFixed(2), exp.toFixed(2), net.toFixed(2)]));
    downloadCSV(`热销商品_${ds}.csv`, ["商品", "销售额", "数量", "均价"], topItems.map(([name, amt, qty, avg]) => [name, amt.toFixed(2), qty, avg.toFixed(2)]));
    downloadCSV(`原料消耗_${ds}.csv`, ["原料", "消耗量", "平均成本", "总成本"], consumptionData.map(([name, qty, avgCost, totalCost]) => [name, qty.toFixed(4), avgCost.toFixed(2), totalCost.toFixed(2)]));
  }

  async function loadReports() {
    if (!startDate || !endDate) return;
    setLoading(true);
    setError(null);
    try {
      const [sales, categories, profit, top, consumption, hours, weekdays] = await Promise.all([
        invoke<[string, number, number, number][]>("get_sales_report", { startDate, endDate }),
        invoke<[string, number, number][]>("get_sales_by_category", { startDate, endDate }),
        invoke<[string, number, number, number, number, number][]>("get_gross_profit_report", { startDate, endDate }),
        invoke<[string, number, number, number][]>("get_top_selling_items", { startDate, endDate, limit: 10 }),
        invoke<[string, number, number, number][]>("get_material_consumption_report", { startDate, endDate }),
        invoke<[number, number, number][]>("get_sales_by_hour", { startDate, endDate }),
        invoke<[number, number, number][]>("get_sales_by_weekday", { startDate, endDate }),
      ]);
      setSalesData(sales);
      setCategoryData(categories);
      setProfitData(profit);
      setTopItems(top);
      setConsumptionData(consumption);
      setHourData(hours);
      setWeekdayData(weekdays);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => { loadReports(); }, []);

  const totalSales = salesData.reduce((sum, [, amt]) => sum + amt, 0);
  const totalOrders = salesData.reduce((sum, [, , cnt]) => sum + cnt, 0);
  const totalCollected = salesData.reduce((sum, [, , , col]) => sum + col, 0);
  const totalRevenue = profitData.reduce((sum, [, rev]) => sum + rev, 0);
  const totalCost = profitData.reduce((sum, [, , cost]) => sum + cost, 0);
  const totalGrossProfit = totalRevenue - totalCost;
  const totalExpenses = profitData.reduce((sum, [, , , , exp]) => sum + exp, 0);
  const totalNetProfit = totalGrossProfit - totalExpenses;
  const grossMargin = totalRevenue > 0 ? ((totalGrossProfit / totalRevenue) * 100).toFixed(1) : "0.0";
  const netMargin = totalRevenue > 0 ? ((totalNetProfit / totalRevenue) * 100).toFixed(1) : "0.0";

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">数据报表</h2>
          <p className="text-sm text-muted-foreground">销售分析、毛利计算、热销排行</p>
        </div>
      </div>

      {error && (
        <div className="rounded-lg border border-destructive/50 bg-destructive/10 p-4 text-sm text-destructive">
          报表加载失败: {error}
          <Button variant="link" onClick={() => setError(null)} className="ml-2">关闭</Button>
        </div>
      )}

      {/* Date Range Filter */}
      <Card>
        <CardContent className="pt-6">
          <div className="flex items-end gap-4">
            <div className="space-y-2">
              <Label>开始日期</Label>
              <Input type="date" value={startDate} onChange={(e) => setStartDate(e.target.value)} />
            </div>
            <div className="space-y-2">
              <Label>结束日期</Label>
              <Input type="date" value={endDate} onChange={(e) => setEndDate(e.target.value)} />
            </div>
            <Button onClick={loadReports} disabled={loading}>查詢</Button>
            <Button variant="outline" onClick={exportAllCSV} disabled={loading || salesData.length === 0}>
              <Download className="mr-2 h-4 w-4" />導出 CSV
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Summary Cards */}
      <div className="grid gap-4 md:grid-cols-6">
        <Card>
          <CardHeader className="pb-2"><CardTitle className="text-sm font-medium text-muted-foreground">应收</CardTitle></CardHeader>
          <CardContent><div className="text-2xl font-bold">¥{totalSales.toFixed(2)}</div></CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2"><CardTitle className="text-sm font-medium text-muted-foreground">实收</CardTitle></CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-blue-500">¥{totalCollected.toFixed(2)}</div>
            {totalSales > 0 && totalCollected < totalSales && <div className="text-xs text-amber-500 mt-0.5">未收 ¥{(totalSales - totalCollected).toFixed(2)}</div>}
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2"><CardTitle className="text-sm font-medium text-muted-foreground">总订单数</CardTitle></CardHeader>
          <CardContent><div className="text-2xl font-bold">{totalOrders}</div></CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2"><CardTitle className="text-sm font-medium text-muted-foreground">毛利</CardTitle></CardHeader>
          <CardContent><div className={`text-2xl font-bold ${totalGrossProfit >= 0 ? "text-emerald-500" : "text-destructive"}`}>¥{totalGrossProfit.toFixed(2)}</div><div className="text-xs text-muted-foreground mt-0.5">{grossMargin}%</div></CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2"><CardTitle className="text-sm font-medium text-muted-foreground">支出</CardTitle></CardHeader>
          <CardContent><div className="text-2xl font-bold text-amber-500">¥{totalExpenses.toFixed(2)}</div></CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2"><CardTitle className="text-sm font-medium text-muted-foreground">净利</CardTitle></CardHeader>
          <CardContent><div className={`text-2xl font-bold ${totalNetProfit >= 0 ? "text-emerald-600" : "text-destructive"}`}>¥{totalNetProfit.toFixed(2)}</div><div className="text-xs text-muted-foreground mt-0.5">{netMargin}%</div></CardContent>
        </Card>
      </div>

      {/* Report Tabs */}
      <Tabs defaultValue="sales" className="w-full" orientation="horizontal">
        <TabsList variant="line" className="w-full">
          <TabsTrigger value="sales" className="gap-1.5"><BarChart3 className="h-4 w-4" />销售报表</TabsTrigger>
          <TabsTrigger value="profit" className="gap-1.5"><TrendingUp className="h-4 w-4" />毛利报表</TabsTrigger>
          <TabsTrigger value="top" className="gap-1.5"><Trophy className="h-4 w-4" />热销排行</TabsTrigger>
          <TabsTrigger value="category" className="gap-1.5"><PieChartIcon className="h-4 w-4" />分类销售</TabsTrigger>
          <TabsTrigger value="consumption" className="gap-1.5"><Package className="h-4 w-4" />原料消耗</TabsTrigger>
          <TabsTrigger value="time" className="gap-1.5"><Clock className="h-4 w-4" />时段分析</TabsTrigger>
        </TabsList>

        {/* Sales Report */}
        <TabsContent value="sales" className="mt-6">
          <Card>
            <CardHeader>
              <CardTitle>每日销售额</CardTitle>
              <CardDescription>按日期统计销售金额和订单数</CardDescription>
            </CardHeader>
            <CardContent>
              {salesData.length === 0 ? (
                <EmptyState icon={BarChart3} title="暂无数据" description="选择日期范围查询销售数据" />
              ) : (
                <>
                  <div className="h-[300px] mb-6">
                    <ResponsiveContainer width="100%" height="100%">
                      <BarChart data={salesData.map(([date, amount, count]) => ({ date, amount, count }))}>
                        <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
                        <XAxis dataKey="date" className="text-xs" tick={{ fontSize: 12 }} />
                        <YAxis className="text-xs" tick={{ fontSize: 12 }} />
                        <Tooltip formatter={(value: unknown, name: unknown) => {
                          const display = typeof value === "number" ? `¥${value.toFixed(2)}` : String(value ?? "");
                          const label = name === "amount" ? "销售额" : name === "count" ? "订单数" : String(name ?? "");
                          return [display, label];
                        }} />
                        <Bar dataKey="amount" fill="#3B82F6" radius={[4, 4, 0, 0]} name="销售额" />
                      </BarChart>
                    </ResponsiveContainer>
                  </div>
                  <Table>
                    <TableHeader>
                      <TableRow><TableHead>日期</TableHead><TableHead className="text-right">应收</TableHead><TableHead className="text-right">实收</TableHead><TableHead className="text-right">订单数</TableHead></TableRow>
                    </TableHeader>
                    <TableBody>
                      {salesData.map(([date, amount, count, collected]) => (
                        <TableRow key={date}>
                          <TableCell className="font-mono text-xs">{date}</TableCell>
                          <TableCell className="text-right font-medium">¥{amount.toFixed(2)}</TableCell>
                          <TableCell className={`text-right font-medium ${collected < amount ? "text-amber-500" : "text-emerald-600"}`}>¥{collected.toFixed(2)}</TableCell>
                          <TableCell className="text-right">{count}</TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        {/* Profit Report */}
        <TabsContent value="profit" className="mt-6">
          <Card>
            <CardHeader>
              <CardTitle>每日毛利</CardTitle>
              <CardDescription>收入、成本、毛利对比</CardDescription>
            </CardHeader>
            <CardContent>
              {profitData.length === 0 ? (
                <EmptyState icon={TrendingUp} title="暂无数据" description="选择日期范围查询毛利数据" />
              ) : (
                <>
                  <div className="h-[300px] mb-6">
                    <ResponsiveContainer width="100%" height="100%">
                      <LineChart data={profitData.map(([date, revenue, cost, grossProfit, expenses, netProfit]) => ({ date, revenue, cost, grossProfit, expenses, netProfit }))}>
                        <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
                        <XAxis dataKey="date" className="text-xs" tick={{ fontSize: 12 }} />
                        <YAxis className="text-xs" tick={{ fontSize: 12 }} />
                        <Tooltip formatter={(value: unknown) => [`¥${typeof value === "number" ? value.toFixed(2) : value}`, ""]} />
                        <Legend />
                        <Line type="monotone" dataKey="revenue" stroke="#3B82F6" strokeWidth={2} name="收入" />
                        <Line type="monotone" dataKey="cost" stroke="#EF4444" strokeWidth={2} name="食材成本" />
                        <Line type="monotone" dataKey="grossProfit" stroke="#10B981" strokeWidth={2} name="毛利" />
                        <Line type="monotone" dataKey="expenses" stroke="#F59E0B" strokeWidth={2} strokeDasharray="4 2" name="支出" />
                        <Line type="monotone" dataKey="netProfit" stroke="#8B5CF6" strokeWidth={2} name="净利" />
                      </LineChart>
                    </ResponsiveContainer>
                  </div>
                  <Table>
                    <TableHeader>
                      <TableRow><TableHead>日期</TableHead><TableHead className="text-right">收入</TableHead><TableHead className="text-right">食材成本</TableHead><TableHead className="text-right">毛利</TableHead><TableHead className="text-right">支出</TableHead><TableHead className="text-right">净利</TableHead></TableRow>
                    </TableHeader>
                    <TableBody>
                      {profitData.map(([date, revenue, cost, grossProfit, expenses, netProfit]) => {
                        const gm = revenue > 0 ? ((grossProfit / revenue) * 100).toFixed(1) : "0.0";
                        return (
                          <TableRow key={date}>
                            <TableCell className="font-mono text-xs">{date}</TableCell>
                            <TableCell className="text-right">¥{revenue.toFixed(2)}</TableCell>
                            <TableCell className="text-right text-muted-foreground">¥{cost.toFixed(2)}</TableCell>
                            <TableCell className={`text-right font-medium ${grossProfit >= 0 ? "text-emerald-500" : "text-destructive"}`}>¥{grossProfit.toFixed(2)} <span className="text-xs opacity-60">({gm}%)</span></TableCell>
                            <TableCell className="text-right text-amber-500">¥{expenses.toFixed(2)}</TableCell>
                            <TableCell className={`text-right font-bold ${netProfit >= 0 ? "text-emerald-600" : "text-destructive"}`}>¥{netProfit.toFixed(2)}</TableCell>
                          </TableRow>
                        );
                      })}
                    </TableBody>
                  </Table>
                </>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        {/* Top Items */}
        <TabsContent value="top" className="mt-6">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2"><Trophy className="h-4 w-4" />热销商品排行</CardTitle>
              <CardDescription>按销量排序 Top 10</CardDescription>
            </CardHeader>
            <CardContent>
              {topItems.length === 0 ? (
                <EmptyState icon={Trophy} title="暂无数据" description="销售数据将显示热销排行" />
              ) : (
                <Table>
                  <TableHeader>
                    <TableRow><TableHead className="w-12">排名</TableHead><TableHead>商品</TableHead><TableHead className="text-right">销量</TableHead><TableHead className="text-right">销售额</TableHead><TableHead className="text-right">均价</TableHead></TableRow>
                  </TableHeader>
                  <TableBody>
                    {topItems.map(([name, revenue, qty, avgPrice], idx) => (
                      <TableRow key={name}>
                        <TableCell>
                          <Badge variant={idx < 3 ? "default" : "secondary"} className="w-8 h-6 flex items-center justify-center">{idx + 1}</Badge>
                        </TableCell>
                        <TableCell className="font-medium">{name}</TableCell>
                        <TableCell className="text-right">{qty}</TableCell>
                        <TableCell className="text-right font-medium">¥{revenue.toFixed(2)}</TableCell>
                        <TableCell className="text-right text-muted-foreground">¥{avgPrice.toFixed(2)}</TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        {/* Category Sales */}
        <TabsContent value="category" className="mt-6">
          <Card>
            <CardHeader>
              <CardTitle>分类销售</CardTitle>
              <CardDescription>按菜单分类统计</CardDescription>
            </CardHeader>
            <CardContent>
              {categoryData.length === 0 ? (
                <EmptyState icon={PieChartIcon} title="暂无数据" description="分类销售统计将在此显示" />
              ) : (
                <>
                  <div className="h-[300px] mb-6">
                    <ResponsiveContainer width="100%" height="100%">
                      <PieChart>
                        <Pie
                          data={categoryData.map(([name, amount, qty]) => ({ name: name || "未分類", value: amount, qty }))}
                          cx="50%"
                          cy="50%"
                          labelLine={false}
                          label={({ name, percent }) => `${name ?? ""} ${percent != null ? (percent * 100).toFixed(0) : 0}%`}
                          outerRadius={100}
                          fill="#8884d8"
                          dataKey="value"
                        >
                          {categoryData.map((_entry, index) => (
                            <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                          ))}
                        </Pie>
                        <Tooltip formatter={(value: unknown) => [`¥${typeof value === "number" ? value.toFixed(2) : value}`, "销售额"]} />
                      </PieChart>
                    </ResponsiveContainer>
                  </div>
                  <Table>
                    <TableHeader>
                      <TableRow><TableHead>分类</TableHead><TableHead className="text-right">销售额</TableHead><TableHead className="text-right">销量</TableHead></TableRow>
                    </TableHeader>
                    <TableBody>
                      {categoryData.map(([name, amount, qty]) => (
                        <TableRow key={name}>
                          <TableCell className="font-medium">{name || "未分類"}</TableCell>
                          <TableCell className="text-right font-medium">¥{amount.toFixed(2)}</TableCell>
                          <TableCell className="text-right">{qty}</TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        {/* Material Consumption */}
        <TabsContent value="consumption" className="mt-6">
          <Card>
            <CardHeader>
              <CardTitle>原料消耗报表</CardTitle>
              <CardDescription>按日期范围统计原料消耗数量和成本</CardDescription>
            </CardHeader>
            <CardContent>
              {consumptionData.length === 0 ? (
                <EmptyState icon={Package} title="暂无数据" description="选择日期范围查询原料消耗数据" />
              ) : (
                <>
                  <div className="h-[300px] mb-6">
                    <ResponsiveContainer width="100%" height="100%">
                      <BarChart data={consumptionData.map(([name, qty, , cost]) => ({ name, qty, cost }))}>
                        <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
                        <XAxis dataKey="name" className="text-xs" />
                        <YAxis yAxisId="left" orientation="left" stroke="#3b82f6" className="text-xs" />
                        <YAxis yAxisId="right" orientation="right" stroke="#10b981" className="text-xs" />
                        <Tooltip formatter={(value: unknown, name) => [String(name) === "qty" ? `${value}` : `¥${value}`, String(name) === "qty" ? "消耗数量" : "成本"]} />
                        <Legend />
                        <Bar yAxisId="left" dataKey="qty" name="消耗数量" fill="#3b82f6" radius={[4, 4, 0, 0]} />
                        <Bar yAxisId="right" dataKey="cost" name="成本(¥)" fill="#10b981" radius={[4, 4, 0, 0]} />
                      </BarChart>
                    </ResponsiveContainer>
                  </div>
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>原料</TableHead>
                        <TableHead className="text-right">消耗数量</TableHead>
                        <TableHead className="text-right">平均成本</TableHead>
                        <TableHead className="text-right">总成本</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {consumptionData.map(([name, qty, avgCost, totalCost]) => (
                        <TableRow key={name}>
                          <TableCell className="font-medium">{name || "未知名"}</TableCell>
                          <TableCell className="text-right">{qty.toFixed(2)}</TableCell>
                          <TableCell className="text-right">¥{avgCost.toFixed(2)}</TableCell>
                          <TableCell className="text-right font-medium">¥{totalCost.toFixed(2)}</TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </>
              )}
            </CardContent>
          </Card>
        </TabsContent>
        {/* Time Analysis */}
        <TabsContent value="time" className="mt-6">
          <div className="grid gap-6 lg:grid-cols-2">
            <Card>
              <CardHeader>
                <CardTitle>时段销售分布</CardTitle>
                <CardDescription>按小时统计销售额（0–23时）</CardDescription>
              </CardHeader>
              <CardContent>
                {hourData.length === 0 ? (
                  <EmptyState icon={Clock} title="暂无数据" description="选择日期范围查询时段数据" />
                ) : (
                  <div className="h-[300px]">
                    <ResponsiveContainer width="100%" height="100%">
                      <BarChart data={(() => {
                        const map = Object.fromEntries(hourData.map(([h, , amt]) => [h, amt]));
                        return Array.from({ length: 24 }, (_, h) => ({ hour: `${h}时`, amount: map[h] ?? 0 }));
                      })()}>
                        <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
                        <XAxis dataKey="hour" className="text-xs" tick={{ fontSize: 11 }} interval={1} />
                        <YAxis className="text-xs" tick={{ fontSize: 12 }} />
                        <Tooltip formatter={(value: unknown) => [`¥${typeof value === "number" ? value.toFixed(2) : value}`, "销售额"]} />
                        <Bar dataKey="amount" fill="#3B82F6" radius={[4, 4, 0, 0]} />
                      </BarChart>
                    </ResponsiveContainer>
                  </div>
                )}
              </CardContent>
            </Card>
            <Card>
              <CardHeader>
                <CardTitle>星期销售分布</CardTitle>
                <CardDescription>按星期统计销售额</CardDescription>
              </CardHeader>
              <CardContent>
                {weekdayData.length === 0 ? (
                  <EmptyState icon={Clock} title="暂无数据" description="选择日期范围查询星期数据" />
                ) : (
                  <div className="h-[300px]">
                    <ResponsiveContainer width="100%" height="100%">
                      <BarChart data={(() => {
                        const labels = ["日", "一", "二", "三", "四", "五", "六"];
                        const map = Object.fromEntries(weekdayData.map(([d, , amt]) => [d, amt]));
                        return labels.map((label, i) => ({ day: `周${label}`, amount: map[i] ?? 0 }));
                      })()}>
                        <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
                        <XAxis dataKey="day" className="text-xs" tick={{ fontSize: 12 }} />
                        <YAxis className="text-xs" tick={{ fontSize: 12 }} />
                        <Tooltip formatter={(value: unknown) => [`¥${typeof value === "number" ? value.toFixed(2) : value}`, "销售额"]} />
                        <Bar dataKey="amount" fill="#10B981" radius={[4, 4, 0, 0]} />
                      </BarChart>
                    </ResponsiveContainer>
                  </div>
                )}
              </CardContent>
            </Card>
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
}
