# Cuckoo 文档索引 (Documentation Index)

> **更新日期**: 2026-05-22  
> **当前版本**: v2.0.0

---

## 📁 文档结构

```
Cuckoo/
├── README.md                      # 项目简介与快速开始
├── ROADMAP.md                     # 版本路线图（v1.3 ~ v3.0）
│
├── docs/
│   ├── 📖 用户文档
│   │   ├── user-guide.md          # 用户操作手册（面向店铺使用者）
│   │   └── install-guide.md       # 安装指南
│   │
│   ├── 📋 开发者文档
│   │   ├── DOCS_INDEX.md          # 本文档 — 文档总索引
│   │   ├── api-design.md          # Tauri IPC 命令设计（110+ 命令）
│   │   ├── database-schema.md     # 数据库架构（31 表）
│   │   ├── packaging-guide.md     # 打包、签名、CI 流程
│   │   └── audit-workflow.md      # 审计工作流（12 维度标准化流水线）
│   │
│   ├── 🔍 审计报告
│   │   ├── comprehensive-audit-report-v1.2.2.md   # 综合审计 v1.2.2
│   │   ├── info-flow-audit-2026-04-30.md          # 信息流安全审计
│   │   ├── multidimensional-audit-2026-05-22.md   # 多维度业务/安全/市场/技术债审计
│   │   ├── recipe-management-audit-2026-05-05.md  # 配方管理审计
│   │   ├── recipe-multi-role-roadmap-2026-05-05.md # 配方多角色路线图
│   │   ├── implementation-audit-report-v1.2.2.md  # 实现审计
│   │   └── backlog-and-fix-list.md                # 待修复清单（审计产出）
│   │
│   ├── 🧪 测试文档
│   │   ├── test-plan-user-journey-v1.2.2.md       # 用户旅程测试计划
│   │   └── test-plan-atomic-v1.2.2.md             # 原子测试计划
│   │
│   ├── 🔧 运维指南
│   │   ├── debug-pipeline.md       # 调试流水线（黑屏/崩溃诊断）
│   │   ├── audit-black-screen-macos26.md  # macOS 26 Beta 兼容问题
│   │   └── remote-assistance-guide.md     # 远程协助（日志获取方法）
│   │
│   ├── 📊 历史归档
│   │   └── archived/              # 旧版审计报告与开发文档
│   │
│   └── 📈 开发进度
│       └── progress/              # 开发进度追踪
```

---

## 📖 快速导航

### 我是用户

| 需求 | 文档 |
|------|------|
| 如何安装 | [install-guide.md](install-guide.md) |
| 如何使用 | [user-guide.md](user-guide.md) |
| 遇到问题 | [remote-assistance-guide.md](remote-assistance-guide.md) |

### 我是开发者

| 需求 | 文档 |
|------|------|
| 项目概览 | [README.md](../README.md) |
| 版本规划 | [ROADMAP.md](../ROADMAP.md) |
| 当前开发任务 | [TODOs.md](../TODOs.md) |
| 待修复问题 | [backlog-and-fix-list.md](backlog-and-fix-list.md) |
| API 设计 | [api-design.md](api-design.md) |
| 数据库结构 | [database-schema.md](database-schema.md) |
| 打包部署 | [packaging-guide.md](packaging-guide.md) |
| 开发后审计 | [audit-workflow.md](audit-workflow.md) |
| 调试问题 | [debug-pipeline.md](debug-pipeline.md) |

### 我是审计者

| 需求 | 文档 |
|------|------|
| 多维度现状审计 | [multidimensional-audit-2026-05-22.md](multidimensional-audit-2026-05-22.md) |
| 最新综合审计 | [comprehensive-audit-report-v1.2.2.md](comprehensive-audit-report-v1.2.2.md) |
| 数据流审计 | [info-flow-audit-2026-04-30.md](info-flow-audit-2026-04-30.md) |
| 配方专项审计 | [recipe-management-audit-2026-05-05.md](recipe-management-audit-2026-05-05.md) |
| 审计流水线 | [audit-workflow.md](audit-workflow.md) |

---

## 🔄 更新日志

| 日期 | 操作 |
|------|------|
| 2026-05-22 | 新增多维度审计文档，补充商业/安全/市场/技术债视角 |
| 2026-05-05 | 文档大重建：重写 README、ROADMAP、DOCS_INDEX，新增用户指南 |
| 2026-05-05 | 新增配方管理专项审计文档 |
| 2026-04-30 | 新增信息流安全审计 |
| 2026-04-29 | 新增审计工作流标准化文档 |
| 2026-04-28 | v1.2.2 综合审计报告 |

---

*本索引在每次文档结构变更时应同步更新。*
