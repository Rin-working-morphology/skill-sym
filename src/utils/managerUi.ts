import type {
  PublishMode,
  PublishTargetStatus,
  SkillEntry,
  TargetId,
  TargetOption,
} from "../types/manager";

export function skillKindLabel(kind: SkillEntry["kind"]) {
  const labels: Record<SkillEntry["kind"], string> = {
    directory: "文件夹",
    file: "文件",
    symlink: "链接",
    other: "其他",
  };

  return labels[kind];
}

export function publishModeLabel(mode: PublishMode) {
  return mode === "symlink" ? "相对链接" : "物理复制";
}

export function updateStatusLabel(status?: string) {
  if (!status) {
    return "未知";
  }

  const labels: Record<string, string> = {
    notConfigured: "未配置",
    ready: "已就绪",
    checking: "检查中",
    available: "发现新版",
    downloading: "下载中",
    installing: "安装中",
    installed: "已安装",
    current: "已是最新",
    noRelease: "暂无发布",
    failed: "检查失败",
  };

  return labels[status] ?? "未知状态";
}

export function toolMetaForTarget(target: PublishTargetStatus): TargetOption {
  const meta = targetMeta[target.id] ?? {
    shortLabel: targetInitial(target.name),
    iconSrc: null,
    tone: "generic" as const,
  };

  return {
    id: target.id,
    path: target.baseFolder,
    skillsPath: target.skillsPath,
    name: target.name,
    folderName: target.folderName,
    shortLabel: meta.shortLabel,
    iconSrc: meta.iconSrc,
    tone: meta.tone,
    enabled: target.enabled,
    isSource: target.isSource,
    protectsSourceChildren: target.protectsSourceChildren,
    hasSkillsFolder: target.hasSkillsFolder,
    installedSkillNames: target.installedSkillNames,
  };
}

function targetInitial(name: string) {
  return Array.from(name.trim())[0]?.toUpperCase() ?? "?";
}

const targetMeta: Record<
  TargetId,
  Pick<TargetOption, "shortLabel" | "iconSrc" | "tone">
> = {
  claude: {
    shortLabel: "CL",
    iconSrc: "/claude.svg",
    tone: "claude",
  },
  codex: {
    shortLabel: "CX",
    iconSrc: "/codex.svg",
    tone: "codex",
  },
  gemini: {
    shortLabel: "GM",
    iconSrc: "/gemini.svg",
    tone: "gemini",
  },
  qoder: {
    shortLabel: "QD",
    iconSrc: "/qoder.svg",
    tone: "qoder",
  },
  trae: {
    shortLabel: "TR",
    iconSrc: "/trae.svg",
    tone: "trae",
  },
  codebuddy: {
    shortLabel: "CB",
    iconSrc: "/codebuddy.svg",
    tone: "codebuddy",
  },
};
