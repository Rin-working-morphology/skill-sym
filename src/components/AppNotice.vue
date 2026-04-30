<template>
  <TransitionGroup
    name="notice-motion"
    tag="div"
    class="app-notice"
    aria-live="polite"
  >
    <div
      v-for="item in items"
      :key="item.key"
      class="notice-item"
      :class="item.kind"
      :role="item.kind === 'danger' ? 'alert' : 'status'"
    >
      <span class="notice-dot" aria-hidden="true" />
      <span class="notice-message">{{ item.message }}</span>
      <button
        type="button"
        class="notice-close"
        aria-label="关闭提示"
        @click="dismissNotice(item.key)"
      >
        ×
      </button>
    </div>
  </TransitionGroup>
</template>

<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from "vue";

type NoticeKind = "neutral" | "success" | "danger";

type NoticeItem = {
  key: string;
  kind: NoticeKind;
  message: string;
};

const AUTO_HIDE_DELAY = 3000;

const props = defineProps<{
  busy: boolean;
  statusMessage: string;
  errorMessage: string;
}>();

const hiddenKeys = ref(new Set<string>());
const hideTimers = new Map<string, ReturnType<typeof window.setTimeout>>();

function noticeKey(kind: NoticeKind, message: string) {
  return `${kind}:${message}`;
}

function clearHideTimer(key: string) {
  const timer = hideTimers.get(key);

  if (!timer) {
    return;
  }

  window.clearTimeout(timer);
  hideTimers.delete(key);
}

function dismissNotice(key: string) {
  const nextHiddenKeys = new Set(hiddenKeys.value);
  nextHiddenKeys.add(key);
  hiddenKeys.value = nextHiddenKeys;
  clearHideTimer(key);
}

const sourceItems = computed<NoticeItem[]>(() => {
  const next: NoticeItem[] = [];

  if (props.busy) {
    const message = "处理中";
    next.push({ key: noticeKey("neutral", message), kind: "neutral", message });
  }

  if (props.statusMessage) {
    next.push({
      key: noticeKey("success", props.statusMessage),
      kind: "success",
      message: props.statusMessage,
    });
  }

  if (props.errorMessage) {
    next.push({
      key: noticeKey("danger", props.errorMessage),
      kind: "danger",
      message: props.errorMessage,
    });
  }

  return next;
});

const items = computed(() =>
  sourceItems.value.filter((item) => !hiddenKeys.value.has(item.key)),
);

watch(
  sourceItems,
  (nextItems) => {
    const activeKeys = new Set(nextItems.map((item) => item.key));

    for (const key of hideTimers.keys()) {
      if (!activeKeys.has(key)) {
        clearHideTimer(key);
      }
    }

    const nextHiddenKeys = new Set(
      [...hiddenKeys.value].filter((key) => activeKeys.has(key)),
    );
    hiddenKeys.value = nextHiddenKeys;

    for (const item of nextItems) {
      if (hiddenKeys.value.has(item.key) || hideTimers.has(item.key)) {
        continue;
      }

      hideTimers.set(
        item.key,
        window.setTimeout(() => {
          dismissNotice(item.key);
        }, AUTO_HIDE_DELAY),
      );
    }
  },
  { immediate: true },
);

onUnmounted(() => {
  for (const key of hideTimers.keys()) {
    clearHideTimer(key);
  }
});
</script>

<style scoped>
.app-notice {
  position: fixed;
  z-index: 50;
  top: 10px;
  right: 12px;
  width: min(320px, calc(100vw - 232px));
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 6px;
  pointer-events: none;
}

:global(.app-shell.settings-mode) .app-notice {
  width: min(320px, calc(100vw - 24px));
}

.notice-item {
  max-width: 100%;
  min-height: 28px;
  display: inline-flex;
  align-items: center;
  gap: 7px;
  padding: 5px 8px;
  border: 1px solid var(--line);
  border-radius: 6px;
  background: var(--surface);
  box-shadow: 0 6px 16px color-mix(in oklch, var(--shadow) 72%, transparent);
  color: var(--text);
  font-size: 0.82rem;
  font-weight: var(--font-body);
  pointer-events: auto;
  overflow-wrap: anywhere;
  transition:
    border-color var(--motion-base) var(--ease-out-quint),
    background-color var(--motion-base) var(--ease-out-quint),
    box-shadow var(--motion-base) var(--ease-out-quint),
    color var(--motion-base) var(--ease-out-quint),
    transform var(--motion-fast) var(--ease-out-quart);
}

.notice-message {
  min-width: 0;
}

.notice-close {
  width: 18px;
  height: 18px;
  flex: 0 0 auto;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  margin-inline-start: 2px;
  border: 0;
  border-radius: 4px;
  background: transparent;
  color: color-mix(in oklch, currentColor 64%, transparent);
  font: inherit;
  line-height: 1;
  cursor: pointer;
  transition:
    background-color var(--motion-fast) var(--ease-out-quint),
    color var(--motion-fast) var(--ease-out-quint);
}

.notice-close:hover {
  background: color-mix(in oklch, currentColor 8%, transparent);
  color: currentColor;
}

.notice-close:focus-visible {
  outline: 2px solid color-mix(in oklch, currentColor 32%, transparent);
  outline-offset: 1px;
}

.notice-dot {
  width: 6px;
  height: 6px;
  flex: 0 0 auto;
  border-radius: 50%;
  background: var(--text-muted);
  transition:
    background-color var(--motion-base) var(--ease-out-quint),
    transform var(--motion-fast) var(--ease-out-quart);
}

.notice-item:hover .notice-dot {
  transform: scale(1.18);
}

.notice-item.success {
  border-color: color-mix(in oklch, var(--ok-text) 22%, var(--line));
  background: color-mix(in oklch, var(--ok-bg) 58%, var(--surface));
  color: var(--text);
}

.notice-item.success .notice-dot {
  background: var(--ok-text);
}

.notice-item.danger {
  border-color: color-mix(in oklch, var(--danger-text) 32%, var(--line));
  background: var(--danger-bg);
  color: var(--danger-text);
}

.notice-item.danger .notice-dot {
  background: var(--danger-text);
}

@media (max-width: 720px) {
  .app-notice {
    width: min(300px, calc(100vw - 218px));
  }

  :global(.app-shell.settings-mode) .app-notice {
    width: min(300px, calc(100vw - 24px));
  }
}

.notice-motion-enter-active,
.notice-motion-leave-active {
  transition:
    opacity var(--motion-base) var(--ease-out-quint),
    transform var(--motion-base) var(--ease-out-quint);
}

.notice-motion-enter-from,
.notice-motion-leave-to {
  opacity: 0;
  transform: translateY(-6px) scale(0.98);
}

.notice-motion-move {
  transition: transform var(--motion-slow) var(--ease-out-expo);
}
</style>
