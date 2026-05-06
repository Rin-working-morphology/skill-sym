export type ScopeKind = "global" | "workspace";
export type PublishMode = "symlink" | "copy";
export type ViewMode = "manager" | "settings";
export type ThemeMode = "light" | "dark";
export type TargetId =
  | "claude"
  | "codex"
  | "qoder"
  | "gemini"
  | "trae"
  | "codebuddy";
export type BaseFolderPreset =
  | ".claude"
  | ".codex"
  | ".qoder"
  | ".trae"
  | ".codebuddy";
export type ToolTone =
  | "claude"
  | "codex"
  | "qoder"
  | "gemini"
  | "trae"
  | "codebuddy"
  | "generic";

export interface Workspace {
  id: string;
  name: string;
  path: string;
  baseFolder: string;
}

export interface AppState {
  globalBaseFolder: string;
  workspaces: Workspace[];
  targetBaseFolders: string[];
  enabledTargetIds: TargetId[];
  defaultPublishMode: PublishMode;
}

export interface ScopeSelection {
  kind: ScopeKind;
  workspaceId?: string;
}

export interface SkillEntry {
  name: string;
  path: string;
  kind: "directory" | "file" | "symlink" | "other";
}

export interface ScanResult {
  skillsPath: string;
  exists: boolean;
  skills: SkillEntry[];
}

export interface PublishTargetStatus {
  id: TargetId;
  name: string;
  folderName: string;
  baseFolder: string;
  skillsPath: string;
  enabled: boolean;
  isSource: boolean;
  protectsSourceChildren: boolean;
  hasSkillsFolder: boolean;
  installedSkillNames: string[];
}

export interface TargetScanResult {
  targets: PublishTargetStatus[];
}

export interface OperationResult {
  message: string;
  source: string;
  target: string;
}

export interface DeleteSkillRequest {
  scope: ScopeSelection;
  skillName: string;
}

export interface UpdateStatus {
  status:
    | "checking"
    | "available"
    | "downloading"
    | "installing"
    | "installed"
    | "current"
    | "noRelease"
    | "failed"
    | string;
  endpoint?: string | null;
  message: string;
  integrationNote: string;
  currentVersion?: string | null;
  latestVersion?: string | null;
  releaseName?: string | null;
  releaseUrl?: string | null;
  downloadUrl?: string | null;
  assetName?: string | null;
  publishedAt?: string | null;
}

export interface ScopeOption {
  key: string;
  kind: ScopeKind;
  label: string;
  path: string;
  detail: string;
  scope: ScopeSelection;
  baseFolder: string;
}

export interface WorkspaceGitDetail {
  path: string;
  detail: string;
}

export interface TargetOption {
  id: TargetId;
  path: string;
  skillsPath: string;
  name: string;
  folderName: string;
  shortLabel: string;
  iconSrc: string | null;
  tone: ToolTone;
  enabled: boolean;
  isSource: boolean;
  protectsSourceChildren: boolean;
  hasSkillsFolder: boolean;
  installedSkillNames: string[];
}
