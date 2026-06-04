import { useState, useEffect } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Sheet, SheetContent, SheetHeader, SheetTitle } from "@/components/ui/sheet";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { EmptyState } from "@/components/ui/empty-state";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { toast } from "sonner";
import { parseSafeFloat } from "@/lib/utils";
import { Plus, Minus, ShoppingCart, Send, X, MessageSquare, Tag, FileText, Star, Layers } from "lucide-react";
import { call as invoke } from "@/lib/transport";
import type { ComboItem } from "@/types";

function formatPrice(price: number): string {
  return price.toLocaleString("zh-CN", { style: "currency", currency: "CNY" });
}

function formatPriceMini(price: number): string {
  return `¥${price.toFixed(0)}`;
}

interface MenuCategory {
  id: number;
  name: string;
}

interface MenuItem {
  id: number;
  code: string | null;
  name: string;
  sales_price: number;
  is_available: boolean;
  is_favorite: boolean;
  image_path: string | null;
  description: string | null;
  recipe_id: number | null;
  category_id: number | null;
  created_at: string;
}

interface MenuItemSpec {
  id: number;
  menu_item_id: number;
  spec_code: string;
  spec_name: string;
  price_delta: number;
  qty_multiplier: number;
}

interface CartModifier {
  id?: number;
  modifier_type: string;
  material_id?: number;
  material_name?: string;
  qty: number;
  price_delta: number;
}

interface CartItem {
  menu_item: MenuItem;
  spec: MenuItemSpec | null;
  qty: number;
  note: string;
  modifiers: CartModifier[];
}

interface POSPageProps {
  menuCategories: MenuCategory[];
  menuItems: MenuItem[];
  onCreateOrder: (items: CartItem[], dineType: string, tableNo: string | null) => Promise<boolean>;
  onCreateAndSubmit: (items: CartItem[], dineType: string, tableNo: string | null) => Promise<boolean>;
  onGetSpecs: (menuItemId: number) => Promise<MenuItemSpec[]>;
  searchQuery?: string;
  loading?: boolean;
}

export function POSPage({
  menuCategories,
  menuItems,
  onCreateOrder,
  onCreateAndSubmit,
  onGetSpecs,
  searchQuery,
  loading = false,
}: POSPageProps) {
  const [selectedCategory, setSelectedCategory] = useState<number | null>(null);
  const [cart, setCart] = useState<CartItem[]>([]);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [specDialogOpen, setSpecDialogOpen] = useState(false);
  const [noteDialogOpen, setNoteDialogOpen] = useState(false);
  const [modifierDialogOpen, setModifierDialogOpen] = useState(false);
  const [currentItem, setCurrentItem] = useState<MenuItem | null>(null);
  const [currentCartItemIndex, setCurrentCartItemIndex] = useState<number | null>(null);
  const [specs, setSpecs] = useState<MenuItemSpec[]>([]);
  const [selectedSpec, setSelectedSpec] = useState<MenuItemSpec | null>(null);
  const [tempNote, setTempNote] = useState("");
  const [clearCartConfirmOpen, setClearCartConfirmOpen] = useState(false);
  const [dineType, setDineType] = useState("dine_in");
  const [cartSheetOpen, setCartSheetOpen] = useState(false);

  const presetModifiers = [
    { modifier_type: "加料", price_delta: 2, qty: 1 },
    { modifier_type: "加料", price_delta: 5, qty: 1 },
    { modifier_type: "加料", price_delta: 10, qty: 1 },
    { modifier_type: "去料", price_delta: -2, qty: 1 },
    { modifier_type: "少糖", price_delta: 0, qty: 1 },
    { modifier_type: "少冰", price_delta: 0, qty: 1 },
    { modifier_type: "去冰", price_delta: 0, qty: 1 },
  ];
  const [tableNo, setTableNo] = useState("");

  useEffect(() => {
    if (menuItems.length === 0) return;
    try {
      const saved = localStorage.getItem("cuckoo_cart");
      if (saved) {
        const arr = JSON.parse(saved);
        if (Array.isArray(arr) && arr.length > 0) {
          const rehydrated = arr
            .map((item: CartItem) => {
              const fresh = menuItems.find((m) => m.id === item.menu_item.id);
              return fresh ? { ...item, menu_item: fresh } : null;
            })
            .filter((item): item is CartItem => item !== null && item.menu_item.is_available);
          if (rehydrated.length < arr.length) {
            // some items were removed (deleted or unavailable) — silent cleanup
          }
          setCart(rehydrated);
        }
      }
    } catch {
      // ignore
    }
  }, [menuItems]);

  useEffect(() => {
    try {
      localStorage.setItem("cuckoo_cart", JSON.stringify(cart));
    } catch {
      // ignore
    }
  }, [cart]);

  const clearCart = () => {
    setCart([]);
    localStorage.removeItem("cuckoo_cart");
  };

  const [combos, setCombos] = useState<ComboItem[]>([]);
  useEffect(() => {
    invoke<ComboItem[]>("list_combos", {}).then(setCombos).catch(() => {});
  }, []);

  const favoriteItems = menuItems.filter((item) => item.is_favorite);
  const filteredItems = menuItems.filter((item) => {
    if (selectedCategory === -1) return item.is_favorite && (!searchQuery || item.name.toLowerCase().includes(searchQuery.toLowerCase()));
    if (selectedCategory === -2) return false; // combos tab handled separately
    const matchesCategory = !selectedCategory || item.category_id === selectedCategory;
    const matchesSearch = !searchQuery || item.name.toLowerCase().includes(searchQuery.toLowerCase());
    return matchesCategory && matchesSearch;
  });
  const filteredCombos = selectedCategory === -2
    ? combos.filter((c) => c.is_available && (!searchQuery || c.name.toLowerCase().includes(searchQuery.toLowerCase())))
    : [];

  function addComboToCart(combo: ComboItem) {
    const syntheticItem: MenuItem = {
      id: combo.menu_item_id,
      code: null,
      name: combo.name,
      sales_price: combo.sales_price,
      is_available: combo.is_available,
      is_favorite: false,
      image_path: null,
      description: combo.components.map(c => `${c.component_name}×${c.qty}`).join(" + "),
      recipe_id: null,
      category_id: null,
      created_at: "",
    };
    addToCart(syntheticItem, null);
  }

  const cartTotal = cart.reduce((sum, item) => {
    const itemTotal = (item.menu_item.sales_price + (item.spec?.price_delta || 0)) * item.qty;
    const modifierTotal = item.modifiers.reduce((m, mod) => m + mod.price_delta * mod.qty, 0);
    return sum + itemTotal + modifierTotal;
  }, 0);

  const cartCount = cart.reduce((sum, item) => sum + item.qty, 0);

  function addToCart(item: MenuItem, spec: MenuItemSpec | null = null, qty: number = 1) {
    const existingIndex = cart.findIndex(
      (c) => c.menu_item.id === item.id && c.spec?.id === spec?.id
    );

    if (existingIndex >= 0) {
      const newCart = [...cart];
      newCart[existingIndex].qty += qty;
      setCart(newCart);
    } else {
      setCart([...cart, { menu_item: item, spec, qty, note: "", modifiers: [] }]);
    }
  }

  function updateQty(index: number, delta: number) {
    const newCart = [...cart];
    newCart[index].qty += delta;
    if (newCart[index].qty <= 0) {
      newCart.splice(index, 1);
    }
    setCart(newCart);
  }

  function removeFromCart(index: number) {
    setCart(cart.filter((_, i) => i !== index));
  }

  async function openSpecDialog(item: MenuItem) {
    setCurrentItem(item);
    setSelectedSpec(null);
    try {
      const itemSpecs = await onGetSpecs(item.id);
      setSpecs(itemSpecs);
      if (itemSpecs.length > 0) {
        setSpecDialogOpen(true);
      } else {
        addToCart(item);
      }
    } catch {
      addToCart(item);
    }
  }

  function confirmSpec() {
    if (currentItem) {
      addToCart(currentItem, selectedSpec);
    }
    setSpecDialogOpen(false);
  }

  function openNoteDialog(index: number) {
    setCurrentCartItemIndex(index);
    setTempNote(cart[index].note);
    setNoteDialogOpen(true);
  }

  function confirmNote() {
    if (currentCartItemIndex !== null) {
      const newCart = [...cart];
      newCart[currentCartItemIndex].note = tempNote;
      setCart(newCart);
    }
    setNoteDialogOpen(false);
  }

  function openModifierDialog(index: number) {
    setCurrentCartItemIndex(index);
    setModifierDialogOpen(true);
  }

  function addModifier(modifier: CartModifier) {
    if (currentCartItemIndex !== null) {
      const newCart = [...cart];
      newCart[currentCartItemIndex].modifiers.push(modifier);
      setCart(newCart);
    }
  }

  function removeModifier(itemIndex: number, modIndex: number) {
    const newCart = [...cart];
    newCart[itemIndex].modifiers.splice(modIndex, 1);
    setCart(newCart);
  }

  function getItemPrice(item: CartItem) {
    const basePrice = item.menu_item.sales_price + (item.spec?.price_delta || 0);
    const modifierPrice = item.modifiers.reduce((sum, m) => sum + m.price_delta * m.qty, 0);
    return basePrice + modifierPrice;
  }

  return (
    <div className="flex h-[calc(100vh-8rem)] gap-4">
      <div className="flex flex-1 flex-col gap-4">
        <Card className="flex-shrink-0">
          <CardHeader className="py-3 px-4">
            <CardTitle className="text-lg">菜单分类</CardTitle>
          </CardHeader>
          <CardContent className="px-4 pb-3">
            <div className="flex gap-2 overflow-x-auto pb-1 scrollbar-hide">
              <Button
                variant={!selectedCategory ? "default" : "outline"}
                size="sm"
                onClick={() => setSelectedCategory(null)}
              >
                全部
              </Button>
              {favoriteItems.length > 0 && (
                <Button
                  variant={selectedCategory === -1 ? "default" : "outline"}
                  size="sm"
                  className="gap-1"
                  onClick={() => setSelectedCategory(-1)}
                >
                  <Star className="h-3 w-3" />常用
                </Button>
              )}
              {combos.length > 0 && (
                <Button
                  variant={selectedCategory === -2 ? "default" : "outline"}
                  size="sm"
                  className="gap-1"
                  onClick={() => setSelectedCategory(-2)}
                >
                  <Layers className="h-3 w-3" />套餐
                </Button>
              )}
              {menuCategories.map((cat) => (
                <Button
                  key={cat.id}
                  variant={selectedCategory === cat.id ? "default" : "outline"}
                  size="sm"
                  onClick={() => setSelectedCategory(cat.id)}
                >
                  {cat.name}
                </Button>
              ))}
            </div>
          </CardContent>
        </Card>

        <Card className="flex-1 min-h-0 flex flex-col">
          <CardHeader className="py-3 px-4 flex-shrink-0">
            <CardTitle className="text-lg">
              商品列表
              <Badge variant="secondary" className="ml-2">
                {filteredItems.length}
              </Badge>
            </CardTitle>
          </CardHeader>
          <CardContent className="px-4 pb-4 flex-1 min-h-0 overflow-hidden">
            <ScrollArea className="h-full">
              {loading ? (
                <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-3">
                  {Array.from({ length: 8 }).map((_, i) => (
                    <div key={i} className="flex flex-col items-start gap-2 rounded-lg border bg-card p-4">
                      <Skeleton className="h-4 w-full" />
                      <Skeleton className="h-4 w-[60%]" />
                      <Skeleton className="h-6 w-16 mt-2" />
                    </div>
                  ))}
                </div>
              ) : (
                <>
                  {selectedCategory === -2 ? (
                    <>
                      <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-3">
                        {filteredCombos.map((combo, i) => (
                          <Button
                            key={combo.menu_item_id}
                            variant="ghost"
                            className="group relative flex flex-col items-start gap-2 rounded-lg border bg-card p-4 text-left transition-all hover:border-primary hover:shadow-md active:scale-95 animate-stagger h-auto"
                            style={{ animationDelay: `${i * 30}ms` }}
                            onClick={() => addComboToCart(combo)}
                          >
                            <div className="flex w-full items-start justify-between gap-1">
                              <span className="font-medium text-sm line-clamp-2">{combo.name}</span>
                              <Layers className="h-3.5 w-3.5 shrink-0 text-muted-foreground mt-0.5" />
                            </div>
                            <span className="text-[10px] text-muted-foreground line-clamp-1">
                              {combo.components.map(c => `${c.component_name}×${c.qty}`).join(" + ")}
                            </span>
                            <span className="text-sm font-bold text-primary">{formatPriceMini(combo.sales_price)}</span>
                          </Button>
                        ))}
                      </div>
                      {filteredCombos.length === 0 && (
                        <EmptyState icon={Layers} title="暂无套餐" description="在菜单管理中创建套餐" />
                      )}
                    </>
                  ) : (
                    <>
                      <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-3">
                        {filteredItems.map((item, i) => (
                          <Button
                            key={item.id}
                            variant="ghost"
                            className="group relative flex flex-col items-start gap-2 rounded-lg border bg-card p-4 text-left transition-all hover:border-primary hover:shadow-md active:scale-95 disabled:opacity-50 disabled:pointer-events-none animate-stagger h-auto"
                            style={{ animationDelay: `${i * 30}ms` }}
                            onClick={() => item.is_available && openSpecDialog(item)}
                            disabled={!item.is_available}
                          >
                            <div className="flex w-full items-start justify-between">
                              <span className="font-medium text-sm line-clamp-2">{item.name}</span>
                              {!item.is_available && (
                                <Badge variant="destructive" className="text-xs">停售</Badge>
                              )}
                            </div>
                            <span className="text-sm font-bold text-primary">
                              {formatPriceMini(item.sales_price)}
                            </span>
                          </Button>
                        ))}
                      </div>
                      {filteredItems.length === 0 && (
                        <EmptyState icon={FileText} title="暂无商品" description="搜索或选择分类查找商品" />
                      )}
                    </>
                  )}
                </>
              )}
            </ScrollArea>
          </CardContent>
        </Card>
      </div>

      {/* Desktop cart sidebar — hidden on small screens */}
      <Card className="hidden md:flex md:w-80 lg:w-96 flex-col min-h-0">
        <CardHeader className="py-3 px-4 flex-shrink-0">
          <CardTitle className="flex items-center gap-2">
            <ShoppingCart className="h-5 w-5" />
            当前订单
            {cartCount > 0 && (
              <Badge variant="default" className="ml-auto">
                {cartCount} 件
              </Badge>
            )}
          </CardTitle>
        </CardHeader>
        <Separator />
        <ScrollArea className="flex-1 px-4 py-3">
          {cart.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-12 text-muted-foreground gap-2">
              <ShoppingCart className="h-12 w-12 opacity-20" />
              <span className="text-sm">请选择商品</span>
            </div>
          ) : (
            <div className="space-y-3">
              {cart.map((item, index) => (
                <div key={index} className="rounded-lg border bg-muted/30 p-3 space-y-2">
                  <div className="flex items-start justify-between gap-2">
                    <div className="flex-1 min-w-0">
                      <div className="font-medium text-sm truncate">{item.menu_item.name}</div>
                      {item.spec && (
                        <div className="flex items-center gap-1 mt-1">
                          <Tag className="h-3 w-3 text-muted-foreground" />
                          <span className="text-xs text-muted-foreground">{item.spec.spec_name}</span>
                        </div>
                      )}
                      {item.note && (
                        <div className="flex items-center gap-1 mt-1">
                          <MessageSquare className="h-3 w-3 text-muted-foreground" />
                          <span className="text-xs text-muted-foreground truncate">{item.note}</span>
                        </div>
                      )}
                      {item.modifiers.length > 0 && (
                        <div className="flex flex-wrap gap-1 mt-1">
                          {item.modifiers.map((mod, mi) => (
                            <Badge key={mi} variant="outline" className="text-[10px] py-0 h-5">
                              {mod.modifier_type}
                              {mod.price_delta !== 0 && ` (${mod.price_delta > 0 ? "+" : ""}${mod.price_delta})`}
                            </Badge>
                          ))}
                        </div>
                      )}
                    </div>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-10 w-10 flex-shrink-0"
                      onClick={() => removeFromCart(index)}
                    >
                      <X className="h-3 w-3" />
                    </Button>
                  </div>
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-1">
                      <Button
                        variant="outline"
                        size="icon"
                        className="h-10 w-10"
                        onClick={() => updateQty(index, -1)}
                      >
                        <Minus className="h-3 w-3" />
                      </Button>
                      <span className="w-8 text-center text-sm font-medium">{item.qty}</span>
                      <Button
                        variant="outline"
                        size="icon"
                        className="h-10 w-10"
                        onClick={() => updateQty(index, 1)}
                      >
                        <Plus className="h-3 w-3" />
                      </Button>
                    </div>
                    <div className="flex items-center gap-2">
                      <span className="font-medium text-sm">
                        {formatPrice(getItemPrice(item) * item.qty)}
                      </span>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-10 w-10 text-muted-foreground"
                        onClick={() => openNoteDialog(index)}
                      >
                        <MessageSquare className="h-3.5 w-3.5" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-10 w-10 text-muted-foreground"
                        onClick={() => openModifierDialog(index)}
                      >
                        <Tag className="h-3.5 w-3.5" />
                      </Button>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </ScrollArea>
        <Separator />
        <div className="p-4 space-y-3 flex-shrink-0">
          <div className="flex items-center justify-between">
            <span className="text-muted-foreground">合计</span>
            <div className="flex items-center gap-2">
              <Button variant="ghost" size="sm" onClick={() => { if (cart.length > 0) setClearCartConfirmOpen(true); }} disabled={cart.length === 0} className="text-muted-foreground hover:text-destructive">
                清空
              </Button>
              <span className="text-2xl font-bold text-primary">{formatPrice(cartTotal)}</span>
            </div>
          </div>
          <div className="flex gap-2">
            <Button
              variant={dineType === "dine_in" ? "default" : "outline"}
              size="sm" className="flex-1"
              onClick={() => setDineType("dine_in")}
            >堂食</Button>
            <Button
              variant={dineType === "takeout" ? "default" : "outline"}
              size="sm" className="flex-1"
              onClick={() => setDineType("takeout")}
            >外带</Button>
            <Button
              variant={dineType === "delivery" ? "default" : "outline"}
              size="sm" className="flex-1"
              onClick={() => setDineType("delivery")}
            >外送</Button>
          </div>
          {dineType === "dine_in" && (
            <Input
              placeholder="桌号（可选）"
              value={tableNo}
              onChange={(e) => setTableNo(e.target.value)}
              className="h-9"
            />
          )}
          <div className="grid grid-cols-3 gap-2">
            <Button variant="outline" className="h-12 col-span-1" onClick={async () => {
              if (isSubmitting) return;
              setIsSubmitting(true);
              try { const ok = await onCreateOrder(cart, dineType, tableNo || null); if (ok) clearCart(); }
              finally { setIsSubmitting(false); }
            }} disabled={cart.length === 0 || isSubmitting}>
              暂存
            </Button>
            <Button className="h-12 col-span-2 text-base" onClick={async () => {
              if (isSubmitting) return;
              setIsSubmitting(true);
              try { const ok = await onCreateAndSubmit(cart, dineType, tableNo || null); if (ok) clearCart(); }
              finally { setIsSubmitting(false); }
            }} disabled={cart.length === 0 || isSubmitting}>
              <Send className="mr-2 h-5 w-5" />
              提交
            </Button>
          </div>
        </div>
      </Card>

      {/* Mobile floating cart button — only shown on small screens */}
      <div className="md:hidden fixed bottom-6 right-6 z-40">
        <Button
          size="lg"
          className="relative h-14 w-14 rounded-full shadow-xl"
          onClick={() => setCartSheetOpen(true)}
        >
          <ShoppingCart className="h-6 w-6" />
          {cartCount > 0 && (
            <span className="absolute -top-1 -right-1 bg-red-500 text-white text-xs rounded-full h-5 w-5 flex items-center justify-center font-bold leading-none">
              {cartCount > 9 ? "9+" : cartCount}
            </span>
          )}
        </Button>
      </div>

      {/* Mobile cart sheet */}
      <Sheet open={cartSheetOpen} onOpenChange={setCartSheetOpen}>
        <SheetContent side="right" className="flex flex-col p-0 w-80 max-w-[85vw]">
          <SheetHeader className="px-4 py-3 border-b flex-shrink-0">
            <SheetTitle className="flex items-center gap-2">
              <ShoppingCart className="h-5 w-5" />
              当前订单
              {cartCount > 0 && (
                <Badge variant="default" className="ml-auto mr-8">
                  {cartCount} 件
                </Badge>
              )}
            </SheetTitle>
          </SheetHeader>
          <ScrollArea className="flex-1 px-4 py-3">
            {cart.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-12 text-muted-foreground gap-2">
                <ShoppingCart className="h-12 w-12 opacity-20" />
                <span className="text-sm">请选择商品</span>
              </div>
            ) : (
              <div className="space-y-3">
                {cart.map((item, index) => (
                  <div key={index} className="rounded-lg border bg-muted/30 p-3 space-y-2">
                    <div className="flex items-start justify-between gap-2">
                      <div className="flex-1 min-w-0">
                        <div className="font-medium text-sm truncate">{item.menu_item.name}</div>
                        {item.spec && (
                          <div className="flex items-center gap-1 mt-1">
                            <Tag className="h-3 w-3 text-muted-foreground" />
                            <span className="text-xs text-muted-foreground">{item.spec.spec_name}</span>
                          </div>
                        )}
                        {item.note && (
                          <div className="flex items-center gap-1 mt-1">
                            <MessageSquare className="h-3 w-3 text-muted-foreground" />
                            <span className="text-xs text-muted-foreground truncate">{item.note}</span>
                          </div>
                        )}
                        {item.modifiers.length > 0 && (
                          <div className="flex flex-wrap gap-1 mt-1">
                            {item.modifiers.map((mod, mi) => (
                              <Badge key={mi} variant="outline" className="text-[10px] py-0 h-5">
                                {mod.modifier_type}
                                {mod.price_delta !== 0 && ` (${mod.price_delta > 0 ? "+" : ""}${mod.price_delta})`}
                              </Badge>
                            ))}
                          </div>
                        )}
                      </div>
                      <div className="flex flex-col items-end gap-1">
                        <span className="text-sm font-medium text-primary whitespace-nowrap">
                          {formatPriceMini(getItemPrice(item) * item.qty)}
                        </span>
                        <div className="flex items-center gap-1">
                          <Button variant="outline" size="icon" className="h-10 w-10" onClick={() => updateQty(index, -1)}>
                            <Minus className="h-3 w-3" />
                          </Button>
                          <span className="w-5 text-center text-sm font-medium">{item.qty}</span>
                          <Button variant="outline" size="icon" className="h-10 w-10" onClick={() => updateQty(index, 1)}>
                            <Plus className="h-3 w-3" />
                          </Button>
                        </div>
                        <div className="flex gap-1">
                          <Button variant="ghost" size="icon" className="h-10 w-10" onClick={() => openNoteDialog(index)}>
                            <MessageSquare className="h-3 w-3" />
                          </Button>
                          <Button variant="ghost" size="icon" className="h-10 w-10" onClick={() => openModifierDialog(index)}>
                            <Tag className="h-3 w-3" />
                          </Button>
                          <Button variant="ghost" size="icon" className="h-10 w-10 text-destructive" onClick={() => removeFromCart(index)}>
                            <X className="h-3 w-3" />
                          </Button>
                        </div>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </ScrollArea>
          <Separator />
          <div className="p-4 space-y-3 flex-shrink-0">
            <div className="flex items-center justify-between">
              <span className="text-muted-foreground">合计</span>
              <div className="flex items-center gap-2">
                <Button variant="ghost" size="sm" onClick={() => { if (cart.length > 0) setClearCartConfirmOpen(true); }} disabled={cart.length === 0} className="text-muted-foreground hover:text-destructive">
                  清空
                </Button>
                <span className="text-2xl font-bold text-primary">{formatPrice(cartTotal)}</span>
              </div>
            </div>
            <div className="flex gap-2">
              <Button variant={dineType === "dine_in" ? "default" : "outline"} size="sm" className="flex-1" onClick={() => setDineType("dine_in")}>堂食</Button>
              <Button variant={dineType === "takeout" ? "default" : "outline"} size="sm" className="flex-1" onClick={() => setDineType("takeout")}>外带</Button>
              <Button variant={dineType === "delivery" ? "default" : "outline"} size="sm" className="flex-1" onClick={() => setDineType("delivery")}>外送</Button>
            </div>
            {dineType === "dine_in" && (
              <Input
                placeholder="桌号（可选）"
                value={tableNo}
                onChange={(e) => setTableNo(e.target.value)}
                className="h-9"
              />
            )}
            <div className="grid grid-cols-3 gap-2">
              <Button variant="outline" className="h-12 col-span-1" onClick={async () => {
                if (isSubmitting) return;
                setIsSubmitting(true);
                try { const ok = await onCreateOrder(cart, dineType, tableNo || null); if (ok) { clearCart(); setCartSheetOpen(false); } }
                finally { setIsSubmitting(false); }
              }} disabled={cart.length === 0 || isSubmitting}>
                暂存
              </Button>
              <Button className="h-12 col-span-2 text-base" onClick={async () => {
                if (isSubmitting) return;
                setIsSubmitting(true);
                try { const ok = await onCreateAndSubmit(cart, dineType, tableNo || null); if (ok) { clearCart(); setCartSheetOpen(false); } }
                finally { setIsSubmitting(false); }
              }} disabled={cart.length === 0 || isSubmitting}>
                <Send className="mr-2 h-5 w-5" />
                提交
              </Button>
            </div>
          </div>
        </SheetContent>
      </Sheet>

      <Dialog open={clearCartConfirmOpen} onOpenChange={setClearCartConfirmOpen}>
        <DialogContent>
          <DialogHeader><DialogTitle>清空购物车</DialogTitle></DialogHeader>
          <p className="py-4 text-sm text-muted-foreground">确定要清空购物车中的所有商品吗？</p>
          <DialogFooter>
            <Button variant="outline" onClick={() => setClearCartConfirmOpen(false)}>取消</Button>
            <Button variant="destructive" onClick={() => { clearCart(); setClearCartConfirmOpen(false); }}>清空</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={specDialogOpen} onOpenChange={setSpecDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>选择规格 - {currentItem?.name}</DialogTitle>
          </DialogHeader>
          <div className="space-y-3 py-4">
            {specs.map((spec) => (
              <Button
                key={spec.id}
                variant={selectedSpec?.id === spec.id ? "default" : "outline"}
                className="w-full flex items-center justify-between h-auto py-3 px-4"
                onClick={() => setSelectedSpec(spec)}
              >
                <span className="font-medium">{spec.spec_name}</span>
                <div className="flex items-center gap-3">
                  {spec.price_delta !== 0 && (
                    <span className={`text-sm ${spec.price_delta > 0 ? "text-destructive" : "text-primary"}`}>
                      {spec.price_delta > 0 ? "+" : ""}{formatPrice(spec.price_delta)}
                    </span>
                  )}
                  {spec.qty_multiplier !== 1 && (
                    <span className="text-xs text-muted-foreground">x{spec.qty_multiplier}</span>
                  )}
                </div>
              </Button>
            ))}
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setSpecDialogOpen(false)}>
              取消
            </Button>
            <Button onClick={confirmSpec}>
              确認
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={noteDialogOpen} onOpenChange={setNoteDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>添加备注</DialogTitle>
          </DialogHeader>
          <div className="py-4">
            <Label htmlFor="note">备注内容</Label>
            <Textarea
              id="note"
              value={tempNote}
              onChange={(e) => setTempNote(e.target.value)}
              placeholder="如：少辣、不要葱、加份米饭..."
              className="mt-2"
              rows={3}
            />
            <div className="flex gap-2 mt-3 flex-wrap">
              {["少辣", "多辣", "不要葱", "不要蒜", "加急", "打包"].map((note) => (
                <Button
                  key={note}
                  variant="outline"
                  size="sm"
                  onClick={() => setTempNote(tempNote ? `${tempNote} ${note}` : note)}
                >
                  {note}
                </Button>
              ))}
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setNoteDialogOpen(false)}>
              取消
            </Button>
            <Button onClick={confirmNote}>确认</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={modifierDialogOpen} onOpenChange={setModifierDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>加料 / 去料</DialogTitle>
          </DialogHeader>
          {currentCartItemIndex !== null && (
            <div className="py-4">
              {cart[currentCartItemIndex].modifiers.length > 0 && (
                <div className="mb-4">
                  <Label className="mb-2 block">已添加:</Label>
                  <div className="flex flex-wrap gap-2">
                    {cart[currentCartItemIndex].modifiers.map((mod, idx) => (
                      <Badge key={idx} variant="secondary" className="flex items-center gap-1">
                        {mod.modifier_type}
                        {mod.price_delta !== 0 && ` (${mod.price_delta > 0 ? "+" : ""}${mod.price_delta})`}
                        <Button variant="ghost" size="icon" className="h-6 w-6 ml-1" onClick={() => removeModifier(currentCartItemIndex, idx)}>
                          <X className="h-3 w-3" />
                        </Button>
                      </Badge>
                    ))}
                  </div>
                </div>
              )}
              <Label className="mb-2 block">快速添加:</Label>
              <div className="flex flex-wrap gap-2">
                {presetModifiers.map((mod, idx) => (
                  <Button
                    key={idx}
                    variant="outline"
                    size="sm"
                    onClick={() => addModifier({ ...mod })}
                  >
                    {mod.modifier_type}
                    {mod.price_delta !== 0 && ` (${mod.price_delta > 0 ? "+" : ""}${mod.price_delta})`}
                  </Button>
                ))}
              </div>
              <div className="mt-4">
                <Label className="mb-2 block">自定義:</Label>
                <div className="flex gap-2">
                  <Input
                    placeholder="名稱"
                    id="mod-name"
                    className="flex-1"
                    onKeyDown={(e) => {
                      if (e.key === "Enter") {
                        const nameInput = document.getElementById("mod-name") as HTMLInputElement;
                        const priceInput = document.getElementById("mod-price") as HTMLInputElement;
                        if (nameInput.value.trim()) {
                          const priceDelta = parseSafeFloat(priceInput.value);
                          if (priceDelta === null) {
                            toast("价格调整无效，已设为 0", { icon: "⚠️" });
                          }
                          addModifier({ modifier_type: nameInput.value.trim(), qty: 1, price_delta: priceDelta ?? 0 });
                          nameInput.value = "";
                          priceInput.value = "";
                        }
                      }
                    }}
                  />
                  <Input
                    placeholder="价格"
                    id="mod-price"
                    type="number"
                    className="w-20"
                    defaultValue={0}
                  />
                </div>
              </div>
            </div>
          )}
          <DialogFooter>
            <Button variant="outline" onClick={() => setModifierDialogOpen(false)}>
              完成
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
