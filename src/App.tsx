import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { appLogger } from "@/lib/logger";
import { Routes, Route, useNavigate, useLocation } from "react-router-dom";
import { SidebarProvider, SidebarInset } from "@/components/ui/sidebar";
import { TooltipProvider } from "@/components/ui/tooltip";
import { AppSidebar } from "@/components/app-sidebar";
import { AppHeader } from "@/components/app-header";
import { DashboardPage } from "@/pages/dashboard-page";
import { MaterialsPage } from "@/pages/materials-page";
import { RecipesPage } from "@/pages/recipes-page";
import { InventoryPage } from "@/pages/inventory-page";
import { MenuPage } from "@/pages/menu-page";
import { POSPage } from "@/pages/pos-page";
import { SuppliersPage } from "@/pages/suppliers-page";
import { KDSPage } from "@/pages/kds-page";
import { AttributesPage } from "@/pages/attributes-page";
import { SettingsPage } from "@/pages/settings-page";
import { MaterialStatesPage } from "@/pages/material-states-page";
import { PurchaseOrdersPage } from "@/pages/purchase-orders-page";
import { ProductionOrdersPage } from "@/pages/production-orders-page";
import { StocktakesPage } from "@/pages/stocktakes-page";
import { ReportsPage } from "@/pages/reports-page";
import { OrdersPage } from "@/pages/orders-page";
import { PrintPage } from "@/pages/print-page";
import { PrintSettingsPage } from "@/pages/print-settings-page";
import { PrintTemplatesPage } from "@/pages/print-templates-page";
import { ExpensesPage } from "@/pages/expenses-page";
import { Toaster } from "@/components/ui/toaster";
import { ConfirmDialog } from "@/components/ui/confirm-dialog";
import { toast } from "sonner";
import { useAppData } from "@/hooks/useAppData";
import { useAppActions } from "@/hooks/useAppActions";
import { Skeleton } from "@/components/ui/skeleton";
import type { OrderWithItems, OrderItemModifier } from "./types";
import { useAutoUpdate } from "@/hooks/useAutoUpdate";
import { UpdateBanner } from "@/components/UpdateBanner";

// ==================== App ====================

function App() {
  const navigate = useNavigate();
  const location = useLocation();
  const activeTab = location.pathname.slice(1) || "dashboard";
  const [searchQuery, setSearchQuery] = useState("");
  const [confirmAction, setConfirmAction] = useState<{ title: string; description: string; onConfirm: () => void } | null>(null);
  const [appStartTime] = useState(Date.now());
  const [unseenErrorCount, setUnseenErrorCount] = useState(0);
  const { updateInfo, dismiss: dismissUpdate, skip: skipUpdate } = useAutoUpdate();

  // Increment badge whenever the logger writes a new entry
  useEffect(() => {
    const handler = () => setUnseenErrorCount((n) => n + 1);
    window.addEventListener("cuckoo:logged", handler);
    return () => window.removeEventListener("cuckoo:logged", handler);
  }, []);

  // Clear badge when user opens settings
  useEffect(() => {
    if (activeTab === "settings") setUnseenErrorCount(0);
  }, [activeTab]);

  const {
    loading, connected,
    units,
    categories,
    tags,
    materials,
    recipes, recipeTypes, selectedRecipe, setSelectedRecipe,
    recipeCost, setRecipeCost,
    menuCategories,
    menuItems,
    orders, ordersHasMore, setOrders, setOrdersHasMore,
    selectedOrder, setSelectedOrder,
    stations,
    kdsTickets, setKdsTickets,
    inventoryBatches,
    inventorySummary,
    inventoryTxns,
    attributeTemplates,
    suppliers,
    materialStates,
    purchaseOrders, selectedPurchaseOrder, setSelectedPurchaseOrder,
    productionOrders, selectedProductionOrder, setSelectedProductionOrder,
    stocktakes, selectedStocktake, setSelectedStocktake,
    expenses, setExpenses,
    supplierProducts, setSupplierProducts,
    loadData,
  } = useAppData();

  const actions = useAppActions({
    loadData,
    categories,
    menuCategories,
    orders,
    materials,
    suppliers,
    units,
    inventoryBatches,
    menuItems,
    setOrders,
    setOrdersHasMore,
    setSelectedRecipe,
    setRecipeCost,
    setSelectedOrder,
    setKdsTickets,
    setSelectedPurchaseOrder,
    setSelectedProductionOrder,
    setSelectedStocktake,
    setExpenses,
    setSupplierProducts,
  });

  useEffect(() => { loadData(); }, []);

  const {
    handleCreateMaterial,
    handleUpdateMaterial,
    handleDeleteMaterial,
    handleRemoveMaterialTag,
    handleCreateCategory,
    handleDeleteCategory,
    handleCreateTag,
    handleDeleteTag,
    handleCreateRecipe,
    handleViewRecipe,
    handleDeleteRecipe,
    handleUpdateRecipe,
    handleCreateRecipeType,
    handleUpdateRecipeType,
    handleDeleteRecipeType,
    handleSeedSampleRecipes,
    handleCreatePendingRecipeForMenu,
    handleBindMenuItemToRecipe,
    handleAddRecipeItem,
    handleDeleteRecipeItem,
    handleUpdateRecipeItem,
    handleRecalculateCost,
    handleCreateMenuCategory,
    handleUpdateMenuCategory,
    handleDeleteMenuCategory,
    handleToggleMenuItem,
    handleBatchToggleMenuItem,
    handleUpdateMenuItem,
    handleDeleteMenuItem,
    handleCreateOrder,
    handlePOSOrder,
    handlePOSAndSubmit,
    handleSubmitOrder,
    handleCancelOrder,
    handleBatchCancelOrder,
    handleViewOrder,
    handleLoadMoreOrders,
    handleGetSpecs,
    handleCreateSpec,
    handleUpdateSpec,
    handleDeleteSpec,
    handleCreateSupplier,
    handleUpdateSupplier,
    handleDeleteSupplier,
    handleCreateMaterialState,
    handleUpdateMaterialState,
    handleDeleteMaterialState,
    handleCreatePurchaseOrder,
    handleAddPurchaseOrderItem,
    handleViewPurchaseOrder,
    handleDeletePurchaseOrder,
    handleReceivePurchaseOrder,
    handleReceivePurchaseOrderItems,
    handleUpdateOrderPayment,
    handleCreateProductionOrder,
    handleStartProductionOrder,
    handleCompleteProductionOrder,
    handleViewProductionOrder,
    handleDeleteProductionOrder,
    handleCreateStocktake,
    handleUpdateStocktakeItem,
    handleCompleteStocktake,
    handleViewStocktake,
    handleDeleteStocktake,
    handleCreateExpense,
    handleUpdateExpense,
    handleDeleteExpense,
    handleCreateSupplierProduct,
    handleUpdateSupplierProduct,
    handleDeleteSupplierProduct,
    handleCreateBatch,
    handleAdjustInventory,
    handleRecordWastage,
    handleDeleteBatch,
    handleLoadKDS,
    handleFinishTicket,
    handleReprintTicket,
    handleAddModifier,
    handleDeleteModifier,
    handleLoadModifiers,
    handleReportTelemetry,
  } = actions;

  const handleCreateMenuItemFull = actions.handleCreateMenuItem;

  // Keep a ref to orders so the telemetry effect can read the latest value
  // without re-running every time orders change.
  const ordersRef = useRef(orders);
  useEffect(() => { ordersRef.current = orders; }, [orders]);

  const handleReportTelemetryRef = useRef(handleReportTelemetry);
  useEffect(() => { handleReportTelemetryRef.current = handleReportTelemetry; }, [handleReportTelemetry]);

  useEffect(() => {
    const startTelemetry = async (eventType = "heartbeat", metadata: any = null) => {
      const today = new Date().toDateString();
      const todayOrders = ordersRef.current.filter(o => new Date(o.created_at).toDateString() === today);
      const todaySales = todayOrders.reduce((sum, o) => sum + (o.status === "submitted" ? o.amount_total : 0), 0);
      const uptimeHours = (Date.now() - appStartTime) / (1000 * 60 * 60);
      // Silently skip — errors are suppressed so network failures don't pollute the error log
      try {
        await handleReportTelemetryRef.current({
          client_id: "cuckoo_local",
          version: "1.2.2",
          event_type: eventType,
          uptime_hours: uptimeHours,
          today_sales: todaySales,
          today_orders: todayOrders.length,
          metadata: metadata,
        });
      } catch { /* telemetry is best-effort; never surface to user */ }
    };

    // 全局錯誤捕捉
    const handleError = (event: ErrorEvent) => {
      appLogger.logRuntimeError(event);
      startTelemetry("error", { message: event.message, stack: event.error?.stack });
    };
    const handleRejection = (event: PromiseRejectionEvent) => {
      appLogger.logRuntimeError(event);
      startTelemetry("error", { message: String(event.reason), type: "unhandled_rejection" });
    };

    window.addEventListener("error", handleError);
    window.addEventListener("unhandledrejection", handleRejection);

    startTelemetry();
    const interval = setInterval(() => startTelemetry(), 60 * 60 * 1000);

    return () => {
      window.removeEventListener("error", handleError);
      window.removeEventListener("unhandledrejection", handleRejection);
      clearInterval(interval);
    };
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [appStartTime]); // intentionally omit orders — read via ref to avoid re-firing on every order change

  if (loading) {
    return (
      <div className="flex h-screen w-full bg-background">
        <div className="w-[280px] border-r p-4 space-y-3">
          <Skeleton className="h-8 w-24" />
          <Skeleton className="h-4 w-32" />
          <Skeleton className="h-4 w-28" />
          <Skeleton className="h-4 w-20" />
          <Skeleton className="h-4 w-24" />
          <Skeleton className="h-4 w-16" />
        </div>
        <div className="flex-1 p-6 space-y-4">
          <Skeleton className="h-12 w-48" />
          <Skeleton className="h-64 w-full" />
          <div className="grid grid-cols-4 gap-4">
            <Skeleton className="h-24 w-full" />
            <Skeleton className="h-24 w-full" />
            <Skeleton className="h-24 w-full" />
            <Skeleton className="h-24 w-full" />
          </div>
        </div>
      </div>
    );
  }

  return (
    <TooltipProvider>
      <SidebarProvider>
        <div className="flex h-screen w-full bg-background">
          <AppSidebar activeTab={activeTab} onTabChange={(tab) => navigate("/" + tab)} connected={connected} errorCount={unseenErrorCount} />
          <SidebarInset className="flex flex-col">
            <AppHeader searchQuery={searchQuery} onSearchChange={setSearchQuery} onRefresh={loadData} refreshing={loading} />
            <main className="flex-1 overflow-auto p-6">
              <Routes>
                <Route path="/dashboard" element={<DashboardPage materialsCount={materials.length} recipesCount={recipes.length} ordersCount={orders.length} batchesCount={inventoryBatches.length} orders={orders} inventorySummary={inventorySummary} loading={loading} />} />
                <Route path="/materials" element={<MaterialsPage materials={materials} recipes={recipes} categories={categories} tags={tags} units={units} onCreateMaterial={handleCreateMaterial} onUpdateMaterial={handleUpdateMaterial} onDeleteMaterial={handleDeleteMaterial} onRemoveMaterialTag={handleRemoveMaterialTag} onCreateCategory={handleCreateCategory} onDeleteCategory={handleDeleteCategory} onCreateTag={handleCreateTag} onDeleteTag={handleDeleteTag} searchQuery={searchQuery} />} />
                <Route path="/recipes" element={<RecipesPage recipes={recipes} recipeTypes={recipeTypes} selectedRecipe={selectedRecipe} recipeCost={recipeCost} materials={materials} menuItems={menuItems} units={units} onCreateRecipe={handleCreateRecipe} onViewRecipe={handleViewRecipe} onDeleteRecipe={handleDeleteRecipe} onUpdateRecipe={handleUpdateRecipe} onCreateRecipeType={handleCreateRecipeType} onUpdateRecipeType={handleUpdateRecipeType} onDeleteRecipeType={handleDeleteRecipeType} onSeedSampleRecipes={handleSeedSampleRecipes} onCreatePendingRecipeForMenu={handleCreatePendingRecipeForMenu} onBindMenuItemToRecipe={handleBindMenuItemToRecipe} onAddRecipeItem={handleAddRecipeItem} onDeleteRecipeItem={handleDeleteRecipeItem} onUpdateRecipeItem={handleUpdateRecipeItem} onRecalculateCost={handleRecalculateCost} searchQuery={searchQuery} />} />
                <Route path="/inventory" element={<InventoryPage inventorySummary={inventorySummary} inventoryBatches={inventoryBatches} inventoryTxns={inventoryTxns} materials={materials} recipes={recipes} suppliers={suppliers} onCreateBatch={handleCreateBatch} onAdjustInventory={handleAdjustInventory} onRecordWastage={handleRecordWastage} onDeleteBatch={handleDeleteBatch} onUpdateMaterial={handleUpdateMaterial} searchQuery={searchQuery} />} />
                <Route path="/menu" element={<MenuPage menuCategories={menuCategories} menuItems={menuItems} recipes={recipes} onCreateMenuCategory={handleCreateMenuCategory} onCreateMenuItem={handleCreateMenuItemFull} onCreatePendingRecipeForMenu={handleCreatePendingRecipeForMenu} onToggleAvailability={handleToggleMenuItem} onBatchToggleAvailability={handleBatchToggleMenuItem} onUpdateMenuItem={handleUpdateMenuItem} onDeleteMenuItem={handleDeleteMenuItem} onUpdateMenuCategory={handleUpdateMenuCategory} onDeleteMenuCategory={handleDeleteMenuCategory} onGetSpecs={handleGetSpecs} onCreateSpec={handleCreateSpec} onUpdateSpec={handleUpdateSpec} onDeleteSpec={handleDeleteSpec} searchQuery={searchQuery} />} />
                <Route path="/pos" element={<POSPage menuCategories={menuCategories} menuItems={menuItems} onCreateOrder={handlePOSOrder} onCreateAndSubmit={handlePOSAndSubmit} onGetSpecs={handleGetSpecs} searchQuery={searchQuery} loading={loading} />} />
                <Route path="/suppliers" element={<SuppliersPage suppliers={suppliers} supplierProducts={supplierProducts} onCreateSupplier={handleCreateSupplier} onUpdateSupplier={handleUpdateSupplier} onDeleteSupplier={handleDeleteSupplier} onCreateSupplierProduct={handleCreateSupplierProduct} onUpdateSupplierProduct={handleUpdateSupplierProduct} onDeleteSupplierProduct={handleDeleteSupplierProduct} searchQuery={searchQuery} />} />
                <Route path="/orders" element={<OrdersPage orders={orders} selectedOrder={selectedOrder} menuItems={Object.fromEntries(menuItems.map((item) => [item.id, item.name]))} materials={materials} onCreateOrder={handleCreateOrder} onSubmitOrder={handleSubmitOrder} onCancelOrder={handleCancelOrder} onBatchCancelOrder={handleBatchCancelOrder} onViewOrder={handleViewOrder} onViewOrderWithModifiers={async (id: number) => { const orderData = await invoke<OrderWithItems>("get_order_with_items", { orderId: id }); const modifiers: Record<number, OrderItemModifier[]> = {}; for (const item of orderData.items) { try { modifiers[item.id] = await handleLoadModifiers(item.id); } catch { modifiers[item.id] = []; } } return { orderData, modifiers }; }} onAddModifier={handleAddModifier} onDeleteModifier={handleDeleteModifier} onLoadModifiers={handleLoadModifiers} onUpdatePayment={handleUpdateOrderPayment} onLoadMore={handleLoadMoreOrders} hasMore={ordersHasMore} searchQuery={searchQuery} />} />
                <Route path="/kds" element={<KDSPage allTickets={kdsTickets} stations={stations} menuItemNames={Object.fromEntries(menuItems.map((m) => [m.id, m.name]))} onStartTicket={async (id) => { try { await invoke("start_ticket", { ticketId: id, operator: null }); toast.success("工单已开始"); loadData(); } catch (e) { toast.error("开始工单失败", { description: String(e) }); } }} onFinishTicket={async (id) => { const ticket = kdsTickets.find((t) => t.id === id); if (ticket) { await handleFinishTicket(ticket); } else { await invoke("finish_ticket", { ticketId: id, operator: null }); toast.success("工单已完成"); loadData(); } }} onReprintTicket={handleReprintTicket} onRefresh={handleLoadKDS} />} />
                <Route path="/attributes" element={<AttributesPage attributeTemplates={attributeTemplates} onRefresh={loadData} />} />
                <Route path="/settings" element={<SettingsPage connected={connected} />} />
                <Route path="/material-states" element={<MaterialStatesPage materialStates={materialStates} materials={materials} units={units} onCreateState={handleCreateMaterialState} onUpdateState={handleUpdateMaterialState} onDeleteState={handleDeleteMaterialState} searchQuery={searchQuery} />} />
                <Route path="/purchase-orders" element={<PurchaseOrdersPage orders={purchaseOrders} materials={materials} units={units} suppliers={suppliers} onCreateOrder={handleCreatePurchaseOrder} onAddItem={handleAddPurchaseOrderItem} onViewOrder={handleViewPurchaseOrder} onDeleteOrder={handleDeletePurchaseOrder} onReceiveOrder={handleReceivePurchaseOrder} onReceiveItems={handleReceivePurchaseOrderItems} selectedOrder={selectedPurchaseOrder} searchQuery={searchQuery} />} />
                <Route path="/production-orders" element={<ProductionOrdersPage orders={productionOrders} recipes={recipes} onCreateOrder={handleCreateProductionOrder} onStartOrder={handleStartProductionOrder} onCompleteOrder={handleCompleteProductionOrder} onViewOrder={handleViewProductionOrder} onDeleteOrder={handleDeleteProductionOrder} selectedOrder={selectedProductionOrder} searchQuery={searchQuery} />} />
                <Route path="/stocktakes" element={<StocktakesPage stocktakes={stocktakes} onCreateStocktake={handleCreateStocktake} onUpdateItem={handleUpdateStocktakeItem} onCompleteStocktake={handleCompleteStocktake} onViewStocktake={handleViewStocktake} onDeleteStocktake={handleDeleteStocktake} selectedStocktake={selectedStocktake} searchQuery={searchQuery} />} />
                <Route path="/reports" element={<ReportsPage />} />
                <Route path="/expenses" element={<ExpensesPage expenses={expenses} onCreateExpense={handleCreateExpense} onUpdateExpense={handleUpdateExpense} onDeleteExpense={handleDeleteExpense} />} />
                <Route path="/print" element={<PrintPage />} />
                <Route path="/print-templates" element={<PrintTemplatesPage />} />
                <Route path="/print-settings" element={<PrintSettingsPage />} />
                <Route path="*" element={<DashboardPage materialsCount={materials.length} recipesCount={recipes.length} ordersCount={orders.length} batchesCount={inventoryBatches.length} orders={orders} inventorySummary={inventorySummary} loading={loading} />} />
              </Routes>
            </main>
          </SidebarInset>
        </div>
      </SidebarProvider>
      <Toaster />
      <ConfirmDialog
        open={!!confirmAction}
        onOpenChange={(open) => { if (!open) setConfirmAction(null); }}
        title={confirmAction?.title || ""}
        description={confirmAction?.description || ""}
        onConfirm={() => confirmAction?.onConfirm()}
      />
      {updateInfo && <UpdateBanner info={updateInfo} onDismiss={dismissUpdate} onSkip={skipUpdate} />}
    </TooltipProvider>
  );
}

export default App;
