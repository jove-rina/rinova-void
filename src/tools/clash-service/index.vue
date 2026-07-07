<script setup lang="ts">
/**
 * ClashServiceTool.vue（index.vue）
 * Clash 订阅服务工具页
 */
import { onMounted } from 'vue'
import { Check, Copy, Loader2, Play, RefreshCw, Shield, Square } from '@lucide/vue'
import ToolEntryLayout from '@/components/ToolEntryLayout.vue'
import VoidButton from '@/components/VoidButton.vue'
import VoidToast from '@/components/VoidToast.vue'
import { useClashService } from '@/composables/useClashService'

const {
  url,
  port,
  status,
  serviceUrl,
  toast,
  copied,
  refreshing,
  runner,
  portHint,
  portReclaimable,
  allowPortFallback,
  syncStatus,
  handleReclaimPort,
  handleStart,
  handleStop,
  handleRefresh,
  copyUrl,
  formatStatus,
  isFormDisabled,
} = useClashService()

onMounted(syncStatus)
</script>

<template>
  <ToolEntryLayout title="Clash 订阅服务" :icon="Shield" class="clash-tool">
    <div class="clash-tool__body">
      <div class="clash-tool__field">
        <label class="clash-tool__label" for="sub-url">订阅 URL</label>
        <input
          id="sub-url"
          v-model="url"
          type="url"
          class="clash-tool__input"
          placeholder="https://your-subscription-url"
          :disabled="isFormDisabled()"
          @keyup.enter="handleStart"
        />
      </div>

      <div class="clash-tool__field">
        <label class="clash-tool__label" for="sub-port">端口</label>
        <div class="clash-tool__port-row">
          <input
            id="sub-port"
            v-model.number="port"
            type="number"
            class="clash-tool__input clash-tool__input--port"
            min="1024"
            max="65535"
            :disabled="isFormDisabled()"
            @keyup.enter="handleStart"
          />
          <div class="clash-tool__actions">
            <template v-if="status === 'idle' || status === 'error'">
              <VoidButton size="large" @click="handleStart">
                <Play :size="16" :stroke-width="2" />
                启动服务
              </VoidButton>
            </template>
            <div v-else-if="status === 'starting'" class="clash-tool__loading">
              <Loader2 :size="16" :stroke-width="2" class="clash-tool__spin" />
              启动中...
            </div>
            <div v-else-if="status === 'stopping'" class="clash-tool__loading">
              <Loader2 :size="16" :stroke-width="2" class="clash-tool__spin" />
              停止中...
            </div>
            <template v-else-if="status === 'running'">
              <VoidButton variant="danger" size="large" @click="handleStop">
                <Square :size="14" :stroke-width="2" fill="currentColor" />
                停止服务
              </VoidButton>
            </template>
          </div>
        </div>
        <p v-if="portHint" class="clash-tool__port-hint">{{ portHint }}</p>
        <div v-if="portHint" class="clash-tool__port-actions">
          <button
            v-if="portReclaimable"
            type="button"
            class="clash-tool__port-action"
            :disabled="isFormDisabled()"
            @click="handleReclaimPort"
          >
            立即释放此端口
          </button>
          <label v-if="!portReclaimable" class="clash-tool__port-fallback">
            <input v-model="allowPortFallback" type="checkbox" :disabled="isFormDisabled()" />
            占用时自动换端口
          </label>
        </div>
      </div>

      <div class="clash-tool__status">
        <span class="clash-tool__status-dot" :class="`clash-tool__status-dot--${status}`"></span>
        {{ formatStatus(status) }}
        <span v-if="status === 'running' && runner" class="clash-tool__runner-tag">
          {{ runner === 'builtin' ? '内置' : runner }}
        </span>
      </div>

      <div v-if="status === 'running' && serviceUrl" class="clash-tool__endpoint">
        <div class="clash-tool__endpoint-header">
          <span class="clash-tool__endpoint-label">服务地址：</span>
          <button class="clash-tool__copy-btn" @click="copyUrl">
            <Check v-if="copied" :size="14" :stroke-width="2" />
            <Copy v-else :size="14" :stroke-width="2" />
            {{ copied ? '已复制' : '复制地址' }}
          </button>
        </div>
        <code class="clash-tool__endpoint-url">{{ serviceUrl }}/clash.yaml</code>
        <div class="clash-tool__endpoint-hint">可在 Clash Verge 中配置为远程订阅源</div>
        <div class="clash-tool__endpoint-actions">
          <button
            class="clash-tool__refresh-btn"
            :disabled="refreshing"
            @click="handleRefresh"
          >
            <RefreshCw :size="14" :stroke-width="2" :class="{ 'clash-tool__spin': refreshing }" />
            {{ refreshing ? '刷新中...' : '手动刷新' }}
          </button>
          <span class="clash-tool__interval-note">每 60 分钟自动刷新</span>
        </div>
      </div>
    </div>
  </ToolEntryLayout>

  <VoidToast :controller="toast" />
</template>

<style lang="less" scoped>
.clash-tool {
  &__body {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 14px;
    overflow-y: auto;
    overflow-x: hidden;
    scrollbar-width: none;
    -ms-overflow-style: none;

    &::-webkit-scrollbar {
      display: none;
    }
  }

  &__field {
    display: flex;
    flex-direction: column;
    gap: 4px;
    flex-shrink: 0;
  }

  &__port-row {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  &__port-hint {
    margin: 0;
    font-size: 11px;
    line-height: 1.4;
    color: #f59e0b;
  }

  &__port-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  &__port-action {
    padding: 2px 8px;
    font-size: 11px;
    border: 1px solid rgba(245, 158, 11, 0.4);
    border-radius: 4px;
    background: transparent;
    color: #f59e0b;
    cursor: pointer;

    &:hover:not(:disabled) {
      background: rgba(245, 158, 11, 0.1);
    }

    &:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }
  }

  &__port-fallback {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    color: var(--void-text-dim);
    cursor: pointer;
    user-select: none;
  }

  &__label {
    font-size: 12px;
    color: var(--void-text-dim);
    font-weight: 500;
  }

  &__input {
    padding: 10px 12px;
    border-radius: 8px;
    border: 1px solid var(--void-border);
    background: rgba(255, 255, 255, 0.05);
    color: var(--void-text);
    font-size: 14px;
    outline: none;
    transition: border-color 0.15s;
    user-select: text;

    &:focus {
      border-color: var(--void-accent);
    }

    &:disabled {
      opacity: 0.4;
    }

    &::placeholder {
      color: var(--void-text-dim);
      opacity: 0.4;
    }

    &--port {
      width: 120px;
    }
  }

  &__actions {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }

  &__status {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    color: var(--void-text-dim);
    flex-shrink: 0;
  }

  &__status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;

    &--idle { background: #6b7280; }
    &--starting { background: #f59e0b; animation: pulse 0.8s infinite; }
    &--running { background: #22c55e; }
    &--stopping { background: #f59e0b; animation: pulse 0.8s infinite; }
    &--error { background: #ef4444; }
  }

  &__runner-tag {
    margin-left: 4px;
    padding: 1px 6px;
    font-size: 11px;
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.06);
    color: var(--void-text-dim);
  }

  &__endpoint {
    padding: 12px;
    border-radius: 8px;
    background: rgba(192, 132, 252, 0.08);
    border: 1px solid rgba(192, 132, 252, 0.15);
    flex-shrink: 0;
  }

  &__endpoint-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 4px;
  }

  &__endpoint-label {
    font-size: 12px;
    color: var(--void-text-dim);
  }

  &__endpoint-url {
    font-size: 13px;
    color: var(--void-accent);
    user-select: text;
    word-break: break-all;
  }

  &__endpoint-hint {
    font-size: 11px;
    color: var(--void-text-dim);
    opacity: 0.6;
    margin-top: 4px;
  }

  &__endpoint-actions {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-top: 10px;
  }

  &__copy-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    border-radius: 6px;
    border: 1px solid var(--void-border);
    background: transparent;
    color: var(--void-accent);
    font-size: 12px;
    cursor: pointer;
    transition: background 0.15s;

    &:hover {
      background: rgba(192, 132, 252, 0.1);
    }
  }

  &__refresh-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 14px;
    border-radius: 6px;
    border: 1px solid var(--void-border);
    background: transparent;
    color: var(--void-text);
    font-size: 12px;
    cursor: pointer;
    transition: background 0.15s;

    &:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }

    &:hover:not(:disabled) {
      background: rgba(255, 255, 255, 0.08);
    }
  }

  &__interval-note {
    font-size: 11px;
    color: var(--void-text-dim);
    opacity: 0.5;
  }

  &__loading {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    color: var(--void-text-dim);
  }

  &__spin {
    animation: spin 1s linear infinite;
  }
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}
</style>
