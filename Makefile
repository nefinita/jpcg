# ============================================================================
# JPCG 构建矩阵
#
# 两种 app 构建模式（见 examples/jpcg_app/src-tauri/Cargo.toml）：
#   static  — 编译期链接 jpcg_core（默认，日常开发）
#   dynamic — dlopen libjpcg_core.{dylib,so,dll}（B 模式：更新只换 dll）
#
# 动态模式产物布局（模块版本目录）：
#   dist/app/<version>/          app 本体 + libjpcg_core / libjpcg_update
#   dist/modules/<version>/      core/update/const 三个 dll（同一版本目录，供跨语言/增量更新）
# ============================================================================

SHELL := /bin/bash
APP_DIR := examples/jpcg_app/src-tauri
BUILD ?= debug

CARGO ?= cargo
CARGO_BUILD_FLAGS :=
ifeq ($(BUILD),release)
CARGO_BUILD_FLAGS += --release
endif

UNAME_S := $(shell uname -s)
MODULE_LIB := libjpcg_core.dylib
ifeq ($(UNAME_S),Linux)
MODULE_LIB := libjpcg_core.so
endif
ifeq ($(UNAME_S),MINGW32_NT-*)
MODULE_LIB := jpcg_core.dll
endif

# ----------------------------------------------------------------------------
.PHONY: build-static build-dynamic build-modules modules-dir test check-all

## 静态模式 app（默认）
build-static:
	$(CARGO) build -p jpcg_app $(CARGO_BUILD_FLAGS)

## 动态模式 app
build-dynamic:
	$(CARGO) build -p jpcg_app --no-default-features --features dynamic $(CARGO_BUILD_FLAGS)

## 三个模块 dylib（jpcg_core / jpcg_update / jpcg_const）
build-modules:
	$(CARGO) build -p jpcg_core -p jpcg_update -p jpcg_const $(CARGO_BUILD_FLAGS)

## 把三个 dylib 复制到与 app 相同目录（动态模式运行所需）
modules-dir: build-dynamic build-modules
	@BIN_DIR="target/$(BUILD)"; \
	echo "==> 复制模块库到 $$BIN_DIR"; \
	for lib in libjpcg_core libjpcg_update libjpcg_const; do \
	  if [ -f "target/$(BUILD)/$$lib.dylib" ]; then \
	    cp "target/$(BUILD)/$$lib.dylib" "$$BIN_DIR/"; \
	  elif [ -f "target/$(BUILD)/$$lib.so" ]; then \
	    cp "target/$(BUILD)/$$lib.so" "$$BIN_DIR/"; \
	  fi; \
	done; \
	ls -la "$$BIN_DIR" | grep -E "jpcg_(core|update|const)|jpcg_app"

## 全量测试（含 dynamic 端到端冒烟）
test:
	$(CARGO) test --workspace
	$(CARGO) test -p jpcg_app --lib --no-default-features --features dynamic

## 双模式 + 测试 全绿检查
check-all: build-static build-dynamic build-modules test
	@echo "==> 全部通过"
