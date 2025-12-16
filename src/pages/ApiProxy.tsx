import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import {
    Power,
    Copy,
    RefreshCw,
    CheckCircle,
    Settings,
    Terminal,
    Code,
    Image as ImageIcon,
    BrainCircuit,
    Sparkles,
    Zap,
    Cpu
} from 'lucide-react';

interface ProxyStatus {
    running: boolean;
    port: number;
    base_url: string;
    active_accounts: number;
}

interface ProxyConfig {
    enabled: boolean;
    port: number;
    api_key: string;
    auto_start: boolean;
}

interface AppConfig {
    // Other fields omitted for brevity as we only need proxy here, 
    // but in a real app we might want to type them all. 
    // Since we pass the whole object back to save_config, we should try to type it loosely or fetch it fully.
    // For now, let's type what we know.
    language: string;
    theme: string;
    auto_refresh: boolean;
    refresh_interval: number;
    auto_sync: boolean;
    sync_interval: number;
    default_export_path: string | null;
    proxy: ProxyConfig;
}



export default function ApiProxy() {
    const { t } = useTranslation();


    const models = [
        {
            id: 'gemini-2.5-flash',
            name: 'Gemini 2.5 Flash',
            desc: t('proxy.model.flash'),
            icon: <Zap size={16} />
        },
        {
            id: 'gemini-2.5-flash-thinking',
            name: 'Gemini 2.5 Flash Thinking',
            desc: t('proxy.model.flash_thinking'),
            icon: <BrainCircuit size={16} />
        },
        {
            id: 'gemini-3-pro-low',
            name: 'Gemini 3 Pro (Low)',
            desc: t('proxy.model.pro_low'),
            icon: <Sparkles size={16} />
        },
        {
            id: 'gemini-3-pro-high',
            name: 'Gemini 3 Pro (High)',
            desc: t('proxy.model.pro_high'),
            icon: <Cpu size={16} />
        },
        {
            id: 'gemini-3-pro-image',
            name: 'Gemini 3 Pro Vision',
            desc: t('proxy.model.pro_image'),
            icon: <ImageIcon size={16} />
        },
        {
            id: 'claude-sonnet-4-5',
            name: 'Claude 4.5 Sonnet',
            desc: t('proxy.model.claude_sonnet'),
            icon: <Sparkles size={16} />
        },
        {
            id: 'claude-sonnet-4-5-thinking',
            name: 'Claude 4.5 Sonnet Thinking',
            desc: t('proxy.model.claude_sonnet_thinking'),
            icon: <BrainCircuit size={16} />
        },
        {
            id: 'claude-opus-4-5-thinking',
            name: 'Claude 4.5 Opus Thinking',
            desc: t('proxy.model.claude_opus_thinking'),
            icon: <BrainCircuit size={16} />
        }
    ];

    const [status, setStatus] = useState<ProxyStatus>({
        running: false,
        port: 0,
        base_url: '',
        active_accounts: 0,
    });

    const [appConfig, setAppConfig] = useState<AppConfig | null>(null);
    const [loading, setLoading] = useState(false);
    const [copied, setCopied] = useState<string | null>(null);
    const [activeTab, setActiveTab] = useState('gemini-2.5-flash');

    // 初始化加载
    useEffect(() => {
        loadConfig();
        loadStatus();
        const interval = setInterval(loadStatus, 3000);
        return () => clearInterval(interval);
    }, []);

    const loadConfig = async () => {
        try {
            const config = await invoke<AppConfig>('load_config');
            setAppConfig(config);
        } catch (error) {
            console.error('加载配置失败:', error);
        }
    };

    const loadStatus = async () => {
        try {
            const s = await invoke<ProxyStatus>('get_proxy_status');
            setStatus(s);
        } catch (error) {
            console.error('获取状态失败:', error);
        }
    };

    const saveConfig = async (newConfig: AppConfig) => {
        try {
            await invoke('save_config', { config: newConfig });
            setAppConfig(newConfig);
        } catch (error) {
            console.error('保存配置失败:', error);
            alert('保存配置失败: ' + error);
        }
    };

    const updateProxyConfig = (updates: Partial<ProxyConfig>) => {
        if (!appConfig) return;
        const newConfig = {
            ...appConfig,
            proxy: {
                ...appConfig.proxy,
                ...updates
            }
        };
        saveConfig(newConfig);
    };

    const handleToggle = async () => {
        if (!appConfig) return;
        setLoading(true);
        try {
            if (status.running) {
                await invoke('stop_proxy_service');
            } else {
                // 使用当前的 appConfig.proxy 启动
                await invoke('start_proxy_service', { config: appConfig.proxy });
            }
            await loadStatus();
        } catch (error: any) {
            alert(t('proxy.dialog.operate_failed', { error }));
        } finally {
            setLoading(false);
        }
    };

    const handleGenerateApiKey = async () => {
        if (confirm(t('proxy.dialog.confirm_regenerate'))) {
            try {
                const newKey = await invoke<string>('generate_api_key');
                updateProxyConfig({ api_key: newKey });
            } catch (error) {
                console.error('生成 API Key 失败:', error);
                alert(t('proxy.dialog.operate_failed', { error }));
            }
        }
    };

    const copyToClipboard = (text: string, label: string) => {
        navigator.clipboard.writeText(text).then(() => {
            setCopied(label);
            setTimeout(() => setCopied(null), 2000);
        });
    };

    const getCurlExample = (modelId: string) => {
        const port = status.running ? status.port : (appConfig?.proxy.port || 8045);
        const baseUrl = `http://localhost:${port}`;
        const apiKey = appConfig?.proxy.api_key || 'YOUR_API_KEY';

        if (modelId === 'gemini-3-pro-image') {
            return `curl ${baseUrl}/v1/chat/completions \\
  -H "Content-Type: application/json" \\
  -H "Authorization: Bearer ${apiKey}" \\
  -d '{
    "model": "gemini-3-pro-image",
    "messages": [
      {
        "role": "user", 
        "content": [
          {"type": "text", "text": "Draw a cute cat"}
        ]
      }
    ]
  }'`;
        }

        return `curl ${baseUrl}/v1/chat/completions \\
  -H "Content-Type: application/json" \\
  -H "Authorization: Bearer ${apiKey}" \\
  -d '{
    "model": "${modelId}",
    "messages": [{"role": "user", "content": "Hello"}]
  }'`;
    };

    const getPythonExample = (modelId: string) => {
        const port = status.running ? status.port : (appConfig?.proxy.port || 8045);
        const baseUrl = `http://localhost:${port}/v1`;
        const apiKey = appConfig?.proxy.api_key || 'YOUR_API_KEY';

        if (modelId === 'gemini-3-pro-image') {
            return `from openai import OpenAI

client = OpenAI(
    base_url="${baseUrl}",
    api_key="${apiKey}"
)

response = client.chat.completions.create(
    model="gemini-3-pro-image",
    messages=[{
        "role": "user",
        "content": [
            {"type": "text", "text": "Draw a futuristic city"}
        ]
    }]
)

print(response.choices[0].message.content)`;
        }

        return `from openai import OpenAI

client = OpenAI(
    base_url="${baseUrl}",
    api_key="${apiKey}"
)

response = client.chat.completions.create(
    model="${modelId}",
    messages=[{"role": "user", "content": "Hello"}]
)

print(response.choices[0].message.content)`;
    };

    return (
        <div className="h-full w-full overflow-y-auto">
            <div className="p-5 space-y-4 max-w-7xl mx-auto">
                <div className="flex items-center justify-between">
                    <h1 className="text-2xl font-bold text-gray-900 dark:text-base-content">{t('proxy.title')}</h1>
                </div>

                {/* 服务状态卡片 */}
                <div className="bg-white dark:bg-base-100 rounded-xl p-4 shadow-sm border border-gray-100 dark:border-base-200">
                    <div className="flex items-center justify-between">
                        <div className="flex items-center gap-3">
                            <div className={`w-3 h-3 rounded-full ${status.running ? 'bg-green-500 animate-pulse' : 'bg-gray-400'}`} />
                            <div>
                                <h2 className="text-lg font-semibold text-gray-900 dark:text-base-content">
                                    {status.running ? t('proxy.status.running') : t('proxy.status.stopped')}
                                </h2>
                                {status.running && (
                                    <p className="text-sm text-gray-500 dark:text-gray-400">
                                        {t('proxy.status.accounts_available', { count: status.active_accounts })}
                                    </p>
                                )}
                            </div>
                        </div>
                        <button
                            onClick={handleToggle}
                            disabled={loading || !appConfig}
                            className={`px-4 py-2 rounded-lg font-medium transition-colors flex items-center gap-2 ${status.running
                                ? 'bg-red-500 hover:bg-red-600 text-white'
                                : 'bg-blue-500 hover:bg-blue-600 text-white'
                                } ${(loading || !appConfig) ? 'opacity-50 cursor-not-allowed' : ''}`}
                        >
                            <Power size={18} />
                            {loading ? t('proxy.status.processing') : (status.running ? t('proxy.action.stop') : t('proxy.action.start'))}
                        </button>
                    </div>
                </div>

                {/* 配置区 */}
                {appConfig && (
                    <div className="bg-white dark:bg-base-100 rounded-xl shadow-sm border border-gray-100 dark:border-base-200">
                        <div className="p-4 border-b border-gray-100 dark:border-base-200">
                            <h2 className="text-lg font-semibold flex items-center gap-2 text-gray-900 dark:text-base-content">
                                <Settings size={20} />
                                {t('proxy.config.title')}
                            </h2>
                        </div>
                        <div className="p-4 space-y-4">
                            {/* 监听端口和自启动 */}
                            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                                <div>
                                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                        {t('proxy.config.port')}
                                    </label>
                                    <input
                                        type="number"
                                        value={appConfig.proxy.port}
                                        onChange={(e) => updateProxyConfig({ port: parseInt(e.target.value) })}
                                        min={8000}
                                        max={65535}
                                        disabled={status.running}
                                        className="w-full px-3 py-2 border border-gray-300 dark:border-base-200 rounded-lg bg-white dark:bg-base-200 text-gray-900 dark:text-base-content focus:ring-2 focus:ring-blue-500 focus:border-transparent disabled:opacity-50 disabled:cursor-not-allowed"
                                    />
                                    <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
                                        {t('proxy.config.port_hint')}
                                    </p>
                                </div>
                                <div className="flex items-center">
                                    <label className="flex items-center cursor-pointer gap-3">
                                        <div className="relative">
                                            <input
                                                type="checkbox"
                                                className="sr-only"
                                                checked={appConfig.proxy.auto_start}
                                                onChange={(e) => updateProxyConfig({ auto_start: e.target.checked })}
                                            />
                                            <div className={`block w-14 h-8 rounded-full transition-colors ${appConfig.proxy.auto_start ? 'bg-blue-500' : 'bg-gray-300 dark:bg-base-300'}`}></div>
                                            <div className={`dot absolute left-1 top-1 bg-white w-6 h-6 rounded-full transition-transform ${appConfig.proxy.auto_start ? 'transform translate-x-6' : ''}`}></div>
                                        </div>
                                        <span className="text-sm font-medium text-gray-900 dark:text-base-content">
                                            {t('proxy.config.auto_start')}
                                        </span>
                                    </label>
                                </div>
                            </div>

                            {/* API 密钥 */}
                            <div>
                                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                    {t('proxy.config.api_key')}
                                </label>
                                <div className="flex gap-2">
                                    <input
                                        type="text" // 改为 text 以便复制，或者可以保留 password 点击显示
                                        value={appConfig.proxy.api_key}
                                        readOnly
                                        className="flex-1 px-3 py-2 border border-gray-300 dark:border-base-200 rounded-lg bg-gray-50 dark:bg-base-300 text-gray-600 dark:text-gray-400 font-mono"
                                    />
                                    <button
                                        onClick={handleGenerateApiKey}
                                        className="px-3 py-2 border border-gray-300 dark:border-base-200 rounded-lg bg-white dark:bg-base-200 hover:bg-gray-50 dark:hover:bg-base-300 transition-colors"
                                        title={t('proxy.config.btn_regenerate')}
                                    >
                                        <RefreshCw size={18} />
                                    </button>
                                    <button
                                        onClick={() => copyToClipboard(appConfig.proxy.api_key, 'api_key')}
                                        className="px-3 py-2 border border-gray-300 dark:border-base-200 rounded-lg bg-white dark:bg-base-200 hover:bg-gray-50 dark:hover:bg-base-300 transition-colors"
                                        title={t('proxy.config.btn_copy')}
                                    >
                                        {copied === 'api_key' ? (
                                            <CheckCircle size={18} className="text-green-500" />
                                        ) : (
                                            <Copy size={18} />
                                        )}
                                    </button>
                                </div>
                                <p className="mt-1 text-xs text-amber-600 dark:text-amber-500">
                                    {t('proxy.config.warning_key')}
                                </p>
                            </div>
                        </div>
                    </div>
                )}

                {/* 使用说明 */}
                {appConfig && (
                    <div className="bg-white dark:bg-base-100 rounded-xl shadow-sm border border-gray-100 dark:border-base-200 overflow-hidden">
                        <div className="p-4 border-b border-gray-100 dark:border-base-200">
                            <h2 className="text-lg font-semibold text-gray-900 dark:text-base-content">{t('proxy.example.title')}</h2>
                        </div>

                        {/* Tabs */}
                        <div className="flex border-b border-gray-100 dark:border-base-200 overflow-x-auto">
                            {models.map((model) => (
                                <button
                                    key={model.id}
                                    onClick={() => setActiveTab(model.id)}
                                    className={`flex items-center gap-2 px-4 py-3 text-sm font-medium transition-colors whitespace-nowrap ${activeTab === model.id
                                        ? 'text-blue-600 dark:text-blue-400 border-b-2 border-blue-600 dark:border-blue-400 bg-blue-50/50 dark:bg-blue-900/10'
                                        : 'text-gray-600 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-base-200'
                                        }`}
                                >
                                    {model.icon}
                                    {model.name}
                                    <span className="text-xs opacity-60 ml-1">({model.desc})</span>
                                </button>
                            ))}
                        </div>

                        <div className="p-4 space-y-4">
                            <div>
                                <h3 className="flex items-center justify-between font-medium mb-2 text-gray-900 dark:text-base-content">
                                    <span className="flex items-center gap-2">
                                        <Terminal size={16} />
                                        {t('proxy.example.curl')}
                                    </span>
                                    <button
                                        onClick={() => copyToClipboard(getCurlExample(activeTab), 'curl')}
                                        className="text-xs flex items-center gap-1 text-blue-600 hover:text-blue-700"
                                    >
                                        {copied === 'curl' ? <CheckCircle size={14} /> : <Copy size={14} />}
                                        {copied === 'curl' ? t('proxy.config.btn_copied') : t('proxy.config.btn_copy')}
                                    </button>
                                </h3>
                                <pre className="p-3 bg-gray-900 rounded-lg text-sm overflow-x-auto text-gray-100 font-mono">
                                    {getCurlExample(activeTab)}
                                </pre>
                            </div>

                            <div>
                                <h3 className="flex items-center justify-between font-medium mb-2 text-gray-900 dark:text-base-content">
                                    <span className="flex items-center gap-2">
                                        <Code size={16} />
                                        {t('proxy.example.python')}
                                    </span>
                                    <button
                                        onClick={() => copyToClipboard(getPythonExample(activeTab), 'python')}
                                        className="text-xs flex items-center gap-1 text-blue-600 hover:text-blue-700"
                                    >
                                        {copied === 'python' ? <CheckCircle size={14} /> : <Copy size={14} />}
                                        {copied === 'python' ? t('proxy.config.btn_copied') : t('proxy.config.btn_copy')}
                                    </button>
                                </h3>
                                <pre className="p-3 bg-gray-900 rounded-lg text-sm overflow-x-auto text-gray-100 font-mono">
                                    {getPythonExample(activeTab)}
                                </pre>
                            </div>
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
}
