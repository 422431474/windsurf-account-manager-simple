<template>
  <el-dialog
    v-model="dialogVisible"
    title="批量获取试用链接"
    width="700px"
    :close-on-click-modal="false"
    :close-on-press-escape="false"
    @close="handleClose"
  >
    <!-- 步骤提示 -->
    <el-steps :active="currentStep" finish-status="success" simple style="margin-bottom: 20px;">
      <el-step title="验证" />
      <el-step title="获取链接" />
      <el-step title="完成" />
    </el-steps>

    <!-- 第一步：验证码验证 -->
    <div v-if="currentStep === 0" class="step-content">
      <el-alert
        :title="`需要验证 ${needVerifyAccounts.length} 个账号`"
        type="info"
        :closable="false"
        show-icon
        style="margin-bottom: 15px;"
      >
        <template #default>
          <p>当前正在验证: {{ currentVerifyIndex + 1 }} / {{ needVerifyAccounts.length }}</p>
          <p style="color: #909399; font-size: 12px;">Pro 计划需要 Turnstile 验证，Teams 计划自动跳过验证</p>
        </template>
      </el-alert>

      <!-- 当前验证的账号信息 -->
      <div v-if="currentVerifyAccount" class="verify-account-info">
        <el-tag type="primary" size="large">{{ currentVerifyAccount.nickname || currentVerifyAccount.email }}</el-tag>
        <span class="verify-status" v-if="verifyResults[currentVerifyAccount.id]">
          <el-icon color="#67c23a"><CircleCheck /></el-icon> 已验证
        </span>
      </div>

      <!-- Turnstile 验证区域 -->
      <div class="turnstile-container" v-if="currentVerifyAccount && !verifyResults[currentVerifyAccount.id]">
        <div ref="turnstileRef" class="cf-turnstile"></div>
        <p v-if="turnstileStatus === 'loading'" class="status-text loading">
          <el-icon class="is-loading"><Loading /></el-icon>
          加载验证中...
        </p>
        <p v-else-if="turnstileStatus === 'success'" class="status-text success">
          <el-icon><CircleCheck /></el-icon>
          验证成功！
        </p>
        <p v-else-if="turnstileStatus === 'error'" class="status-text error">
          <el-icon><CircleClose /></el-icon>
          验证失败，请重试
        </p>
      </div>

      <!-- 验证进度列表 -->
      <div class="verify-progress-list">
        <div 
          v-for="(account, index) in needVerifyAccounts" 
          :key="account.id"
          class="verify-item"
          :class="{ 
            'current': index === currentVerifyIndex,
            'verified': verifyResults[account.id],
            'pending': index > currentVerifyIndex && !verifyResults[account.id]
          }"
        >
          <span class="verify-index">{{ index + 1 }}</span>
          <span class="verify-email">{{ account.nickname || account.email }}</span>
          <el-icon v-if="verifyResults[account.id]" color="#67c23a"><CircleCheck /></el-icon>
          <el-icon v-else-if="index === currentVerifyIndex" color="#409eff" class="is-loading"><Loading /></el-icon>
          <el-icon v-else color="#909399"><Clock /></el-icon>
        </div>
      </div>
    </div>

    <!-- 第二步：获取链接 -->
    <div v-else-if="currentStep === 1" class="step-content">
      <el-alert
        title="正在批量获取试用链接..."
        type="info"
        :closable="false"
        show-icon
        style="margin-bottom: 15px;"
      />
      <el-progress 
        :percentage="Math.round((fetchProgress.current / fetchProgress.total) * 100)" 
        :stroke-width="20"
        :format="() => `${fetchProgress.current}/${fetchProgress.total}`"
      />
      <p class="fetch-status">{{ fetchProgress.status }}</p>
    </div>

    <!-- 第三步：结果展示 -->
    <div v-else-if="currentStep === 2" class="step-content">
      <el-alert
        :title="`获取完成：成功 ${successResults.length} 个，失败 ${failedResults.length} 个`"
        :type="failedResults.length === 0 ? 'success' : 'warning'"
        :closable="false"
        show-icon
        style="margin-bottom: 15px;"
      />

      <!-- 成功结果列表 -->
      <div v-if="successResults.length > 0" class="result-section">
        <h4>成功获取的链接 ({{ successResults.length }})</h4>
        <el-scrollbar max-height="300px">
          <div v-for="result in successResults" :key="result.accountId" class="result-item success">
            <div class="result-account">
              <el-tag size="small">{{ result.accountName }}</el-tag>
            </div>
            <div class="result-url">
              <el-input 
                v-model="result.url" 
                readonly 
                size="small"
              >
                <template #append>
                  <el-button :icon="CopyDocument" @click="copyUrl(result.url!)" />
                </template>
              </el-input>
            </div>
          </div>
        </el-scrollbar>
      </div>

      <!-- 失败结果列表 -->
      <div v-if="failedResults.length > 0" class="result-section">
        <h4>获取失败 ({{ failedResults.length }})</h4>
        <div v-for="result in failedResults" :key="result.accountId" class="result-item failed">
          <el-tag size="small" type="danger">{{ result.accountName }}</el-tag>
          <span class="error-msg">{{ result.error }}</span>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleClose" :disabled="isProcessing">取消</el-button>
        
        <!-- 第一步：验证按钮 -->
        <template v-if="currentStep === 0">
          <el-button 
            type="primary" 
            @click="confirmCurrentVerify"
            :disabled="!canConfirmVerify"
          >
            确认验证 ({{ currentVerifyIndex + 1 }}/{{ needVerifyAccounts.length }})
          </el-button>
          <el-button 
            type="success" 
            @click="startFetchLinks"
            :disabled="!allVerified"
          >
            开始获取链接
          </el-button>
        </template>

        <!-- 第三步：结果操作按钮 -->
        <template v-if="currentStep === 2">
          <el-button 
            type="primary" 
            @click="copyAllUrls"
            :disabled="successResults.length === 0"
          >
            <el-icon><CopyDocument /></el-icon>
            复制全部链接
          </el-button>
          <el-button 
            type="success" 
            @click="openAllInIncognito"
            :disabled="successResults.length === 0"
            :loading="isOpeningBrowsers"
          >
            <el-icon><Monitor /></el-icon>
            批量打开无痕浏览器
          </el-button>
        </template>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onUnmounted } from 'vue';
import { ElMessage } from 'element-plus';
import { invoke } from '@tauri-apps/api/core';
import { 
  CircleCheck, 
  CircleClose, 
  Loading, 
  Clock, 
  CopyDocument,
  Monitor
} from '@element-plus/icons-vue';
import { useSettingsStore, useAccountsStore } from '@/store';
import { apiService } from '@/api';

const TURNSTILE_SITE_KEY = '0x4AAAAAAA447Bur1xJStKg5';

const props = defineProps<{
  modelValue: boolean;
  selectedAccountIds: string[];
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
}>();

const settingsStore = useSettingsStore();
const accountsStore = useAccountsStore();

const dialogVisible = ref(false);
const currentStep = ref(0);
const isProcessing = ref(false);
const isOpeningBrowsers = ref(false);

// Turnstile 相关
const turnstileRef = ref<HTMLElement | null>(null);
const turnstileStatus = ref<'idle' | 'loading' | 'success' | 'error'>('loading');
const currentTurnstileToken = ref('');
const widgetId = ref<string | null>(null);

// 验证相关
const currentVerifyIndex = ref(0);
const verifyResults = ref<Record<string, string>>({}); // accountId -> turnstileToken

// 获取链接相关
const fetchProgress = ref({ current: 0, total: 0, status: '' });

// 结果
interface LinkResult {
  accountId: string;
  accountName: string;
  url?: string;
  error?: string;
}
const successResults = ref<LinkResult[]>([]);
const failedResults = ref<LinkResult[]>([]);

// 获取选中的账号列表
const selectedAccounts = computed(() => {
  return accountsStore.accounts.filter(a => props.selectedAccountIds.includes(a.id));
});

// 需要验证的账号（Pro 计划需要验证）
const needVerifyAccounts = computed(() => {
  const teamsTier = settingsStore.settings?.subscriptionPlan ?? 2;
  // Pro 计划需要 Turnstile 验证
  if (teamsTier === 2) {
    return selectedAccounts.value;
  }
  // Teams 计划不需要验证
  return [];
});

// 当前正在验证的账号
const currentVerifyAccount = computed(() => {
  return needVerifyAccounts.value[currentVerifyIndex.value];
});

// 是否可以确认当前验证
const canConfirmVerify = computed(() => {
  return turnstileStatus.value === 'success' && currentTurnstileToken.value;
});

// 是否所有账号都已验证
const allVerified = computed(() => {
  if (needVerifyAccounts.value.length === 0) return true;
  return needVerifyAccounts.value.every(a => verifyResults.value[a.id]);
});

// 同步 modelValue
watch(() => props.modelValue, (val) => {
  dialogVisible.value = val;
  if (val) {
    resetState();
    initVerification();
  }
});

watch(dialogVisible, (val) => {
  emit('update:modelValue', val);
});

// 重置状态
function resetState() {
  currentStep.value = 0;
  currentVerifyIndex.value = 0;
  verifyResults.value = {};
  turnstileStatus.value = 'loading';
  currentTurnstileToken.value = '';
  fetchProgress.value = { current: 0, total: 0, status: '' };
  successResults.value = [];
  failedResults.value = [];
  isProcessing.value = false;
  isOpeningBrowsers.value = false;
}

// 初始化验证
async function initVerification() {
  const teamsTier = settingsStore.settings?.subscriptionPlan ?? 2;
  
  console.log(`[BatchTrialLink] 初始化验证, teamsTier: ${teamsTier}, 需要验证账号数: ${needVerifyAccounts.value.length}`);
  console.log(`[BatchTrialLink] 选中的账号:`, selectedAccounts.value.map(a => a.nickname || a.email));
  
  if (teamsTier !== 2 || needVerifyAccounts.value.length === 0) {
    // Teams 计划不需要验证，直接跳到获取链接
    // 为所有账号设置空 token
    console.log('[BatchTrialLink] Teams 计划，跳过 Turnstile 验证');
    selectedAccounts.value.forEach(a => {
      verifyResults.value[a.id] = '';
    });
    return;
  }

  console.log('[BatchTrialLink] Pro 计划，开始 Turnstile 验证');
  await nextTick();
  loadTurnstile();
}

// 加载 Turnstile 脚本
function loadTurnstileScript(): Promise<void> {
  return new Promise((resolve, reject) => {
    if ((window as any).turnstile) {
      resolve();
      return;
    }
    
    const script = document.createElement('script');
    script.src = 'https://challenges.cloudflare.com/turnstile/v0/api.js?render=explicit';
    script.async = true;
    script.defer = true;
    
    script.onload = () => resolve();
    script.onerror = () => reject(new Error('Failed to load Turnstile script'));
    
    document.head.appendChild(script);
  });
}

// 渲染 Turnstile
async function loadTurnstile() {
  try {
    turnstileStatus.value = 'loading';
    currentTurnstileToken.value = '';
    
    await loadTurnstileScript();
    
    await new Promise<void>((resolve) => {
      const checkTurnstile = () => {
        if ((window as any).turnstile) {
          resolve();
        } else {
          setTimeout(checkTurnstile, 100);
        }
      };
      checkTurnstile();
    });

    const turnstile = (window as any).turnstile;
    
    if (widgetId.value) {
      try {
        turnstile.remove(widgetId.value);
      } catch (e) {
        console.log('[Turnstile] Remove old widget failed:', e);
      }
    }

    if (turnstileRef.value) {
      turnstileRef.value.innerHTML = '';
    }

    await nextTick();

    // 等待 DOM 元素可用
    let attempts = 0;
    while (!turnstileRef.value && attempts < 10) {
      await new Promise(resolve => setTimeout(resolve, 100));
      attempts++;
    }
    
    console.log(`[BatchTrialLink] turnstileRef 状态: ${turnstileRef.value ? '可用' : '不可用'}, 尝试次数: ${attempts}`);

    if (turnstileRef.value) {
      widgetId.value = turnstile.render(turnstileRef.value, {
        sitekey: TURNSTILE_SITE_KEY,
        theme: 'light',
        callback: (token: string) => {
          console.log('[Turnstile] Verification success');
          currentTurnstileToken.value = token;
          turnstileStatus.value = 'success';
          // 自动确认并进入下一个账号
          autoConfirmAndNext(token);
        },
        'error-callback': () => {
          console.error('[Turnstile] Verification failed');
          turnstileStatus.value = 'error';
        },
        'expired-callback': () => {
          console.log('[Turnstile] Token expired');
          turnstileStatus.value = 'idle';
          currentTurnstileToken.value = '';
        }
      });
      
      turnstileStatus.value = 'idle';
    }
  } catch (error) {
    console.error('[Turnstile] Load error:', error);
    turnstileStatus.value = 'error';
  }
}

// 自动确认并进入下一个账号（Turnstile 验证成功后自动调用）
function autoConfirmAndNext(token: string) {
  if (!currentVerifyAccount.value) return;
  
  const accountId = currentVerifyAccount.value.id;
  const accountName = currentVerifyAccount.value.nickname || currentVerifyAccount.value.email;
  
  // 保存当前验证结果
  verifyResults.value[accountId] = token;
  console.log(`[BatchTrialLink] 账号 ${accountName} (${accountId}) 验证成功，token: ${token.substring(0, 20)}...`);
  
  const currentIndex = currentVerifyIndex.value;
  const total = needVerifyAccounts.value.length;
  
  ElMessage.success(`已验证 ${currentIndex + 1}/${total}`);
  
  // 移动到下一个
  if (currentIndex < total - 1) {
    currentVerifyIndex.value++;
    // 等待 DOM 更新后重新加载 Turnstile
    // 使用双重 nextTick 确保 v-if 条件更新后 DOM 完全渲染
    nextTick(() => {
      nextTick(() => {
        console.log(`[BatchTrialLink] 加载第 ${currentVerifyIndex.value + 1} 个账号的 Turnstile`);
        loadTurnstile();
      });
    });
  } else {
    // 所有验证完成，自动开始获取链接
    ElMessage.success('所有账号验证完成，开始获取链接...');
    setTimeout(() => {
      startFetchLinks();
    }, 500);
  }
}

// 确认当前验证（手动按钮，作为备用）
function confirmCurrentVerify() {
  if (!currentVerifyAccount.value || !currentTurnstileToken.value) return;
  autoConfirmAndNext(currentTurnstileToken.value);
}

// 开始获取链接
async function startFetchLinks() {
  currentStep.value = 1;
  isProcessing.value = true;
  
  const accounts = selectedAccounts.value;
  fetchProgress.value = { current: 0, total: accounts.length, status: '准备中...' };
  
  const teamsTier = settingsStore.settings?.subscriptionPlan ?? 2;
  const paymentPeriod = settingsStore.settings?.paymentPeriod ?? 1;
  const teamName = teamsTier === 1 ? (settingsStore.settings?.teamName || undefined) : undefined;
  const seatCount = settingsStore.settings?.seatCount ?? 1;
  
  // 调试：打印所有验证结果
  console.log('[BatchTrialLink] 开始获取链接，验证结果:', JSON.stringify(verifyResults.value, null, 2));
  console.log('[BatchTrialLink] 账号顺序:', accounts.map(a => `${a.nickname || a.email} (${a.id})`));
  
  for (const account of accounts) {
    fetchProgress.value.status = `正在获取: ${account.nickname || account.email}`;
    
    try {
      const turnstileToken = verifyResults.value[account.id] || undefined;
      console.log(`[BatchTrialLink] 获取 ${account.nickname || account.email} 的链接，使用 token: ${turnstileToken ? turnstileToken.substring(0, 20) + '...' : 'undefined'}`);
      
      const result = await apiService.getTrialPaymentLink(
        account.id,
        teamsTier,
        paymentPeriod,
        teamName,
        teamsTier === 1 ? seatCount : undefined,
        turnstileToken
      );
      
      console.log(`[BatchTrialLink] ${account.nickname || account.email} API 返回结果:`, result);
      
      if (result.success && result.stripe_url) {
        successResults.value.push({
          accountId: account.id,
          accountName: account.nickname || account.email,
          url: result.stripe_url
        });
      } else {
        console.error(`[BatchTrialLink] ${account.nickname || account.email} 获取失败:`, result.error);
        failedResults.value.push({
          accountId: account.id,
          accountName: account.nickname || account.email,
          error: result.error || '获取失败'
        });
      }
    } catch (error: any) {
      failedResults.value.push({
        accountId: account.id,
        accountName: account.nickname || account.email,
        error: error.toString()
      });
    }
    
    fetchProgress.value.current++;
  }
  
  fetchProgress.value.status = '获取完成';
  currentStep.value = 2;
  isProcessing.value = false;
}

// 复制单个链接
async function copyUrl(url: string) {
  try {
    await navigator.clipboard.writeText(url);
    ElMessage.success('链接已复制');
  } catch {
    ElMessage.error('复制失败');
  }
}

// 复制所有链接
async function copyAllUrls() {
  const urls = successResults.value.map(r => `${r.accountName}: ${r.url}`).join('\n');
  try {
    await navigator.clipboard.writeText(urls);
    ElMessage.success(`已复制 ${successResults.value.length} 个链接`);
  } catch {
    ElMessage.error('复制失败');
  }
}

// 批量打开无痕浏览器
async function openAllInIncognito() {
  isOpeningBrowsers.value = true;
  
  let successCount = 0;
  for (const result of successResults.value) {
    try {
      if (!result.url) continue;
      await invoke('open_external_link_incognito', { url: result.url });
      successCount++;
      // 添加延迟，避免同时打开太多窗口
      await new Promise(resolve => setTimeout(resolve, 500));
    } catch (error) {
      console.error(`打开链接失败 (${result.accountName}):`, error);
    }
  }
  
  isOpeningBrowsers.value = false;
  ElMessage.success(`已在无痕模式打开 ${successCount} 个链接`);
}

// 关闭对话框
function handleClose() {
  if (isProcessing.value) return;
  dialogVisible.value = false;
}

// 清理
onUnmounted(() => {
  if (widgetId.value && (window as any).turnstile) {
    try {
      (window as any).turnstile.remove(widgetId.value);
    } catch (e) {
      console.log('[Turnstile] Cleanup failed:', e);
    }
  }
});
</script>

<style scoped>
.step-content {
  min-height: 300px;
}

.verify-account-info {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 20px;
  padding: 10px;
  background: #f5f7fa;
  border-radius: 8px;
}

.verify-status {
  display: flex;
  align-items: center;
  gap: 4px;
  color: #67c23a;
  font-size: 14px;
}

.turnstile-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 20px 0;
  margin-bottom: 20px;
  background: #fafafa;
  border-radius: 8px;
}

.status-text {
  margin-top: 15px;
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 14px;
}

.status-text.loading {
  color: #909399;
}

.status-text.success {
  color: #67c23a;
}

.status-text.error {
  color: #f56c6c;
}

.verify-progress-list {
  max-height: 200px;
  overflow-y: auto;
  border: 1px solid #ebeef5;
  border-radius: 8px;
}

.verify-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  border-bottom: 1px solid #ebeef5;
}

.verify-item:last-child {
  border-bottom: none;
}

.verify-item.current {
  background: #ecf5ff;
}

.verify-item.verified {
  background: #f0f9eb;
}

.verify-item.pending {
  opacity: 0.6;
}

.verify-index {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #909399;
  color: white;
  border-radius: 50%;
  font-size: 12px;
}

.verify-item.current .verify-index {
  background: #409eff;
}

.verify-item.verified .verify-index {
  background: #67c23a;
}

.verify-email {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.fetch-status {
  margin-top: 15px;
  text-align: center;
  color: #606266;
}

.result-section {
  margin-top: 15px;
}

.result-section h4 {
  margin-bottom: 10px;
  color: #303133;
}

.result-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  margin-bottom: 8px;
  border-radius: 6px;
}

.result-item.success {
  background: #f0f9eb;
}

.result-item.failed {
  background: #fef0f0;
}

.result-account {
  flex-shrink: 0;
  min-width: 120px;
}

.result-url {
  flex: 1;
  overflow: hidden;
}

.error-msg {
  color: #f56c6c;
  font-size: 12px;
}

.dialog-footer {
  display: flex;
  gap: 10px;
  justify-content: flex-end;
}

.is-loading {
  animation: rotating 1s linear infinite;
}

@keyframes rotating {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

/* 深色模式 */
html.dark .verify-account-info {
  background: #262729;
}

html.dark .turnstile-container {
  background: #1d1e1f;
}

html.dark .verify-progress-list {
  border-color: #4c4d4f;
}

html.dark .verify-item {
  border-bottom-color: #4c4d4f;
}

html.dark .verify-item.current {
  background: rgba(64, 158, 255, 0.1);
}

html.dark .verify-item.verified {
  background: rgba(103, 194, 58, 0.1);
}

html.dark .result-item.success {
  background: rgba(103, 194, 58, 0.1);
}

html.dark .result-item.failed {
  background: rgba(245, 108, 108, 0.1);
}
</style>
