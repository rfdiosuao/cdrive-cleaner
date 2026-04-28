# C 盘清理助手

基于 Tauri 2 + Vue 3 的 Windows C 盘清理工具。项目采用 Rust 后端负责扫描、评估和清理执行，Vue 前端负责桌面端交互。

## 功能概览

- 快速扫描：扫描临时文件、缓存、日志、回收站、浏览器缓存等常见可清理位置。
- 深度扫描：遍历 C 盘并按类型聚合结果，展示可释放空间和风险等级。
- 安全防护：内置系统目录保护、扩展名保护、隐私目录保护和签名校验。
- 清理预览：执行清理前生成任务预览，提示风险并支持备份策略。
- 大文件分析：定位大文件，默认要求用户人工确认。
- 软件迁移：识别可迁移软件，为后续迁移和回滚提供数据基础。
- AI 迁移建议：支持本地规则优先，也可配置云端模型增强分析。

## 技术栈

前端：

- Vue 3 + TypeScript + Vite
- Pinia + Vue Router
- Tailwind CSS
- lucide-vue-next

后端：

- Rust + Tauri 2
- Tokio
- rusqlite
- windows-rs
- ort / ONNX Runtime
- aes-gcm
- xxhash-rust
- reqwest

## 开发

环境要求：

- Windows 10/11
- Node.js 20+
- Rust stable

安装依赖：

```bash
npm ci
```

前端开发：

```bash
npm run dev
```

Tauri 开发：

```bash
npm run tauri dev
```

构建前端：

```bash
npm run build
```

检查后端：

```bash
cd src-tauri
cargo check
```

发布构建：

```bash
npm run tauri build
```

## 项目结构

```text
src/                  Vue 前端
src/api/              Tauri command 调用封装
src/components/       通用布局组件
src/stores/           Pinia 状态
src/views/            页面
src-tauri/            Tauri/Rust 后端
src-tauri/src/        扫描、清理、安全、配置等核心逻辑
```

## 当前完善重点

- 保持扫描和清理链路可编译、可验证。
- 默认保护系统目录和用户隐私目录。
- 对高风险项目使用显式确认，不做自动删除。
- 前端页面围绕“扫描 -> 评估 -> 预览 -> 清理/迁移”的主流程组织。

## License

MIT
