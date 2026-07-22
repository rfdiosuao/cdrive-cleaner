<p align="center"><strong>简体中文</strong> · <a href="README.en.md">English</a></p>

# C盘清理助手

一款基于 Tauri 2.0 的高性能 Windows C盘清理工具，Rust 后端 + Vue 3 前端。

## 功能特性

### 智能扫描
- **NTFS USN 日志扫描** — 利用 USN Change Journal 快速枚举文件变更，毫秒级响应
- **异步并发文件遍历** — 信号量控制的并行目录遍历，高效利用多核
- **文件指纹缓存** — SQLite 存储的 MD5 + XXHash 指纹，避免重复计算

### 安全清理
- **Authenticode 签名验证** — WinVerifyTrust FFI 校验数字签名，保护系统文件
- **三级安全防护** — 目录白名单 / 扩展名白名单 / 签名验证逐层过滤
- **加权安全评分** — 基于路径、类型、时间、频率、签名等多维度加权评分
- **备份还原** — 清理前自动创建备份，支持一键恢复

### 9 大清理插件

| 插件 | 说明 |
|------|------|
| 系统垃圾 | Temp 文件、日志、更新缓存、缩略图缓存 |
| 系统冗余 | Windows.old、WinSxS 冗余、旧还原点 |
| 社交软件 | 微信、QQ、企业微信聊天缓存 |
| 浏览器 | Chrome、Edge、Firefox 缓存 |
| 大文件 | 智能定位 100MB+ 大文件 |
| 重复文件 | 相同文件 & 相似图片检测 |
| 软件迁移 | 已安装软件无损迁移到其他盘 |
| AI 智能迁移 | AI 驱动的文件迁移方案推荐 |
| 定时清理 | 定时自动清理 & 磁盘空间紧急清理 |

### AI 加持
- **本地推理** — ONNX Runtime 本地模型，离线可用
- **云端分析** — 支持 OpenAI / Anthropic / DeepSeek / 通义千问 / 自定义 API
- **智能切换** — 根据文件数量和路径长度自动选择本地/云端

## 技术栈

**前端**
- Vue 3 + TypeScript + Vite 5
- TailwindCSS + Pinia + Vue Router
- Lucide Icons + Recharts

**后端**
- Rust + Tauri 2.0
- Tokio 异步运行时
- rusqlite (SQLite 持久化)
- windows-rs (Win32 API: 注册表、USN 日志、Authenticode)
- ort (ONNX Runtime 本地推理)
- aes-gcm (备份加密)
- xxhash-rust (高速文件指纹)
- reqwest (云端 AI 通信)

## 开发

```bash
# 安装前端依赖
npm install

# 开发模式
npm run tauri dev

# 构建发布版
npm run tauri build
```

### 环境要求
- Node.js 20+
- Rust (stable)
- Windows 10/11

## CI/CD

| 工作流 | 触发条件 | 说明 |
|--------|----------|------|
| CI | push / PR 到 master | cargo check + 前端构建 + Release 构建 |
| Release | 推送 `v*` 标签 | 自动构建 exe/msi 并创建 GitHub Release |

```bash
# 发布新版本
git tag v0.1.0
git push origin v0.1.0
```

## 截图

> 仪表盘 — C盘空间概览、安全评分、可清理空间

> 扫描结果 — 分类筛选、多选清理、大小统计

> 设置 — AI 配置、通用设置、白名单管理

## License

MIT
