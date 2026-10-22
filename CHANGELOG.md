# 更新日志

## [0.1.5] - 2026-10-22

### 新增

- **发布目标扩展到 9 个 agent**：在原有 Claude、Codex、Gemini、Qoder、Trae、CodeBuddy 的基础上新增 OpenCode、Pi、DeepSeek，并为每个 agent 分别定义工作区目录、全局目录候选与安装探测目录。Trae 额外兼容 `.trae-cn`。
- **自动探测已安装的 agent**：启动时检查用户主目录中是否存在对应 agent 的配置目录。管理页只渲染「已启用且本机已检测到」的目标；设置页保留全部已知目标，未检测到的项灰显、标注「未检测到」且不可启用；一个都没检测到时，技能列表旁会提示「未检测到已安装的 coding agent」。
- **新 agent 首次自动启用**：新增 `known_detected_target_ids`，只记录已经见过的 agent。这样新装一个 agent 时它会自动出现，而用户手动关闭过的目标不会在重启后被重新打开。
- **物理复制差异提示**：当发布方式为「物理复制」时，扫描源目录与各目标目录中同名技能的副本，比对 `SKILL.md` 内容（单文件技能比对整个文件）以及其余文件的文件树结构（相对路径 + 文件大小），并在技能名前显示「有差异」徽标，悬浮可查看是内容差异、文件树差异还是两者都有。
  - 只统计真正的物理副本：符号链接副本跟随源目录，天然一致，不参与比对；源目录本身也作为一份副本参与，因此只发布到部分目标时同样能发现差异。
  - 同一路径（例如全局源就是 `.agents`）不会被重复计数；不足两份副本的技能不做提示。
- **Arch Linux AppImage 产物**：新增在 `archlinux:base-devel` 容器中构建的 AppImage，链接滚动版的 `webkit2gtk`，并重命名为 `SkillSym_<version>_archlinux_amd64.AppImage` 上传到同一个 Release，避免与 Ubuntu 22.04 构建的同名产物冲突。
- **动态的全局根目录快捷按钮**：预设按钮改为根据本机检测到的 agent 动态生成（如 `.claude`、`.config/opencode`、`.pi/agent`、`.dsh`），并始终保留 `.agents`。

### 变更

- **默认基础目录由 `.claude` 改为 `.agents`**（共享的 Agent Skills 目录），适合被多个 agent 直接复用。已保存过 `globalBaseFolder` 的老用户不会被强制改写，原有技能来源不受影响。
- **Codex 的目录约定更新**：全局与工作区均使用 `.agents`，`.codex` 仅作为安装探测目录。
- **全局目录按候选顺序解析**：每个 agent 可声明多个全局目录候选，取本机实际存在的第一个；Trae 优先 `.trae`、其次 `.trae-cn`，Pi 固定使用 `~/.pi/agent`，OpenCode 只使用 `~/.config/opencode`（`~/.opencode` 在某些环境是安装/二进制目录）。
- **基础目录允许相对多级路径**：现在可以填写 `.config/opencode`、`.pi/agent` 这类嵌套目录，仍然拒绝绝对路径和 `..` 越界。
- 设置页拆分为「通用」「发布目标」「关于」三个独立组件，界面行为保持不变。

### 修复

- **Firefox 深浅色下的图标配色**：不再依赖 SVG 内部的 `prefers-color-scheme`（Firefox 在 `<img>` 中不会求值，而且它跟随的是系统主题而非应用内的 `data-theme`），改为在暗色主题下对 codex / pi / trae 单色图标做反相处理。

### 样式

- 差异徽标配色由弱化文字色改为强调色。
- 补充 Pi、DeepSeek、OpenCode 图标资源，README 截图同步更新。

### 文档

- README 新增 9 个 agent 的「探测目录 / 全局技能目录 / 工作区技能目录」对照表，补充 `.agents` 默认约定说明，并附上各 agent 官方文档与 Agent Skills 规范的参考链接。
