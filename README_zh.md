# feature-scope

![Crates.io License](https://img.shields.io/crates/l/feature-scope)
[![Crates.io Version](https://img.shields.io/crates/v/feature-scope)](https://docs.rs/feature-scope)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/langyo/feature-scope/test.yml)

> **[English](README.md) | 中文**

## 简介

一个帮助库，让工作空间里的各个 crate 可以独立控制自己需要的特性，不会相互干扰。

> 项目还在开发中，API 可能会有变化。

## 工作原理

这个库基于 Rust 的 `-cfg` 编译参数，实现了自定义的特性作用域机制，从根本上解决了 Rust 工作空间中的 **特性统一问题**。

### 问题描述

传统的 Cargo 工作空间中，如果多个 crate 依赖同一个库但需要不同的特性，Cargo 会把所有特性合并到一起。这会带来一些问题：

- **特性冲突**：不同 crate 需要同一个依赖的互斥特性
- **意外编译**：代码被编译时启用了你没要求的特性
- **钻石依赖问题**：间接依赖导致的特性组合冲突

### 解决思路

`feature-scope` 绕过了 Cargo 的特性统一机制：

1. **自定义 cfg 标志**：不用 Cargo 特性，改用自定义的 `--cfg __scope_<feature>` 标志
2. **CLI 包装器**：`cargo feature-scope` 命令会拦截构建过程，注入正确的 cfg 标志
3. **过程宏**：`#[feature_scope]` 和 `#[feature_scope_default]` 宏把你的特性声明转换成基于 cfg 的条件编译
4. **独立控制**：工作空间里每个 crate 都能精确指定要用哪个依赖的哪些特性

这样一来，同个工作空间里的不同 crate 就能各自使用同一个依赖的不同特性集合，完全不会相互干扰，从编译层面彻底解决了特性统一问题。

## 安装

要使用这个库，你需要先安装 `cargo-feature-scope` CLI 工具：

```bash
cargo install cargo-feature-scope
```

也可以从源码安装：

```bash
git clone https://github.com/langyo/feature-scope.git
cd feature-scope
cargo install --path packages/cli
```

## 快速开始

这个库采用两步配置：

1. **在库 crate 里声明特性**，用 `package.metadata.feature-scope-decl`：

```toml
# 在你的库 crate 的 Cargo.toml 里
[package.metadata.feature-scope-decl]
default = ["a"]
a = []
b = []
c = []
```

1. **在使用方 crate 里配置特性**，用 `package.metadata.feature-scope`：

```toml
# 在你的应用/使用方 crate 的 Cargo.toml 里
[[package.metadata.feature-scope]]
package = "your-library-name"
features = ["b"]
default-features = false
```

这个库需要配合 `cargo-feature-scope` CLI 工具来提供正确的编译器参数。构建和运行项目时，你需要用 `cargo feature-scope` 代替普通的 `cargo` 命令：

```bash
# 构建项目
cargo feature-scope build

# 运行项目
cargo feature-scope run

# 运行工作空间里的指定包
cargo feature-scope run -p your-package-name

# 运行测试
cargo feature-scope test
```

然后就可以在代码里使用 `feature_scope` 宏了：

```rust
// 启用 `a` 特性或使用默认配置时，这个函数可用
#[feature_scope_default(a)]
pub fn feature_a_function() {
    println!("feature_scope_a");
}

// 只有启用 `b` 特性时，这个函数才会被编译
#[feature_scope(b)]
pub fn feature_b_function() {
    println!("feature_scope_b");
}

// 默认情况下这个函数可用
#[feature_scope_default]
pub fn default_function() {
    println!("默认函数");
}

// 根据 Cargo.toml 里设置的特性标志，调用相应的函数
feature_a_function();
feature_b_function();
default_function();
```

## 示例

项目在 `examples/` 目录里提供了可运行的示例。试试基础工作空间示例：

```bash
# 进入示例目录
cd examples/basic_workspace

# 构建工作空间里的所有包
cargo feature-scope build

# 运行默认入口（使用默认特性）
cargo feature-scope run -p entry_default
# 输出：
# a type
# default type

# 运行自定义入口（使用特性 'b'）
cargo feature-scope run -p entry_custom
# 输出：
# b type
# b type
```

这个示例展示了同一个工作空间里的不同包可以有不同的特性配置：

- `entry_default`：使用 `types` 包里定义的默认特性（特性 `a`）
- `entry_custom`：使用自定义特性配置（特性 `b`）
- `types`：根据启用的特性提供不同实现的共享库

## 开发

### 运行测试

项目包含了完整的 CI 工作流，会测试主代码库和示例：

1. **单元测试**：用 `cargo test --all-targets --all-features --workspace` 运行
2. **示例测试**：自动测试 `basic_workspace` 示例，确保输出正确
3. **代码质量**：Clippy 检查和代码格式化检查

本地运行示例测试：

```bash
# 用 cargo-make（推荐）
cargo install cargo-make
cargo make test-examples

# 或者运行包括示例在内的所有测试
cargo make test-all

# 或者手动运行
cd examples/basic_workspace
cargo feature-scope build
cargo feature-scope run -p entry_default
cargo feature-scope run -p entry_custom
```

---

**备注**：从 0.2.0 版本开始，所有源代码均基于 Copilot 的人机对话实现，只有架构构建、结果审核与提交记录有人工介入。
