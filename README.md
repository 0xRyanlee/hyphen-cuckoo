# Cuckoo — 餐饮作业系统

> 配方驱动、本地优先的餐厅全流程桌面管理应用

<div align="center">

[![Version](https://img.shields.io/badge/版本-v3.13.0-blue?style=for-the-badge)](https://github.com/0xRyanlee/hyphen-cuckoo/releases)
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
| POS 点餐 | 分类浏览、常用品项快捷区、规格选择、加料/去料、购物车、一键提交 |
| 订单管理 | 订单列表、状态追踪、单品退单（逐品退款）、日期/状态筛选、收据打印 |
| KDS 厨房 | 厨房显示系统，按工作站分单、超时警告、出单确认 |
| 顾客管理 | 顾客档案、积分/集点、消费记录、手动调整积分 |

### 后台管理

| 模块 | 功能 |
|------|------|
| 仪表板 | 本周 vs 上周销售对比、今日热销 Top 5、待出餐、库存预警 |
| 菜单 | 菜品/分类管理，关联配方，规格定价，可售状态切换，常用标记 |
| 材料管理 | 原料档案、分类、标签、单位换算、保质期设定 |
| 配方 | BOM 配方编辑（树状结构）、半成品嵌套、成本自动核算、类型管理 |
| 库存 | 批次追踪（FIFO/FEFO）、到期预警、出入库记录、废弃管理 |
| 库存盘点 | 周期盘点、差异自动计算、调整入库 |
| 数据报表 | 销售趋势、毛利分析、热销排行、分类统计、时段/星期分布 |

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
| 角色权限 | PIN 码保护的四角色系统（老板/收银/厨师/仓库），菜单级访问控制 |
| 局域网同步 | 主机模式（HTTP 服务）+ 从机模式（4 秒轮询），多设备实时共享订单 |
| 日常支出 | 非订单支出记录与统计 |
| 属性模板 | 自定义批次/材料/配方追踪字段 |
| 打印中心 | 打印机管理、LAN 扫描、模板设计、打印预览、飞鹅云打印 |
| 系统设置 | 版本信息、自动更新、数据备份、PIN 管理、错误日志 |

---

## 下载安装

从 **[Releases 页面](https://github.com/0xRyanlee/hyphen-cuckoo/releases)** 下载最新版本：

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
| 局域网同步 | 自建 TCP/HTTP 服务（纯标准库，零额外依赖）|
| 遥测 | Rust Webhook + 前端错误捕获 |

---

## 项目结构

```
Cuckoo/
├── src/                        # React 前端
│   ├── pages/                  # 页面组件（22 个模块页面）
│   ├── components/             # 通用 UI 组件
│   ├── hooks/                  # 数据加载 & 业务逻辑
│   ├── lib/                    # 工具函数、日志、角色权限
│   └── types/                  # TypeScript 统一类型定义
│
├── src-tauri/                  # Rust 后端
│   ├── src/
│   │   ├── lib.rs              # 应用入口，命令注册
│   │   ├── commands.rs         # Tauri IPC 命令（120+）
│   │   ├── database.rs         # SQLite 数据访问层（33 表）
│   │   ├── sync_server.rs      # 局域网同步 HTTP 服务
│   │   └── printer.rs          # ESC/POS 打印驱动
│   └── tauri.conf.json
│
├── docs/                       # 开发者与用户文档
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
git clone https://github.com/0xRyanlee/hyphen-cuckoo.git
cd Cuckoo
npm install
npm run tauri dev
```

### 打包发布

推送版本 tag，GitHub Actions 自动构建 macOS `.dmg` + Windows `.exe`：

```bash
git tag v2.x.x
git push origin v2.x.x
```

### 测试

```bash
npm test              # 运行测试
npm run test:run      # 单次运行
npm run test:coverage # 覆盖率报告
```

---

## 产品分析 — KANO 模型（2026-05-23，v2.2 规划期）

> 以两岸小餐厅老板（1–3 人团队，月营业额 5–50 万）为核心用户画像。

### Must-be 基本需求（缺失即流失）

2026 年市场标准线：这些功能是用户的最低预期，有了不加分，缺了直接走人。

| 功能 | 状态 | 备注 |
|------|------|------|
| POS 收款、订单流转、菜单管理 | ✅ 完整 | — |
| 基础库存追踪与采购 | ✅ 完整 | — |
| 本地数据备份与 CSV 导出 | ✅ 完整 | 报表/订单/库存/支出均可导出 |
| 订单作废与退款流程 | ✅ 完整 | 单品退单 v2.0 |
| **多设备支持（iPad/手机 POS）** | ❌ 开发中 | **2026 年已是行业标配**；无此功能无法进入绝大多数实体店场景 |

### One-dimensional 期望需求（越好越满意）

这些功能存在竞争梯度，做得越细致客户越满意。

| 功能 | 状态 | 备注 |
|------|------|------|
| 报表维度（时段/星期/热销/毛利） | ✅ 完整 | — |
| 订单筛选与搜索 | ✅ 完整 | — |
| 打印可靠性 | ✅ 完整 | — |
| 仪表板关键指标 | ✅ 完整 | — |
| **KDS 实时性**（同步延迟） | ⚠️ 4 秒轮询 | 够用但非最优；iPad web client 上线后可考虑 SSE 推送 |
| **桌位管理清晰度** | ❌ 规划中 | 多设备场景下桌号归属变重要 |

### Attractive 魅力需求（超预期差异化）

用户没有预期，做了会惊喜，是与同价位竞品拉开差距的关键。

| 功能 | 状态 | 备注 |
|------|------|------|
| BOM 多层配方嵌套 + 成本核算 | ✅ 已实现 | 同价位竞品几乎无此功能 |
| FIFO/FEFO 批次自动扣减 | ✅ 已实现 | — |
| 顾客积分 / 集点系统 | ✅ 已实现 | — |
| 角色权限（PIN 四角色）| ✅ 已实现 | 小餐厅软件罕见 |
| **QR Code 桌位自助点单** | ❌ 规划中 | **核心差异化**：顾客扫码点餐直进 KDS，减少人工；定价 4000 以下软件无此功能 |
| **多端运行（无需安装）** | ❌ 规划中 | iPad 开浏览器即用，零部署摩擦，是销售话术的核心卖点之一 |

### Indifferent 无差异（暂不做）

| 功能 | 原因 |
|------|------|
| 多语言 i18n | 当前客户均为中文用户，资源优先级低 |
| 云端多店铺 | 架构复杂度过高，超出当前阶段 |

### Reverse 反向需求（做了反而减分）

| 功能 | 原因 |
|------|------|
| 强制月费订阅 | 用户选择 Cuckoo 的核心原因就是无订阅；引入等于背刺 |
| 强制云端同步 | 违反本地优先承诺，数据安全是小老板最大顾虑 |
| 复杂安装流程 | iPad 端若需要 App Store 安装，会直接流失非技术型客户 |

---

## 开发路线图

### v1.4.x — 打印完善 & 数据安全 ✅

收据打印、入库标签、工作站绑定打印机、本地自动备份。

### v1.5.0 — 体验打磨 ✅

订单筛选、食材到期通知、取消原因、退款 UI、时段/星期报表。

### v1.6.0 — 快捷与仪表板 ✅

POS 常用品项快捷区、仪表板本周 vs 上周对比、今日热销 Top 5。

### v3.13.0 — 营销协同 ✅（已完成）

| 功能 | 详情 |
|------|------|
| 顾客积分 / 集点 | 顾客档案、积分增减、消费累计、手动调整、历史记录 |
| 单品退单 | 逐品退款、`order_items.refunded` 标记、累计退款金额 |
| 角色权限系统 | 老板/收银/厨师/仓库四角色，PIN 保护，菜单级访问控制 |
| 局域网多设备同步 | 零依赖 TCP/HTTP 主机服务 + 从机 4 秒轮询，订单实时共享 |

### v2.1.0 — 下一阶段（规划中）

| 方向 | 关键任务 |
|------|----------|
| 双向同步 | KDS 设备完成出餐同步回 POS |
| 同步加固 | 共享密钥、协议版本校验、同步错误可观测 |
| CSV 导出 | 报表/订单/库存数据导出 |
| 批量操作 | 批量菜品上下架、批量采购入库 |

### v3.0.0 — 云端与生态（远期）

积分兑换、顾客绑单、微信小程序接入、多店铺云端管理、AI 辅助经营分析。

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

---

## 参与贡献

1. Fork 本仓库
2. 创建 feature 分支：`git checkout -b feature/my-feature`
3. 提交代码：`git commit -m 'feat: add my feature'`
4. 推送分支：`git push origin feature/my-feature`
5. 创建 Pull Request

> 提交前请运行 `npx tsc --noEmit` 确保无 TypeScript 编译错误。

---

## License

[MIT](LICENSE) © 2026 Cuckoo Team
