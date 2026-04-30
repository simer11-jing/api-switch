# API Switch

一个基于 Rust 的 API 代理管理平台，支持多渠道管理、模型路由、熔断机制和 Token 统计。

## 功能特性

- **多渠道管理**: 支持 OpenAI、Claude、Gemini、Azure 等多种 API 类型
- **模型路由**: 支持按模型名称路由到不同渠道
- **模型发现**: 自动发现渠道支持的模型列表
- **权重配置**: 支持渠道权重和模型权重配置
- **熔断机制**: 模型级别的熔断保护，支持自动恢复
- **Token 统计**: 实时统计 Token 使用量
- **API Keys**: 支持多 API Key 管理
- **请求日志**: 完整的请求日志记录

## 快速开始

### Docker 部署

```bash
# 构建镜像
docker build -t api-switch .

# 运行容器
docker run -d --name api-switch \
  --restart unless-stopped \
  -p 9091:9091 \
  -v /path/to/data:/app/data \
  -e DATABASE_PATH=/app/data/api-switch.db \
  -e PORT=9091 \
  api-switch
```

### 从源码构建

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装 musl 工具链 (用于 Alpine)
rustup target add x86_64-unknown-linux-musl

# 构建
cargo build --release --target x86_64-unknown-linux-musl

# 运行
./target/x86_64-unknown-linux-musl/release/api-switch
```

## API 文档

### 认证

```bash
# 登录
POST /api/login
{"username": "admin", "password": "admin"}
```

### 渠道管理

```bash
# 获取渠道列表
GET /api/channels

# 创建渠道
POST /api/channels
{"name": "OpenAI", "api_type": "openai", "base_url": "https://api.openai.com/v1", "api_key": "sk-..."}

# 测试渠道连通性
POST /api/channels/{id}/test

# 发现模型
POST /api/channels/{id}/discover

# 测试单个模型
POST /api/channels/{id}/test-model
{"model": "gpt-4"}
```

### 模型路由

```bash
# 获取路由列表
GET /api/entries

# 创建路由
POST /api/entries
{"model": "gpt-4", "channel_id": "...", "display_name": "GPT-4", "weight": 1, "priority": 0}
```

### 代理接口

```bash
# Chat Completions
POST /v1/chat/completions
Authorization: Bearer your-api-key
```

## 配置说明

### 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| DATABASE_PATH | 数据库路径 | /app/data/api-switch.db |
| PORT | 服务端口 | 9091 |
| RUST_LOG | 日志级别 | info |

### 熔断配置

- **熔断阈值**: 连续失败次数达到阈值后触发熔断
- **恢复时间**: 熔断后等待时间自动恢复

## 界面预览

访问 http://localhost:9091 使用默认账号登录:

- 用户名: admin
- 密码: admin

## License

MIT
