<template>
  <section class="settings-panel" aria-labelledby="settings-about-title">
    <div class="settings-section-head">
      <h2 id="settings-about-title">版本信息</h2>
    </div>

    <article class="setting-card wide about-card">
      <div class="about-overview">
        <div class="about-product">
          <span class="about-mark" aria-hidden="true">S</span>
          <div class="about-title-block">
            <h3>{{ appName }}</h3>
            <small>本机技能同步管理器</small>
          </div>
        </div>

        <dl class="about-version-list">
          <div>
            <dt>当前版本</dt>
            <dd>{{ appVersion || "-" }}</dd>
          </div>
          <div v-if="updateStatus?.latestVersion">
            <dt>可用版本</dt>
            <dd>{{ updateStatus.latestVersion }}</dd>
          </div>
        </dl>
      </div>

      <div v-if="updateStatus" class="update-summary">
        <div class="update-summary-head">
          <span
            class="status-pill"
            :class="`status-${updateStatusClass(updateStatus.status)}`"
          >
            {{ updateStatusLabel(updateStatus.status) }}
          </span>
        </div>
        <p>{{ updateStatus.message }}</p>
        <small v-if="updateStatus.integrationNote">
          {{ updateStatus.integrationNote }}
        </small>

        <dl
          v-if="
            updateStatus.publishedAt ||
            updateStatus.releaseName ||
            updateStatus.assetName ||
            updateStatus.endpoint
          "
          class="update-detail-list"
        >
          <div v-if="updateStatus.publishedAt">
            <dt>发布时间</dt>
            <dd>{{ updateStatus.publishedAt }}</dd>
          </div>
          <div v-if="updateStatus.releaseName">
            <dt>发布名称</dt>
            <dd>{{ updateStatus.releaseName }}</dd>
          </div>
          <div v-if="updateStatus.assetName">
            <dt>安装包</dt>
            <dd>{{ updateStatus.assetName }}</dd>
          </div>
          <div v-if="updateStatus.endpoint">
            <dt>更新源</dt>
            <dd>{{ updateStatus.endpoint }}</dd>
          </div>
        </dl>
      </div>

      <div class="settings-actions about-actions">
        <button type="button" :disabled="busy" @click="emit('openReleasePage')">
          更新日志
        </button>
        <button
          type="button"
          class="primary-action"
          :disabled="busy || updateStatus?.status === 'checking'"
          @click="emit('installLatestUpdate')"
        >
          检查更新
        </button>
      </div>
    </article>
  </section>
</template>

<script setup lang="ts">
import type { UpdateStatus } from "../types/manager";
import { updateStatusLabel } from "../utils/managerUi";

defineProps<{
  busy: boolean;
  appName: string;
  appVersion: string;
  updateStatus: UpdateStatus | null;
}>();

const emit = defineEmits<{
  installLatestUpdate: [];
  openReleasePage: [];
}>();

function updateStatusClass(status: string) {
  return status.toLowerCase();
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

.about-card {
  gap: 14px;
}

.about-overview {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(180px, 260px);
  gap: 14px;
  align-items: stretch;
}

.about-product {
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 0;
}

.about-mark {
  width: 34px;
  min-width: 34px;
  height: 34px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  background: var(--surface-muted);
  color: var(--text);
  font-size: 0.9rem;
  font-weight: var(--font-heading);
  line-height: 1;
}

.about-title-block {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.about-title-block h3,
.about-title-block small {
  overflow-wrap: anywhere;
}

.about-version-list {
  display: grid;
  align-content: center;
  gap: 0;
  margin: 0;
}

.about-version-list div {
  min-width: 0;
  display: grid;
  grid-template-columns: 72px minmax(0, 1fr);
  gap: 8px;
  align-items: baseline;
  padding: 7px 0;
  border-bottom: 1px solid color-mix(in oklch, var(--line) 68%, transparent);
}

.about-version-list div:last-child {
  border-bottom: 0;
}

.about-version-list dt,
.update-detail-list dt {
  color: var(--text-muted);
  font-size: 0.76rem;
  font-weight: var(--font-label);
}

.about-version-list dd,
.update-detail-list dd {
  margin: 0;
  overflow-wrap: anywhere;
  color: var(--text);
  font-size: 0.84rem;
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

.status-downloading,
.status-installing,
.status-installed {
  background: var(--control-bg-active);
  color: var(--text);
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
  gap: 8px;
  padding: 10px 12px;
  border-radius: 6px;
  background: var(--surface-muted);
}

.update-summary-head {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 7px;
}

.update-summary-head strong {
  color: var(--text);
  font-size: 0.84rem;
}

.update-summary p {
  margin: 0;
  font-size: 0.84rem;
}

.update-summary small {
  overflow-wrap: anywhere;
}

.update-detail-list {
  display: grid;
  gap: 0;
  margin: 2px 0 0;
  border-top: 1px solid color-mix(in oklch, var(--line) 72%, transparent);
}

.update-detail-list div {
  min-width: 0;
  display: grid;
  grid-template-columns: 72px minmax(0, 1fr);
  gap: 8px;
  padding: 7px 0;
  border-bottom: 1px solid color-mix(in oklch, var(--line) 62%, transparent);
}

.update-detail-list div:last-child {
  border-bottom: 0;
}

.settings-actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 7px;
}

.about-actions {
  padding-top: 2px;
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

@media (max-width: 720px) {
  .settings-panel {
    padding-left: 14px;
    padding-right: 14px;
  }

  .about-overview {
    grid-template-columns: minmax(0, 1fr);
  }

  .about-product {
    padding: 4px 0 0;
  }
}
</style>
