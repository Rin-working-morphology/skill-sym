<template>
  <Teleport to="body">
    <Transition name="custom-target-pop">
      <div class="custom-target-backdrop" @mousedown.self="emit('cancel')">
        <form
          class="custom-target-dialog"
          aria-labelledby="custom-target-title"
          @submit.prevent="submit"
          @keydown.esc.prevent="emit('cancel')"
        >
          <div class="custom-target-head">
            <h2 id="custom-target-title">添加发布目标</h2>
          </div>

          <label class="form-row">
            <span>名称</span>
            <input
              ref="nameInput"
              v-model="name"
              type="text"
              autocomplete="off"
              :disabled="busy"
              placeholder="例如 Claude"
            />
          </label>

          <label class="form-row">
            <span>skills 文件存放位置</span>
            <input
              v-model="folderName"
              type="text"
              autocomplete="off"
              :disabled="busy"
              placeholder="例如 .claude"
            />
          </label>

          <div class="dialog-actions">
            <button type="button" :disabled="busy" @click="emit('cancel')">
              取消
            </button>
            <button
              type="submit"
              class="primary-action"
              :disabled="busy || !isValid"
            >
              添加
            </button>
          </div>
        </form>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from "vue";

const props = defineProps<{
  busy: boolean;
}>();

const emit = defineEmits<{
  cancel: [];
  submit: [payload: { name: string; folderName: string }];
}>();

const name = ref("");
const folderName = ref("");
const nameInput = ref<HTMLInputElement | null>(null);

const isValid = computed(
  () => name.value.trim().length > 0 && folderName.value.trim().length > 0,
);

onMounted(async () => {
  await nextTick();
  nameInput.value?.focus();
});

function submit() {
  if (!isValid.value || props.busy) {
    return;
  }

  emit("submit", {
    name: name.value.trim(),
    folderName: folderName.value.trim(),
  });
}
</script>

<style scoped>
.custom-target-backdrop {
  position: fixed;
  z-index: 40;
  inset: 0;
  display: grid;
  place-items: center;
  padding: 18px;
  background: color-mix(in oklch, var(--bg) 72%, transparent);
}

.custom-target-dialog {
  width: min(100%, 360px);
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 14px;
  border: 1px solid var(--line);
  border-radius: 6px;
  background: var(--surface);
  box-shadow: 0 8px 18px color-mix(in oklch, var(--shadow) 74%, transparent);
}

.custom-target-head {
  padding-bottom: 2px;
}

.form-row {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 5px;
  color: var(--text-muted);
  font-size: var(--type-button-compact);
  font-weight: var(--font-label);
}

.form-row input {
  width: 100%;
  min-height: 30px;
  padding: 4px 8px;
  border: 1px solid var(--line);
  border-radius: 5px;
  background: var(--surface);
  color: var(--text);
  font: inherit;
  font-size: 0.84rem;
  font-weight: var(--font-body);
  outline: none;
  transition:
    border-color var(--motion-base) var(--ease-out-quint),
    background-color var(--motion-base) var(--ease-out-quint),
    box-shadow var(--motion-base) var(--ease-out-quint);
}

.form-row input:hover:not(:disabled) {
  background: var(--control-bg-hover);
}

.form-row input:focus {
  border-color: color-mix(in oklch, var(--accent) 48%, var(--line));
  box-shadow: 0 0 0 2px color-mix(in oklch, var(--accent) 18%, transparent);
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 7px;
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

.custom-target-pop-enter-active,
.custom-target-pop-leave-active {
  transition:
    opacity var(--motion-base) var(--ease-out-quint),
    transform var(--motion-base) var(--ease-out-quint);
}

.custom-target-pop-enter-from,
.custom-target-pop-leave-to {
  opacity: 0;
  transform: translateY(5px);
}
</style>
