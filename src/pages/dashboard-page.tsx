import { useState, useEffect } from "react";
import { call as invoke } from "@/lib/transport";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { ChartContainer, ChartTooltip, type ChartConfig } from "@/components/ui/chart";
import { EmptyState } from "@/components/ui/empty-state";
import { Package, ChefHat, ShoppingCart, TrendingUp, AlertTriangle, BarChart3, Calendar, Trophy, Clock, Zap } from "lucide-react";
import { Button } from "@/components/ui/button";
import { AreaChart, Area, CartesianGrid, XAxis, YAxis, PieChart, Pie, Cell } from "recharts";

const COLORS = ["#3b82f6", "#10b981", "#f59e0b", "#ef4444", "#8b5cf6", "#ec4899", "#06b6d4", "#84cc16"];

interface DashboardProps {
  materialsCount: number;
  recipesCount: number;
  ordersCount: number;
  batchesCount: number;
  orders: Array<{
    id: number;
    order_no: string;
    status: string;
    amount_total: number;
    created_at: string;
  }>;
  inventorySummary: Array<{
    material_id: number;
    material_name: string;
    available_qty: number;
  }>;
  loading?: boolean;
}

export function DashboardPage({
  materialsCount,
  recipesCount,
  ordersCount: _ordersCount,
  batchesCount,
  orders,
  inventorySummary,
loading = false,
}: DashboardProps) {
  const [timeRange, setTimeRange] = useState<"today" | "week" | "month" | "all">("today");
  const [todayTopItems, setTodayTopItems] = useState<[string, number, number, number][]>([]);
  const [marketingStats, setMarketingStats] = useState<{ redemptions_today: number; coupons_issued_today: number; coupons_redeemed_today: number } | null>(null);
  const LOW_STOCK_THRESHOLD = 10;
  const lowStockItems = inventorySummary.filter((s) => s.available_qty < LOW_STOCK_THRESHOLD);

  // 時間篩選邏輯
  const now = new Date();
  const todayStr = now.toISOString().split("T")[0];
  const weekAgo = new Date(now.getTime() - 7 * 86400000).toISOString().split("T")[0];
  const lastWeekStart = new Date(now.getTime() - 14 * 86400000).toISOString().split("T")[0];
  const monthAgo = new Date(now.getTime() - 30 * 86400000).toISOString().split("T")[0];

  useEffect(() => {
    invoke<[string, number, number, number][]>("get_top_selling_items", { startDate: todayStr, endDate: todayStr, limit: 5 })
      .then((items) => setTodayTopItems(items))
      .catch(() => {});
    invoke<{ redemptions_today: number; coupons_issued_today: number; coupons_redeemed_today: number }>("get_marketing_stats_today")
      .then(setMarketingStats).catch(() => {});
  }, [todayStr]);

  const filteredOrders = orders.filter((o) => {
    if (timeRange === "all") return true;
    const orderDate = (o.created_at || "").replace("T", " ").split(" ")[0];
    if (timeRange === "today") return orderDate === todayStr;
    if (timeRange === "week") return orderDate >= weekAgo;
    if (timeRange === "month") return orderDate >= monthAgo;
    return true;
  });

  const activeOrders = filteredOrders.filter((o) => o.status !== "cancelled");
  const totalOrderValue = activeOrders.reduce((sum, o) => sum + o.amount_total, 0);
  const avgOrderValue = activeOrders.length > 0 ? totalOrderValue / activeOrders.length : 0;
  const pendingCount = orders.filter((o) => o.status === "submitted").length;

  // 本週 vs 上週 收入對比
  const thisWeekRevenue = orders.filter((o) => {
    const d = (o.created_at || "").split("T")[0].split(" ")[0];
    return d >= weekAgo && o.status !== "cancelled";
  }).reduce((sum, o) => sum + o.amount_total, 0);
  const lastWeekRevenue = orders.filter((o) => {
    const d = (o.created_at || "").split("T")[0].split(" ")[0];
    return d >= lastWeekStart && d < weekAgo && o.status !== "cancelled";
  }).reduce((sum, o) => sum + o.amount_total, 0);
  const weekChangePercent = lastWeekRevenue > 0
    ? ((thisWeekRevenue - lastWeekRevenue) / lastWeekRevenue * 100)
    : null;

  const chartData = inventorySummary.slice(0, 8).map((s) => ({
    name: s.material_name.length > 6 ? s.material_name.slice(0, 6) + "..." : s.material_name,
    qty: s.available_qty,
  }));

  const inventoryChartConfig = {
    qty: {
      label: "库存量",
      color: "hsl(var(--chart-1))",
    },
  } satisfies ChartConfig;

  const statusChartConfig = {
    value: {
      label: "订单数",
    },
  } satisfies ChartConfig;

  const statusData = [
    { name: "待提交", value: filteredOrders.filter((o) => o.status === "pending").length },
    { name: "已提交", value: filteredOrders.filter((o) => o.status === "submitted").length },
    { name: "已完成", value: filteredOrders.filter((o) => o.status === "ready").length },
    { name: "已取消", value: filteredOrders.filter((o) => o.status === "cancelled").length },
  ].filter((s) => s.value > 0);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">仪表板</h2>
          <p className="text-sm text-muted-foreground">系统概览与关键指标</p>
        </div>
        <div className="flex items-center gap-1 rounded-lg border p-1">
          <Button variant={timeRange === "today" ? "default" : "ghost"} size="sm" className="h-7 text-xs" onClick={() => setTimeRange("today")}>
            <Calendar className="mr-1 h-3 w-3" />今日
          </Button>
          <Button variant={timeRange === "week" ? "default" : "ghost"} size="sm" className="h-7 text-xs" onClick={() => setTimeRange("week")}>本周</Button>
          <Button variant={timeRange === "month" ? "default" : "ghost"} size="sm" className="h-7 text-xs" onClick={() => setTimeRange("month")}>本月</Button>
          <Button variant={timeRange === "all" ? "default" : "ghost"} size="sm" className="h-7 text-xs" onClick={() => setTimeRange("all")}>全部</Button>
        </div>
      </div>

      <div className="grid gap-4 md:grid-cols-3 lg:grid-cols-6">
        <Card className="col-span-2 transition-all duration-200 hover:shadow-lg hover:-translate-y-0.5">
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">本期销售额</CardTitle>
            <TrendingUp className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            {loading ? <Skeleton className="h-8 w-24 mb-1" /> : <div className="text-2xl font-bold text-emerald-600">¥{totalOrderValue.toFixed(2)}</div>}
            <p className="text-xs text-muted-foreground">
              均值 ¥{loading ? "-" : avgOrderValue.toFixed(2)} · {activeOrders.length} 笔
              {timeRange === "week" && weekChangePercent !== null && (
                <span className={`ml-2 font-medium ${weekChangePercent >= 0 ? "text-emerald-500" : "text-destructive"}`}>
                  {weekChangePercent >= 0 ? "▲" : "▼"}{Math.abs(weekChangePercent).toFixed(1)}% vs 上周
                </span>
              )}
            </p>
          </CardContent>
        </Card>

        <Card className="transition-all duration-200 hover:shadow-lg hover:-translate-y-0.5">
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">待出餐</CardTitle>
            <Clock className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            {loading ? <Skeleton className="h-8 w-16 mb-1" /> : <div className={`text-2xl font-bold ${pendingCount > 0 ? "text-amber-500" : ""}`}>{pendingCount}</div>}
            <p className="text-xs text-muted-foreground">已提交待完成</p>
          </CardContent>
        </Card>

        <Card className="transition-all duration-200 hover:shadow-lg hover:-translate-y-0.5">
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">库存预警</CardTitle>
            <AlertTriangle className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            {loading ? <Skeleton className="h-8 w-16 mb-1" /> : <div className={`text-2xl font-bold ${lowStockItems.length > 0 ? "text-destructive" : ""}`}>{lowStockItems.length}</div>}
            <p className="text-xs text-muted-foreground">低于 {LOW_STOCK_THRESHOLD} 的材料</p>
          </CardContent>
        </Card>

        <Card className="transition-all duration-200 hover:shadow-lg hover:-translate-y-0.5">
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">原材料</CardTitle>
            <Package className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            {loading ? <Skeleton className="h-8 w-16 mb-1" /> : <div className="text-2xl font-bold">{materialsCount}</div>}
            <p className="text-xs text-muted-foreground">活跃材料</p>
          </CardContent>
        </Card>

        <Card className="transition-all duration-200 hover:shadow-lg hover:-translate-y-0.5">
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">配方</CardTitle>
            <ChefHat className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            {loading ? <Skeleton className="h-8 w-16 mb-1" /> : <div className="text-2xl font-bold">{recipesCount}</div>}
            <p className="text-xs text-muted-foreground">{batchesCount} 批次在库</p>
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2"><BarChart3 className="h-4 w-4" />库存分布</CardTitle>
            <CardDescription>Top 8 材料可用库存</CardDescription>
          </CardHeader>
          <CardContent>
            {loading ? (
              <div className="space-y-3">
                <Skeleton className="h-[250px] w-full" />
              </div>
            ) : chartData.length === 0 ? (
              <EmptyState icon={BarChart3} title="暂无数据" description="库存数据将在此显示" />
            ) : (
              <ChartContainer config={inventoryChartConfig} className="h-[250px] w-full">
                <AreaChart data={chartData}>
                  <defs>
                    <linearGradient id="fillQty" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="5%" stopColor="var(--color-qty)" stopOpacity={0.8} />
                      <stop offset="95%" stopColor="var(--color-qty)" stopOpacity={0.1} />
                    </linearGradient>
                  </defs>
                  <CartesianGrid vertical={false} className="stroke-muted" />
                  <XAxis
                    dataKey="name"
                    tickLine={false}
                    axisLine={false}
                    tickMargin={8}
                    className="text-xs"
                    tick={{ fontSize: 11 }}
                  />
                  <YAxis
                    tickLine={false}
                    axisLine={false}
                    tickMargin={8}
                    className="text-xs"
                    tick={{ fontSize: 11 }}
                  />
                  <ChartTooltip
                    indicator="dot"
                    labelClassName="font-medium"
                  />
                  <Area
                    dataKey="qty"
                    type="natural"
                    fill="url(#fillQty)"
                    stroke="var(--color-qty)"
                    strokeWidth={2}
                    animationDuration={600}
                    animationBegin={200}
                  />
                </AreaChart>
              </ChartContainer>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2"><TrendingUp className="h-4 w-4" />订单状态分布</CardTitle>
            <CardDescription>按状态统计订单数量</CardDescription>
          </CardHeader>
          <CardContent>
            {loading ? (
              <div className="space-y-3">
                <Skeleton className="h-[200px] w-full" />
              </div>
            ) : statusData.length === 0 ? (
              <EmptyState icon={ShoppingCart} title="暂无数据" description="订单状态分布将在此显示" />
            ) : (
              <div className="flex items-center gap-6">
                <ChartContainer config={statusChartConfig} className="h-[200px] w-[50%]">
                  <PieChart>
                    <Pie
                      data={statusData}
                      cx="50%"
                      cy="50%"
                      innerRadius={50}
                      outerRadius={80}
                      paddingAngle={4}
                      dataKey="value"
                      label={({ name, percent }) => `${name} ${((percent || 0) * 100).toFixed(0)}%`}
                    >
                      {statusData.map((_, index) => (
                        <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                      ))}
                    </Pie>
                    <ChartTooltip
                      indicator="dot"
                      formatter={(value: unknown) => [`${value}`, "订单数"]}
                    />
                  </PieChart>
                </ChartContainer>
                <div className="space-y-2 flex-1">
                  {statusData.map((s, i) => (
                    <div key={s.name} className="flex items-center justify-between text-sm">
                      <div className="flex items-center gap-2">
                        <div className="h-3 w-3 rounded-full" style={{ backgroundColor: COLORS[i % COLORS.length] }} />
                        <span>{s.name}</span>
                      </div>
                      <span className="font-medium">{s.value}</span>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-7">
        <Card className="col-span-4">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <ShoppingCart className="h-4 w-4" />
              最近订单
            </CardTitle>
          </CardHeader>
          <CardContent>
            {loading ? (
              <div className="space-y-3">
                <Skeleton className="h-4 w-full" />
                <Skeleton className="h-4 w-[90%]" />
                <Skeleton className="h-4 w-[80%]" />
              </div>
            ) : filteredOrders.length === 0 ? (
              <EmptyState icon={ShoppingCart} title="暂无订单" description="创建订单后将在此显示" />
            ) : (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>订单号</TableHead>
                    <TableHead>状态</TableHead>
                    <TableHead className="text-right">总额</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {filteredOrders.slice(0, 5).map((order, i) => (
                    <TableRow key={order.id} className="animate-stagger" style={{ animationDelay: `${i * 50}ms` }}>
                      <TableCell className="font-medium">{order.order_no}</TableCell>
                      <TableCell>
                        <Badge variant={order.status === "ready" || order.status === "cancelled" ? "default" : "secondary"}>
                          {order.status === "pending" ? "待提交" : order.status === "submitted" ? "已提交" : order.status === "ready" ? "已完成" : order.status === "cancelled" ? "已取消" : order.status}
                        </Badge>
                      </TableCell>
                      <TableCell className="text-right">
                        ¥{order.amount_total.toFixed(2)}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            )}
          </CardContent>
        </Card>

        <Card className="col-span-3">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <AlertTriangle className="h-4 w-4" />
              库存预警
            </CardTitle>
            <CardDescription>可用库存低于 {LOW_STOCK_THRESHOLD} 的材料</CardDescription>
          </CardHeader>
          <CardContent>
            {loading ? (
              <div className="space-y-3">
                <Skeleton className="h-4 w-full" />
                <Skeleton className="h-4 w-[85%]" />
                <Skeleton className="h-4 w-[70%]" />
              </div>
            ) : lowStockItems.length === 0 ? (
              <EmptyState icon={AlertTriangle} title="无库存预警" description="所有库存充足" />
            ) : (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>材料</TableHead>
                    <TableHead className="text-right">可用量</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {lowStockItems.map((summary) => (
                    <TableRow key={summary.material_id}>
                      <TableCell className="font-medium">{summary.material_name}</TableCell>
                      <TableCell className="text-right text-destructive font-medium">
                        {summary.available_qty.toFixed(2)}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            )}
          </CardContent>
        </Card>
      </div>

      {todayTopItems.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2"><Trophy className="h-4 w-4 text-amber-400" />今日热销 Top {todayTopItems.length}</CardTitle>
            <CardDescription>今日销售数量排行</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-5 gap-3">
              {todayTopItems.map(([name, , qty], idx) => (
                <div key={name} className="flex flex-col items-center gap-1 rounded-lg border p-3 text-center">
                  <span className={`text-lg font-bold ${idx === 0 ? "text-amber-400" : idx === 1 ? "text-slate-400" : idx === 2 ? "text-orange-400" : "text-muted-foreground"}`}>
                    #{idx + 1}
                  </span>
                  <span className="text-xs font-medium line-clamp-2">{name}</span>
                  <span className="text-sm text-muted-foreground">{qty} 份</span>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {/* 行销效果日报 */}
      {marketingStats && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Zap className="h-4 w-4 text-amber-500" />今日行销效果
            </CardTitle>
            <CardDescription>兑奖次数与折价券核销</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-3 gap-4">
              <div className="rounded-lg border p-3 text-center">
                <div className="text-2xl font-bold text-amber-600">{marketingStats.redemptions_today}</div>
                <div className="text-xs text-muted-foreground mt-1">今日兑奖次数</div>
              </div>
              <div className="rounded-lg border p-3 text-center">
                <div className="text-2xl font-bold text-blue-600">{marketingStats.coupons_issued_today}</div>
                <div className="text-xs text-muted-foreground mt-1">折价券发放</div>
              </div>
              <div className="rounded-lg border p-3 text-center">
                <div className="text-2xl font-bold text-green-600">{marketingStats.coupons_redeemed_today}</div>
                <div className="text-xs text-muted-foreground mt-1">折价券核销</div>
              </div>
            </div>
            {marketingStats.coupons_issued_today > 0 && (
              <div className="mt-3 text-xs text-muted-foreground text-center">
                核销率 {Math.round(marketingStats.coupons_redeemed_today / marketingStats.coupons_issued_today * 100)}%
                　·　前往<a href="#/marketing" className="text-primary underline ml-1">行销中心</a>查看兑奖记录
              </div>
            )}
          </CardContent>
        </Card>
      )}
    </div>
  );
}
