# ALICE-ASR-SaaS

Automatic Speech Recognition SaaS built on the ALICE-ASR engine. Provides high-accuracy transcription, streaming recognition, and multi-language support via a simple REST API.

## Architecture

```
Client --> API Gateway (8110) --> Core Engine (8111)
```

- **API Gateway**: Authentication, rate limiting, request proxying
- **Core Engine**: ASR inference, language detection, model management

## Features

- Real-time and batch speech transcription
- Multi-language support (50+ languages)
- Streaming recognition with partial results
- Speaker diarization
- Confidence scoring per word/segment
- Custom vocabulary and acoustic model selection

## API Endpoints

### Core Engine (port 8111)

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Health check with uptime and stats |
| POST | `/api/v1/asr/transcribe` | Transcribe audio file or base64 audio |
| POST | `/api/v1/asr/stream` | Streaming recognition session |
| GET | `/api/v1/asr/languages` | List supported languages |
| GET | `/api/v1/asr/models` | List available ASR models |
| GET | `/api/v1/asr/stats` | Operational statistics |

### API Gateway (port 8110)

Proxies all `/api/v1/*` routes to the Core Engine with JWT/API-Key auth and token-bucket rate limiting.

## Quick Start

```bash
# Core Engine
cd services/core-engine
ASR_ADDR=0.0.0.0:8111 cargo run --release

# API Gateway
cd services/api-gateway
GATEWAY_ADDR=0.0.0.0:8110 CORE_ENGINE_URL=http://localhost:8111 cargo run --release
```

## Example Request

```bash
curl -X POST http://localhost:8111/api/v1/asr/transcribe \
  -H "Content-Type: application/json" \
  -d '{"audio_b64":"...","language":"ja","model":"alice-asr-v2"}'
```

## License

AGPL-3.0-or-later. SaaS operators must publish complete service source code under AGPL-3.0.
