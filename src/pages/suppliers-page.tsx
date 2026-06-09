import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Plus, Building2, Pencil, Trash2, Save, Truck, ShoppingBag, Globe, Store } from "lucide-react";
import { EmptyState } from "@/components/ui/empty-state";
import type { SupplierProduct } from "@/types";

interface Supplier {
  id: number;
  name: string;
  phone: string | null;
  contact_person: string | null;
  address: string | null;
  note: string | null;
}

interface SuppliersPageProps {
  suppliers: Supplier[];
  supplierProducts: SupplierProduct[];
  onCreateSupplier: (data: { name: string; phone: string; contact_person: string; address: string; note: string }) => void;
  onUpdateSupplier: (id: number, data: { name?: string; phone?: string | null; contact_person?: string | null; address?: string | null; note?: string | null }) => void;
  onDeleteSupplier: (id: number) => void;
  onCreateSupplierProduct: (data: { product_name: string; supplier_name: string; channel: string }) => void;
  onUpdateSupplierProduct: (id: number, data: { product_name: string; supplier_name: string; channel: string }) => void;
  onDeleteSupplierProduct: (id: number) => void;
  searchQuery?: string;
}

export function SuppliersPage({
  suppliers,
  supplierProducts,
  onCreateSupplier,
  onUpdateSupplier,
  onDeleteSupplier,
  onCreateSupplierProduct,
  onUpdateSupplierProduct,
  onDeleteSupplierProduct,
  searchQuery,
}: SuppliersPageProps) {
  const [channelFilter, setChannelFilter] = useState<string>("all");
  const [newName, setNewName] = useState("");
  const [newPhone, setNewPhone] = useState("");
  const [newContact, setNewContact] = useState("");
  const [newAddress, setNewAddress] = useState("");
  const [newNote, setNewNote] = useState("");

  const [editSupplier, setEditSupplier] = useState<Supplier | null>(null);
  const [editName, setEditName] = useState("");
  const [editPhone, setEditPhone] = useState("");
  const [editContact, setEditContact] = useState("");
  const [editAddress, setEditAddress] = useState("");
  const [editNote, setEditNote] = useState("");

  const [deleteConfirm, setDeleteConfirm] = useState<Supplier | null>(null);
  const [deleteProductConfirm, setDeleteProductConfirm] = useState<SupplierProduct | null>(null);
  const [editProduct, setEditProduct] = useState<SupplierProduct | null>(null);
  const [editProductName, setEditProductName] = useState("");
  const [editProductSupplier, setEditProductSupplier] = useState("");
  const [editProductChannel, setEditProductChannel] = useState("local");

  const [addProductOpen, setAddProductOpen] = useState(false);
  const [newProductName, setNewProductName] = useState("");
  const [newProductSupplier, setNewProductSupplier] = useState("");
  const [newProductChannel, setNewProductChannel] = useState("local");

  const filteredProducts = supplierProducts.filter((p) => {
    const matchChannel = channelFilter === "all" || p.channel === channelFilter;
    if (!searchQuery) return matchChannel;
    const q = searchQuery.toLowerCase();
    const matchSearch = p.product_name.toLowerCase().includes(q) || p.supplier_name.toLowerCase().includes(q);
    return matchSearch && matchChannel;
  });

  const filteredSuppliers = suppliers.filter((s) => {
    if (!searchQuery) return true;
    const q = searchQuery.toLowerCase();
    return s.name.toLowerCase().includes(q) || (s.contact_person || "").toLowerCase().includes(q) ||
      (s.phone || "").toLowerCase().includes(q) || (s.address || "").toLowerCase().includes(q);
  });

  function openEdit(s: Supplier) {
    setEditSupplier(s);
    setEditName(s.name);
    setEditPhone(s.phone || "");
    setEditContact(s.contact_person || "");
    setEditAddress(s.address || "");
    setEditNote(s.note || "");
  }

  function saveEdit() {
    if (!editSupplier) return;
    onUpdateSupplier(editSupplier.id, {
      name: editName || undefined,
      phone: editPhone || null,
      contact_person: editContact || null,
      address: editAddress || null,
      note: editNote || null,
    });
    setEditSupplier(null);
  }

  function handleAddProduct() {
    if (!newProductName.trim() || !newProductSupplier.trim()) return;
    onCreateSupplierProduct({ product_name: newProductName.trim(), supplier_name: newProductSupplier.trim(), channel: newProductChannel });
    setNewProductName(""); setNewProductSupplier(""); setNewProductChannel("local");
    setAddProductOpen(false);
  }

  const navigate = useNavigate();
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">采购渠道管理</h2>
          <p className="text-sm text-muted-foreground">管理商品与采购渠道（本地 / 网络）</p>
        </div>
      </div>
      <div className="flex border-b border-border">
        <button className="-mb-px pb-2 px-4 text-sm font-medium border-b-2 border-transparent text-muted-foreground hover:text-foreground" onClick={() => navigate("/purchase-orders")}>采购单</button>
        <button className="-mb-px pb-2 px-4 text-sm font-medium border-b-2 border-primary text-primary">供应商 & 商品</button>
      </div>

      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-2">
                <ShoppingBag className="h-4 w-4" />
                商品列表
              </CardTitle>
              <CardDescription>共 {filteredProducts.length} 个商品</CardDescription>
            </div>
            <div className="flex items-center gap-3">
              <Tabs value={channelFilter} onValueChange={setChannelFilter}>
                <TabsList>
                  <TabsTrigger value="all">全部</TabsTrigger>
                  <TabsTrigger value="local">
                    <Store className="mr-1 h-3 w-3" />
                    本地
                  </TabsTrigger>
                  <TabsTrigger value="network">
                    <Globe className="mr-1 h-3 w-3" />
                    网络
                  </TabsTrigger>
                </TabsList>
              </Tabs>
              <Button size="sm" onClick={() => setAddProductOpen(true)}>
                <Plus className="mr-1 h-3 w-3" />新增商品
              </Button>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          {filteredProducts.length === 0 ? (
            <EmptyState icon={ShoppingBag} title="暂无商品" description="点击「新增商品」按钮添加" />
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>商品</TableHead>
                  <TableHead>供应商</TableHead>
                  <TableHead>采购渠道</TableHead>
                  <TableHead className="text-right">操作</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {filteredProducts.map((p) => (
                  <TableRow key={p.id}>
                    <TableCell className="font-medium">{p.product_name}</TableCell>
                    <TableCell className="text-muted-foreground">{p.supplier_name}</TableCell>
                    <TableCell>
                      {p.channel === "local" ? (
                        <span className="inline-flex items-center gap-1 rounded-full bg-secondary px-2.5 py-0.5 text-xs font-medium text-secondary-foreground">
                          <Store className="h-3 w-3" />
                          本地
                        </span>
                      ) : (
                        <span className="inline-flex items-center gap-1 rounded-full bg-secondary px-2.5 py-0.5 text-xs font-medium text-secondary-foreground">
                          <Globe className="h-3 w-3" />
                          网络
                        </span>
                      )}
                    </TableCell>
                    <TableCell className="text-right">
                      <div className="flex justify-end gap-1">
                        <Button variant="ghost" size="icon" className="h-8 w-8" onClick={() => { setEditProduct(p); setEditProductName(p.product_name); setEditProductSupplier(p.supplier_name); setEditProductChannel(p.channel); }}>
                          <Pencil className="h-4 w-4" />
                        </Button>
                        <Button variant="ghost" size="icon" className="h-8 w-8 text-destructive" onClick={() => setDeleteProductConfirm(p)}>
                          <Trash2 className="h-4 w-4" />
                        </Button>
                      </div>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      <div className="grid gap-6 lg:grid-cols-3">
        <Card className="lg:col-span-2">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Building2 className="h-4 w-4" />
              供应商列表
            </CardTitle>
            <CardDescription>共 {filteredSuppliers.length} 个供应商</CardDescription>
          </CardHeader>
          <CardContent>
            {filteredSuppliers.length === 0 ? (
              <EmptyState icon={Truck} title="暂无供应商" description="添加供应商开始管理" />
            ) : (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>名称</TableHead>
                    <TableHead>联系人</TableHead>
                    <TableHead>电话</TableHead>
                    <TableHead>地址</TableHead>
                    <TableHead className="text-right">操作</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {filteredSuppliers.map((s) => (
                    <TableRow key={s.id}>
                      <TableCell className="font-medium">{s.name}</TableCell>
                      <TableCell className="text-muted-foreground">{s.contact_person || "-"}</TableCell>
                      <TableCell className="text-muted-foreground font-mono text-xs">{s.phone || "-"}</TableCell>
                      <TableCell className="text-muted-foreground text-xs truncate max-w-[200px]">{s.address || "-"}</TableCell>
                      <TableCell className="text-right">
                        <div className="flex justify-end gap-1">
                          <Button variant="ghost" size="icon" className="h-8 w-8" onClick={() => openEdit(s)}><Pencil className="h-4 w-4" /></Button>
                          <Button variant="ghost" size="icon" className="h-8 w-8 text-destructive" onClick={() => setDeleteConfirm(s)}><Trash2 className="h-4 w-4" /></Button>
                        </div>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>新增供应商</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label>供应商名称</Label>
              <Input value={newName} onChange={(e) => setNewName(e.target.value)} placeholder="如：东海鲜物贸易" />
            </div>
            <div className="space-y-2">
              <Label>联系人</Label>
              <Input value={newContact} onChange={(e) => setNewContact(e.target.value)} placeholder="如：张先生" />
            </div>
            <div className="space-y-2">
              <Label>电话</Label>
              <Input value={newPhone} onChange={(e) => setNewPhone(e.target.value)} placeholder="如：02-12345678" />
            </div>
            <div className="space-y-2">
              <Label>地址</Label>
              <Input value={newAddress} onChange={(e) => setNewAddress(e.target.value)} placeholder="如：台北市中山区" />
            </div>
            <div className="space-y-2">
              <Label>备注</Label>
              <Input value={newNote} onChange={(e) => setNewNote(e.target.value)} placeholder="备注信息" />
            </div>
            <Separator />
            <Button className="w-full" onClick={() => {
              if (newName.trim()) {
                onCreateSupplier({ name: newName.trim(), phone: newPhone.trim(), contact_person: newContact.trim(), address: newAddress.trim(), note: newNote.trim() });
                setNewName(""); setNewPhone(""); setNewContact(""); setNewAddress(""); setNewNote("");
              }
            }} disabled={!newName.trim()}>
              <Plus className="mr-2 h-4 w-4" />新增
            </Button>

            <Separator />
            <div className="space-y-2">
              <h4 className="text-sm font-medium">快速添加</h4>
              <div className="flex flex-wrap gap-2">
                {["海鲜供应商", "肉类供应商", "蔬菜供应商", "调味料供应商"].map((name) => (
                  <Button key={name} variant="outline" size="sm" onClick={() => setNewName(name)}>{name}</Button>
                ))}
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* 新增商品 Dialog */}
      <Dialog open={addProductOpen} onOpenChange={setAddProductOpen}>
        <DialogContent>
          <DialogHeader><DialogTitle>新增商品</DialogTitle></DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label>商品名称</Label>
              <Input value={newProductName} onChange={(e) => setNewProductName(e.target.value)} placeholder="如：新鲜猪肉" />
            </div>
            <div className="space-y-2">
              <Label>供应商</Label>
              <Select value={newProductSupplier} onValueChange={setNewProductSupplier}>
                <SelectTrigger>
                  <SelectValue placeholder="选择供应商" />
                </SelectTrigger>
                <SelectContent>
                  {suppliers.map((s) => (
                    <SelectItem key={s.id} value={s.name}>{s.name}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label>采购渠道</Label>
              <Select value={newProductChannel} onValueChange={setNewProductChannel}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="local">
                    <span className="flex items-center gap-1"><Store className="h-3 w-3" />本地</span>
                  </SelectItem>
                  <SelectItem value="network">
                    <span className="flex items-center gap-1"><Globe className="h-3 w-3" />网络</span>
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setAddProductOpen(false)}>取消</Button>
            <Button onClick={handleAddProduct} disabled={!newProductName.trim() || !newProductSupplier.trim()}>
              <Plus className="mr-1 h-4 w-4" />添加
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* 编辑供应商 Dialog */}
      <Dialog open={!!editSupplier} onOpenChange={() => setEditSupplier(null)}>
        <DialogContent>
          <DialogHeader><DialogTitle>编辑供应商</DialogTitle></DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label>名称</Label>
              <Input value={editName} onChange={(e) => setEditName(e.target.value)} />
            </div>
            <div className="space-y-2">
              <Label>联系人</Label>
              <Input value={editContact} onChange={(e) => setEditContact(e.target.value)} />
            </div>
            <div className="space-y-2">
              <Label>电话</Label>
              <Input value={editPhone} onChange={(e) => setEditPhone(e.target.value)} />
            </div>
            <div className="space-y-2">
              <Label>地址</Label>
              <Input value={editAddress} onChange={(e) => setEditAddress(e.target.value)} />
            </div>
            <div className="space-y-2">
              <Label>备注</Label>
              <Input value={editNote} onChange={(e) => setEditNote(e.target.value)} />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setEditSupplier(null)}>取消</Button>
            <Button onClick={saveEdit}><Save className="mr-1 h-4 w-4" />保存</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* 删除供应商确认 */}
      <Dialog open={!!deleteConfirm} onOpenChange={() => setDeleteConfirm(null)}>
        <DialogContent>
          <DialogHeader><DialogTitle>确认删除</DialogTitle></DialogHeader>
          <p className="py-4 text-sm text-muted-foreground">确定要删除供应商「{deleteConfirm?.name}」吗？</p>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteConfirm(null)}>取消</Button>
            <Button variant="destructive" onClick={() => { if (deleteConfirm) { onDeleteSupplier(deleteConfirm.id); } setDeleteConfirm(null); }}>删除</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* 编辑商品 */}
      <Dialog open={!!editProduct} onOpenChange={() => setEditProduct(null)}>
        <DialogContent>
          <DialogHeader><DialogTitle>编辑采购商品</DialogTitle></DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label>商品名称</Label>
              <Input value={editProductName} onChange={(e) => setEditProductName(e.target.value)} placeholder="商品名称" />
            </div>
            <div className="space-y-2">
              <Label>供应商</Label>
              <Select value={editProductSupplier} onValueChange={setEditProductSupplier}>
                <SelectTrigger>
                  <SelectValue placeholder="选择供应商" />
                </SelectTrigger>
                <SelectContent>
                  {suppliers.map((s) => (
                    <SelectItem key={s.id} value={s.name}>{s.name}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label>渠道</Label>
              <Select value={editProductChannel} onValueChange={setEditProductChannel}>
                <SelectTrigger><SelectValue /></SelectTrigger>
                <SelectContent>
                  <SelectItem value="local">本地</SelectItem>
                  <SelectItem value="online">网络</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setEditProduct(null)}>取消</Button>
            <Button onClick={() => {
              if (editProduct && editProductName.trim() && editProductSupplier.trim()) {
                onUpdateSupplierProduct(editProduct.id, { product_name: editProductName.trim(), supplier_name: editProductSupplier.trim(), channel: editProductChannel });
                setEditProduct(null);
              }
            }}><Save className="mr-1 h-4 w-4" />保存</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* 删除商品确认 */}
      <Dialog open={!!deleteProductConfirm} onOpenChange={() => setDeleteProductConfirm(null)}>
        <DialogContent>
          <DialogHeader><DialogTitle>确认删除</DialogTitle></DialogHeader>
          <p className="py-4 text-sm text-muted-foreground">确定要删除商品「{deleteProductConfirm?.product_name}」吗？</p>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteProductConfirm(null)}>取消</Button>
            <Button variant="destructive" onClick={() => { if (deleteProductConfirm) { onDeleteSupplierProduct(deleteProductConfirm.id); } setDeleteProductConfirm(null); }}>删除</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
