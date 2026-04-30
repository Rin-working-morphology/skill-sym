<template>
  <section class="settings-view">
    <header class="settings-topbar">
      <div class="settings-title-row">
        <button
          type="button"
          class="settings-back-button"
          :disabled="busy"
          aria-label="返回管理页"
          title="返回管理页"
          @click="emit('back')"
        >
          <span class="svg-icon icon-back" aria-hidden="true"></span>
        </button>

        <div>
          <h1>设置</h1>
        </div>
      </div>
    </header>

    <div class="settings-layout">
      <nav class="settings-tabs" aria-label="设置分类">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          type="button"
          :class="{ active: activeTab === tab.id }"
          @click="activeTab = tab.id"
        >
          {{ tab.label }}
        </button>
      </nav>

      <Transition name="settings-panel-swap" mode="out-in">
        <section
          v-if="activeTab === 'general'"
          key="general"
          class="settings-panel"
          aria-labelledby="settings-general-title"
        >
          <div class="settings-section-head">
            <h2 id="settings-general-title">基础配置</h2>
          </div>

          <article class="setting-card wide">
            <div class="setting-head">
              <h3>全局根目录</h3>
              <small>默认位置</small>
            </div>
            <strong class="setting-path">{{
              state?.globalBaseFolder ?? "-"
            }}</strong>
            <div class="mini-actions">
              <button
                type="button"
                :disabled="busy"
                @click="emit('setQuickBase', '.claude')"
              >
                .claude
              </button>
              <button
                type="button"
                :disabled="busy"
                @click="emit('setQuickBase', '.codex')"
              >
                .codex
              </button>
              <button
                type="button"
                :disabled="busy"
                @click="emit('setQuickBase', '.qoder')"
              >
                .qoder
              </button>
              <button
                type="button"
                :disabled="busy"
                @click="emit('chooseGlobalBaseFolder')"
              >
                选择目录
              </button>
            </div>
          </article>

          <article class="setting-card wide">
            <div class="setting-head">
              <h3>默认复用方式</h3>
              <small>全局默认</small>
            </div>
            <div class="mode-toggle large">
              <button
                type="button"
                :class="{ active: publishMode === 'symlink' }"
                :disabled="busy"
                @click="emit('setPublishMode', 'symlink')"
              >
                相对链接
              </button>
              <button
                type="button"
                :class="{ active: publishMode === 'copy' }"
                :disabled="busy"
                @click="emit('setPublishMode', 'copy')"
              >
                物理复制
              </button>
            </div>
          </article>

          <article class="setting-card wide">
            <div class="setting-head">
              <h3>界面主题</h3>
              <small>本机显示</small>
            </div>
            <div class="mode-toggle large theme-toggle">
              <button
                type="button"
                :class="{ active: themeMode === 'light' }"
                :disabled="busy"
                @click="emit('setThemeMode', 'light')"
              >
                浅色
              </button>
              <button
                type="button"
                :class="{ active: themeMode === 'dark' }"
                :disabled="busy"
                @click="emit('setThemeMode', 'dark')"
              >
                深色
              </button>
            </div>
          </article>
        </section>

        <section
          v-else-if="activeTab === 'targets'"
          key="targets"
          class="settings-panel"
          aria-labelledby="settings-targets-title"
        >
          <div class="settings-section-head">
            <h2 id="settings-targets-title">目标启用状态</h2>
          </div>

          <article class="setting-card wide">
            <div class="target-button-group">
              <button
                v-for="target in targetOptions"
                :key="target.id"
                type="button"
                class="target-toggle"
                :class="{ active: target.enabled }"
                :aria-pressed="target.enabled"
                :disabled="busy"
                :title="target.path"
                @click="emit('togglePublishTarget', target.id)"
              >
                <span class="tool-avatar" :class="target.tone">
                  <img
                    v-if="target.iconSrc"
                    :src="target.iconSrc"
                    :alt="target.name"
                  />
                  <span v-else>{{ target.shortLabel }}</span>
                </span>
                <span>
                  <strong>{{ target.name }}</strong>
                  <small>{{ target.folderName }}</small>
                </span>
              </button>
            </div>
          </article>
        </section>

        <section
          v-else
          key="about"
          class="settings-panel"
          aria-labelledby="settings-about-title"
        >
          <div class="settings-section-head">
            <h2 id="settings-about-title">版本信息</h2>
          </div>

          <article class="setting-card wide">
            <dl class="meta-grid">
              <div>
                <dt>项目名</dt>
                <dd>{{ appName }}</dd>
              </div>
              <div>
                <dt>版本</dt>
                <dd>{{ appVersion || "-" }}</dd>
              </div>
            </dl>

            <div class="settings-actions">
              <button
                type="button"
                :disabled="busy"
                @click="emit('openReleasePage')"
              >
                更新日志
              </button>
              <button
                v-if="updateStatus?.downloadUrl"
                type="button"
                :disabled="busy"
                @click="emit('openLatestDownload')"
              >
                下载安装包
              </button>
              <button
                type="button"
                class="primary-action"
                :disabled="busy || updateStatus?.status === 'checking'"
                @click="emit('refreshUpdateStatus')"
              >
                检查更新
              </button>
            </div>
          </article>
        </section>
      </Transition>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";

import type {
  AppState,
  BaseFolderPreset,
  PublishMode,
  TargetId,
  TargetOption,
  ThemeMode,
  UpdateStatus,
} from "../types/manager";
import { updateStatusLabel } from "../utils/managerUi";

type SettingsTab = "general" | "targets" | "about";

const props = defineProps<{
  busy: boolean;
  state: AppState | null;
  publishMode: PublishMode;
  themeMode: ThemeMode;
  targetOptions: TargetOption[];
  appName: string;
  appVersion: string;
  updateStatus: UpdateStatus | null;
}>();

const emit = defineEmits<{
  back: [];
  setQuickBase: [folderName: BaseFolderPreset];
  chooseGlobalBaseFolder: [];
  setPublishMode: [mode: PublishMode];
  setThemeMode: [mode: ThemeMode];
  togglePublishTarget: [targetId: TargetId];
  refreshUpdateStatus: [];
  openReleasePage: [];
  openLatestDownload: [];
}>();

const tabs: { id: SettingsTab; label: string }[] = [
  { id: "general", label: "通用" },
  { id: "targets", label: "发布目标" },
  { id: "about", label: "关于" },
];

const activeTab = ref<SettingsTab>("general");
</script>

<style scoped>
.settings-view {
  min-width: 0;
  min-height: 0;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--bg);
  animation: settings-view-settle var(--motion-slow) var(--ease-out-expo);
}

.settings-topbar {
  min-height: 58px;
  display: flex;
  align-items: center;
  padding: 12px 18px;
  border-bottom: 1px solid var(--line);
  background: var(--surface);
}

.settings-title-row {
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 10px;
}

.settings-back-button {
  width: 30px;
  min-width: 30px;
  height: 30px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  border: 0;
  background: transparent;
  box-shadow: var(--control-shadow);
  color: var(--text-muted);
}

.settings-back-button:hover:not(:disabled) {
  background: var(--control-bg-hover);
  box-shadow: var(--control-shadow-hover);
  color: var(--text);
}

.settings-back-button .svg-icon {
  width: 17px;
  height: 17px;
}

.svg-icon {
  display: inline-block;
  background: currentColor;
  mask-position: center;
  mask-repeat: no-repeat;
  mask-size: contain;
  -webkit-mask-position: center;
  -webkit-mask-repeat: no-repeat;
  -webkit-mask-size: contain;
}

.icon-back {
  mask-image: url("../assets/back.svg");
  -webkit-mask-image: url("../assets/back.svg");
}

.settings-layout {
  min-height: 0;
  flex: 1;
  display: grid;
  grid-template-columns: var(--app-rail-width) minmax(0, 1fr);
  overflow: hidden;
}

.settings-tabs {
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 3px;
  padding: 14px 10px;
  overflow-y: auto;
  border-right: 1px solid var(--line);
  background: var(--rail-bg);
}

.settings-tabs button {
  min-height: 30px;
  display: flex;
  align-items: center;
  justify-content: flex-start;
  border-color: transparent;
  background: transparent;
  box-shadow: none;
  color: var(--text-muted);
  font-size: var(--type-button);
  text-align: left;
  transition:
    background-color var(--motion-base) var(--ease-out-quint),
    color var(--motion-base) var(--ease-out-quint),
    transform var(--motion-fast) var(--ease-out-quart);
}

.settings-tabs button:hover:not(:disabled) {
  background: color-mix(in oklch, var(--surface) 64%, transparent);
  box-shadow: none;
  color: var(--text);
}

.settings-tabs button.active {
  background: var(--surface-active);
  box-shadow: none;
  color: var(--text);
  font-weight: var(--font-label);
}

.settings-panel {
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 18px;
  overflow-x: hidden;
  overflow-y: auto;
  scrollbar-gutter: stable;
}

.settings-section-head {
  display: flex;
  flex-direction: column;
  gap: 3px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--line);
}

.meta-grid dt {
  font-size: 0.76rem;
  font-weight: var(--font-label);
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0;
}

.setting-card {
  display: flex;
  flex-direction: column;
  gap: 9px;
  max-width: 760px;
  padding: 0 0 14px;
  border-bottom: 1px solid var(--line);
  background: transparent;
}

.setting-card.wide {
  width: min(100%, 760px);
}

.setting-head {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.setting-path {
  font-size: 0.8rem;
  font-weight: var(--font-body);
  overflow-wrap: anywhere;
}

.mini-actions,
.settings-actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 7px;
}

.mode-toggle {
  display: flex;
  align-items: center;
  gap: 3px;
  padding: 2px;
  border: 0;
  border-radius: 5px;
  background: var(--surface-muted);
  width: fit-content;
}

.mode-toggle button {
  min-height: 24px;
  border-color: transparent;
  border-radius: 4px;
  background: transparent;
  box-shadow: none;
  font-size: var(--type-button-compact);
}

.mode-toggle button.active {
  border-color: var(--selection-stroke);
  background: var(--surface);
  color: var(--text);
  font-weight: var(--font-label);
}

.target-toggle.active {
  background: var(--surface);
  box-shadow: none;
  color: var(--text);
  font-weight: var(--font-label);
}

.mode-toggle.large button {
  min-width: 92px;
  font-size: var(--type-button);
}

.target-button-group {
  display: flex;
  flex-wrap: wrap;
  gap: 7px;
}

.target-toggle {
  min-width: 132px;
  flex: 1 1 132px;
  display: flex;
  align-items: center;
  justify-content: flex-start;
  gap: 7px;
  min-height: 42px;
  padding: 6px 8px;
  border: 0;
  background: transparent;
  box-shadow: none;
  font-size: var(--type-button);
  text-align: left;
  transition:
    background-color var(--motion-base) var(--ease-out-quint),
    color var(--motion-base) var(--ease-out-quint),
    transform var(--motion-fast) var(--ease-out-quart);
}

.target-toggle:hover:not(:disabled) {
  background: var(--control-bg-hover);
  box-shadow: none;
}

.target-toggle.active {
  background: var(--control-bg-active);
}

.target-toggle > span:last-child {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.target-toggle small {
  font-size: var(--type-button-compact);
}

.tool-avatar {
  width: 26px;
  height: 26px;
  display: inline-flex;
  flex: 0 0 auto;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: 5px;
  background: var(--surface-muted);
  color: var(--text-muted);
  transition:
    background-color var(--motion-base) var(--ease-out-quint),
    color var(--motion-base) var(--ease-out-quint),
    transform var(--motion-fast) var(--ease-out-quart);
}

.target-toggle:hover:not(:disabled) .tool-avatar,
.target-toggle.active .tool-avatar {
  transform: translateY(-1px);
}

.tool-avatar img {
  width: 15px;
  height: 15px;
  opacity: 0.76;
  transition:
    opacity var(--motion-base) var(--ease-out-quint),
    transform var(--motion-fast) var(--ease-out-quart);
}

.target-toggle:hover:not(:disabled) .tool-avatar img {
  opacity: 0.92;
  transform: scale(1.04);
}

.tool-avatar span {
  font-size: var(--type-button-compact);
  font-weight: var(--font-label);
}

.meta-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 7px;
  margin: 0;
}

.meta-grid div {
  padding: 8px;
  border: 0;
  border-radius: 6px;
  background: var(--surface-muted);
}

.meta-grid dd {
  margin-top: 4px;
  overflow-wrap: anywhere;
}

.status-pill {
  display: inline-flex;
  align-items: center;
  min-height: 20px;
  padding: 2px 7px;
  border-radius: 999px;
  background: var(--surface);
  color: var(--text-muted);
  font-size: var(--type-button-compact);
  font-weight: var(--font-label);
}

.status-available {
  background: var(--accent);
  color: var(--accent-text);
}

.status-current {
  background: var(--control-bg-active);
  color: var(--text);
}

.status-failed,
.status-norelease {
  background: var(--surface-muted);
  color: var(--text-muted);
}

.update-summary {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 8px;
  border-radius: 6px;
  background: var(--surface-muted);
}

.update-summary p {
  margin: 0;
  font-size: 0.84rem;
}

.update-summary small {
  overflow-wrap: anywhere;
}

.update-detail-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 7px;
  margin: 2px 0 0;
}

.update-detail-grid div {
  min-width: 0;
}

.update-detail-grid dt {
  font-size: 0.72rem;
  font-weight: var(--font-label);
  color: var(--text-muted);
}

.update-detail-grid dd {
  margin: 2px 0 0;
  overflow-wrap: anywhere;
  font-size: 0.8rem;
}

.primary-action {
  border-color: transparent;
  background: var(--accent);
  box-shadow: none;
  color: var(--accent-text);
}

.primary-action:hover:not(:disabled) {
  background: var(--accent-hover);
  box-shadow: none;
}

.settings-panel-swap-enter-active,
.settings-panel-swap-leave-active {
  transition:
    opacity var(--motion-base) var(--ease-out-quint),
    transform var(--motion-base) var(--ease-out-quint);
}

.settings-panel-swap-enter-from,
.settings-panel-swap-leave-to {
  opacity: 0;
  transform: translateY(5px);
}

@keyframes settings-view-settle {
  from {
    opacity: 0.9;
    transform: translateY(3px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@media (max-width: 720px) {
  .settings-topbar,
  .settings-panel {
    padding-left: 14px;
    padding-right: 14px;
  }

  .settings-layout {
    grid-template-columns: minmax(0, 1fr);
    grid-template-rows: auto minmax(0, 1fr);
  }

  .settings-tabs {
    min-height: auto;
    flex-direction: row;
    padding: 8px 14px;
    overflow-x: auto;
    overflow-y: hidden;
    border-right: 0;
    border-bottom: 1px solid var(--line);
  }

  .settings-tabs button {
    min-width: max-content;
    justify-content: center;
  }

  .meta-grid {
    grid-template-columns: minmax(0, 1fr);
  }

  .mode-toggle.large button {
    min-width: 82px;
  }
}
</style>
