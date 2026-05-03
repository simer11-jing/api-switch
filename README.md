# API Switch

<p align="center">
  <strong>A high-performance LLM API gateway written in Rust</strong>
</p>

<p align="center">
  Multi-provider support • Load balancing • Circuit breaker • Token tracking
</p>

<p align="center">
  <a href="#features">Features</a> •
  <a href="#quick-start">Quick Start</a> •
  <a href="#api-documentation">API Docs</a> •
  <a href="#deployment">Deployment</a>
</p>

---

## Features

- **Multi-Provider Support** - OpenAI, Claude, Gemini, Azure, and any OpenAI-compatible endpoint
- **Load Balancing** - Priority-based routing with automatic failover
- **Circuit Breaker** - Model-level fault tolerance with auto-recovery
- **Token Tracking** - Real-time usage statistics and cost monitoring
- **Auto Discovery** - Automatically discover available models from providers
- **Web Console** - Built-in admin dashboard for easy management
- **Single Binary** - No dependencies, just download and run
- **Docker Ready** - Official Docker image for easy deployment

## Quick Start

### Option 1: Docker (Recommended)

```bash
docker run -d --name api-switch \
  -p 9091:9091 \
  -v api-switch-data:/app/data \
  simer11/api-switch:latest
```

Open http://localhost:9091 and login with `admin` / `admin`.

### Option 2: Binary

```bash
# Download from releases
curl -L https://github.com/simer11-jing/api-switch/releases/latest/download/api-switch-linux-amd64 -o api-switch
chmod +x api-switch
./api-switch
```

### Option 3: Build from Source

```bash
git clone https://github.com/simer11-jing/api-switch.git
cd api-switch
cargo build --release
./target/release/api-switch
```

## Usage

### 1. Add a Channel

```bash
curl -X POST http://localhost:9091/api/channels \
  -H "Content-Type: application/json" \
  -H "Cookie: session=your-session" \
  -d '{
    "name": "OpenAI",
    "api_type": "openai",
    "base_url": "https://api.openai.com/v1",
    "api_key": "sk-your-key"
  }'
```

### 2. Discover Models

```bash
curl -X POST http://localhost:9091/api/channels/{id}/discover
```

### 3. Create API Key

```bash
curl -X POST http://localhost:9091/api/keys \
  -H "Content-Type: application/json" \
  -d '{"name": "my-key"}'
```

### 4. Use the Proxy

```bash
curl http://localhost:9091/v1/chat/completions \
  -H "Authorization: Bearer your-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-4",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

## API Documentation

### Authentication

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/login` | POST | Login with username/password |
| `/api/logout` | POST | Logout current session |
| `/api/me` | GET | Get current user info |

### Channels

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/channels` | GET | List all channels |
| `/api/channels` | POST | Create a channel |
| `/api/channels/{id}` | GET/PUT/DELETE | Channel CRUD |
| `/api/channels/{id}/test` | POST | Test connectivity |
| `/api/channels/{id}/discover` | POST | Discover models |

### Routing

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/entries` | GET/POST | List/create route entries |
| `/api/entries/{id}` | PUT/DELETE | Update/delete entry |
| `/api/entries/reorder` | POST | Reorder priorities |

### Proxy

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/v1/chat/completions` | POST | Chat completions (OpenAI compatible) |
| `/v1/models` | GET | List available models |

## Deployment

### Docker Compose

```yaml
version: '3.8'
services:
  api-switch:
    image: simer11/api-switch:latest
    ports:
      - "9091:9091"
    volumes:
      - ./data:/app/data
    environment:
      - RUST_LOG=info
      - DATABASE_PATH=/app/data/api-switch.db
    restart: unless-stopped
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | 9091 | Server port |
| `DATABASE_PATH` | /app/data/api-switch.db | SQLite database path |
| `RUST_LOG` | info | Log level |

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                    API Switch                        │
├─────────────────────────────────────────────────────┤
│  ┌─────────┐  ┌─────────┐  ┌─────────┐            │
│  │ OpenAI  │  │ Claude  │  │ Gemini  │  ...       │
│  └────┬────┘  └────┬────┘  └────┬────┘            │
│       │            │            │                  │
│       └────────────┴────────────┘                  │
│                    │                                │
│              ┌─────┴─────┐                         │
│              │   Router  │  ← Priority-based       │
│              └─────┬─────┘                         │
│                    │                                │
│         ┌──────────┼──────────┐                    │
│         │          │          │                    │
│    ┌────┴────┐ ┌───┴───┐ ┌───┴───┐                │
│    │ Circuit │ │ Token │ │ Logging│                │
│    │ Breaker │ │ Stats │ │        │                │
│    └─────────┘ └───────┘ └───────┘                │
│                    │                                │
│              ┌─────┴─────┐                         │
│              │  /v1/...  │  ← OpenAI compatible    │
│              └───────────┘                         │
└─────────────────────────────────────────────────────┘
```

## Roadmap

- [ ] OpenAPI documentation (Swagger UI)
- [ ] Prometheus metrics endpoint
- [ ] Multi-tenant support
- [ ] Rate limiting per API key
- [ ] Webhook notifications
- [ ] Kubernetes Helm chart

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

Inspired by [LiteLLM](https://github.com/BerriAI/litellm) and [gproxy](https://github.com/LeenHawk/gproxy).
