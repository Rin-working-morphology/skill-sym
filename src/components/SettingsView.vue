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
        <SettingsGeneralTab
          v-if="activeTab === 'general'"
          key="general"
          :busy="busy"
          :state="state"
          :publish-mode="publishMode"
          :theme-mode="themeMode"
          @set-quick-base="emit('setQuickBase', $event)"
          @choose-global-base-folder="emit('chooseGlobalBaseFolder')"
          @set-publish-mode="emit('setPublishMode', $event)"
          @set-theme-mode="emit('setThemeMode', $event)"
        />

        <SettingsTargetsTab
          v-else-if="activeTab === 'targets'"
          key="targets"
          :busy="busy"
          :target-options="targetOptions"
          @toggle-publish-target="emit('togglePublishTarget', $event)"
          @add-custom-publish-target="emit('addCustomPublishTarget', $event)"
        />

        <SettingsAboutTab
          v-else
          key="about"
          :busy="busy"
          :app-name="appName"
          :app-version="appVersion"
          :update-status="updateStatus"
          @install-latest-update="emit('installLatestUpdate')"
          @open-release-page="emit('openReleasePage')"
        />
      </Transition>
    </div>
  </section>
</template>

<script setup lang="ts">
import { ref } from "vue";

import SettingsAboutTab from "./SettingsAboutTab.vue";
import SettingsGeneralTab from "./SettingsGeneralTab.vue";
import SettingsTargetsTab from "./SettingsTargetsTab.vue";
import type {
  AppState,
  BaseFolderPreset,
  PublishMode,
  TargetId,
  TargetOption,
  ThemeMode,
  UpdateStatus,
} from "../types/manager";

type SettingsTab = "general" | "targets" | "about";

defineProps<{
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
  addCustomPublishTarget: [payload: { name: string; folderName: string }];
  installLatestUpdate: [];
  openReleasePage: [];
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
  .settings-topbar {
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
}
</style>
