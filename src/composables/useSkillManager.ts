import { computed, ref } from "vue";
import { getName, getVersion } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";
import {
  confirm as confirmDialog,
  open as openDialog,
} from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";

import type {
  AppState,
  BaseFolderPreset,
  DeleteSkillRequest,
  OperationResult,
  PublishMode,
  ScanResult,
  ScopeOption,
  ScopeSelection,
  TargetId,
  TargetOption,
  ThemeMode,
  TargetScanResult,
  UpdateStatus,
  ViewMode,
  Workspace,
  WorkspaceGitDetail,
} from "../types/manager";
import { publishModeLabel, toolMetaForTarget } from "../utils/managerUi";

const THEME_MODE_STORAGE_KEY = "skillsym-theme-mode";
const RELEASES_PAGE_URL =
  "https://github.com/Rin-working-morphology/skill-sym/releases";
const LATEST_RELEASE_API_URL =
  "https://api.github.com/repos/Rin-working-morphology/skill-sym/releases/latest";

interface GitHubReleaseAsset {
  name: string;
  browser_download_url: string;
}

interface GitHubRelease {
  name: string | null;
  tag_name: string;
  html_url: string;
  published_at: string | null;
  assets: GitHubReleaseAsset[];
}

function readStoredThemeMode(): ThemeMode {
  if (typeof window === "undefined") {
    return "light";
  }

  try {
    return window.localStorage.getItem(THEME_MODE_STORAGE_KEY) === "dark"
      ? "dark"
      : "light";
  } catch {
    return "light";
  }
}

function applyThemeMode(mode: ThemeMode) {
  if (typeof document === "undefined") {
    return;
  }

  document.documentElement.dataset.theme = mode;
  document.documentElement.style.colorScheme = mode;
}

function normalizeVersion(version: string) {
  return version.trim().replace(/^[^\d]*/, "").split(/[+-]/)[0];
}

function compareVersions(left: string, right: string) {
  const leftParts = normalizeVersion(left).split(".").map(Number);
  const rightParts = normalizeVersion(right).split(".").map(Number);
  const length = Math.max(leftParts.length, rightParts.length);

  for (let index = 0; index < length; index += 1) {
    const leftPart = Number.isFinite(leftParts[index]) ? leftParts[index] : 0;
    const rightPart = Number.isFinite(rightParts[index]) ? rightParts[index] : 0;

    if (leftPart !== rightPart) {
      return leftPart > rightPart ? 1 : -1;
    }
  }

  return 0;
}

function selectInstallerAsset(assets: GitHubReleaseAsset[]) {
  return (
    assets.find((asset) => /_x64_zh-CN\.msi$/i.test(asset.name)) ??
    assets.find((asset) => /\.msi$/i.test(asset.name)) ??
    assets.find((asset) => /setup.*\.exe$/i.test(asset.name)) ??
    null
  );
}

function formatReleaseDate(value?: string | null) {
  if (!value) {
    return null;
  }

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return date.toLocaleString("zh-CN", {
    dateStyle: "medium",
    timeStyle: "short",
  });
}

export function useSkillManager() {
  const state = ref<AppState | null>(null);
  const scan = ref<ScanResult | null>(null);
  const targetScan = ref<TargetScanResult | null>(null);
  const updateStatus = ref<UpdateStatus | null>(null);
  const workspaceGitDetails = ref<Record<string, string>>({});
  const appName = ref("SkillSym");
  const appVersion = ref("");
  const viewMode = ref<ViewMode>("manager");
  const selectedScopeKey = ref("global");
  const selectedSkillName = ref("");
  const themeMode = ref<ThemeMode>(readStoredThemeMode());
  const busy = ref(false);
  const statusMessage = ref("");
  const errorMessage = ref("");

  applyThemeMode(themeMode.value);

  const scopeOptions = computed<ScopeOption[]>(() => {
    if (!state.value) {
      return [];
    }

    return [
      {
        key: "global",
        kind: "global",
        label: "全局技能",
        path: state.value.globalBaseFolder,
        detail: state.value.globalBaseFolder,
        scope: { kind: "global" },
        baseFolder: state.value.globalBaseFolder,
      },
      ...state.value.workspaces.map((workspace) => ({
        key: `workspace:${workspace.id}`,
        kind: "workspace" as const,
        label: workspace.name,
        path: workspace.path,
        detail: workspaceGitDetails.value[workspace.path] ?? "",
        scope: { kind: "workspace" as const, workspaceId: workspace.id },
        baseFolder: workspace.baseFolder,
      })),
    ];
  });

  const workspaceOptions = computed(() =>
    scopeOptions.value.filter((option) => option.kind === "workspace"),
  );

  const activeScope = computed(() => {
    return (
      scopeOptions.value.find((option) => option.key === selectedScopeKey.value) ??
      scopeOptions.value[0]
    );
  });

  const allTargetOptions = computed<TargetOption[]>(() =>
    (targetScan.value?.targets ?? []).map((target) => toolMetaForTarget(target)),
  );

  const targetOptions = computed(() =>
    allTargetOptions.value.filter((target) => target.enabled),
  );

  const selectedSkill = computed(() => {
    return scan.value?.skills.find((skill) => skill.name === selectedSkillName.value);
  });

  const publishMode = computed<PublishMode>(
    () => state.value?.defaultPublishMode ?? "symlink",
  );

  async function initialize() {
    await Promise.all([loadState(), loadAppMeta()]);
    await Promise.all([refreshScopeData(), refreshUpdateStatus()]);
  }

  async function command<T>(name: string, args?: Record<string, unknown>): Promise<T> {
    return invoke<T>(name, args);
  }

  async function runAction(
    action: () => Promise<boolean | void>,
    successMessage?: string,
  ) {
    busy.value = true;
    errorMessage.value = "";
    statusMessage.value = "";

    try {
      const completed = await action();
      if (completed !== false && successMessage) {
        statusMessage.value = successMessage;
      }
    } catch (error) {
      errorMessage.value = error instanceof Error ? error.message : String(error);
    } finally {
      busy.value = false;
    }
  }

  async function loadState() {
    state.value = await command<AppState>("load_manager_state");
    await refreshWorkspaceGitDetails();

    if (!scopeOptions.value.some((option) => option.key === selectedScopeKey.value)) {
      selectedScopeKey.value = "global";
    }
  }

  async function refreshWorkspaceGitDetails() {
    const workspaces = state.value?.workspaces ?? [];
    if (!workspaces.length) {
      workspaceGitDetails.value = {};
      return;
    }

    const details = await command<WorkspaceGitDetail[]>("scan_workspace_git_details", {
      paths: workspaces.map((workspace) => workspace.path),
    });

    workspaceGitDetails.value = Object.fromEntries(
      details.map((detail) => [detail.path, detail.detail]),
    );
  }

  async function loadAppMeta() {
    const [nameResult, versionResult] = await Promise.allSettled([
      getName(),
      getVersion(),
    ]);

    if (nameResult.status === "fulfilled") {
      appName.value = nameResult.value;
    }

    if (versionResult.status === "fulfilled") {
      appVersion.value = versionResult.value;
    }
  }

  async function refreshSkills() {
    if (!activeScope.value) {
      scan.value = null;
      return;
    }

    scan.value = await command<ScanResult>("scan_skills", {
      scope: activeScope.value.scope,
    });

    if (!scan.value.skills.some((skill) => skill.name === selectedSkillName.value)) {
      selectedSkillName.value = scan.value.skills[0]?.name ?? "";
    }
  }

  async function refreshPublishTargets() {
    if (!activeScope.value) {
      targetScan.value = null;
      return;
    }

    targetScan.value = await command<TargetScanResult>("scan_publish_targets", {
      scope: activeScope.value.scope,
    });
  }

  async function refreshScopeData() {
    await Promise.all([refreshSkills(), refreshPublishTargets()]);
  }

  async function refreshUpdateStatus() {
    const currentVersion =
      appVersion.value ||
      (await getVersion().catch(() => "")) ||
      "";

    updateStatus.value = {
      status: "checking",
      endpoint: LATEST_RELEASE_API_URL,
      currentVersion,
      message: "正在检查 GitHub 最新发布版本...",
      integrationNote: "通过 GitHub Releases latest 接口检查版本。",
    };

    try {
      const response = await fetch(LATEST_RELEASE_API_URL, {
        headers: {
          Accept: "application/vnd.github+json",
        },
      });

      if (response.status === 404) {
        updateStatus.value = {
          status: "noRelease",
          endpoint: LATEST_RELEASE_API_URL,
          currentVersion,
          releaseUrl: RELEASES_PAGE_URL,
          message: "GitHub 上还没有可用发布版本。",
          integrationNote: "创建 release 后这里会自动读取最新 tag 和安装包。",
        };
        return;
      }

      if (!response.ok) {
        throw new Error(`GitHub 返回 ${response.status}`);
      }

      const release = (await response.json()) as GitHubRelease;
      const latestVersion = normalizeVersion(release.tag_name);
      const installerAsset = selectInstallerAsset(release.assets ?? []);
      const hasNewVersion =
        currentVersion && latestVersion
          ? compareVersions(latestVersion, currentVersion) > 0
          : false;

      updateStatus.value = {
        status: hasNewVersion ? "available" : "current",
        endpoint: LATEST_RELEASE_API_URL,
        currentVersion,
        latestVersion,
        releaseName: release.name,
        releaseUrl: release.html_url,
        downloadUrl: installerAsset?.browser_download_url ?? null,
        assetName: installerAsset?.name ?? null,
        publishedAt: formatReleaseDate(release.published_at),
        message: hasNewVersion
          ? `发现新版本 ${latestVersion}，当前版本 ${currentVersion || "-"}。`
          : `当前已是最新版本 ${currentVersion || latestVersion || "-"}。`,
        integrationNote: installerAsset
          ? `已匹配安装包：${installerAsset.name}`
          : "该 release 没有找到安装包资产，可打开发布页查看。",
      };
    } catch (error) {
      updateStatus.value = {
        status: "failed",
        endpoint: LATEST_RELEASE_API_URL,
        currentVersion,
        releaseUrl: RELEASES_PAGE_URL,
        message: "检查更新失败。",
        integrationNote: error instanceof Error ? error.message : String(error),
      };
    }
  }

  async function openReleasePage() {
    await runAction(async () => {
      await openUrl(updateStatus.value?.releaseUrl ?? RELEASES_PAGE_URL);
    });
  }

  async function openLatestDownload() {
    const url = updateStatus.value?.downloadUrl ?? updateStatus.value?.releaseUrl;
    if (!url) {
      return;
    }

    await runAction(async () => {
      await openUrl(url);
    });
  }

  async function pickDirectory(): Promise<string | null> {
    const picked = await openDialog({
      directory: true,
      multiple: false,
    });

    return typeof picked === "string" ? picked : null;
  }

  async function openFolderInExplorer(path: string) {
    if (!path) {
      return;
    }

    await runAction(async () => {
      await command<void>("open_managed_folder", { path });
    });
  }

  async function addWorkspace() {
    await runAction(async () => {
      const path = await pickDirectory();
      if (!path) {
        return false;
      }

      const previousIds = new Set(state.value?.workspaces.map((workspace) => workspace.id) ?? []);
      state.value = await command<AppState>("add_workspace", { path });
      await refreshWorkspaceGitDetails();
      viewMode.value = "manager";

      const workspace =
        state.value.workspaces.find((item) => item.path === path) ??
        state.value.workspaces.find((item) => !previousIds.has(item.id));

      if (workspace) {
        selectedScopeKey.value = `workspace:${workspace.id}`;
      }

      await refreshScopeData();
      return true;
    }, "工作区已保存");
  }

  async function removeWorkspace(workspace: Workspace) {
    await runAction(async () => {
      state.value = await command<AppState>("remove_workspace", {
        id: workspace.id,
      });
      await refreshWorkspaceGitDetails();
      selectedScopeKey.value = "global";
      await refreshScopeData();
    }, "工作区已移除");
  }

  async function selectScope(key: string) {
    selectedScopeKey.value = key;
    viewMode.value = "manager";
    await runAction(async () => {
      await refreshScopeData();
    });
  }

  async function setQuickBase(folderName: BaseFolderPreset) {
    if (!activeScope.value) {
      return;
    }

    await setQuickBaseForScope(activeScope.value.scope, folderName);
  }

  async function setQuickBaseForScope(
    scope: ScopeSelection,
    folderName: BaseFolderPreset,
  ) {
    await runAction(async () => {
      state.value = await command<AppState>("set_scope_base_child", {
        scope,
        folderName,
      });
      await refreshScopeData();
    }, `${folderName} 已设为当前基础目录`);
  }

  async function chooseScopeBaseFolder(
    scope: ScopeSelection = activeScope.value?.scope ?? { kind: "global" },
  ) {
    await runAction(async () => {
      const baseFolder = await pickDirectory();
      if (!baseFolder) {
        return false;
      }

      state.value = await command<AppState>("set_scope_base_folder", {
        scope,
        baseFolder,
      });
      await refreshScopeData();
      return true;
    }, "基础目录已保存");
  }

  async function setDefaultPublishMode(mode: PublishMode) {
    if (publishMode.value === mode) {
      return;
    }

    await runAction(async () => {
      state.value = await command<AppState>("set_default_publish_mode", { mode });
    }, `默认方式已切换为${publishModeLabel(mode)}`);
  }

  function setThemeMode(mode: ThemeMode) {
    if (themeMode.value === mode) {
      return;
    }

    themeMode.value = mode;
    try {
      window.localStorage.setItem(THEME_MODE_STORAGE_KEY, mode);
    } catch {
      // Keep the visible theme change even when storage is unavailable.
    }
    applyThemeMode(mode);
  }

  function setViewMode(mode: ViewMode) {
    viewMode.value = mode;
  }

  function selectSkill(name: string) {
    selectedSkillName.value = name;
  }

  function removeWorkspaceOption(option: ScopeOption) {
    const workspace = state.value?.workspaces.find(
      (item) => `workspace:${item.id}` === option.key,
    );

    if (workspace) {
      void removeWorkspace(workspace);
    }
  }

  async function togglePublishTarget(targetId: TargetId) {
    if (!state.value) {
      return;
    }

    const current = new Set(state.value.enabledTargetIds);
    if (current.has(targetId)) {
      current.delete(targetId);
    } else {
      current.add(targetId);
    }

    await runAction(async () => {
      state.value = await command<AppState>("set_enabled_publish_targets", {
        targetIds: [...current],
      });
      await refreshPublishTargets();
    }, "发布目标已更新");
  }

  function isSkillPublished(skillName: string, target: TargetOption) {
    return target.installedSkillNames.includes(skillName);
  }

  function isFolderPublished(target: TargetOption) {
    return target.hasSkillsFolder;
  }

  async function publishWholeFolderToTarget(payload: {
    targetBaseFolder: string;
    currentPublished: boolean;
    targetName: string;
  }) {
    if (!activeScope.value) {
      return;
    }

    const actionText = payload.currentPublished ? "移除" : "发布";
    const confirmed = await confirmDialog(
      payload.currentPublished
        ? `确认从 ${payload.targetName} 移除整包 skills？`
        : `确认发布整包 skills 到 ${payload.targetName}？目标中的 skills 文件或文件夹会被覆盖。`,
      { title: "确认操作", kind: "warning" },
    );

    if (!confirmed) {
      return;
    }

    await runAction(async () => {
      const result = payload.currentPublished
        ? await command<OperationResult>("remove_published_skills_folder", {
            request: {
              scope: activeScope.value.scope,
              targetBaseFolder: payload.targetBaseFolder,
            },
          })
        : await command<OperationResult>("publish_skills_folder", {
            request: {
              scope: activeScope.value.scope,
              targetBaseFolder: payload.targetBaseFolder,
              mode: publishMode.value,
            },
          });
      statusMessage.value = `${actionText}完成：${result.target}`;
      await refreshPublishTargets();
    });
  }

  async function toggleSkillTarget(
    skillName: string,
    targetBaseFolder: string,
    currentPublished: boolean,
  ) {
    if (!activeScope.value) {
      return;
    }

    selectedSkillName.value = skillName;

    await runAction(async () => {
      const result = currentPublished
        ? await command<OperationResult>("remove_published_skill", {
            request: {
              scope: activeScope.value.scope,
              skillName,
              targetBaseFolder,
            },
          })
        : await command<OperationResult>("publish_skill", {
            request: {
              scope: activeScope.value.scope,
              skillName,
              targetBaseFolder,
              mode: publishMode.value,
            },
          });
      statusMessage.value = `${result.message} ${result.target}`;
      await refreshPublishTargets();
    });
  }

  async function deleteSkill(skillName: string) {
    if (!activeScope.value) {
      return;
    }

    const confirmed = await confirmDialog(
      `确认删除 ${skillName}？这会移除各目标目录中的同名发布项，并删除当前基础目录中的源技能。`,
      { title: "确认删除技能", kind: "warning" },
    );

    if (!confirmed) {
      return;
    }

    selectedSkillName.value = skillName;

    await runAction(async () => {
      const request: DeleteSkillRequest = {
        scope: activeScope.value.scope,
        skillName,
      };
      const result = await command<OperationResult>("delete_skill", { request });
      statusMessage.value = result.message;
      await refreshScopeData();
    });
  }

  return {
    state,
    scan,
    targetScan,
    updateStatus,
    appName,
    appVersion,
    viewMode,
    selectedScopeKey,
    themeMode,
    busy,
    statusMessage,
    errorMessage,
    scopeOptions,
    workspaceOptions,
    activeScope,
    allTargetOptions,
    targetOptions,
    selectedSkill,
    publishMode,
    initialize,
    addWorkspace,
    removeWorkspaceOption,
    openFolderInExplorer,
    selectScope,
    setQuickBase,
    setQuickBaseForScope,
    chooseScopeBaseFolder,
    setDefaultPublishMode,
    setThemeMode,
    setViewMode,
    selectSkill,
    togglePublishTarget,
    isSkillPublished,
    isFolderPublished,
    publishWholeFolderToTarget,
    toggleSkillTarget,
    deleteSkill,
    refreshSkills,
    refreshPublishTargets,
    refreshScopeData,
    refreshUpdateStatus,
    openReleasePage,
    openLatestDownload,
  };
}
