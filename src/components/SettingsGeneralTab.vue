<template>
  <section
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
      <strong class="setting-path">{{ state?.globalBaseFolder ?? "-" }}</strong>
      <div class="mini-actions">
        <button
          v-for="preset in baseFolderPresets"
          :key="preset"
          type="button"
          :disabled="busy"
          @click="emit('setQuickBase', preset)"
        >
          {{ preset }}
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
</template>

<script setup lang="ts">
import type {
  AppState,
  BaseFolderPreset,
  PublishMode,
  ThemeMode,
} from "../types/manager";

defineProps<{
  busy: boolean;
  state: AppState | null;
  publishMode: PublishMode;
  themeMode: ThemeMode;
}>();

const emit = defineEmits<{
  setQuickBase: [folderName: BaseFolderPreset];
  chooseGlobalBaseFolder: [];
  setPublishMode: [mode: PublishMode];
  setThemeMode: [mode: ThemeMode];
}>();

const baseFolderPresets: BaseFolderPreset[] = [
  ".claude",
  ".codex",
  ".qoder",
  ".trae",
  ".codebuddy",
];
</script>

<style scoped>
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

.mini-actions {
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

.mode-toggle.large button {
  min-width: 92px;
  font-size: var(--type-button);
}

@media (max-width: 720px) {
  .settings-panel {
    padding-left: 14px;
    padding-right: 14px;
  }

  .mode-toggle.large button {
    min-width: 82px;
  }
}
</style>
