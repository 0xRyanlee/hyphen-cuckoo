# Cuckoo — 餐饮作业系统

> 配方驱动、本地优先的餐厅全流程桌面管理应用

<div align="center">

[![Version](https://img.shields.io/github/v/release/0xRyanlee/hyphen-cuckoo?style=for-the-badge&label=最新版本)](https://github.com/0xRyanlee/hyphen-cuckoo/releases/latest)
[![Tauri 2](https://img.shields.io/badge/Tauri-2-FFC131?style=for-the-badge&logo=tauri&logoColor=white)](https://tauri.app/)
[![React](https://img.shields.io/badge/React-18-61DAFB?style=for-the-badge&logo=react&logoColor=black)](https://react.dev/)
[![Rust](https://img.shields.io/badge/Rust-2021-CE422B?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)

</div>

---

## 简介

Cuckoo 是面向中小餐厅的全流程后台管理系统。核心理念是**配方驱动**——所有库存扣料、成本核算均基于 BOM 配方自动完成，无需手工记账。

- **完全本地运行**：数据存储于本机 SQLite，无需联网、无订阅费
- **跨平台**：支持 macOS 和 Windows
- **一体化**：点餐 → 厨房 → 库存 → 采购 → 报表，全链路贯通

---

## 下载安装

从 **[Releases 页面](https://github.com/0xRyanlee/hyphen-cuckoo/releases/latest)** 下载最新版本：

| 平台 | 文件 | 说明 |
|------|------|------|
| macOS | `Cuckoo_x.x.x_universal.dmg` | Apple Silicon + Intel 通用版 |
| Windows | `Cuckoo_x.x.x_x64-setup.exe` | 64 位安装包 |

### macOS 安装说明

1. 下载 `.dmg` 文件
2. 打开后将 Cuckoo 拖入 Applications
3. 首次启动右键点击「打开」（绕过 Gatekeeper）

### Windows 安装说明

1. 下载 `.exe` 安装包
2. 运行安装向导

---

## 功能模块

### 前台操作

| 模块 | 功能 |
|------|------|
| POS 点餐 | 分类浏览、规格选择、加料/去料、购物车、一键提交 |
| 订单管理 | 订单列表、状态追踪、单品退单、收据打印 |
| KDS 厨房 | 按工作站分单、超时警告、出单确认 |
| 顾客管理 | 顾客档案、积分/集点、消费记录 |

### 后台管理

| 模块 | 功能 |
|------|------|
| 仪表板 | 营业额图表、库存预警、今日热销 |
| 菜单管理 | 分类、规格、加料、停售控制、套餐 |
| 材料管理 | 材料档案、分类标签、最低库存阈值 |
| 配方管理 | BOM 配方、成本核算、配方与菜单绑定 |
| 库存管理 | 批次入库、库存流水、盘点草稿 |

### 进货 / 生产

| 模块 | 功能 |
|------|------|
| 进货管理 | 采购单、收货确认、供应商管理 |
| 生产单 | 生产批次、原料消耗、入库记录 |

### 营销 / 设置

| 模块 | 功能 |
|------|------|
| 营销中心 | 自助点单页、集点卡、优惠券、营销图片 |
| 打印中心 | ESC/POS 打印机、自定义票据模板 |
| 系统设置 | 数据备份/恢复、角色权限、自动更新 |

---

## 自动更新

应用启动后会自动检查新版本，也可在**系统设置 → 自动更新**中手动触发检查。

---

## 数据安全

所有数据存储于本机 SQLite 文件（`~/Library/Application Support/com.cuckoo.ops/`）。

建议定期使用内置备份功能导出至外部存储。

---

*Proprietary software. All rights reserved.*
