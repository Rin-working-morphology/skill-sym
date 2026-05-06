<template>
  <section
    class="settings-panel"
    aria-labelledby="settings-targets-title"
  >
    <div class="settings-section-head">
      <div class="settings-section-title-row">
        <h2 id="settings-targets-title">目标启用状态</h2>
        <button
          type="button"
          class="add-target-button"
          :disabled="busy"
          @click="customTargetFormOpen = true"
        >
          添加
        </button>
      </div>
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

    <CustomTargetForm
      v-if="customTargetFormOpen"
      :busy="busy"
      @cancel="customTargetFormOpen = false"
      @submit="submitCustomTarget"
    />
  </section>
</template>

<script setup lang="ts">
import { ref } from "vue";

import CustomTargetForm from "./CustomTargetForm.vue";
import type { TargetId, TargetOption } from "../types/manager";

defineProps<{
  busy: boolean;
  targetOptions: TargetOption[];
}>();

const emit = defineEmits<{
  togglePublishTarget: [targetId: TargetId];
  addCustomPublishTarget: [payload: { name: string; folderName: string }];
}>();

const customTargetFormOpen = ref(false);

function submitCustomTarget(payload: { name: string; folderName: string }) {
  emit("addCustomPublishTarget", payload);
  customTargetFormOpen.value = false;
}
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

.settings-section-title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.add-target-button {
  min-height: 24px;
  border-color: transparent;
  background: var(--accent);
  box-shadow: none;
  color: var(--accent-text);
  font-size: var(--type-button-compact);
}

.add-target-button:hover:not(:disabled) {
  background: var(--accent-hover);
  box-shadow: none;
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
  box-shadow: none;
  color: var(--text);
  font-weight: var(--font-label);
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

.target-toggle strong,
.target-toggle small {
  overflow-wrap: anywhere;
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
  color: inherit;
  font-size: var(--type-button-compact);
  font-weight: var(--font-label);
}

.tool-avatar.generic {
  color: var(--custom-target-icon-text);
}

@media (max-width: 720px) {
  .settings-panel {
    padding-left: 14px;
    padding-right: 14px;
  }
}
</style>
