<template>
  <aside class="rail">
    <div class="rail-top">
      <button
        type="button"
        class="rail-button"
        :disabled="busy"
        @click="emit('addWorkspace')"
      >
        + 添加工作区
      </button>
    </div>

    <div class="rail-scroll">
      <section class="rail-section">
        <div class="section-kicker">根目录</div>
        <button
          type="button"
          class="scope-card"
          :class="{
            active: selectedScopeKey === 'global' && viewMode === 'manager',
          }"
          :disabled="busy"
          @click="emit('selectScope', 'global')"
        >
          <strong>全局技能</strong>
          <small>{{ globalBaseFolder || "-" }}</small>
        </button>
      </section>

      <section class="rail-section">
        <div class="section-headline">
          <span>工作区</span>
          <small>{{ workspaceOptions.length }} 个</small>
        </div>

        <TransitionGroup
          name="workspace-row-motion"
          tag="div"
          class="workspace-stack"
        >
          <div
            v-for="option in workspaceOptions"
            :key="option.key"
            class="workspace-row"
          >
            <button
              type="button"
              class="scope-card workspace-card"
              :class="{
                active:
                  selectedScopeKey === option.key && viewMode === 'manager',
              }"
              :disabled="busy"
              @click="emit('selectScope', option.key)"
            >
              <strong>{{ option.label }}</strong>
              <small v-if="option.detail">{{ option.detail }}</small>
            </button>
            <div class="workspace-actions">
              <button
                type="button"
                class="icon-button subtle"
                data-workspace-menu-trigger
                :disabled="busy"
                title="更多"
                @click.stop="toggleMenu(option.key)"
              >
                <span class="svg-icon icon-more" aria-hidden="true"></span>
              </button>
              <button
                type="button"
                class="icon-button subtle workspace-edit"
                :disabled="busy"
                title="编辑技能基础目录"
                @click.stop="
                  closeMenu();
                  emit('editWorkspaceBase', option);
                "
              >
                <span class="svg-icon icon-edit" aria-hidden="true"></span>
              </button>
            </div>

            <Transition name="menu-pop">
              <div
                v-if="openMenuKey === option.key"
                class="workspace-menu"
                data-workspace-menu
              >
                <button
                  type="button"
                  :disabled="busy"
                  @click.stop="
                    emit('openWorkspaceFolder', option.path);
                    closeMenu();
                  "
                >
                  在资源管理器中打开
                </button>
                <button
                  type="button"
                  class="danger"
                  :disabled="busy"
                  @click.stop="
                    emit('removeWorkspace', option);
                    closeMenu();
                  "
                >
                  移除
                </button>
              </div>
            </Transition>
          </div>
        </TransitionGroup>

        <div v-if="!workspaceOptions.length" class="rail-empty">暂无工作区</div>
      </section>
    </div>

    <div
      class="rail-bottom"
      :class="{ active: viewMode === 'settings', disabled: busy }"
      role="button"
      :tabindex="busy ? -1 : 0"
      :aria-disabled="busy ? 'true' : 'false'"
      @click="openSettings"
      @keydown.enter.prevent="openSettings"
      @keydown.space.prevent="openSettings"
    >
      <div class="settings-entry">
        <span class="settings-gear" aria-hidden="true">
          <span class="svg-icon icon-setting" aria-hidden="true"></span>
        </span>
        <span class="settings-copy">
          <strong>设置</strong>
        </span>
      </div>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from "vue";
import type { ScopeOption, ViewMode } from "../types/manager";

const props = defineProps<{
  appName: string;
  appVersion: string;
  busy: boolean;
  globalBaseFolder: string;
  workspaceOptions: ScopeOption[];
  selectedScopeKey: string;
  viewMode: ViewMode;
}>();

const emit = defineEmits<{
  addWorkspace: [];
  selectScope: [key: string];
  removeWorkspace: [option: ScopeOption];
  openWorkspaceFolder: [path: string];
  editWorkspaceBase: [option: ScopeOption];
  openSettings: [];
}>();

const openMenuKey = ref("");

function handleDocumentClick(event: MouseEvent) {
  if (!openMenuKey.value) return;

  const target = event.target;
  if (!(target instanceof Node)) return;

  const targetElement =
    target instanceof Element ? target : target.parentElement;
  if (!targetElement) {
    closeMenu();
    return;
  }

  if (
    targetElement.closest("[data-workspace-menu]") ||
    targetElement.closest("[data-workspace-menu-trigger]")
  ) {
    return;
  }

  closeMenu();
}

function toggleMenu(key: string) {
  openMenuKey.value = openMenuKey.value === key ? "" : key;
}

function closeMenu() {
  openMenuKey.value = "";
}

function openSettings() {
  if (props.busy) return;
  emit("openSettings");
}

onMounted(() => {
  document.addEventListener("click", handleDocumentClick);
});

onBeforeUnmount(() => {
  document.removeEventListener("click", handleDocumentClick);
});
</script>

<style scoped>
.rail {
  min-height: 0;
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 10px 8px 0;
  overflow-x: hidden;
  overflow-y: hidden;
  border-right: 1px solid color-mix(in oklch, var(--line) 72%, transparent);
  background: var(--rail-bg);
  animation: rail-settle var(--motion-slow) var(--ease-out-expo);
}

.rail-top,
.workspace-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.rail-top {
  position: relative;
  z-index: 2;
  padding: 20px 0;
  flex: 0 0 auto;
  align-items: flex-start;
  flex-wrap: wrap;
  gap: 7px;
  background: linear-gradient(
    to bottom,
    var(--rail-bg) 0%,
    color-mix(in oklch, var(--rail-bg) 72%, transparent) 78%,
    transparent 100%
  );
}

.rail-button {
  width: 100%;
  min-height: 24px;
  padding-block: 2px;
  border: 0;
  background: var(--surface-muted);
  box-shadow: none;
  color: var(--text);
  font-size: var(--type-button);
}

.rail-button:hover:not(:disabled) {
  background: var(--control-bg-hover);
  box-shadow: none;
}

.rail-section {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.rail-scroll {
  min-height: 0;
  flex: 1 1 auto;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 10px 0 10px;
  overflow-x: hidden;
  overflow-y: auto;
  scrollbar-gutter: stable;
  mask-image: linear-gradient(
    to bottom,
    rgb(0 0 0 / 0.58) 0,
    rgb(0 0 0) var(--scroll-fade-compact),
    rgb(0 0 0) calc(100% - var(--scroll-fade-compact)),
    rgb(0 0 0 / 0.62) 100%
  );
  -webkit-mask-image: linear-gradient(
    to bottom,
    rgb(0 0 0 / 0.58) 0,
    rgb(0 0 0) var(--scroll-fade-compact),
    rgb(0 0 0) calc(100% - var(--scroll-fade-compact)),
    rgb(0 0 0 / 0.62) 100%
  );
}

.section-kicker {
  font-size: 0.76rem;
  font-weight: var(--font-label);
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0;
}

.section-headline {
  min-height: 20px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.section-headline span,
.section-kicker {
  color: var(--text);
  font-size: 0.82rem;
}

.scope-card {
  position: relative;
  width: 100%;
  min-height: 40px;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  justify-content: center;
  gap: 2px;
  padding: 5px 6px 5px 28px;
  overflow: hidden;
  border-color: transparent;
  background: transparent;
  box-shadow: none;
  text-align: left;
  transition:
    background-color var(--motion-base) var(--ease-out-quint),
    color var(--motion-base) var(--ease-out-quint),
    transform var(--motion-fast) var(--ease-out-quart);
}

.scope-card::before {
  content: "";
  position: absolute;
  left: 11px;
  top: 50%;
  width: 6px;
  height: 6px;
  border: 1px solid var(--line-strong);
  border-radius: 50%;
  background: var(--rail-bg);
  transform: translateY(-50%);
  transition:
    border-color var(--motion-base) var(--ease-out-quint),
    background-color var(--motion-base) var(--ease-out-quint),
    box-shadow var(--motion-base) var(--ease-out-quint),
    transform var(--motion-base) var(--ease-out-quint);
}

.scope-card:hover:not(:disabled) {
  background: color-mix(in oklch, var(--surface) 62%, transparent);
  box-shadow: none;
}

.scope-card.active,
.rail-bottom.active .settings-entry {
  background: var(--surface-active);
  box-shadow: none;
  color: var(--text);
}

.scope-card.active::before {
  border-color: var(--accent-point);
  background: var(--accent-point);
  box-shadow: 0 0 0 3px color-mix(in oklch, var(--accent-point) 12%, transparent);
  transform: translateY(-50%) scale(1.08);
}

.scope-card strong,
.scope-card small {
  overflow-wrap: anywhere;
}

.scope-card strong {
  font-weight: var(--font-label);
}

.scope-card small,
.settings-copy small {
  font-size: 0.76rem;
}

.workspace-stack {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.workspace-row {
  position: relative;
  align-items: stretch;
}

.workspace-card {
  min-height: 42px;
  padding-right: 62px;
}

.workspace-actions {
  position: absolute;
  top: 8px;
  right: 4px;
  display: flex;
  gap: 3px;
  opacity: 0;
  pointer-events: none;
  transform: translateX(4px);
  transition:
    opacity var(--motion-base) var(--ease-out-quint),
    transform var(--motion-base) var(--ease-out-quint);
}

.workspace-row:hover .workspace-actions,
.workspace-row:focus-within .workspace-actions {
  opacity: 1;
  pointer-events: auto;
  transform: translateX(0);
}

.icon-button {
  position: relative;
  width: 20px;
  min-width: 20px;
  height: 20px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 6px;
}

.svg-icon {
  width: 14px;
  height: 14px;
  display: inline-block;
  background: currentColor;
  mask-position: center;
  mask-repeat: no-repeat;
  mask-size: contain;
  -webkit-mask-position: center;
  -webkit-mask-repeat: no-repeat;
  -webkit-mask-size: contain;
}

.icon-more {
  mask-image: url("../assets/more.svg");
  -webkit-mask-image: url("../assets/more.svg");
}

.icon-edit {
  mask-image: url("../assets/edit.svg");
  -webkit-mask-image: url("../assets/edit.svg");
}

.icon-setting {
  mask-image: url("../assets/setting.svg");
  -webkit-mask-image: url("../assets/setting.svg");
}

.icon-button.subtle {
  border: 0;
  background: transparent;
  box-shadow: none;
  color: var(--text-muted);
}

.icon-button.subtle:hover:not(:disabled) {
  background: var(--control-bg-hover);
  color: var(--text);
}

.workspace-edit {
  width: auto;
  min-width: 28px;
  padding: 0 4px;
  font-size: var(--type-button-compact);
}

.workspace-menu {
  position: absolute;
  z-index: 20;
  top: 34px;
  right: 7px;
  width: 158px;
  display: flex;
  flex-direction: column;
  padding: 4px;
  border: 1px solid var(--line);
  border-radius: 6px;
  background: var(--surface);
  box-shadow: 0 8px 18px color-mix(in oklch, var(--shadow) 74%, transparent);
  transform-origin: top right;
}

.workspace-menu button {
  justify-content: flex-start;
  border-color: transparent;
  background: transparent;
  box-shadow: none;
  font-size: var(--type-button);
  text-align: left;
}

.workspace-menu button.danger {
  color: var(--danger-text);
}

.rail-empty {
  padding: 8px;
  border: 1px dashed var(--line);
  border-radius: 5px;
  background: transparent;
  color: var(--text-muted);
}

.rail-bottom {
  position: relative;
  z-index: 2;
  flex: 0 0 auto;
  margin: 0 -8px;
  padding: 9px 8px 8px;
  background: linear-gradient(
    to bottom,
    transparent 0%,
    color-mix(in oklch, var(--rail-bg) 72%, transparent) 34%,
    var(--rail-bg) 100%
  );
  color: var(--text);
  cursor: default;
}

.rail-bottom:not(.disabled) {
  cursor: pointer;
}

.settings-entry {
  margin-top: auto;
  display: flex;
  align-items: center;
  gap: 6px;
  min-height: 30px;
  padding: 3px 6px;
  border: 0;
  border-radius: 5px;
  background: transparent;
  box-shadow: var(--control-shadow);
  text-align: left;
  transition:
    background-color var(--motion-base) var(--ease-out-quint),
    color var(--motion-base) var(--ease-out-quint);
}

.rail-bottom:not(.disabled):hover .settings-entry {
  background: var(--control-bg-hover);
  box-shadow: var(--control-shadow-hover);
}

.rail-bottom:focus-visible .settings-entry {
  outline: 2px solid color-mix(in oklch, var(--accent-point) 70%, transparent);
  outline-offset: 2px;
}

.rail-bottom.disabled {
  color: var(--text-muted);
  opacity: 0.62;
  pointer-events: none;
}

.settings-gear {
  width: 20px;
  height: 20px;
  display: inline-flex;
  flex: 0 0 auto;
  align-items: center;
  justify-content: center;
  border-radius: 5px;
  background: var(--surface-muted);
  color: var(--text-muted);
}

.settings-gear .svg-icon {
  width: 13px;
  height: 13px;
  transition: transform var(--motion-base) var(--ease-out-quint);
}

.rail-bottom:not(.disabled):hover .settings-gear .svg-icon {
  transform: rotate(90deg);
}

.settings-copy {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.settings-copy strong {
  font-size: 0.82rem;
  font-weight: var(--font-label);
}

.menu-pop-enter-active,
.menu-pop-leave-active {
  transition:
    opacity var(--motion-fast) var(--ease-out-quint),
    transform var(--motion-fast) var(--ease-out-quint);
}

.menu-pop-enter-from,
.menu-pop-leave-to {
  opacity: 0;
  transform: translateY(-4px) scale(0.98);
}

.workspace-row-motion-enter-active,
.workspace-row-motion-leave-active {
  transition:
    opacity var(--motion-base) var(--ease-out-quint),
    transform var(--motion-base) var(--ease-out-quint);
}

.workspace-row-motion-enter-from,
.workspace-row-motion-leave-to {
  opacity: 0;
  transform: translateY(5px);
}

.workspace-row-motion-move {
  transition: transform var(--motion-slow) var(--ease-out-expo);
}

@keyframes rail-settle {
  from {
    opacity: 0.88;
    transform: translateX(-4px);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}
</style>
