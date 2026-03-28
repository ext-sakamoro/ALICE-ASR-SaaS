use axum::{extract::State, response::Json, routing::{get, post}, Router};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

struct AppState { start_time: Instant, stats: Mutex<Stats> }
struct Stats { total_transcriptions: u64, total_stream_sessions: u64, total_words_recognized: u64, total_errors: u64 }

#[derive(Serialize)]
struct Health { status: String, version: String, uptime_secs: u64, total_ops: u64 }

#[derive(Deserialize)]
struct TranscribeRequest { audio_b64: String, language: Option<String>, model: Option<String>, punctuate: Option<bool>, diarize: Option<bool> }
#[derive(Serialize)]
struct Segment { start_ms: u64, end_ms: u64, text: String, confidence: f32, speaker: Option<u32> }
#[derive(Serialize)]
struct TranscribeResponse { job_id: String, language: String, model: String, text: String, segments: Vec<Segment>, duration_ms: u64, word_count: u32, confidence: f32, processing_ms: u128 }

#[derive(Deserialize)]
struct StreamRequest { session_id: Option<String>, chunk_b64: String, language: Option<String>, final_chunk: Option<bool> }
#[derive(Serialize)]
struct StreamResponse { session_id: String, partial_text: String, is_final: bool, confidence: f32 }

#[derive(Serialize)]
struct LanguageInfo { code: String, name: String, script: String, rtl: bool }

#[derive(Serialize)]
struct ModelInfo { id: String, name: String, languages: Vec<String>, wer: f32, latency_ms: u32, size_mb: u32 }

#[derive(Serialize)]
struct StatsResponse { total_transcriptions: u64, total_stream_sessions: u64, total_words_recognized: u64, total_errors: u64 }

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "asr_engine=info".into())).init();
    let state = Arc::new(AppState { start_time: Instant::now(), stats: Mutex::new(Stats { total_transcriptions: 0, total_stream_sessions: 0, total_words_recognized: 0, total_errors: 0 }) });
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);
    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/asr/transcribe", post(transcribe))
        .route("/api/v1/asr/stream", post(stream))
        .route("/api/v1/asr/languages", get(languages))
        .route("/api/v1/asr/models", get(models))
        .route("/api/v1/asr/stats", get(stats))
        .layer(cors).layer(TraceLayer::new_for_http()).with_state(state);
    let addr = std::env::var("ASR_ADDR").unwrap_or_else(|_| "0.0.0.0:8111".into());
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("ASR Engine on {addr}");
    axum::serve(listener, app).await.unwrap();
}

async fn health(State(s): State<Arc<AppState>>) -> Json<Health> {
    let st = s.stats.lock().unwrap();
    Json(Health { status: "ok".into(), version: env!("CARGO_PKG_VERSION").into(), uptime_secs: s.start_time.elapsed().as_secs(), total_ops: st.total_transcriptions + st.total_stream_sessions })
}

async fn transcribe(State(s): State<Arc<AppState>>, Json(req): Json<TranscribeRequest>) -> Json<TranscribeResponse> {
    let t = Instant::now();
    let lang = req.language.unwrap_or_else(|| "ja".into());
    let model = req.model.unwrap_or_else(|| "alice-asr-v2".into());
    let diarize = req.diarize.unwrap_or(false);
    let text = format!("Transcribed audio ({} bytes base64)", req.audio_b64.len());
    let word_count = text.split_whitespace().count() as u32;
    let segments = vec![
        Segment { start_ms: 0, end_ms: 1200, text: text.clone(), confidence: 0.97, speaker: if diarize { Some(0) } else { None } },
    ];
    { let mut st = s.stats.lock().unwrap(); st.total_transcriptions += 1; st.total_words_recognized += word_count as u64; }
    Json(TranscribeResponse { job_id: uuid::Uuid::new_v4().to_string(), language: lang, model, text, segments, duration_ms: 1200, word_count, confidence: 0.97, processing_ms: t.elapsed().as_millis() })
}

async fn stream(State(s): State<Arc<AppState>>, Json(req): Json<StreamRequest>) -> Json<StreamResponse> {
    let session_id = req.session_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let is_final = req.final_chunk.unwrap_or(false);
    { let mut st = s.stats.lock().unwrap(); if is_final { st.total_stream_sessions += 1; } }
    Json(StreamResponse { session_id, partial_text: format!("...partial ({} bytes)...", req.chunk_b64.len()), is_final, confidence: 0.91 })
}

async fn languages() -> Json<Vec<LanguageInfo>> {
    Json(vec![
        LanguageInfo { code: "ja".into(), name: "Japanese".into(), script: "CJK".into(), rtl: false },
        LanguageInfo { code: "en".into(), name: "English".into(), script: "Latin".into(), rtl: false },
        LanguageInfo { code: "zh".into(), name: "Chinese (Mandarin)".into(), script: "CJK".into(), rtl: false },
        LanguageInfo { code: "ko".into(), name: "Korean".into(), script: "Hangul".into(), rtl: false },
        LanguageInfo { code: "ar".into(), name: "Arabic".into(), script: "Arabic".into(), rtl: true },
        LanguageInfo { code: "es".into(), name: "Spanish".into(), script: "Latin".into(), rtl: false },
    ])
}

async fn models() -> Json<Vec<ModelInfo>> {
    Json(vec![
        ModelInfo { id: "alice-asr-v2".into(), name: "ALICE ASR v2 (default)".into(), languages: vec!["ja".into(), "en".into(), "zh".into(), "ko".into()], wer: 3.2, latency_ms: 120, size_mb: 240 },
        ModelInfo { id: "alice-asr-v2-large".into(), name: "ALICE ASR v2 Large".into(), languages: vec!["ja".into(), "en".into(), "zh".into(), "ko".into(), "ar".into(), "es".into()], wer: 2.1, latency_ms: 350, size_mb: 680 },
        ModelInfo { id: "alice-asr-v2-tiny".into(), name: "ALICE ASR v2 Tiny (edge)".into(), languages: vec!["ja".into(), "en".into()], wer: 6.8, latency_ms: 30, size_mb: 45 },
    ])
}

async fn stats(State(s): State<Arc<AppState>>) -> Json<StatsResponse> {
    let st = s.stats.lock().unwrap();
    Json(StatsResponse { total_transcriptions: st.total_transcriptions, total_stream_sessions: st.total_stream_sessions, total_words_recognized: st.total_words_recognized, total_errors: st.total_errors })
}

fn _fnv1a(data: &[u8]) -> u64 { let mut h: u64 = 0xcbf2_9ce4_8422_2325; for &b in data { h ^= b as u64; h = h.wrapping_mul(0x0100_0000_01b3); } h }
