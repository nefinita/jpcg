# ============================================================================
# JPCG 构建矩阵
#
# 一键流程（干净检出后一条命令）：
#   make all
#     = setup → frontend → updater（复制到 Tauri externalBin 位置）
#       → modules-dir（动态 app + 4 个模块库，动态 app 另存为 jpcg_app-dynamic）
#       → build-static（最后落地：jpcg_app 为默认 static 变体）
#
# 目标一览：
#   make setup         前端依赖安装（npm ci；缓存放 $(NPM_CACHE)，规避只读 $HOME）
#   make frontend      前端构建（npm run build → examples/jpcg_app/dist/）
#   make frontend-test 前端测试（npm test / vitest）
#   make build-static  静态模式 app（编译期链接 jpcg_core；默认）
#   make build-dynamic 动态模式 app（运行时 dlopen libjpcg_core）
#   make build-modules 四个模块 cdylib（core / combo / update / const）
#   make modules-dir   动态 app + 4 个模块库，复制到 $(MODULES_DIR)
#   make updater       构建 jpcg_updater 并复制到 app 的 externalBin 位置
#   make test          cargo test --workspace + dynamic 模式库测试
#   make fmt-check     rustfmt 检查
#   make clippy        clippy 静态检查（与 CI 一致，不 -D warnings）
#   make check-all     fmt + clippy + 构建 + 全量测试 + 前端 test/build
#
# 两种 app 构建模式（见 examples/jpcg_app/src-tauri/Cargo.toml）：
#   static  — 编译期链接 jpcg_core（默认，日常开发）
#   dynamic — dlopen libjpcg_core.{dylib,so,dll}（B 模式：更新只换 dll）
#
# 依赖顺序（重要）：
#   前端 dist 在编译期嵌入 app（tauri frontendDist），因此 build-static /
#   build-dynamic 依赖 frontend，frontend / frontend-test 依赖 setup 安装的
#   node_modules。默认 goal 仍是 build-static（保持既有 `make` 行为）。
#   Tauri 的 build script（tauri-build）在**每次**编译 app 时都会校验
#   frontendDist 并复制 externalBin sidecar，缺一即构建失败（不是只在
#   `tauri build`/打包阶段才需要）。因此所有会编译 app 的目标（build-static /
#   build-dynamic / clippy / test）都挂 $(APP_BUILD_DEPS) = frontend + updater
#   作为前置，干净检出后单跑其中任一目标都不会缺 updater sidecar。
#   static/dynamic 编译输出同名 target/$(BUILD)/jpcg_app，后构建者覆盖前者，
#   因此：modules-dir 在构建 dynamic 后先把它另存为 jpcg_app-dynamic，`all`
#   再把 build-static 放到最后，一键结束后 jpcg_app=默认 static、
#   jpcg_app-dynamic=dynamic 变体，两种产物同时存在（无需额外 target 目录）。
#
# 产物布局（BUILD=debug 默认；BUILD=release 时全部落在 target/release/）：
#   target/$(BUILD)/jpcg_app                      app 本体（make all 后为默认 static 变体）
#   target/$(BUILD)/jpcg_app-dynamic[.exe]        dynamic 变体（modules-dir 另存，便于对比/发布）
#   target/$(BUILD)/libjpcg_{core,combo,update,const}.{so,dylib}
#   target/$(BUILD)/jpcg_*.dll                    四个模块库（动态模式/跨语言/增量更新）
#   target/$(BUILD)/modules/                      上述 4 个库的副本（动态模式加载首选）
#   examples/jpcg_app/dist/                       前端静态资源（编译期嵌入 app）
#   examples/jpcg_app/src-tauri/binaries/jpcg_updater-<host-triple>[.exe]
#                                                 Tauri externalBin（打包必需）
# ============================================================================

SHELL := /bin/bash

APP_DIR := examples/jpcg_app/src-tauri
FRONTEND_DIR := examples/jpcg_app
BUILD ?= debug

CARGO ?= cargo
NPM ?= npm
# $HOME 只读环境下的 npm 缓存（必须在工作区内）；可用 `make NPM_CACHE=/path ...` 覆盖
NPM_CACHE ?= $(CURDIR)/.cache/npm
# 默认在 node_modules 内 stamp 新于 package.json/package-lock.json 时跳过 npm ci；
# FORCE_SETUP=1 强制重装（等价于每次都跑 npm ci）
FORCE_SETUP ?= 0
CARGO_JOBS ?=

CARGO_BUILD_FLAGS :=
ifeq ($(BUILD),release)
CARGO_BUILD_FLAGS += --release
endif
ifneq ($(CARGO_JOBS),)
CARGO_BUILD_FLAGS += -j$(CARGO_JOBS)
endif

UNAME_S := $(shell uname -s)
EXE_SUFFIX :=
ifeq ($(OS),Windows_NT)
EXE_SUFFIX := .exe
else ifneq (,$(filter MINGW% MSYS% CYGWIN%,$(UNAME_S)))
EXE_SUFFIX := .exe
endif

HOST_TRIPLE := $(shell rustc -vV 2>/dev/null | sed -n 's/^host: //p')

BIN_DIR := target/$(BUILD)
MODULES_DIR ?= $(BIN_DIR)/modules
MODULES := jpcg_core jpcg_combo jpcg_update jpcg_const
APP_BIN := $(BIN_DIR)/jpcg_app$(EXE_SUFFIX)
APP_DYN := $(BIN_DIR)/jpcg_app-dynamic$(EXE_SUFFIX)
UPDATER_SRC := $(BIN_DIR)/jpcg_updater$(EXE_SUFFIX)
UPDATER_DST := $(APP_DIR)/binaries/jpcg_updater-$(HOST_TRIPLE)$(EXE_SUFFIX)
FRONTEND_ABS := $(abspath $(FRONTEND_DIR))
SETUP_STAMP := $(FRONTEND_ABS)/node_modules/.jpcg-setup-stamp
NPM_ENV := npm_config_cache="$(NPM_CACHE)"

# 编译 app 必需的资源前置：前端 dist（tauri frontendDist）+ updater sidecar
# （tauri externalBin）。tauri-build 在 cargo build 阶段即会校验/复制两者，
# 因此凡是会编译 app 的目标都必须先备齐，见下方「依赖顺序（重要）」注释。
APP_BUILD_DEPS := frontend updater

.DEFAULT_GOAL := build-static

.PHONY: all setup frontend frontend-test build-static build-dynamic build-modules \
        modules-dir updater test fmt-check clippy check-all help

# 一键/全绿目标内部有产物顺序要求（dynamic app 另存、static 最后落地、
# 模块库先于 dynamic 冒烟测试），即使外部传 -j 也强制其前置目标串行执行。
.NOTPARALLEL: all check-all

# ----------------------------------------------------------------------------
# 一键目标
# ----------------------------------------------------------------------------

## 一键：依赖安装 → 前端构建 → 动态 app + 4 个模块库（另存 dynamic 变体）→ updater
##       → 静态 app（最后落地，jpcg_app = 默认 static）
all: setup frontend updater modules-dir build-static
	@echo "==> make all 完成"
	@echo "    app(static):  $(APP_BIN)"
	@echo "    app(dynamic): $(APP_DYN)"
	@echo "    模块库:       $(MODULES_DIR)/（4 个）"
	@echo "    updater:      $(UPDATER_DST)"

# ----------------------------------------------------------------------------
# 前端（npm）
# ----------------------------------------------------------------------------

## 前端依赖安装：npm ci（缓存写入 $(NPM_CACHE)，$HOME 只读也能跑）
setup:
	@STAMP="$(SETUP_STAMP)"; \
	if [ "$(FORCE_SETUP)" != "1" ] && [ -f "$$STAMP" ] \
	    && [ "$$STAMP" -nt "$(FRONTEND_ABS)/package.json" ] \
	    && [ "$$STAMP" -nt "$(FRONTEND_ABS)/package-lock.json" ]; then \
	  echo "==> 前端依赖已是最新（跳过 npm ci；FORCE_SETUP=1 可强制重装）"; \
	else \
	  echo "==> npm ci（cache=$(NPM_CACHE)）"; \
	  mkdir -p "$(NPM_CACHE)" && \
	  cd "$(FRONTEND_DIR)" && $(NPM_ENV) $(NPM) ci --no-audit --no-fund && \
	  touch "$$STAMP"; \
	fi

## 前端构建：tsc + vite → examples/jpcg_app/dist/（app 编译期嵌入）
frontend: setup
	@echo "==> npm run build（$(FRONTEND_DIR)）"
	@mkdir -p "$(NPM_CACHE)"
	@cd "$(FRONTEND_DIR)" && $(NPM_ENV) $(NPM) run build

## 前端测试：vitest（jsdom）
frontend-test: setup
	@echo "==> npm test（$(FRONTEND_DIR)）"
	@mkdir -p "$(NPM_CACHE)"
	@cd "$(FRONTEND_DIR)" && $(NPM_ENV) $(NPM) test

# ----------------------------------------------------------------------------
# Rust 构建
# ----------------------------------------------------------------------------

## 静态模式 app（默认；编译期链接 jpcg_core；先备齐前端 dist + updater sidecar）
build-static: $(APP_BUILD_DEPS)
	@echo "==> cargo build -p jpcg_app（static）"
	$(CARGO) build -p jpcg_app $(CARGO_BUILD_FLAGS)

## 动态模式 app（dlopen libjpcg_core；先备齐前端 dist + updater sidecar）
build-dynamic: $(APP_BUILD_DEPS)
	@echo "==> cargo build -p jpcg_app（dynamic）"
	$(CARGO) build -p jpcg_app --no-default-features --features dynamic $(CARGO_BUILD_FLAGS)

## 四个模块库 cdylib（jpcg_core / jpcg_combo / jpcg_update / jpcg_const）
build-modules:
	@echo "==> cargo build 四个模块库"
	$(CARGO) build -p jpcg_core -p jpcg_combo -p jpcg_update -p jpcg_const $(CARGO_BUILD_FLAGS)

## 动态 app + 4 个模块库：复制到 $(MODULES_DIR)；dynamic app 另存为 $(APP_DYN)
modules-dir: build-dynamic build-modules
	@echo "==> 复制 4 个模块库到 $(MODULES_DIR)"
	@mkdir -p "$(MODULES_DIR)"; missing=0; \
	for lib in $(MODULES); do \
	  src=""; \
	  for cand in "$(BIN_DIR)/lib$$lib.dylib" "$(BIN_DIR)/lib$$lib.so" "$(BIN_DIR)/$$lib.dll"; do \
	    if [ -f "$$cand" ]; then src="$$cand"; break; fi; \
	  done; \
	  if [ -z "$$src" ]; then \
	    echo "  ✗ 缺少 $$lib（$(BIN_DIR)）"; missing=1; \
	  else \
	    cp -f "$$src" "$(MODULES_DIR)/" && echo "  ✓ $$src"; \
	  fi; \
	done; \
	[ "$$missing" -eq 0 ] || { echo "错误：模块库不完整" >&2; exit 1; }; \
	echo "  已就绪：$$(ls "$(MODULES_DIR)" | grep -E 'jpcg_(core|combo|update|const)' | tr '\n' ' ')"; \
	if [ -f "$(APP_BIN)" ]; then \
	  cp -f "$(APP_BIN)" "$(APP_DYN)" && echo "  ✓ $(APP_BIN) → $(APP_DYN)（dynamic 变体另存）"; \
	else \
	  echo "  ✗ 缺少 $(APP_BIN)（build-dynamic 未产出？）" >&2; exit 1; \
	fi

## updater：构建并复制到 Tauri externalBin 位置（打包必需）
updater:
	@test -n "$(HOST_TRIPLE)" || { echo "错误：无法获取 host triple（rustc 不可用？）" >&2; exit 1; }
	@echo "==> cargo build -p jpcg_updater → $(UPDATER_DST)"
	$(CARGO) build -p jpcg_updater $(CARGO_BUILD_FLAGS)
	@mkdir -p "$(APP_DIR)/binaries"
	@test -f "$(UPDATER_SRC)" || { echo "错误：未生成 $(UPDATER_SRC)" >&2; exit 1; }
	@cp -f "$(UPDATER_SRC)" "$(UPDATER_DST)"
	@echo "  ✓ $(UPDATER_DST)"

# ----------------------------------------------------------------------------
# 测试与检查
# ----------------------------------------------------------------------------

## 全量测试（含 dynamic 端到端冒烟；dll 由 build-modules 生成，缺失时该用例自动跳过）
test: build-modules $(APP_BUILD_DEPS)
	$(CARGO) test --workspace $(CARGO_BUILD_FLAGS)
	$(CARGO) test -p jpcg_app --lib --no-default-features --features dynamic $(CARGO_BUILD_FLAGS)

## rustfmt 检查（不修改文件）
fmt-check:
	$(CARGO) fmt --all -- --check

## clippy（与 CI 一致：仅要求不产生编译错误，既有警告不计入门禁）
clippy: $(APP_BUILD_DEPS)
	$(CARGO) clippy --workspace --all-targets $(CARGO_BUILD_FLAGS)

## 全绿检查：fmt + clippy + 构建 + 全量测试 + 前端 test/build（失败即停；
## build-static 放最后，结束时 jpcg_app 为默认 static 变体）
check-all: fmt-check clippy build-modules test frontend-test frontend build-dynamic build-static
	@echo "==> 全部通过（fmt / clippy / 构建 / 全量测试 / 前端 test+build）"

# ----------------------------------------------------------------------------
# 帮助
# ----------------------------------------------------------------------------

## 列出可用目标
help:
	@echo "JPCG Makefile 目标（BUILD=$(BUILD)，缓存=$(NPM_CACHE)）："
	@echo "  make all            一键：setup + frontend + updater + modules-dir + build-static"
	@echo "                      （结束：$(APP_BIN)=static，$(APP_DYN)=dynamic）"
	@echo "  make setup          npm ci（缓存写入工作区，规避只读 HOME）"
	@echo "  make frontend       npm run build（dist，编译期嵌入 app）"
	@echo "  make frontend-test  npm test（vitest）"
	@echo "  make build-static   静态模式 app（默认 goal）"
	@echo "  make build-dynamic  动态模式 app（dlopen 模块库）"
	@echo "  make build-modules  四个模块 cdylib（core/combo/update/const）"
	@echo "  make modules-dir    动态 app + 4 个模块库 → $(MODULES_DIR)（dynamic app 另存 $(APP_DYN)）"
	@echo "  make updater        构建 updater → $(APP_DIR)/binaries/"
	@echo "  make test           cargo test --workspace + dynamic 库测试"
	@echo "  make fmt-check      rustfmt 检查"
	@echo "  make clippy         clippy 静态检查"
	@echo "  make check-all      fmt + clippy + 构建 + 测试 + 前端 test/build"
	@echo "  可用变量：BUILD=release CARGO_JOBS=N FORCE_SETUP=1 NPM_CACHE=/path MODULES_DIR=/path"
