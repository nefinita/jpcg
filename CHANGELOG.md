# Changelog

本文件按 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/) 格式，于每次发布时由 `scripts/release.sh`
从 `changes/` 聚合生成。版本号遵循语义化版本（SemVer）。

## [Unreleased]

### 新增
- git-flow 维护流程 + CI/Release 工作流 + PR/Issue 模板 + CONTRIBUTING
- 组件独立版本：data_version / jpcg_const 采用 `等级.赛季.日期`（如 `130.3.20260602`）
- app 内展示各模块 dll 版本（`get_module_versions`）
- 模块库（dll）增量更新：modules_manifest 逐 dll 带 version + sha256，逐 dll 差量判断

### 变更
- 数据版本由日历式改为 `等级.赛季.日期`；版本标签 UI 美化（含日期）
- 模块更新改为逐 dll 版本/哈希比较，本地存快照

### 修复
- （待补充）

## [2.1.0-alpha.1] - 2026-06

（历史版本记录待首次正式发布时从 git log / changes/ 补充）
