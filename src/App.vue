<template>
  <main class="app-shell" :class="{ 'settings-mode': viewMode === 'settings' }">
    <template v-if="viewMode === 'manager'">
      <AppRail
        :app-name="appName"
        :app-version="appVersion"
        :busy="busy"
        :global-base-folder="state?.globalBaseFolder ?? ''"
        :workspace-options="workspaceOptions"
        :selected-scope-key="selectedScopeKey"
        :view-mode="viewMode"
        @add-workspace="addWorkspace"
        @select-scope="selectScope"
        @remove-workspace="removeWorkspaceOption"
        @open-workspace-folder="openFolderInExplorer"
        @edit-workspace-base="chooseScopeBaseFolder($event.scope)"
        @open-settings="setViewMode('settings')"
      />

      <section class="content-pane">
        <ManagerView
          :busy="busy"
          :active-scope="activeScope"
          :scan="scan"
          :target-options="targetOptions"
          :selected-skill="selectedSkill"
          :publish-mode="publishMode"
          :is-skill-published="isSkillPublished"
          :is-folder-published="isFolderPublished"
          @set-publish-mode="setDefaultPublishMode"
          @refresh-skills="refreshScopeData"
          @set-quick-base="setQuickBase"
          @choose-scope-base-folder="chooseScopeBaseFolder"
          @publish-whole-folder="publishWholeFolderToTarget"
          @select-skill="selectSkill"
          @toggle-skill-target="
            toggleSkillTarget(
              $event.skillName,
              $event.targetBaseFolder,
              $event.currentPublished,
            )
          "
          @delete-skill="deleteSkill"
        />
      </section>
    </template>

    <SettingsView
      v-else
      :busy="busy"
      :state="state"
      :publish-mode="publishMode"
      :theme-mode="themeMode"
      :target-options="allTargetOptions"
      :app-name="appName"
      :app-version="appVersion"
      @back="setViewMode('manager')"
      @set-quick-base="setQuickBaseForScope({ kind: 'global' }, $event)"
      @choose-global-base-folder="chooseScopeBaseFolder({ kind: 'global' })"
      @set-publish-mode="setDefaultPublishMode"
      @set-theme-mode="setThemeMode"
      @toggle-publish-target="togglePublishTarget"
      @refresh-update-status="refreshUpdateStatus"
    />

    <AppNotice
      :busy="busy"
      :status-message="statusMessage"
      :error-message="errorMessage"
    />
  </main>
</template>

<script setup lang="ts">
import { onMounted } from "vue";

import AppNotice from "./components/AppNotice.vue";
import AppRail from "./components/AppRail.vue";
import ManagerView from "./components/ManagerView.vue";
import SettingsView from "./components/SettingsView.vue";
import { useSkillManager } from "./composables/useSkillManager";

const {
  state,
  scan,
  appName,
  appVersion,
  viewMode,
  selectedScopeKey,
  themeMode,
  busy,
  statusMessage,
  errorMessage,
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
  refreshScopeData,
  refreshUpdateStatus,
} = useSkillManager();

onMounted(async () => {
  await initialize();
});
</script>

<style scoped>
.app-shell {
  width: 100%;
  height: 100%;
  min-height: 0;
  display: grid;
  grid-template-columns: var(--app-rail-width) minmax(0, 1fr);
  overflow: hidden;
  background: var(--bg);
  transition: background-color var(--motion-base) var(--ease-out-quint);
}

.app-shell.settings-mode {
  grid-template-columns: minmax(0, 1fr);
}

.content-pane {
  position: relative;
  min-width: 0;
  min-height: 0;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow-x: hidden;
  overflow-y: hidden;
  background: var(--bg);
  animation: pane-settle var(--motion-slow) var(--ease-out-expo);
  transition: background-color var(--motion-base) var(--ease-out-quint);
}

.content-pane::before,
.content-pane::after {
  content: "";
  position: absolute;
  z-index: 6;
  right: 0;
  left: 0;
  height: var(--scroll-fade-soft);
  pointer-events: none;
}

.content-pane::before {
  top: 0;
  background: linear-gradient(
    to bottom,
    color-mix(in oklch, var(--bg) 48%, transparent) 0%,
    color-mix(in oklch, var(--bg) 18%, transparent) 64%,
    transparent 100%
  );
}

.content-pane::after {
  bottom: 0;
  background: linear-gradient(
    to top,
    color-mix(in oklch, var(--bg) 52%, transparent) 0%,
    color-mix(in oklch, var(--bg) 18%, transparent) 64%,
    transparent 100%
  );
}

@media (max-width: 720px) {
  .app-shell.settings-mode {
    grid-template-columns: minmax(0, 1fr);
  }
}

@keyframes pane-settle {
  from {
    opacity: 0.9;
    transform: translateY(3px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
