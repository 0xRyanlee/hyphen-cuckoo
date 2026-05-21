# Cuckoo — 餐饮作业系统

> 配方驱动、本地优先的餐厅全流程桌面管理应用

<div align="center">

[![Version](https://img.shields.io/badge/版本-v1.4.1-blue?style=for-the-badge)](https://github.com/0xRyanlee/Cuckoo/releases)
[![Tauri 2](https://img.shields.io/badge/Tauri-2-FFC131?style=for-the-badge&logo=tauri&logoColor=white)](https://tauri.app/)
[![React](https://img.shields.io/badge/React-18-61DAFB?style=for-the-badge&logo=react&logoColor=black)](https://react.dev/)
[![Rust](https://img.shields.io/badge/Rust-2021-CE422B?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![SQLite](https://img.shields.io/badge/SQLite-本地-003B57?style=for-the-badge&logo=sqlite&logoColor=white)](https://www.sqlite.org/)
[![License](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)](LICENSE)

</div>

---

## 简介

Cuckoo 是面向中小餐厅的全流程后台管理系统。核心理念是**配方驱动**——所有库存扣料、成本核算均基于 BOM 配方自动完成，无需手工记账。

- **完全本地运行**：数据存储于本机 SQLite，无需联网、无订阅费
- **跨平台**：支持 macOS 和 Windows
- **一体化**：点餐 → 厨房 → 库存 → 采购 → 报表，全链路贯通

> 📖 首次使用？请阅读 [用户指南](docs/user-guide.md)

---

## 功能模块

### 前台操作

| 模块 | 功能 |
|------|------|
| POS 点餐 | 分类浏览、规格选择、加料/去料、购物车、一键提交 |
| 订单管理 | 订单列表、状态追踪（堂食/外带/外送）、分页浏览 |
| KDS 厨房 | 厨房显示系统，按工作站分单、超时警告、出单确认 |

### 后台管理

| 模块 | 功能 |
|------|------|
| 仪表板 | 今日/本周/本月销售、毛利、订单概览 |
| 菜单 | 菜品/分类管理，关联配方，规格定价，可售状态切换 |
| 材料管理 | 原料档案、分类、标签、单位换算、保质期设定 |
| 配方 | BOM 配方编辑（树状结构）、半成品嵌套、成本自动核算、类型管理 |
| 库存 | 批次追踪（FIFO/FEFO）、到期预警、出入库记录、废弃管理 |
| 库存盘点 | 周期盘点、差异自动计算、调整入库 |
| 数据报表 | 销售趋势、毛利分析、热销排行、分类统计、原料消耗 |

### 进货 / 生产

| 模块 | 功能 |
|------|------|
| 供应商 | 供应商档案管理 |
| 采购单 | 创建采购单、收货入库、自动生成批次 |
| 生产单 | 半成品生产、配方驱动原料扣减、产出入库 |
| 材料状态 | 自定义材料属性（冰衣率、出成率等）|

### 设置与运维

| 模块 | 功能 |
|------|------|
| 属性模板 | 自定义批次/材料/配方追踪字段 |
| 打印中心 | 打印机管理、LAN 扫描、模板设计、打印预览、飞鹅云打印 |
| 系统设置 | 版本信息、连线状态、前端错误日志、远程遥测 |

---

## 下载安装

从 **[Releases 页面](https://github.com/0xRyanlee/Cuckoo/releases)** 下载最新版本：

| 平台 | 文件 | 说明 |
|------|------|------|
| macOS | `Cuckoo_x.x.x_universal.dmg` | 拖入 Applications 即可 |
| Windows | `Cuckoo_x.x.x_x64-setup.exe` | 双击安装 |

> **首次运行 macOS**：如提示"无法验证开发者"，前往 系统设置 → 隐私与安全性 → 仍要打开。

详细安装步骤请参考 [安装指南](docs/install-guide.md)。

---

## 数据位置

| 平台 | 路径 |
|------|------|
| macOS | `~/Library/Application Support/Cuckoo/` |
| Windows | `%LOCALAPPDATA%\Cuckoo\` |

数据库文件：`cuckoo.db`（SQLite，可直接复制备份）

---

## 技术栈

| 层 | 技术 |
|----|------|
| 桌面壳 | Tauri 2 (Rust) |
| 前端框架 | React 18 + TypeScript |
| UI 组件 | shadcn/ui + Tailwind CSS 4 |
| 路由 | React Router v7（Hash 模式）|
| 数据库 | SQLite via rusqlite（WAL 模式）|
| 热敏打印 | ESC/POS + TSPL + 飞鹅云打印 |
| 遥测 | Rust Webhook + 前端错误捕获 |

---

## 项目结构

```
Cuckoo/
├── src/                        # React 前端
│   ├── pages/                  # 页面组件（20 个模块页面）
│   ├── components/             # 通用 UI 组件
│   ├── hooks/                  # 数据加载 & 业务逻辑
│   ├── lib/                    # 工具函数、日志、遥测
│   └── types/                  # TypeScript 统一类型定义
│
├── src-tauri/                  # Rust 后端
│   ├── src/
│   │   ├── lib.rs              # 应用入口，命令注册
│   │   ├── commands.rs         # Tauri IPC 命令（110+）
│   │   ├── database.rs         # SQLite 数据访问层（31 表）
│   │   └── printer.rs          # ESC/POS 打印驱动
│   └── tauri.conf.json
│
├── docs/                       # 开发者与用户文档
│   ├── user-guide.md           # 用户指南
│   ├── api-design.md           # API 设计文档
│   ├── database-schema.md      # 数据库架构
│   ├── audit-workflow.md       # 审计工作流
│   └── ...                     # 更多文档见 docs/DOCS_INDEX.md
│
└── assets/                     # 图标与静态资源
```

---

## 开发者指南

### 环境要求

- **Node.js** 18+
- **Rust** 1.78+（[安装 Rustup](https://rustup.rs/)）
- **npm** 9+

### 本地运行

```bash
git clone https://github.com/0xRyanlee/Cuckoo.git
cd Cuckoo
npm install
npm run tauri dev
```

### 打包发布

推送版本 tag，GitHub Actions 自动构建 macOS `.dmg` + Windows `.exe`：

```bash
git tag v1.x.x
git push origin v1.x.x
```

### 测试

```bash
npm test              # 运行测试
npm run test:run      # 单次运行
npm run test:coverage # 覆盖率报告
```

---

## 产品分析 — KANO 模型（2026-05-21）

以 KANO 模型对当前功能进行分层，指导优先级决策：

### Must-be 基本需求（缺失即流失）

| 功能 | 状态 |
|------|------|
| POS 收款、订单流转、菜单管理 | ✅ 完整 |
| 基础库存追踪与采购 | ✅ 完整 |
| 错误提示覆盖率 | ✅ 已系统性修复 |
| **本地数据自动备份** | ✅ v1.4 已实现（设置页可配置）|
| **订单作废与退款流程** | ⚠️ 已可取消，部分退款待完善 |

### One-dimensional 期望需求（越好越满意）

报表维度、订单历史筛选、打印稳定性、库存成本精度——当前均已实现，
持续改善可线性提升用户留存。

### Attractive 魅力需求（超预期差异化）

| 功能 | 状态 |
|------|------|
| KDS 厨房显示 + 工作站打印机路由 | ✅ 已实现 |
| FIFO/FEFO 批次自动扣减 | ✅ 已实现 |
| 多层配方展开（半成品嵌套） | ✅ 已修复 |
| 收款后自动打印收据 | ✅ v1.4 新增 |
| 入库自动打印批次标签 | ✅ v1.4 新增 |
| 顾客积分 / 集点 | 路线图中 |
| 食材到期前桌面通知 | 路线图中 |

### 判断结论

Cuckoo 的 Attractive 层（KDS、FIFO、多层配方）已达中端 ERP 水准，
Must-be 层在 v1.4 完成备份与错误修复后基本达标。
下一重心：将 One-dimensional 项（订单筛选、报表增强）推到更高完成度，
以及引入第一个真正的 Attractive 新功能（积分集点）。

---

## 开发路线图

### v1.4.x — 打印完善 & 数据安全（当前）

| 方向 | 关键任务 |
|------|----------|
| 打印 | 收据打印、入库标签开关、工作站绑定打印机 |
| 数据安全 | 本地自动备份 SQLite（启动时 + 手动触发）|
| 错误提示 | 打印页静默失败修复、收款自动打印 |

### v1.5.0 — 体验打磨（近期目标）

| 方向 | 关键任务 |
|------|----------|
| 订单 | 订单历史日期 + 状态筛选、部分退款流程 |
| 库存 | 食材到期日三天前桌面通知 |
| 报表 | 按星期 / 时段分析、员工维度 |
| POS | 常用品项快捷区、扫码下单支持 |

### v2.0.0 — 功能扩展（季度规划）

| 方向 | 关键任务 |
|------|----------|
| 会员 | 顾客积分 / 集点卡（烘焙店高频需求）|
| 权限 | 角色系统（老板 / 收银 / 厨师 / 仓库）|
| 协同 | 局域网多设备同步 |

### v3.0.0 — 云端与生态（远期）

| 方向 | 关键任务 |
|------|----------|
| 云端 | 微信小程序接入、多店铺管理 |
| 分析 | AI 辅助经营分析 |

> 完整路线图见 [ROADMAP.md](ROADMAP.md)

---

## 文档

| 文档 | 说明 |
|------|------|
| [文档总索引](docs/DOCS_INDEX.md) | 所有文档导航入口 |
| [用户指南](docs/user-guide.md) | 面向店铺使用者的操作手册 |
| [API 设计](docs/api-design.md) | Tauri IPC 命令签名与说明 |
| [数据库结构](docs/database-schema.md) | 完整表结构与关联关系 |
| [打包指南](docs/packaging-guide.md) | 签名、公证、CI 流程 |
| [调试指南](docs/debug-pipeline.md) | 渲染/黑屏问题诊断流程 |
| [审计工作流](docs/audit-workflow.md) | 开发后审计流水线 |
| [待修复清单](docs/backlog-and-fix-list.md) | 审计发现的待处理问题 |
| [远程协助](docs/remote-assistance-guide.md) | 日志位置与获取方法 |

---

## 参与贡献

1. Fork 本仓库
2. 创建 feature 分支：`git checkout -b feature/my-feature`
3. 提交代码：`git commit -m 'feat: add my feature'`
4. 推送分支：`git push origin feature/my-feature`
5. 创建 Pull Request

> 提交前请运行 `npm run lint` 确保无 TypeScript 编译错误。

---

## License

[MIT](LICENSE) © 2026 Cuckoo Team
