use axum::{
    extract::{Query, State, Path},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, warn};
use tracing_subscriber;

use snowflake_generator::Snowflake;

/// Snowflake ID Generator HTTP Server
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Server host
    #[arg(short = 'H', long, default_value = "0.0.0.0")]
    host: String,

    /// Server port
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// Worker ID (0-31)
    #[arg(short, long, default_value_t = 1)]
    worker_id: u64,

    /// Datacenter ID (0-31)
    #[arg(short, long, default_value_t = 1)]
    datacenter_id: u64,

    /// Use configuration file for worker management
    #[arg(short, long)]
    config_file: Option<String>,
}

/// Application state shared across handlers
#[derive(Clone)]
struct AppState {
    snowflake: Arc<Mutex<Snowflake>>,
    stats: Arc<Mutex<ServerStats>>,
}

/// Server statistics
#[derive(Debug, Clone)]
struct ServerStats {
    total_requests: u64,
    successful_generations: u64,
    failed_generations: u64,
    start_time: std::time::Instant,
}

impl ServerStats {
    fn new() -> Self {
        Self {
            total_requests: 0,
            successful_generations: 0,
            failed_generations: 0,
            start_time: std::time::Instant::now(),
        }
    }
}

/// Response for single ID generation
#[derive(Serialize)]
struct IdResponse {
    id: u64,
    worker_id: u64,
    datacenter_id: u64,
    timestamp: u64,
}

/// Response for batch ID generation
#[derive(Serialize)]
struct BatchIdResponse {
    ids: Vec<u64>,
    count: usize,
    worker_id: u64,
    datacenter_id: u64,
}

/// Query parameters for batch generation
#[derive(Deserialize)]
struct BatchQuery {
    count: Option<usize>,
}

/// Server statistics response
#[derive(Serialize)]
struct StatsResponse {
    total_requests: u64,
    successful_generations: u64,
    failed_generations: u64,
    success_rate: f64,
    uptime_seconds: u64,
    requests_per_second: f64,
}

/// Snowflake ID parse response
#[derive(Serialize)]
struct ParseResponse {
    id: u64,
    id_hex: String,
    timestamp: u64,
    timestamp_formatted: String,
    datacenter_id: u64,
    worker_id: u64,
    sequence: u64,
    details: String,
}

/// Health check handler
async fn health() -> &'static str {
    "OK"
}

/// Generate a single snowflake ID
async fn generate_id(State(state): State<AppState>) -> Result<Json<IdResponse>, StatusCode> {
    let mut stats = state.stats.lock().unwrap();
    stats.total_requests += 1;
    drop(stats);

    let mut snowflake = state.snowflake.lock().unwrap();
    match snowflake.next_id() {
        Ok(id) => {
            let worker_id = snowflake.get_worker_id();
            let datacenter_id = snowflake.get_datacenter_id();
            drop(snowflake);

            let mut stats = state.stats.lock().unwrap();
            stats.successful_generations += 1;
            drop(stats);

            // Extract timestamp from ID (first 41 bits after shifting)
            let timestamp = (id >> 22) + 1609459200000; // Add epoch back

            Ok(Json(IdResponse {
                id,
                worker_id,
                datacenter_id,
                timestamp,
            }))
        }
        Err(err) => {
            warn!("Failed to generate ID: {}", err);
            let mut stats = state.stats.lock().unwrap();
            stats.failed_generations += 1;
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Generate batch of snowflake IDs
async fn generate_batch(
    Query(params): Query<BatchQuery>,
    State(state): State<AppState>,
) -> Result<Json<BatchIdResponse>, StatusCode> {
    let count = params.count.unwrap_or(10).min(1000); // Limit to 1000 IDs per request

    let mut stats = state.stats.lock().unwrap();
    stats.total_requests += 1;
    drop(stats);

    let mut snowflake = state.snowflake.lock().unwrap();
    let worker_id = snowflake.get_worker_id();
    let datacenter_id = snowflake.get_datacenter_id();

    let mut ids = Vec::with_capacity(count);
    let mut success_count = 0;

    for _ in 0..count {
        match snowflake.next_id() {
            Ok(id) => {
                ids.push(id);
                success_count += 1;
            }
            Err(err) => {
                warn!("Failed to generate ID in batch: {}", err);
                break;
            }
        }
    }
    drop(snowflake);

    let mut stats = state.stats.lock().unwrap();
    stats.successful_generations += success_count as u64;
    stats.failed_generations += (count - success_count) as u64;
    drop(stats);

    if ids.is_empty() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Json(BatchIdResponse {
        count: ids.len(),
        ids,
        worker_id,
        datacenter_id,
    }))
}

/// Get server statistics
async fn get_stats(State(state): State<AppState>) -> Json<StatsResponse> {
    let stats = state.stats.lock().unwrap();
    let uptime = stats.start_time.elapsed().as_secs();
    let success_rate = if stats.total_requests > 0 {
        stats.successful_generations as f64 / stats.total_requests as f64 * 100.0
    } else {
        0.0
    };
    let rps = if uptime > 0 {
        stats.total_requests as f64 / uptime as f64
    } else {
        0.0
    };

    Json(StatsResponse {
        total_requests: stats.total_requests,
        successful_generations: stats.successful_generations,
        failed_generations: stats.failed_generations,
        success_rate,
        uptime_seconds: uptime,
        requests_per_second: rps,
    })
}

/// Parse a snowflake ID and return its components
async fn parse_id(Path(id): Path<u64>) -> Result<Json<ParseResponse>, StatusCode> {
    let info = Snowflake::parse_id(id);
    
    Ok(Json(ParseResponse {
        id: info.id,
        id_hex: info.id_as_hex(),
        timestamp: info.timestamp,
        timestamp_formatted: info.timestamp_as_string(),
        datacenter_id: info.datacenter_id,
        worker_id: info.worker_id,
        sequence: info.sequence,
        details: info.format_details(),
    }))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    info!(
        "Starting Snowflake ID Generator Server on {}:{}",
        args.host, args.port
    );
    info!(
        "Worker ID: {}, Datacenter ID: {}",
        args.worker_id, args.datacenter_id
    );

    // Create snowflake generator based on configuration
    let snowflake = if let Some(config_file) = args.config_file {
        info!("Using configuration file: {}", config_file);
        match Snowflake::new_with_config(&config_file, args.datacenter_id) {
            Ok(sf) => sf,
            Err(e) => {
                warn!("Failed to load config file, falling back to default: {}", e);
                Snowflake::new(args.worker_id, args.datacenter_id)
            }
        }
    } else {
        Snowflake::new(args.worker_id, args.datacenter_id)
    };

    // Create application state
    let state = AppState {
        snowflake: Arc::new(Mutex::new(snowflake)),
        stats: Arc::new(Mutex::new(ServerStats::new())),
    };

    // Build our application with routes
    let app = Router::new()
        .route("/health", get(health))
        .route("/id", get(generate_id))
        .route("/batch", get(generate_batch))
        .route("/stats", get(get_stats))
        .route("/parse/:id", get(parse_id))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()),
        )
        .with_state(state);

    // Create listener
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", args.host, args.port)).await?;
    
    info!("Server running on http://{}:{}", args.host, args.port);
    info!("Available endpoints:");
    info!("  GET /health - Health check");
    info!("  GET /id - Generate single snowflake ID");
    info!("  GET /batch?count=N - Generate batch of IDs (max 1000)");
    info!("  GET /stats - Server statistics");
    info!("  GET /parse/:id - Parse snowflake ID");

    // Start the server
    axum::serve(listener, app).await?;

    Ok(())
}
