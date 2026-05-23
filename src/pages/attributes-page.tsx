import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Tag, HelpCircle, Plus, Pencil, Trash2 } from "lucide-react";
import { toast } from "sonner";
import { call as invoke } from "@/lib/transport";

interface AttributeTemplate {
  id: number;
  entity_type: string;
  category: string | null;
  attr_code: string;
  attr_name: string;
  data_type: string;
  unit: string | null;
  default_value: number | null;
  formula: string | null;
  is_active?: boolean;
}

interface AttributesPageProps {
  attributeTemplates: AttributeTemplate[];
  onRefresh?: () => void;
}

export function AttributesPage({ attributeTemplates, onRefresh }: AttributesPageProps) {
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editingTemplate, setEditingTemplate] = useState<AttributeTemplate | null>(null);
  const [deleteConfirm, setDeleteConfirm] = useState<AttributeTemplate | null>(null);

  const [formEntityType, setFormEntityType] = useState("batch");
  const [formCategory, setFormCategory] = useState("");
  const [formCode, setFormCode] = useState("");
  const [formName, setFormName] = useState("");
  const [formDataType, setFormDataType] = useState("number");
  const [formUnit, setFormUnit] = useState("");
  const [formDefaultValue, setFormDefaultValue] = useState("");
  const [formFormula, setFormFormula] = useState("");

  function openNew() {
    setEditingTemplate(null);
    setFormEntityType("batch");
    setFormCategory("");
    setFormCode("");
    setFormName("");
    setFormDataType("number");
    setFormUnit("");
    setFormDefaultValue("");
    setFormFormula("");
    setDialogOpen(true);
  }

  function openEdit(template: AttributeTemplate) {
    setEditingTemplate(template);
    setFormEntityType(template.entity_type);
    setFormCategory(template.category || "");
    setFormCode(template.attr_code);
    setFormName(template.attr_name);
    setFormDataType(template.data_type);
    setFormUnit(template.unit || "");
    setFormDefaultValue(template.default_value?.toString() || "");
    setFormFormula(template.formula || "");
    setDialogOpen(true);
  }

  async function handleSave() {
    if (!formCode.trim() || !formName.trim()) {
      toast.error("请填写代码和名称");
      return;
    }

    try {
      if (editingTemplate) {
        await invoke("update_attribute_template", {
          id: editingTemplate.id,
          entityType: formEntityType,
          category: formCategory || null,
          attrCode: formCode,
          attrName: formName,
          dataType: formDataType,
          unit: formUnit || null,
          defaultValue: formDefaultValue ? parseFloat(formDefaultValue) : null,
          formula: formFormula || null,
          isActive: true,
        });
        toast.success("属性模板已更新");
      } else {
        await invoke("create_attribute_template", {
          entityType: formEntityType,
          category: formCategory || null,
          attrCode: formCode,
          attrName: formName,
          dataType: formDataType,
          unit: formUnit || null,
          defaultValue: formDefaultValue ? parseFloat(formDefaultValue) : null,
          formula: formFormula || null,
        });
        toast.success("属性模板已创建");
      }
      setDialogOpen(false);
      onRefresh?.();
    } catch (e) {
      toast.error("保存失败", { description: String(e) });
    }
  }

  async function handleDelete() {
    if (!deleteConfirm) return;
    try {
      await invoke("delete_attribute_template", { id: deleteConfirm.id });
      toast.success("属性模板已删除");
      setDeleteConfirm(null);
      onRefresh?.();
    } catch (e) {
      toast.error("删除失败", { description: String(e) });
    }
  }

  const getEntityTypeLabel = (type: string) => {
    switch (type) {
      case "material": return "材料";
      case "batch": return "批次";
      case "recipe": return "配方";
      default: return type;
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div>
            <h2 className="text-2xl font-semibold tracking-tight">属性模板</h2>
            <p className="text-sm text-muted-foreground">管理批次、材料和配方的自定义属性</p>
          </div>
          <Tooltip>
            <TooltipTrigger asChild>
              <Button variant="outline" size="icon" className="h-7 w-7 rounded-full">
                <HelpCircle className="h-4 w-4" />
              </Button>
            </TooltipTrigger>
            <TooltipContent side="right" className="max-w-sm p-4">
              <div className="space-y-2 text-sm">
                <p className="font-semibold">什么是属性模板？</p>
                <p>属性模板用于定义材料、批次或配方的<strong>自定义追踪字段</strong>，例如：</p>
                <ul className="list-disc pl-4 space-y-1">
                  <li><strong>冰衣率</strong> — 冷冻海鲜表面的冰衣重量占比</li>
                  <li><strong>出成率</strong> — 原材料加工后的可用比例</li>
                  <li><strong>季节性系数</strong> — 不同季节的品质/价格波动因子</li>
                  <li><strong>品质等级</strong> — A/B/C 等级评分</li>
                </ul>
                <p className="text-muted-foreground">创建批次时，系统会根据模板自动添加这些属性字段，用于更精确的成本核算和库存管理。</p>
              </div>
            </TooltipContent>
          </Tooltip>
        </div>
        <Button onClick={openNew}><Plus className="mr-2 h-4 w-4" />新增模板</Button>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Tag className="h-4 w-4" />
            属性模板列表
          </CardTitle>
          <CardDescription>共 {attributeTemplates.length} 个模板</CardDescription>
        </CardHeader>
        <CardContent>
          {attributeTemplates.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">
              暂无属性模板，点击"新增模板"创建
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>代码</TableHead>
                  <TableHead>名称</TableHead>
                  <TableHead>实体类型</TableHead>
                  <TableHead>分类</TableHead>
                  <TableHead>数据类型</TableHead>
                  <TableHead className="text-right">默认值</TableHead>
                  <TableHead>公式</TableHead>
                  <TableHead className="text-right">操作</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {attributeTemplates.map((template) => (
                  <TableRow key={template.id}>
                    <TableCell>
                      <code className="relative rounded bg-muted px-[0.3rem] py-[0.2rem] font-mono text-xs">
                        {template.attr_code}
                      </code>
                    </TableCell>
                    <TableCell className="font-medium">{template.attr_name}</TableCell>
                    <TableCell className="text-muted-foreground">{getEntityTypeLabel(template.entity_type)}</TableCell>
                    <TableCell className="text-muted-foreground">{template.category || "-"}</TableCell>
                    <TableCell className="text-muted-foreground">{template.data_type}</TableCell>
                    <TableCell className="text-right">{template.default_value ?? "-"}</TableCell>
                    <TableCell>
                      <code className="relative rounded bg-muted px-[0.3rem] py-[0.2rem] font-mono text-xs">
                        {template.formula || "-"}
                      </code>
                    </TableCell>
                    <TableCell className="text-right">
                      <div className="flex justify-end gap-1">
                        <Button variant="ghost" size="icon" className="h-8 w-8" onClick={() => openEdit(template)}>
                          <Pencil className="h-4 w-4" />
                        </Button>
                        <Button variant="ghost" size="icon" className="h-8 w-8 text-destructive" onClick={() => setDeleteConfirm(template)}>
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

      <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
        <DialogContent className="max-w-lg">
          <DialogHeader>
            <DialogTitle>{editingTemplate ? "编辑属性模板" : "新增属性模板"}</DialogTitle>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label>实体类型</Label>
                <Select value={formEntityType} onValueChange={setFormEntityType}>
                  <SelectTrigger><SelectValue /></SelectTrigger>
                  <SelectContent>
                    <SelectItem value="material">材料</SelectItem>
                    <SelectItem value="batch">批次</SelectItem>
                    <SelectItem value="recipe">配方</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <Label>分类（可选）</Label>
                <Input value={formCategory} onChange={(e) => setFormCategory(e.target.value)} placeholder="如：冷冻食品" />
              </div>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label>属性代码</Label>
                <Input value={formCode} onChange={(e) => setFormCode(e.target.value)} placeholder="如：ice_coating_rate" />
              </div>
              <div className="space-y-2">
                <Label>属性名称</Label>
                <Input value={formName} onChange={(e) => setFormName(e.target.value)} placeholder="如：冰衣率" />
              </div>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label>数据类型</Label>
                <Select value={formDataType} onValueChange={setFormDataType}>
                  <SelectTrigger><SelectValue /></SelectTrigger>
                  <SelectContent>
                    <SelectItem value="number">数值</SelectItem>
                    <SelectItem value="text">文本</SelectItem>
                    <SelectItem value="boolean">布尔</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <Label>单位（可选）</Label>
                <Input value={formUnit} onChange={(e) => setFormUnit(e.target.value)} placeholder="如：%" />
              </div>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label>默认值（可选）</Label>
                <Input type="number" value={formDefaultValue} onChange={(e) => setFormDefaultValue(e.target.value)} placeholder="0" />
              </div>
              <div className="space-y-2">
                <Label>公式（可选）</Label>
                <Input value={formFormula} onChange={(e) => setFormFormula(e.target.value)} placeholder="如：qty * 0.1" />
              </div>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDialogOpen(false)}>取消</Button>
            <Button onClick={handleSave}>保存</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={!!deleteConfirm} onOpenChange={() => setDeleteConfirm(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>确认删除</DialogTitle>
          </DialogHeader>
          <p className="py-4 text-sm text-muted-foreground">
            确定要删除属性模板「{deleteConfirm?.attr_name}」吗？
          </p>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteConfirm(null)}>取消</Button>
            <Button variant="destructive" onClick={handleDelete}>删除</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}