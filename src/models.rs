use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEntry {
    pub id: String,
    pub channel_id: String,
    pub channel_name: Option<String>,
    pub model: String,
    pub display_name: Option<String>,
    pub enabled: bool,
    pub priority: i32,
    pub sort_index: i32,
    pub weight: i32,
    pub response_ms: Option<String>,
    pub cooldown_until: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateEntry {
    pub channel_id: String,
    pub model: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub priority: Option<i32>,
    #[serde(default)]
    pub weight: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ReorderEntries {
    pub ordered_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub total_requests: i64,
    pub today_requests: i64,
    pub total_prompt_tokens: i64,
    pub total_completion_tokens: i64,
    pub today_prompt_tokens: i64,
    pub today_completion_tokens: i64,
}

#[derive(Debug, Serialize)]
pub struct ChartDataPoint {
    pub time: String,
    pub model: String,
    pub value: i64,
}

#[derive(Debug, Serialize)]
pub struct ModelRanking {
    pub model: String,
    pub count: i64,
    pub tokens: i64,
}

#[derive(Debug, Serialize)]
pub struct ChannelTreeStats {
    pub channel_id: String,
    pub channel_name: String,
    pub total_requests: i64,
    pub total_tokens: i64,
    pub models: Vec<ModelTreeStats>,
}

#[derive(Debug, Serialize)]
pub struct ModelTreeStats {
    pub model: String,
    pub requests: i64,
    pub tokens: i64,
    pub success_count: i64,
    pub error_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub api_type: String,
    pub base_url: String,
    pub api_key: String,
    pub models: String,
    pub enabled: bool,
    pub priority: i32,
    pub weight: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateChannel {
    pub name: String,
    pub api_type: String,
    pub base_url: String,
    pub api_key: String,
    #[serde(default)]
    pub models: String,
    #[serde(default)]
    pub priority: i32,
    #[serde(default = "default_weight")]
    pub weight: i32,
}

fn default_weight() -> i32 { 1 }

#[derive(Debug, Deserialize)]
pub struct UpdateChannel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: String,
    pub name: String,
    pub key: String,
    pub usage_count: i64,
    pub usage_limit: i64,
    pub enabled: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateApiKey {
    #[serde(default = "default_key_name")]
    pub name: String,
    #[serde(default)]
    pub usage_limit: i64,
}

fn default_key_name() -> String { "default".into() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestLog {
    pub id: String,
    pub channel_id: Option<String>,
    pub channel_name: Option<String>,
    pub model: Option<String>,
    pub api_key_id: Option<String>,
    pub request_type: String,
    pub status_code: i32,
    pub latency_ms: i64,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub error: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct LogStats {
    pub total: i64,
    pub success: i64,
    pub errors: i64,
    pub today: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_threshold")]
    pub circuit_breaker_threshold: i32,
    #[serde(default = "default_reset_time")]
    pub circuit_breaker_reset_time: i32,
    #[serde(default = "default_retry")]
    pub retry_times: i32,
    #[serde(default = "default_timeout")]
    pub timeout: i32,
    #[serde(default)]
    pub auto_select_new_models: bool,
    #[serde(default)]
    pub max_tokens_per_month: i64,
    #[serde(default)]
    pub default_model: String,
}

fn default_threshold() -> i32 { 5 }
fn default_reset_time() -> i32 { 300 }
fn default_retry() -> i32 { 3 }
fn default_timeout() -> i32 { 60000 }

impl Default for Settings {
    fn default() -> Self {
        Self {
            circuit_breaker_threshold: default_threshold(),
            circuit_breaker_reset_time: default_reset_time(),
            retry_times: default_retry(),
            timeout: default_timeout(),
            auto_select_new_models: true,
            max_tokens_per_month: 0,
            default_model: String::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<serde_json::Value>,
    #[serde(default)]
    pub stream: bool,
    #[serde(default)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct ModelBreakerStatus {
    pub channel_id: String,
    pub model: String,
    pub failures: i32,
    pub open: bool,
    pub cooldown_until: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ResetModelBreakerRequest {
    pub channel_id: String,
    pub model: String,
}

// ==================== 模型标签系统 ====================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelType {
    #[serde(rename = "chat")]
    Chat,
    #[serde(rename = "embedding")]
    Embedding,
    #[serde(rename = "rerank")]
    Rerank,
    #[serde(rename = "tts")]
    TTS,
    #[serde(rename = "whisper")]
    Whisper,
    #[serde(rename = "vision")]
    Vision,
    #[serde(rename = "image")]
    Image,
}

impl Default for ModelType {
    fn default() -> Self { Self::Chat }
}

impl std::fmt::Display for ModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelType::Chat => write!(f, "chat"),
            ModelType::Embedding => write!(f, "embedding"),
            ModelType::Rerank => write!(f, "rerank"),
            ModelType::TTS => write!(f, "tts"),
            ModelType::Whisper => write!(f, "whisper"),
            ModelType::Vision => write!(f, "vision"),
            ModelType::Image => write!(f, "image"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelTag {
    pub model: String,
    pub model_type: ModelType,
    pub context_window: i32,
    pub max_output: i32,
    pub description: String,
    pub provider: String,
}

impl Default for ModelTag {
    fn default() -> Self {
        Self {
            model: String::new(),
            model_type: ModelType::Chat,
            context_window: 4096,
            max_output: 4096,
            description: String::new(),
            provider: String::new(),
        }
    }
}

// 内置模型标签库
pub fn get_builtin_model_tags() -> Vec<ModelTag> {
    vec![
        // OpenAI Chat
        ModelTag { model: "gpt-4o".into(), model_type: ModelType::Chat, context_window: 128000, max_output: 16384, description: "GPT-4 Omni".into(), provider: "OpenAI".into() },
        ModelTag { model: "gpt-4o-mini".into(), model_type: ModelType::Chat, context_window: 128000, max_output: 16384, description: "GPT-4 Omni Mini".into(), provider: "OpenAI".into() },
        ModelTag { model: "gpt-4-turbo".into(), model_type: ModelType::Chat, context_window: 128000, max_output: 4096, description: "GPT-4 Turbo".into(), provider: "OpenAI".into() },
        ModelTag { model: "gpt-4".into(), model_type: ModelType::Chat, context_window: 8192, max_output: 4096, description: "GPT-4".into(), provider: "OpenAI".into() },
        ModelTag { model: "gpt-3.5-turbo".into(), model_type: ModelType::Chat, context_window: 16385, max_output: 4096, description: "GPT-3.5 Turbo".into(), provider: "OpenAI".into() },

        // OpenAI Embedding
        ModelTag { model: "text-embedding-3-large".into(), model_type: ModelType::Embedding, context_window: 8191, max_output: 3072, description: "OpenAI Embedding Large".into(), provider: "OpenAI".into() },
        ModelTag { model: "text-embedding-3-small".into(), model_type: ModelType::Embedding, context_window: 8191, max_output: 1536, description: "OpenAI Embedding Small".into(), provider: "OpenAI".into() },
        ModelTag { model: "text-embedding-ada-002".into(), model_type: ModelType::Embedding, context_window: 8191, max_output: 1536, description: "OpenAI Embedding Ada".into(), provider: "OpenAI".into() },

        // OpenAI TTS/Whisper
        ModelTag { model: "tts-1".into(), model_type: ModelType::TTS, context_window: 4096, max_output: 0, description: "OpenAI TTS".into(), provider: "OpenAI".into() },
        ModelTag { model: "tts-1-hd".into(), model_type: ModelType::TTS, context_window: 4096, max_output: 0, description: "OpenAI TTS HD".into(), provider: "OpenAI".into() },
        ModelTag { model: "whisper-1".into(), model_type: ModelType::Whisper, context_window: 0, max_output: 0, description: "OpenAI Whisper".into(), provider: "OpenAI".into() },

        // OpenAI Image
        ModelTag { model: "dall-e-3".into(), model_type: ModelType::Image, context_window: 0, max_output: 0, description: "DALL-E 3".into(), provider: "OpenAI".into() },
        ModelTag { model: "dall-e-2".into(), model_type: ModelType::Image, context_window: 0, max_output: 0, description: "DALL-E 2".into(), provider: "OpenAI".into() },

        // Claude
        ModelTag { model: "claude-4-opus".into(), model_type: ModelType::Chat, context_window: 200000, max_output: 32000, description: "Claude 4 Opus".into(), provider: "Anthropic".into() },
        ModelTag { model: "claude-4-sonnet".into(), model_type: ModelType::Chat, context_window: 200000, max_output: 32000, description: "Claude 4 Sonnet".into(), provider: "Anthropic".into() },
        ModelTag { model: "claude-3-5-sonnet".into(), model_type: ModelType::Chat, context_window: 200000, max_output: 8192, description: "Claude 3.5 Sonnet".into(), provider: "Anthropic".into() },
        ModelTag { model: "claude-3-opus".into(), model_type: ModelType::Chat, context_window: 200000, max_output: 4096, description: "Claude 3 Opus".into(), provider: "Anthropic".into() },
        ModelTag { model: "claude-3-sonnet".into(), model_type: ModelType::Chat, context_window: 200000, max_output: 4096, description: "Claude 3 Sonnet".into(), provider: "Anthropic".into() },
        ModelTag { model: "claude-3-haiku".into(), model_type: ModelType::Chat, context_window: 200000, max_output: 4096, description: "Claude 3 Haiku".into(), provider: "Anthropic".into() },

        // Gemini
        ModelTag { model: "gemini-2.5-pro".into(), model_type: ModelType::Chat, context_window: 1048576, max_output: 65536, description: "Gemini 2.5 Pro".into(), provider: "Google".into() },
        ModelTag { model: "gemini-2.0-flash".into(), model_type: ModelType::Chat, context_window: 1048576, max_output: 8192, description: "Gemini 2.0 Flash".into(), provider: "Google".into() },
        ModelTag { model: "gemini-1.5-pro".into(), model_type: ModelType::Chat, context_window: 2097152, max_output: 8192, description: "Gemini 1.5 Pro".into(), provider: "Google".into() },
        ModelTag { model: "text-embedding-004".into(), model_type: ModelType::Embedding, context_window: 2048, max_output: 768, description: "Gemini Embedding".into(), provider: "Google".into() },

        // DeepSeek
        ModelTag { model: "deepseek-chat".into(), model_type: ModelType::Chat, context_window: 64000, max_output: 4096, description: "DeepSeek Chat".into(), provider: "DeepSeek".into() },
        ModelTag { model: "deepseek-reasoner".into(), model_type: ModelType::Chat, context_window: 64000, max_output: 4096, description: "DeepSeek Reasoner".into(), provider: "DeepSeek".into() },

        // Qwen
        ModelTag { model: "qwen-turbo".into(), model_type: ModelType::Chat, context_window: 131072, max_output: 8192, description: "Qwen Turbo".into(), provider: "Alibaba".into() },
        ModelTag { model: "qwen-plus".into(), model_type: ModelType::Chat, context_window: 131072, max_output: 8192, description: "Qwen Plus".into(), provider: "Alibaba".into() },
        ModelTag { model: "qwen-max".into(), model_type: ModelType::Chat, context_window: 32768, max_output: 8192, description: "Qwen Max".into(), provider: "Alibaba".into() },
        ModelTag { model: "qwen-vl".into(), model_type: ModelType::Vision, context_window: 8192, max_output: 4096, description: "Qwen Vision".into(), provider: "Alibaba".into() },
        ModelTag { model: "text-embedding-v3".into(), model_type: ModelType::Embedding, context_window: 8192, max_output: 1024, description: "Qwen Embedding".into(), provider: "Alibaba".into() },

        // SiliconFlow Embedding (BGE)
        ModelTag { model: "BAAI/bge-m3".into(), model_type: ModelType::Embedding, context_window: 8192, max_output: 1024, description: "BGE M3 多语言向量".into(), provider: "BAAI".into() },
        ModelTag { model: "Pro/BAAI/bge-m3".into(), model_type: ModelType::Embedding, context_window: 8192, max_output: 1024, description: "BGE M3 Pro".into(), provider: "BAAI".into() },
        ModelTag { model: "BAAI/bge-large-en-v1.5".into(), model_type: ModelType::Embedding, context_window: 512, max_output: 1024, description: "BGE Large English".into(), provider: "BAAI".into() },
        ModelTag { model: "BAAI/bge-large-zh-v1.5".into(), model_type: ModelType::Embedding, context_window: 512, max_output: 1024, description: "BGE Large Chinese".into(), provider: "BAAI".into() },
        ModelTag { model: "BAAI/bge-small-en-v1.5".into(), model_type: ModelType::Embedding, context_window: 512, max_output: 384, description: "BGE Small English".into(), provider: "BAAI".into() },
        ModelTag { model: "BAAI/bge-small-zh-v1.5".into(), model_type: ModelType::Embedding, context_window: 512, max_output: 512, description: "BGE Small Chinese".into(), provider: "BAAI".into() },

        // SiliconFlow Rerank
        ModelTag { model: "BAAI/bge-reranker-v2-m3".into(), model_type: ModelType::Rerank, context_window: 8192, max_output: 0, description: "BGE Reranker V2 M3".into(), provider: "BAAI".into() },
        ModelTag { model: "Pro/BAAI/bge-reranker-v2-m3".into(), model_type: ModelType::Rerank, context_window: 8192, max_output: 0, description: "BGE Reranker V2 M3 Pro".into(), provider: "BAAI".into() },

        // Qwen Embedding
        ModelTag { model: "Qwen/Qwen3-Embedding-8B".into(), model_type: ModelType::Embedding, context_window: 32768, max_output: 4096, description: "Qwen3 Embedding 8B".into(), provider: "Alibaba".into() },
        ModelTag { model: "Qwen/Qwen3-Embedding-4B".into(), model_type: ModelType::Embedding, context_window: 32768, max_output: 4096, description: "Qwen3 Embedding 4B".into(), provider: "Alibaba".into() },
        ModelTag { model: "Qwen/Qwen3-Embedding-0.6B".into(), model_type: ModelType::Embedding, context_window: 32768, max_output: 1024, description: "Qwen3 Embedding 0.6B".into(), provider: "Alibaba".into() },

        // Other Embedding
        ModelTag { model: "intfloat/e5-large-v2".into(), model_type: ModelType::Embedding, context_window: 512, max_output: 1024, description: "E5 Large V2".into(), provider: "Intfloat".into() },
        ModelTag { model: "intfloat/multilingual-e5-large".into(), model_type: ModelType::Embedding, context_window: 512, max_output: 1024, description: "Multilingual E5 Large".into(), provider: "Intfloat".into() },
        ModelTag { model: "netease-youdao/bce-embedding-base_v1".into(), model_type: ModelType::Embedding, context_window: 512, max_output: 768, description: "BCE Embedding Base".into(), provider: "NetEase".into() },

        // Yi
        ModelTag { model: "yi-lightning".into(), model_type: ModelType::Chat, context_window: 16384, max_output: 4096, description: "Yi Lightning".into(), provider: "01.AI".into() },
        ModelTag { model: "yi-large".into(), model_type: ModelType::Chat, context_window: 32768, max_output: 4096, description: "Yi Large".into(), provider: "01.AI".into() },

        // GLM
        ModelTag { model: "glm-4".into(), model_type: ModelType::Chat, context_window: 128000, max_output: 4096, description: "GLM-4".into(), provider: "Zhipu".into() },
        ModelTag { model: "glm-4-flash".into(), model_type: ModelType::Chat, context_window: 128000, max_output: 4096, description: "GLM-4 Flash".into(), provider: "Zhipu".into() },

        // Llama
        ModelTag { model: "llama-3.3-70b".into(), model_type: ModelType::Chat, context_window: 131072, max_output: 8192, description: "Llama 3.3 70B".into(), provider: "Meta".into() },
        ModelTag { model: "llama-3.1-405b".into(), model_type: ModelType::Chat, context_window: 131072, max_output: 8192, description: "Llama 3.1 405B".into(), provider: "Meta".into() },

        // Mistral
        ModelTag { model: "mistral-large".into(), model_type: ModelType::Chat, context_window: 128000, max_output: 8192, description: "Mistral Large".into(), provider: "Mistral".into() },
        ModelTag { model: "codestral".into(), model_type: ModelType::Chat, context_window: 32768, max_output: 8192, description: "Codestral".into(), provider: "Mistral".into() },
    ]
}

// 根据模型名查找标签（支持前缀匹配）
pub fn find_model_tag(model: &str) -> Option<ModelTag> {
    let tags = get_builtin_model_tags();
    // 精确匹配
    if let Some(tag) = tags.iter().find(|t| t.model == model) {
        return Some(tag.clone());
    }
    // 前缀匹配
    for tag in tags.iter() {
        if model.starts_with(&tag.model) || tag.model.starts_with(model) {
            return Some(tag.clone());
        }
    }
    None
}
