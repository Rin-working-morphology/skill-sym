# SkillSym

一个用于管理和发布 AI Coding Skills 的本地桌面工具。SkillSym 可以扫描全局目录或项目工作区内的 `skills` 内容，并将它们发布到 Claude、Codex、Gemini、OpenCode、Qoder、Trae、CodeBuddy、Pi、DeepSeek 等工具约定的本地目录中。

项目基于 **Tauri 2 + Vue 3 + TypeScript** 构建，前端负责管理界面，Rust 后端负责文件系统扫描、复制、符号链接、清理和本地状态持久化。

## 功能特性

- 默认使用更通用的 Agent Skills 目录 `~/.agents/skills`
- 自动探测用户主目录下已安装的 coding agent，只为已检测到的 agent 渲染发布目标
- 添加多个项目工作区，并为每个工作区配置独立的技能基础目录
- 扫描技能条目，支持识别文件、文件夹、符号链接和其他类型
- 将单个技能或整个 `skills` 目录发布到多个目标工具
- 支持两种发布方式：
  - **相对链接**：创建相对符号链接，适合本机复用和保持源内容同步
  - **物理复制**：复制文件或文件夹，适合需要独立副本的场景
- 支持启用或关闭发布目标：Claude、Codex、Gemini、OpenCode、Qoder、Trae、CodeBuddy、Pi、DeepSeek
- 删除源技能时同步清理已发布到目标目录中的同名技能
- 支持浅色/深色主题
- 提供工作区快捷打开、工作区移除、基础目录切换等桌面操作

## Agent 探测与目录约定

应用会检查用户主目录中的 agent 配置目录。只有检测到安装的 agent 才会出现在管理页的目标图标中；设置页会保留全部已知 agent，并把未检测到的项标记为“未检测到”。

| 工具      | 探测目录（用户主目录）         | 全局技能目录                              | 工作区技能目录     |
| --------- | ------------------------------ | ----------------------------------------- | ------------------ |
| Claude    | `.claude`                      | `~/.claude/skills`                        | `.claude/skills`   |
| Codex     | `.codex`                       | `~/.agents/skills`                        | `.agents/skills`   |
| Gemini    | `.gemini`                      | `~/.gemini/skills`                        | `.gemini/skills`   |
| OpenCode  | `.config/opencode`、`.opencode` | `~/.config/opencode/skills`               | `.opencode/skills` |
| Qoder     | `.qoder`                       | `~/.qoder/skills`                         | `.qoder/skills`    |
| Trae      | `.trae`、`.trae-cn`            | `~/.trae/skills` 或 `~/.trae-cn/skills`   | `.trae/skills`     |
| CodeBuddy | `.codebuddy`                   | `~/.codebuddy/skills`                     | `.codebuddy/skills`|
| Pi        | `.pi`                          | `~/.pi/agent/skills`                      | `.pi/skills`       |
| DeepSeek  | `.dsh`                         | `~/.dsh/skills`                           | `.dsh/skills`      |

> 说明：Codex 新版使用共享的 `.agents/skills` 目录；Gemini、OpenCode、Pi 和 DeepSeek 同时兼容 `~/.agents/skills` 与 `.agents/skills`。因此把默认来源设为 `~/.agents/skills` 可以被多个 agent 直接复用。用户也可以随时在设置页把全局根目录切换为任意 agent 目录。

SkillSym 以“基础目录 + `skills` 子目录”的形式读取技能。路径由系统原生路径 API 处理，Windows、macOS、Linux 均可使用。

例如：

```text
# Windows
C:\Users\Rin\.agents\skills
D:\ProjectA\.opencode\skills

# macOS / Linux
/Users/rin/.agents/skills
/home/rin/ProjectA/.opencode/skills
```

全局范围下，目标目录位于用户主目录下。工作区范围下，目标目录位于对应工作区根目录下。自定义发布目标仍然使用“基础目录 + `skills`”的约定。

## 使用方式

### 全局技能

启动应用后，左侧会显示“全局技能”。默认全局基础目录为：

```text
# Windows
%USERPROFILE%\.agents

# macOS / Linux
~/.agents
```

应用会读取其中的：

```text
# Windows
%USERPROFILE%\.agents\skills

# macOS / Linux
~/.agents/skills
```

如果目录不存在，应用会在初始化时创建全局 `skills` 目录。

设置页的快捷目录按钮会根据本机检测到的 agent 动态生成，例如 `.claude`、`.config/opencode`、`.pi/agent`、`.dsh`，并始终保留 `.agents`。

### 工作区技能

点击左侧的“添加工作区”，选择一个项目目录后，应用会记录该工作区，并默认使用：

```text
# Windows
<workspace>\.agents\skills

# macOS / Linux
<workspace>/.agents/skills
```

作为该工作区的技能来源。你也可以通过工作区旁的编辑按钮，改成 `.claude`、`.opencode`、`.qoder`、`.trae`、`.codebuddy`、`.pi`、`.dsh` 或任意已有目录。

### 发布技能

在技能列表中：

- `skills` 根目录行用于发布或移除整个 `skills` 文件夹
- 单个技能行用于发布或移除对应技能
- Claude、Codex、Gemini、OpenCode、Qoder、Trae、CodeBuddy、Pi、DeepSeek 图标表示目标工具
- 图标上出现状态点表示该技能或目录已经发布到对应目标
- 只有已启用且本机已检测到的 agent 才会显示图标

发布前可以在顶部切换发布方式：

- **链接**：创建相对符号链接
- **复制**：复制实际文件

在 Windows 上使用符号链接时，需要启用开发者模式，或以提升权限运行应用。macOS 和 Linux 通常不需要额外授权，但目标目录必须允许当前用户写入。

## 设置

设置页包含三类配置：

- **通用**：配置全局根目录、默认发布方式、界面主题
- **发布目标**：查看本机检测状态并启用/停用 agent，也可以添加自定义发布目标
- **关于**：查看应用名称、版本、更新状态和一键更新入口

更新流程使用 Tauri 2 updater，从 GitHub Releases 的 `latest.json` 检查签名更新包，并在应用内完成下载、安装和重启。

## 本地状态

应用状态会保存在 Tauri 的应用数据目录中，文件名为：

```text
skillsym-state.json
```

状态内容包括：

- 全局基础目录
- 已添加的工作区
- 每个工作区的技能基础目录
- 已启用的发布目标
- 自定义发布目标
- 默认发布方式

主题设置保存在浏览器侧 `localStorage` 中。

## 参考文档

- [Claude Code skills](https://code.claude.com/docs/en/skills)
- [Codex skills](https://developers.openai.com/codex/skills)
- [Gemini CLI agent skills](https://geminicli.com/docs/cli/using-agent-skills/)
- [OpenCode agent skills](https://opencode.ai/docs/en/skills/)
- [CodeBuddy Code skills](https://www.codebuddy.cn/docs/cli/skills)
- [Qoder CLI skills](https://docs.qoder.com/cli/Skills)
- [Trae skills](https://docs.trae.ai/ide/skills)
- [Pi skills](https://pi.dev/docs/latest/skills)
- [Agent Skills 规范](https://agentskills.io)
- [Vercel skills：各 agent 目录对照表](https://github.com/vercel-labs/skills/blob/main/docs/agents.md)

## 技术栈

- 桌面框架：Tauri 2
- 前端框架：Vue 3
- 构建工具：Vite
- 语言：TypeScript、Rust
- 包管理：Bun
- Tauri 插件：
  - `@tauri-apps/plugin-dialog`
  - `@tauri-apps/plugin-opener`
  - `@tauri-apps/plugin-process`
  - `@tauri-apps/plugin-updater`

## 项目结构

```text
.
├─ public/                 # 工具图标等静态资源
├─ src/                    # Vue 前端
│  ├─ assets/              # 本地 SVG 图标
│  ├─ components/          # 管理页、设置页、侧边栏、通知组件
│  ├─ composables/         # 前端状态和 Tauri 命令封装
│  ├─ styles/              # 全局样式和设计令牌
│  ├─ types/               # 前端类型定义
│  └─ utils/               # UI 文案和目标工具元数据
├─ src-tauri/              # Tauri / Rust 后端
│  ├─ capabilities/        # Tauri 权限配置
│  ├─ icons/               # 应用图标
│  └─ src/
│     ├─ commands/         # Tauri 命令入口
│     ├─ filesystem/       # 扫描、发布、复制、链接、删除逻辑
│     ├─ models/           # Rust 数据结构
│     ├─ state/            # agent 探测、目标定义、状态读取和迁移
│     └─ paths.rs          # 路径校验和工具函数
├─ DESIGN.md               # 设计系统说明
├─ PRODUCT.md              # 产品和视觉方向
├─ package.json            # 前端脚本和依赖
└─ vite.config.ts          # Vite 配置
```

## 开发环境

建议环境：

- Windows 11、macOS 或主流 Linux 发行版
- Node.js 22+
- Bun
- Rust stable
- Tauri 2 所需系统依赖

安装依赖：

```bash
bun install
```

启动前端开发服务器：

```bash
bun run dev
```

启动 Tauri 桌面应用：

```bash
bun run tauri dev
```

构建前端：

```bash
bun run build
```

构建当前系统的桌面安装包：

```bash
bun run tauri build
```

Linux 本地构建前需要安装 WebKitGTK 等系统依赖。Arch Linux 示例：

```bash
sudo pacman -Syu
sudo pacman -S --needed webkit2gtk-4.1 libappindicator-gtk3 librsvg patchelf
```

macOS 本地构建前需要安装 Xcode Command Line Tools：

```bash
xcode-select --install
```

## 可用脚本

| 命令                  | 说明                           |
| --------------------- | ------------------------------ |
| `bun run dev`         | 启动 Vite 开发服务器           |
| `bun run build`       | 执行 TypeScript 检查并构建前端 |
| `bun run preview`     | 预览前端构建产物               |
| `bun run tauri dev`   | 启动 Tauri 开发模式            |
| `bun run tauri build` | 构建 Tauri 桌面应用            |
