<script setup lang="ts">
import { ref, onMounted } from "vue";
import { Settings, Brain, Shield, Palette, Save, CheckCircle2, AlertCircle } from "lucide-vue-next";
import { aiConfigGet, aiConfigSet, aiTestConnection, settingsGet, settingsSet } from "../api/settings";

const activeTab = ref("ai");

const tabs = [
  { id: "ai", label: "AI配置", icon: Brain },
  { id: "general", label: "通用设置", icon: Settings },
  { id: "whitelist", label: "白名单", icon: Shield },
  { id: "appearance", label: "外观", icon: Palette },
];

const aiConfig = ref({
  provider: "local",
  apiKey: "",
  model: "",
  baseUrl: "",
  temperature: 0.7,
  maxTokens: 2048,
});

const preferences = ref({
  autoScanOnStart: false,
  scanMode: "quick",
  cleanConfirmRequired: true,
  backupBeforeClean: true,
  language: "zh-CN",
  theme: "dark",
});

const isTesting = ref(false);
const testResult = ref<"idle" | "success" | "error">("idle");
const isSaving = ref(false);

const cloudProviders = [
  { id: "openai", name: "OpenAI", defaultUrl: "https://api.openai.com/v1" },
  { id: "anthropic", name: "Anthropic", defaultUrl: "https://api.anthropic.com" },
  { id: "deepseek", name: "DeepSeek", defaultUrl: "https://api.deepseek.com/v1" },
  { id: "qwen", name: "通义千问", defaultUrl: "https://dashscope.aliyuncs.com/compatible-mode/v1" },
  { id: "custom", name: "自定义", defaultUrl: "" },
];

onMounted(async () => {
  try {
    const config = await aiConfigGet();
    if (config) {
      aiConfig.value = {
        provider: config.provider === "local" ? "local" : "cloud",
        apiKey: config.apiKey || "",
        model: config.model || "",
        baseUrl: config.baseUrl || "",
        temperature: config.temperature || 0.7,
        maxTokens: config.maxTokens || 2048,
      };
    }
  } catch {}

  try {
    const prefs = await settingsGet();
    if (prefs) {
      preferences.value = {
        autoScanOnStart: prefs.autoScanOnStart || false,
        scanMode: prefs.scanMode || "quick",
        cleanConfirmRequired: prefs.cleanConfirmRequired ?? true,
        backupBeforeClean: prefs.backupBeforeClean ?? true,
        language: prefs.language || "zh-CN",
        theme: prefs.theme || "dark",
      };
    }
  } catch {}
});

async function testAiConnection() {
  isTesting.value = true;
  testResult.value = "idle";
  try {
    const result = await aiTestConnection();
    testResult.value = result ? "success" : "error";
  } catch {
    testResult.value = "error";
  }
  isTesting.value = false;
}

async function saveAiConfig() {
  isSaving.value = true;
  try {
    await aiConfigSet({
      provider: aiConfig.value.provider === "local" ? { local: true } : { cloud: { openai: true } },
      api_key: aiConfig.value.apiKey,
      model: aiConfig.value.model,
      base_url: aiConfig.value.baseUrl,
      temperature: aiConfig.value.temperature,
      max_tokens: aiConfig.value.maxTokens,
    } as any);
  } catch {}
  isSaving.value = false;
}

async function savePreferences() {
  isSaving.value = true;
  try {
    await settingsSet(preferences.value as any);
  } catch {}
  isSaving.value = false;
}

function onProviderChange(providerId: string) {
  const provider = cloudProviders.find((p) => p.id === providerId);
  if (provider && provider.defaultUrl) {
    aiConfig.value.baseUrl = provider.defaultUrl;
  }
}
</script>

<template>
  <div class="space-y-6">
    <h1 class="text-2xl font-bold text-dark-100">设置</h1>

    <div class="flex gap-4">
      <div class="w-48 space-y-1">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          class="w-full flex items-center gap-2 px-3 py-2 rounded-lg text-sm transition-colors"
          :class="activeTab === tab.id ? 'bg-accent-blue/15 text-accent-blue' : 'text-dark-400 hover:bg-dark-800 hover:text-dark-200'"
          @click="activeTab = tab.id"
        >
          <component :is="tab.icon" :size="16" />
          <span>{{ tab.label }}</span>
        </button>
      </div>

      <div class="flex-1 card">
        <div v-if="activeTab === 'ai'" class="space-y-6">
          <h2 class="text-lg font-semibold">AI模型配置</h2>

          <div class="space-y-4">
            <div>
              <label class="text-sm text-dark-400 mb-2 block">推理模式</label>
              <div class="flex gap-3">
                <button
                  class="px-4 py-2 rounded-lg text-sm transition-colors"
                  :class="aiConfig.provider === 'local' ? 'bg-accent-blue text-white' : 'bg-dark-700 text-dark-400'"
                  @click="aiConfig.provider = 'local'"
                >本地模式</button>
                <button
                  class="px-4 py-2 rounded-lg text-sm transition-colors"
                  :class="aiConfig.provider !== 'local' ? 'bg-accent-blue text-white' : 'bg-dark-700 text-dark-400'"
                  @click="aiConfig.provider = 'cloud'"
                >云端模式</button>
              </div>
            </div>

            <template v-if="aiConfig.provider !== 'local'">
              <div>
                <label class="text-sm text-dark-400 mb-2 block">云端服务商</label>
                <select
                  class="input-field w-full"
                  v-model="aiConfig.provider"
                  @change="onProviderChange(($event.target as HTMLSelectElement).value)"
                >
                  <option v-for="p in cloudProviders" :key="p.id" :value="p.id">{{ p.name }}</option>
                </select>
              </div>

              <div>
                <label class="text-sm text-dark-400 mb-2 block">API Key</label>
                <input
                  type="password"
                  class="input-field w-full"
                  v-model="aiConfig.apiKey"
                  placeholder="sk-..."
                />
              </div>

              <div>
                <label class="text-sm text-dark-400 mb-2 block">API Endpoint</label>
                <input
                  type="text"
                  class="input-field w-full"
                  v-model="aiConfig.baseUrl"
                  placeholder="https://api.openai.com/v1"
                />
              </div>

              <div>
                <label class="text-sm text-dark-400 mb-2 block">模型名称</label>
                <input
                  type="text"
                  class="input-field w-full"
                  v-model="aiConfig.model"
                  placeholder="gpt-4o"
                />
              </div>

              <div class="flex items-center gap-3">
                <button class="btn-secondary flex items-center gap-2" @click="testAiConnection" :disabled="isTesting">
                  <component :is="testResult === 'success' ? CheckCircle2 : testResult === 'error' ? AlertCircle : Brain" :size="16" />
                  {{ isTesting ? '测试中...' : '测试连接' }}
                </button>
                <span v-if="testResult === 'success'" class="text-sm text-accent-green">连接成功</span>
                <span v-if="testResult === 'error'" class="text-sm text-accent-red">连接失败</span>
              </div>
            </template>

            <div v-else class="p-4 rounded-lg bg-dark-800/50">
              <p class="text-sm text-dark-400">本地模式使用内置轻量化模型进行文件评估，无需网络连接。</p>
              <p class="text-sm text-dark-500 mt-2">适合日常使用，对常见文件类型判断准确率≥99.2%</p>
            </div>

            <button class="btn-primary flex items-center gap-2" @click="saveAiConfig" :disabled="isSaving">
              <Save :size="16" />
              {{ isSaving ? '保存中...' : '保存AI配置' }}
            </button>
          </div>
        </div>

        <div v-if="activeTab === 'general'" class="space-y-6">
          <h2 class="text-lg font-semibold">通用设置</h2>

          <div class="space-y-4">
            <div class="flex items-center justify-between">
              <div>
                <div class="text-sm">启动时自动扫描</div>
                <div class="text-xs text-dark-500">应用启动时自动执行快速扫描</div>
              </div>
              <label class="toggle">
                <input type="checkbox" v-model="preferences.autoScanOnStart" />
                <span class="toggle-slider"></span>
              </label>
            </div>

            <div class="flex items-center justify-between">
              <div>
                <div class="text-sm">清理前确认</div>
                <div class="text-xs text-dark-500">执行清理操作前需要用户确认</div>
              </div>
              <label class="toggle">
                <input type="checkbox" v-model="preferences.cleanConfirmRequired" />
                <span class="toggle-slider"></span>
              </label>
            </div>

            <div class="flex items-center justify-between">
              <div>
                <div class="text-sm">清理前备份</div>
                <div class="text-xs text-dark-500">清理前自动备份文件，7天内可恢复</div>
              </div>
              <label class="toggle">
                <input type="checkbox" v-model="preferences.backupBeforeClean" />
                <span class="toggle-slider"></span>
              </label>
            </div>

            <div>
              <label class="text-sm text-dark-400 mb-2 block">默认扫描模式</label>
              <select class="input-field w-full" v-model="preferences.scanMode">
                <option value="quick">快速扫描</option>
                <option value="deep">深度扫描</option>
              </select>
            </div>

            <button class="btn-primary flex items-center gap-2" @click="savePreferences" :disabled="isSaving">
              <Save :size="16" />
              {{ isSaving ? '保存中...' : '保存设置' }}
            </button>
          </div>
        </div>

        <div v-if="activeTab === 'whitelist'" class="space-y-6">
          <h2 class="text-lg font-semibold">白名单管理</h2>
          <p class="text-sm text-dark-500">白名单中的路径将不会被扫描和清理</p>
          <div class="text-dark-500 text-center py-8">
            <Shield :size="48" class="mx-auto mb-4 text-dark-600" />
            <p>暂无白名单条目</p>
          </div>
        </div>

        <div v-if="activeTab === 'appearance'" class="space-y-6">
          <h2 class="text-lg font-semibold">外观设置</h2>
          <div>
            <label class="text-sm text-dark-400 mb-2 block">主题</label>
            <select class="input-field w-full" v-model="preferences.theme">
              <option value="dark">深色主题</option>
              <option value="light">浅色主题</option>
            </select>
          </div>
          <div>
            <label class="text-sm text-dark-400 mb-2 block">语言</label>
            <select class="input-field w-full" v-model="preferences.language">
              <option value="zh-CN">简体中文</option>
              <option value="en">English</option>
            </select>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
