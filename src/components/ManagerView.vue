<template>
  <div v-if="activeScope" class="manager-workspace">
    <header class="manager-top">
      <div class="scope-block">
        <p>
          {{
            activeScope.kind === "global" ? "全局技能根目录" : activeScope.path
          }}
        </p>
      </div>
      <p class="scope-base-note">Skills来源：{{ baseFolderText() }}</p>
    </header>

    <section class="panel skill-matrix">
      <div class="panel-header">
        <div class="panel-title">
          <h2>技能列表</h2>
          <small>{{ scan?.skills.length ?? 0 }} 项技能</small>
        </div>
        <div class="panel-tools">
          <span class="command-label">发布方式</span>
          <div class="mode-toggle compact">
            <button
              type="button"
              :class="{ active: publishMode === 'symlink' }"
              :disabled="busy"
              @click="emit('setPublishMode', 'symlink')"
            >
              链接
            </button>
            <button
              type="button"
              :class="{ active: publishMode === 'copy' }"
              :disabled="busy"
              @click="emit('setPublishMode', 'copy')"
            >
              复制
            </button>
          </div>
          <button
            type="button"
            class="icon-button refresh-action"
            :disabled="busy"
            title="重新扫描"
            aria-label="重新扫描"
            @click="emit('refreshSkills')"
          >
            <span class="svg-icon icon-refresh" aria-hidden="true"></span>
          </button>
        </div>
      </div>

      <Transition name="panel-swap" mode="out-in">
        <div v-if="scan" class="skill-list-shell">
          <TransitionGroup name="skill-row-motion" tag="div" class="skill-list">
            <div key="skill-head" class="skill-head">
              <span>名称</span>
              <span>类型</span>
              <span>操作</span>
              <span></span>
            </div>

            <div key="bundle-row" class="skill-row bundle-row">
              <div class="skill-select bundle-select">
                <span class="bundle-icon" aria-hidden="true">
                  <span class="svg-icon icon-folder" aria-hidden="true"></span>
                </span>
                <div class="skill-main">
                  <span class="skill-name">skills</span>
                </div>
              </div>
              <span class="kind-pill">根目录</span>
              <div class="row-actions">
                <button
                  v-for="target in targetOptions"
                  :key="`bundle-${target.path}`"
                  type="button"
                  class="icon-button tool-action"
                  :class="[target.tone, { active: isFolderPublished(target) }]"
                  :aria-pressed="isFolderPublished(target)"
                  :disabled="busy || target.isSource || !scan.exists"
                  :title="
                    target.isSource
                      ? `${target.name} 是当前源目录`
                      : isFolderPublished(target)
                        ? `从 ${target.name} 移除整包 skills`
                        : `以${publishModeLabel(publishMode)}发布 skills 到 ${target.name}`
                  "
                  @click="
                    emit('publishWholeFolder', {
                      targetBaseFolder: target.path,
                      currentPublished: isFolderPublished(target),
                      targetName: target.name,
                    })
                  "
                >
                  <img
                    v-if="target.iconSrc"
                    :src="target.iconSrc"
                    :alt="target.name"
                  />
                  <span v-else>{{ target.shortLabel }}</span>
                </button>
              </div>
              <span></span>
            </div>

            <div
              v-for="skill in scan.skills"
              :key="skill.path"
              class="skill-row child-skill-row"
              :class="{ active: selectedSkill?.path === skill.path }"
            >
              <span
                class="skill-select"
                :disabled="busy"
                @click="emit('selectSkill', skill.name)"
              >
                <div class="skill-main">
                  <span class="skill-name">{{ skill.name }}</span>
                </div>
              </span>

              <span class="kind-pill">{{ skillKindLabel(skill.kind) }}</span>

              <div class="row-actions">
                <button
                  v-for="target in targetOptions"
                  :key="`${skill.path}-${target.path}`"
                  type="button"
                  class="icon-button tool-action"
                  :class="[
                    target.tone,
                    { active: isSkillPublished(skill.name, target) },
                  ]"
                  :aria-pressed="isSkillPublished(skill.name, target)"
                  :disabled="busy || target.protectsSourceChildren"
                  :title="
                    target.protectsSourceChildren
                      ? `${target.name} 的技能项受当前源目录保护`
                      : isSkillPublished(skill.name, target)
                        ? `从 ${target.name} 移除 ${skill.name}`
                        : `以${publishModeLabel(publishMode)}发布 ${skill.name} 到 ${target.name}`
                  "
                  @click="
                    emit('toggleSkillTarget', {
                      skillName: skill.name,
                      targetBaseFolder: target.path,
                      currentPublished: isSkillPublished(skill.name, target),
                    })
                  "
                >
                  <img
                    v-if="target.iconSrc"
                    :src="target.iconSrc"
                    :alt="target.name"
                  />
                  <span v-else>{{ target.shortLabel }}</span>
                </button>
              </div>
              <button
                type="button"
                class="icon-button delete-action"
                :disabled="busy"
                :title="`删除 ${skill.name}`"
                @click="emit('deleteSkill', skill.name)"
              >
                <span class="svg-icon icon-delete" aria-hidden="true"></span>
              </button>
            </div>
          </TransitionGroup>
        </div>

        <div v-else class="panel-empty">暂无技能</div>
      </Transition>
    </section>
  </div>
</template>

<script setup lang="ts">
import type {
  BaseFolderPreset,
  PublishMode,
  ScanResult,
  ScopeOption,
  SkillEntry,
  TargetOption,
} from "../types/manager";
import { publishModeLabel, skillKindLabel } from "../utils/managerUi";

const props = defineProps<{
  busy: boolean;
  activeScope?: ScopeOption;
  scan: ScanResult | null;
  targetOptions: TargetOption[];
  selectedSkill?: SkillEntry;
  publishMode: PublishMode;
  isSkillPublished: (skillName: string, target: TargetOption) => boolean;
  isFolderPublished: (target: TargetOption) => boolean;
}>();

const emit = defineEmits<{
  setPublishMode: [mode: PublishMode];
  refreshSkills: [];
  setQuickBase: [folderName: BaseFolderPreset];
  chooseScopeBaseFolder: [];
  publishWholeFolder: [
    payload: {
      targetBaseFolder: string;
      currentPublished: boolean;
      targetName: string;
    },
  ];
  selectSkill: [name: string];
  toggleSkillTarget: [
    payload: {
      skillName: string;
      targetBaseFolder: string;
      currentPublished: boolean;
    },
  ];
  deleteSkill: [skillName: string];
}>();

function normalizePath(path: string) {
  return path.replace(/\\/g, "/").replace(/\/+$/, "");
}

function baseFolderText() {
  const scope = props.activeScope;

  if (!scope) {
    return "-";
  }

  if (scope.kind === "global") {
    return `全局目录 ${scope.baseFolder}`;
  }

  const workspacePath = normalizePath(scope.path);
  const basePath = normalizePath(scope.baseFolder);
  const relativeBase = basePath.startsWith(`${workspacePath}/`)
    ? basePath.slice(workspacePath.length + 1)
    : scope.baseFolder;

  return `工作区下 ${relativeBase || "根目录"}`;
}
</script>

<style scoped>
.manager-workspace {
  min-width: 0;
  min-height: 0;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg);
  animation: view-settle var(--motion-slow) var(--ease-out-expo);
}

.manager-top {
  min-height: 40px;
  flex: 0 0 auto;
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  padding: 13px 18px 11px;
  border-bottom: 0;
  background: var(--surface);
  box-shadow: inset 0 -1px color-mix(in oklch, var(--line) 68%, transparent);
}

.scope-block {
  min-width: 0;
  flex: 1 1 auto;
  max-width: 86ch;
}

.scope-block p {
  margin-top: 3px;
  overflow-wrap: anywhere;
  font-size: 0.76rem;
}

.scope-base-note {
  flex: 0 1 38ch;
  margin: 2px 0 0;
  max-width: 38ch;
  overflow-wrap: anywhere;
  text-align: right;
  color: var(--text);

  font-size: 0.76rem;
}

.command-label {
  color: var(--text-muted);
  font-size: 0.74rem;
  font-weight: var(--font-label);
}

.mode-toggle {
  display: flex;
  align-items: center;
  gap: 3px;
  padding: 2px;
  border: 0;
  border-radius: 5px;
  background: var(--surface-muted);
}

.mode-toggle button {
  min-height: 24px;
  border-color: transparent;
  border-radius: 4px;
  background: transparent;
  box-shadow: none;
}

.mode-toggle button.active {
  border-color: var(--selection-stroke);
  background: var(--surface);
  color: var(--text);
  font-weight: var(--font-label);
}

.mode-toggle.compact button {
  min-height: 22px;
  padding: 2px 7px;
  font-size: var(--type-button-compact);
}

.panel {
  min-height: 0;
  border-bottom: 1px solid color-mix(in oklch, var(--line) 66%, transparent);
  background: var(--bg);
}

.skill-matrix {
  flex: 1 1 auto;
  display: flex;
  flex-direction: column;
  padding: 12px 18px 18px;
  border-bottom: 0;
  background: var(--bg);
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  flex: 0 0 auto;
  margin-bottom: 0;
  padding-bottom: 9px;
  border-bottom: 0;
}

.panel-title,
.panel-tools {
  display: flex;
  align-items: center;
  gap: 8px;
}

.panel-title {
  min-width: 0;
}

.panel-tools {
  justify-content: flex-end;
  flex-wrap: wrap;
}

.row-actions {
  min-width: max-content;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: flex-end;
  gap: 5px;
}

.skill-list-shell {
  position: relative;
  min-height: 0;
  flex: 1 1 auto;
  overflow: hidden;
  background: var(--surface);
}

.skill-list-shell::before,
.skill-list-shell::after {
  content: "";
  position: absolute;
  z-index: 4;
  right: 0;
  left: 0;
  height: var(--scroll-fade-soft);
  pointer-events: none;
}

.skill-list-shell::before {
  top: 0;
  background: linear-gradient(
    to bottom,
    color-mix(in oklch, var(--surface) 58%, transparent) 0%,
    color-mix(in oklch, var(--surface) 24%, transparent) 58%,
    transparent 100%
  );
}

.skill-list-shell::after {
  bottom: 0;
  background: linear-gradient(
    to top,
    color-mix(in oklch, var(--surface) 62%, transparent) 0%,
    color-mix(in oklch, var(--surface) 26%, transparent) 58%,
    transparent 100%
  );
}

.skill-list {
  min-height: 0;
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: 0;
  overflow-x: hidden;
  overflow-y: auto;
  scrollbar-gutter: stable;
  background: var(--surface);
}

.skill-head,
.skill-row {
  display: grid;
  grid-template-columns: minmax(130px, 1fr) 62px minmax(124px, auto) 28px;
  gap: 7px;
  align-items: center;
  padding-left: 12px;
  padding-right: 8px;
}

.skill-head {
  position: sticky;
  z-index: 3;
  top: 0;
  min-height: 28px;
  border-bottom: 0;
  background: color-mix(in oklch, var(--surface-muted) 92%, var(--surface));
  box-shadow: inset 0 -1px color-mix(in oklch, var(--line) 70%, transparent);
  color: var(--text-muted);
  font-size: 0.76rem;
  font-weight: var(--font-label);
  text-transform: uppercase;
  letter-spacing: 0;
}

.skill-row {
  min-height: 38px;
  padding-top: 4px;
  padding-bottom: 4px;
  border-bottom: 1px solid color-mix(in oklch, var(--line) 76%, transparent);
  background: transparent;
  transition:
    background-color var(--motion-base) var(--ease-out-quint),
    border-color var(--motion-base) var(--ease-out-quint),
    color var(--motion-base) var(--ease-out-quint),
    transform var(--motion-fast) var(--ease-out-quart);
}

.skill-row:last-child {
  border-bottom-color: transparent;
}

.skill-row:hover {
  background: color-mix(in oklch, var(--surface-muted) 62%, transparent);
}

.skill-row:hover .skill-name {
  color: var(--text);
}

.skill-row.active {
  border-color: var(--line);
  background: var(--surface-active);
}

.bundle-row {
  background: color-mix(in oklch, var(--surface-muted) 54%, transparent);
}

.child-skill-row .skill-select {
  position: relative;
  padding-left: 28px;
}

.child-skill-row .skill-select::before {
  content: "";
  position: absolute;
  left: 8px;
  top: 50%;
  width: 13px;
  height: 13px;
  border-left: 1px solid var(--line);
  border-bottom: 1px solid var(--line);
  border-bottom-left-radius: 5px;
  transform: translateY(-70%);
}

.skill-select {
  min-width: 0;
  display: flex;
  align-items: center;
  justify-content: flex-start;
  border: 0;
  background: transparent;
  box-shadow: none;
  padding: 0;
  text-align: left;
}

.skill-select:hover:not(:disabled) {
  background: transparent;
}

.skill-select strong {
  font-weight: var(--font-label);
}

.bundle-select {
  gap: 7px;
}

.bundle-icon {
  width: 24px;
  height: 24px;
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

.bundle-row:hover .bundle-icon {
  transform: translateY(-1px);
  color: var(--text);
}

.svg-icon {
  width: 15px;
  height: 15px;
  display: inline-block;
  background: currentColor;
  mask-position: center;
  mask-repeat: no-repeat;
  mask-size: contain;
  -webkit-mask-position: center;
  -webkit-mask-repeat: no-repeat;
  -webkit-mask-size: contain;
}

.icon-refresh {
  mask-image: url("../assets/refresh.svg");
  -webkit-mask-image: url("../assets/refresh.svg");
}

.icon-folder {
  mask-image: url("../assets/folder.svg");
  -webkit-mask-image: url("../assets/folder.svg");
}

.icon-delete {
  mask-image: url("../assets/delete.svg");
  -webkit-mask-image: url("../assets/delete.svg");
}

.bundle-icon .svg-icon {
  width: 15px;
  height: 15px;
}

.bundle-row .kind-pill {
  border-color: var(--line);
  background: var(--surface);
  color: var(--text-muted);
}

.skill-main {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.skill-main .skill-name {
  font-size: 0.9rem;
  transition: color var(--motion-base) var(--ease-out-quint);
}

.kind-pill {
  width: fit-content;
  padding: 2px 5px;
  border: 0;
  border-radius: 999px;
  background: var(--surface-muted);
  color: var(--text-muted);
  font-size: 0.76rem;
  font-weight: var(--font-label);
  transition:
    background-color var(--motion-base) var(--ease-out-quint),
    color var(--motion-base) var(--ease-out-quint);
}

.refresh-action {
  width: 24px;
  min-width: 24px;
  height: 24px;
  border-color: transparent;
  background: transparent;
  box-shadow: var(--control-shadow);
  color: var(--text-muted);
}

.refresh-action:hover:not(:disabled) {
  background: var(--control-bg-hover);
  box-shadow: var(--control-shadow-hover);
  color: var(--text);
}

.refresh-action .svg-icon {
  width: 14px;
  height: 14px;
}

.refresh-action:disabled .svg-icon {
  animation: refresh-spin 860ms linear infinite;
}

.icon-button {
  position: relative;
  width: 26px;
  min-width: 26px;
  height: 26px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0;
}

.tool-action {
  width: 26px;
  height: 26px;
  border: 0;
  border-radius: 5px;
  background: transparent;
  box-shadow: none;
  color: var(--text-muted);
  transition:
    background-color var(--motion-base) var(--ease-out-quint),
    color var(--motion-base) var(--ease-out-quint),
    opacity var(--motion-base) var(--ease-out-quint),
    transform var(--motion-fast) var(--ease-out-quart);
}

.tool-action:hover:not(:disabled),
.tool-action.active {
  background: var(--control-bg-active);
  box-shadow: none;
  color: var(--text);
}

.tool-action.active::after {
  content: "";
  position: absolute;
  right: 3px;
  bottom: 3px;
  width: 4px;
  height: 4px;
  border-radius: 50%;
  background: var(--accent-point);
  box-shadow: 0 0 0 2px
    color-mix(in oklch, var(--accent-point) 18%, transparent);
  animation: dot-settle var(--motion-slow) var(--ease-out-expo);
}

.tool-action img {
  width: 15px;
  height: 15px;
  opacity: 0.76;
  transition:
    opacity var(--motion-base) var(--ease-out-quint),
    transform var(--motion-fast) var(--ease-out-quart);
}

.tool-action:hover:not(:disabled) img {
  opacity: 0.92;
  transform: scale(1.04);
}

.tool-action span {
  color: inherit;
  font-size: var(--type-button-compact);
  font-weight: var(--font-label);
}

.tool-action.generic {
  color: var(--custom-target-icon-text);
}

.delete-action {
  justify-self: end;
  border: 0;
  background: transparent;
  box-shadow: none;
  color: var(--danger-text);
  opacity: 0;
  pointer-events: none;
  transform: translateX(4px);
}

.delete-action:hover:not(:disabled) {
  background: var(--danger-bg);
  box-shadow: none;
}

.skill-row:hover .delete-action,
.skill-row:focus-within .delete-action {
  opacity: 1;
  pointer-events: auto;
  transform: translateX(0);
}

.panel-empty {
  padding: 8px;
  border: 1px dashed var(--line);
  border-radius: 5px;
  background: transparent;
  color: var(--text-muted);
}

.panel-swap-enter-active,
.panel-swap-leave-active {
  transition:
    opacity var(--motion-base) var(--ease-out-quint),
    transform var(--motion-base) var(--ease-out-quint);
}

.panel-swap-enter-from,
.panel-swap-leave-to {
  opacity: 0;
  transform: translateY(4px);
}

.skill-row-motion-enter-active,
.skill-row-motion-leave-active {
  transition:
    opacity var(--motion-base) var(--ease-out-quint),
    transform var(--motion-base) var(--ease-out-quint);
}

.skill-row-motion-enter-from,
.skill-row-motion-leave-to {
  opacity: 0;
  transform: translateY(5px);
}

.skill-row-motion-move {
  transition: transform var(--motion-slow) var(--ease-out-expo);
}

@keyframes dot-settle {
  from {
    opacity: 0;
    transform: scale(0.6);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

@keyframes refresh-spin {
  to {
    transform: rotate(360deg);
  }
}

@keyframes view-settle {
  from {
    opacity: 0.88;
    transform: translateY(3px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@media (max-width: 720px) {
  .manager-top {
    flex-direction: column;
    gap: 6px;
    padding: 12px 14px 10px;
  }

  .scope-base-note {
    width: 100%;
    max-width: none;
    text-align: left;
  }

  .panel-header {
    align-items: flex-start;
  }

  .skill-matrix {
    padding-left: 14px;
    padding-right: 14px;
  }

  .skill-head,
  .skill-row {
    grid-template-columns: minmax(0, 1fr) 54px minmax(96px, auto) 28px;
  }
}
</style>
