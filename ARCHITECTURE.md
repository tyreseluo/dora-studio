# Dora Studio Architecture Guide

This document describes the technical architecture of Dora Studio, a native desktop dashboard for the Dora dataflow framework. Built with Makepad for GPU-accelerated rendering, following the plugin patterns established by MoFA Studio.

## Project Overview

**Dora Studio** is a self-contained observability and management dashboard for Dora dataflows.

- **Version**: 0.1.0 (Planning)
- **Edition**: Rust 2021
- **License**: Apache-2.0
- **Implementation**: 100% Rust (no C/C++ dependencies)
- **UI Framework**: Makepad (GPU-accelerated, immediate mode, pure Rust)
- **Storage**: Apache Arrow DataFusion + Parquet (embedded, pure Rust)
- **Telemetry**: Built-in OTLP receiver (no external dependencies)

## Directory Structure

```
dora-studio/
├── Cargo.toml                    # Workspace configuration
├── PRD.md                        # Product requirements
├── ARCHITECTURE.md               # This file
├── README.md                     # Quick start
│
├── dora-studio-shell/            # Binary: Main application
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs               # Entry point, event loop
│   │   ├── lib.rs                # SharedState, exports
│   │   ├── app.rs                # Main App widget, shell coordination
│   │   └── widgets/
│   │       ├── mod.rs
│   │       ├── sidebar.rs        # Navigation sidebar
│   │       ├── status_bar.rs     # Infrastructure status (coordinator/daemon)
│   │       └── time_range_picker.rs  # Shared time range selector
│   └── resources/
│       ├── fonts/                # Manrope, LXGWWenKai, Emoji
│       └── icons/                # SVG icons
│
├── dora-studio-widgets/          # Library: Shared visualization components
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                # Module exports, live_design
│       ├── theme.rs              # Colors, fonts (light/dark)
│       ├── app_trait.rs          # DoraApp trait, AppInfo, AppRegistry
│       ├── time_series_chart.rs  # Line/area charts with time axis
│       ├── dataflow_graph.rs     # Interactive node-edge diagram
│       ├── log_table.rs          # Virtualized log list
│       ├── metric_card.rs        # Single value with trend
│       ├── status_badge.rs       # Running/Stopped/Error indicator
│       ├── span_timeline.rs      # Trace waterfall visualization
│       ├── yaml_editor.rs        # Syntax-highlighted YAML editor
│       └── search_input.rs       # Filter input with regex/case options
│
├── dora-studio-client/           # Library: Dora API client + storage
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                # Module exports
│       ├── client.rs             # DoraClient: coordinator communication
│       ├── storage.rs            # DataFusion + Parquet storage
│       ├── otlp_receiver.rs      # OTLP gRPC server for traces
│       └── models.rs             # Shared data types
│
├── apps/
│   ├── dataflow-manager/         # App: Lifecycle management
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs            # DoraApp impl
│   │       ├── screen.rs         # Main screen widget
│   │       ├── dataflow_table.rs # Dataflow list table
│   │       └── node_details.rs   # Expanded node view
│   │
│   ├── yaml-editor/              # App: YAML editing + graph
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── screen.rs         # Split-pane layout
│   │       ├── editor_pane.rs    # YAML editor with validation
│   │       └── graph_pane.rs     # Interactive graph view
│   │
│   ├── log-viewer/               # App: Log streaming
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── screen.rs
│   │       ├── filter_bar.rs     # Dataflow/node/level filters
│   │       └── log_list.rs       # Virtualized log display
│   │
│   └── telemetry-dashboard/      # App: Metrics + Traces
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── screen.rs         # Tab container (Metrics/Traces/Topics)
│           ├── metrics_view.rs   # Time-series charts
│           ├── traces_view.rs    # Trace list + waterfall
│           └── topics_view.rs    # Topic frequency stats
│
└── tests/                        # Integration tests
    ├── client_tests.rs
    └── storage_tests.rs
```

## Crate Dependencies

```
dora-studio-shell (binary)
├── makepad-widgets           # GPU-accelerated UI
├── dora-studio-widgets       # Shared visualization widgets
├── dora-studio-client        # Coordinator client + storage
├── dataflow-manager          # Lifecycle management app
├── yaml-editor               # YAML + graph app
├── log-viewer                # Log streaming app
├── telemetry-dashboard       # Metrics + traces app
├── tokio                     # Async runtime
├── parking_lot               # Synchronization
├── tracing                   # Logging
└── ctrlc                     # Graceful shutdown

dora-studio-widgets (library)
├── makepad-widgets
├── parking_lot
└── log

dora-studio-client (library)
├── dora-core                 # Descriptor parsing, validation
├── datafusion                # SQL query engine (pure Rust)
├── arrow                     # Columnar memory format (pure Rust)
├── parquet                   # Columnar file storage (pure Rust)
├── tonic                     # gRPC server (OTLP)
├── opentelemetry-proto       # OTLP message types
├── tokio                     # Async runtime
├── serde / serde_json        # Serialization
├── uuid                      # Dataflow IDs
└── thiserror / anyhow        # Error handling

dataflow-manager (library)
├── makepad-widgets
├── dora-studio-widgets
└── dora-studio-client

yaml-editor (library)
├── makepad-widgets
├── dora-studio-widgets
├── dora-studio-client
└── dora-core                 # YAML parsing, validation

log-viewer (library)
├── makepad-widgets
├── dora-studio-widgets
└── dora-studio-client

telemetry-dashboard (library)
├── makepad-widgets
├── dora-studio-widgets
└── dora-studio-client
```

---

## Core Architecture

### System Components

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              DORA STUDIO                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐   │
│   │  Dataflow   │  │    YAML     │  │     Log     │  │   Telemetry     │   │
│   │   Manager   │  │   Editor    │  │   Viewer    │  │   Dashboard     │   │
│   │     App     │  │     App     │  │     App     │  │      App        │   │
│   └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └────────┬────────┘   │
│          │                │                │                   │            │
│   ┌──────┴────────────────┴────────────────┴───────────────────┴──────┐    │
│   │                    SHELL (Navigation, Theme, Status Bar)          │    │
│   └──────────────────────────────────┬────────────────────────────────┘    │
│                                      │                                      │
│   ┌──────────────────────────────────┴────────────────────────────────┐    │
│   │                     DORA STUDIO WIDGETS                           │    │
│   │  TimeSeriesChart │ DataflowGraph │ LogTable │ SpanTimeline │ ... │    │
│   └──────────────────────────────────┬────────────────────────────────┘    │
│                                      │                                      │
│   ┌──────────────────────────────────┴────────────────────────────────┐    │
│   │                     DORA STUDIO CLIENT                            │    │
│   │                                                                   │    │
│   │  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────┐   │    │
│   │  │ DoraClient  │    │   Storage   │    │   OTLP Receiver     │   │    │
│   │  │ (TCP/JSON)  │    │ (DataFusion)│    │   (gRPC:4317)       │   │    │
│   │  └──────┬──────┘    └──────┬──────┘    └──────────┬──────────┘   │    │
│   └─────────┼──────────────────┼─────────────────────┼───────────────┘    │
└─────────────┼──────────────────┼─────────────────────┼────────────────────┘
              │                  │                     │
              ▼                  ▼                     ▼
    ┌─────────────────┐   ┌───────────┐      ┌─────────────────┐
    │   Coordinator   │   │  ~/.dora/ │      │  Dora Daemons   │
    │  (TCP :53290)   │   │ studio.db │      │ (OTLP export)   │
    └────────┬────────┘   └───────────┘      └─────────────────┘
             │
    ┌────────┴────────┐
    │     Daemon(s)   │
    │ (Node spawning) │
    └─────────────────┘
```

### Data Flow Architecture

```
                                 USER INTERACTION
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                              UI LAYER                                   │
│                                                                         │
│   App Screen ──▶ Widget Event ──▶ Action ──▶ Shell Dispatch            │
│                                                                         │
└──────────────────────────────────┬──────────────────────────────────────┘
                                   │
                    ┌──────────────┴──────────────┐
                    ▼                             ▼
         ┌─────────────────┐           ┌─────────────────┐
         │   DoraClient    │           │     Storage     │
         │  (Real-time)    │           │  (Historical)   │
         └────────┬────────┘           └────────┬────────┘
                  │                             │
    ┌─────────────┼─────────────┐               │
    ▼             ▼             ▼               ▼
┌───────┐   ┌─────────┐   ┌─────────┐    ┌───────────┐
│ List  │   │ Metrics │   │  Logs   │    │ DataFusion│
│Dataflow│  │ Polling │   │ Stream  │    │  Queries  │
└───────┘   └─────────┘   └─────────┘    └───────────┘
    │             │             │               │
    └─────────────┴──────────────┴──────────────┘
                          │
                          ▼
                 ┌─────────────────┐
                 │   UI Update     │
                 │  (Redraw Cx)    │
                 └─────────────────┘
```

---

## Plugin System: DoraApp Trait

Following MoFA Studio patterns, apps implement a standardized trait:

```rust
// dora-studio-widgets/src/app_trait.rs

pub trait DoraApp {
    /// App metadata (name, id, icon, description)
    fn info() -> AppInfo where Self: Sized;

    /// Register widgets with Makepad
    fn live_design(cx: &mut Cx);
}

pub struct AppInfo {
    pub name: &'static str,         // "Dataflow Manager"
    pub id: &'static str,           // "dataflow-manager"
    pub icon: &'static str,         // "icons/dataflow.svg"
    pub description: &'static str,  // "Manage dataflow lifecycle"
}

pub struct AppRegistry {
    apps: Vec<AppInfo>,
}

impl AppRegistry {
    pub fn new() -> Self {
        Self { apps: Vec::new() }
    }

    pub fn register(&mut self, info: AppInfo) {
        self.apps.push(info);
    }

    pub fn apps(&self) -> &[AppInfo] {
        &self.apps
    }
}
```

### App Implementation Example

```rust
// apps/dataflow-manager/src/lib.rs

use dora_studio_widgets::{DoraApp, AppInfo};
use makepad_widgets::Cx;

pub mod screen;
pub use screen::DataflowManagerScreen;

pub struct DataflowManagerApp;

impl DoraApp for DataflowManagerApp {
    fn info() -> AppInfo {
        AppInfo {
            name: "Dataflow Manager",
            id: "dataflow-manager",
            icon: "icons/dataflows.svg",
            description: "Start, stop, and monitor dataflows",
        }
    }

    fn live_design(cx: &mut Cx) {
        screen::live_design(cx);
    }
}

pub fn live_design(cx: &mut Cx) {
    DataflowManagerApp::live_design(cx);
}
```

### Shell Registration

```rust
// dora-studio-shell/src/app.rs

use dataflow_manager::{DataflowManagerApp, DataflowManagerScreen};
use yaml_editor::{YamlEditorApp, YamlEditorScreen};
use log_viewer::{LogViewerApp, LogViewerScreen};
use telemetry_dashboard::{TelemetryDashboardApp, TelemetryDashboardScreen};

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        // Core widgets first
        makepad_widgets::live_design(cx);
        dora_studio_widgets::live_design(cx);

        // Shell widgets
        crate::widgets::sidebar::live_design(cx);
        crate::widgets::status_bar::live_design(cx);

        // Apps (order matters for dependencies)
        dataflow_manager::live_design(cx);
        yaml_editor::live_design(cx);
        log_viewer::live_design(cx);
        telemetry_dashboard::live_design(cx);
    }
}

impl LiveHook for App {
    fn after_new_from_doc(&mut self, _cx: &mut Cx) {
        // Register app metadata
        self.app_registry.register(DataflowManagerApp::info());
        self.app_registry.register(YamlEditorApp::info());
        self.app_registry.register(LogViewerApp::info());
        self.app_registry.register(TelemetryDashboardApp::info());
    }
}
```

---

## Dora Client Layer

### DoraClient: Coordinator Communication

```rust
// dora-studio-client/src/client.rs

use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use uuid::Uuid;

pub struct DoraClient {
    coordinator_addr: SocketAddr,
    stream: Option<TcpStream>,
}

impl DoraClient {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            coordinator_addr: addr,
            stream: None,
        }
    }

    /// Connect to coordinator
    pub async fn connect(&mut self) -> Result<(), ClientError> {
        let stream = TcpStream::connect(self.coordinator_addr).await?;
        self.stream = Some(stream);
        Ok(())
    }

    /// List all dataflows with status and metrics
    pub async fn list_dataflows(&mut self) -> Result<Vec<DataflowEntry>, ClientError> {
        let request = ControlRequest::List;
        let response = self.send_request(request).await?;

        match response {
            ControlReply::DataflowList { dataflows } => Ok(dataflows),
            _ => Err(ClientError::UnexpectedResponse),
        }
    }

    /// Start a dataflow from YAML path
    pub async fn start_dataflow(
        &mut self,
        path: &Path,
        name: Option<&str>,
    ) -> Result<Uuid, ClientError> {
        let descriptor = std::fs::read_to_string(path)?;
        let request = ControlRequest::Start {
            dataflow: descriptor,
            name: name.map(String::from),
            detach: false,
        };

        let response = self.send_request(request).await?;
        match response {
            ControlReply::DataflowStarted { uuid } => Ok(uuid),
            ControlReply::Error(e) => Err(ClientError::CoordinatorError(e)),
            _ => Err(ClientError::UnexpectedResponse),
        }
    }

    /// Stop a running dataflow
    pub async fn stop_dataflow(
        &mut self,
        id: Uuid,
        grace_duration: Option<Duration>,
    ) -> Result<DataflowResult, ClientError> {
        let request = ControlRequest::Stop {
            dataflow_uuid: id,
            grace_duration,
        };

        let response = self.send_request(request).await?;
        match response {
            ControlReply::DataflowStopped { result } => Ok(result),
            ControlReply::Error(e) => Err(ClientError::CoordinatorError(e)),
            _ => Err(ClientError::UnexpectedResponse),
        }
    }

    /// Get node metrics for a dataflow
    pub async fn get_node_metrics(
        &mut self,
        dataflow_id: Uuid,
    ) -> Result<HashMap<NodeId, NodeMetrics>, ClientError> {
        let request = ControlRequest::GetNodeInfo { dataflow_uuid: dataflow_id };
        let response = self.send_request(request).await?;

        match response {
            ControlReply::NodeInfo { metrics } => Ok(metrics),
            _ => Err(ClientError::UnexpectedResponse),
        }
    }

    /// Subscribe to log stream
    pub async fn subscribe_logs(
        &mut self,
        filter: LogFilter,
    ) -> Result<LogStream, ClientError> {
        let request = ControlRequest::LogSubscribe { filter };
        self.send_request(request).await?;

        // Return async stream that yields LogMessage
        Ok(LogStream::new(self.stream.as_mut().unwrap()))
    }

    /// Check coordinator connectivity
    pub async fn check_coordinator(&mut self) -> Result<bool, ClientError> {
        match self.send_request(ControlRequest::Check).await {
            Ok(ControlReply::Ok) => Ok(true),
            Ok(_) => Ok(false),
            Err(_) => Ok(false),
        }
    }

    // Internal: serialize and send request
    async fn send_request(&mut self, request: ControlRequest) -> Result<ControlReply, ClientError> {
        let stream = self.stream.as_mut().ok_or(ClientError::NotConnected)?;

        // Serialize request as JSON
        let json = serde_json::to_vec(&request)?;
        let len = (json.len() as u32).to_le_bytes();

        // Send length-prefixed message
        stream.write_all(&len).await?;
        stream.write_all(&json).await?;

        // Read response
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await?;
        let response_len = u32::from_le_bytes(len_buf) as usize;

        let mut response_buf = vec![0u8; response_len];
        stream.read_exact(&mut response_buf).await?;

        let response: ControlReply = serde_json::from_slice(&response_buf)?;
        Ok(response)
    }
}

/// Log stream that yields messages asynchronously
pub struct LogStream {
    // ... implementation
}

impl LogStream {
    pub async fn next(&mut self) -> Option<LogMessage> {
        // Read next log message from stream
    }
}
```

### Data Types (models.rs)

```rust
// dora-studio-client/src/models.rs

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

/// Dataflow list entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataflowEntry {
    pub uuid: Uuid,
    pub name: Option<String>,
    pub status: DataflowStatus,
    pub node_count: usize,
    pub cpu_percent: f32,
    pub memory_mb: f64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataflowStatus {
    Running,
    Finished,
    Failed,
}

/// Node metrics from daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    pub pid: u32,
    pub cpu_percent: f32,
    pub memory_mb: f64,
    pub disk_read_mb_s: Option<f64>,
    pub disk_write_mb_s: Option<f64>,
}

/// Log message from coordinator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogMessage {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub dataflow_id: Option<Uuid>,
    pub node_id: Option<String>,
    pub level: LogLevel,
    pub target: Option<String>,
    pub message: String,
    pub fields: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Log filter for subscription
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LogFilter {
    pub dataflow_id: Option<Uuid>,
    pub node_id: Option<String>,
    pub min_level: Option<LogLevel>,
    pub search: Option<String>,
}
```

---

## Storage Layer (DataFusion + Parquet)

### Architecture

Dora Studio uses Apache Arrow DataFusion for SQL queries and Parquet files for persistent storage. This provides a **100% Rust** solution without any C/C++ dependencies.

```
~/.dora/studio/
├── dataflows.parquet      # Dataflow metadata
├── nodes.parquet          # Node metadata
├── metrics/               # Partitioned by date
│   ├── 2026-01-10.parquet
│   └── ...
├── logs/                  # Partitioned by date
│   └── ...
└── spans/                 # Partitioned by date
    └── ...
```

### Storage Implementation

```rust
// dora-studio-client/src/storage.rs

use arrow::array::*;
use arrow::datatypes::{DataType, Field, Schema, TimeUnit};
use arrow::record_batch::RecordBatch;
use datafusion::prelude::*;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::arrow::ArrowWriter;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct Storage {
    base_path: PathBuf,
    ctx: SessionContext,
}

impl Storage {
    /// Open or create storage at path
    pub async fn open(path: &Path) -> Result<Self, StorageError> {
        std::fs::create_dir_all(path)?;

        let ctx = SessionContext::new();
        let storage = Self {
            base_path: path.to_path_buf(),
            ctx,
        };

        // Register tables for existing parquet files
        storage.register_tables().await?;

        Ok(storage)
    }

    /// Register parquet files as DataFusion tables
    async fn register_tables(&self) -> Result<(), StorageError> {
        // Register metrics table (partitioned directory)
        let metrics_path = self.base_path.join("metrics");
        if metrics_path.exists() {
            self.ctx.register_parquet(
                "metrics",
                metrics_path.to_str().unwrap(),
                ParquetReadOptions::default(),
            ).await?;
        }

        // Register logs table
        let logs_path = self.base_path.join("logs");
        if logs_path.exists() {
            self.ctx.register_parquet(
                "logs",
                logs_path.to_str().unwrap(),
                ParquetReadOptions::default(),
            ).await?;
        }

        // Register spans table
        let spans_path = self.base_path.join("spans");
        if spans_path.exists() {
            self.ctx.register_parquet(
                "spans",
                spans_path.to_str().unwrap(),
                ParquetReadOptions::default(),
            ).await?;
        }

        Ok(())
    }

    /// Get Arrow schema for metrics
    fn metrics_schema() -> Schema {
        Schema::new(vec![
            Field::new("ts", DataType::Timestamp(TimeUnit::Microsecond, None), false),
            Field::new("dataflow_uuid", DataType::Utf8, false),
            Field::new("node_id", DataType::Utf8, false),
            Field::new("cpu_percent", DataType::Float32, true),
            Field::new("memory_mb", DataType::Float64, true),
            Field::new("disk_read_mb_s", DataType::Float64, true),
            Field::new("disk_write_mb_s", DataType::Float64, true),
        ])
    }

    /// Insert metrics samples (batched write)
    pub async fn insert_metrics(&self, samples: &[MetricsSample]) -> Result<(), StorageError> {
        let schema = Arc::new(Self::metrics_schema());

        // Build Arrow arrays from samples
        let ts_array = TimestampMicrosecondArray::from_iter_values(
            samples.iter().map(|s| s.timestamp.timestamp_micros())
        );
        let uuid_array = StringArray::from_iter_values(
            samples.iter().map(|s| s.dataflow_uuid.to_string())
        );
        let node_array = StringArray::from_iter_values(
            samples.iter().map(|s| s.node_id.as_str())
        );
        let cpu_array = Float32Array::from_iter(
            samples.iter().map(|s| s.cpu_percent)
        );
        let mem_array = Float64Array::from_iter(
            samples.iter().map(|s| s.memory_mb)
        );
        let read_array = Float64Array::from_iter(
            samples.iter().map(|s| s.disk_read_mb_s)
        );
        let write_array = Float64Array::from_iter(
            samples.iter().map(|s| s.disk_write_mb_s)
        );

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(ts_array),
                Arc::new(uuid_array),
                Arc::new(node_array),
                Arc::new(cpu_array),
                Arc::new(mem_array),
                Arc::new(read_array),
                Arc::new(write_array),
            ],
        )?;

        // Write to date-partitioned parquet file
        let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let file_path = self.base_path.join("metrics").join(format!("{}.parquet", date));
        std::fs::create_dir_all(file_path.parent().unwrap())?;

        self.append_to_parquet(&file_path, batch).await?;

        Ok(())
    }

    /// Append batch to parquet file (create or append)
    async fn append_to_parquet(&self, path: &Path, batch: RecordBatch) -> Result<(), StorageError> {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        let mut writer = ArrowWriter::try_new(file, batch.schema(), None)?;
        writer.write(&batch)?;
        writer.close()?;

        Ok(())
    }

    /// Query metrics for time range using SQL
    pub async fn query_metrics(
        &self,
        dataflow_uuid: Uuid,
        node_id: Option<&str>,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<MetricsSample>, StorageError> {
        let node_filter = node_id
            .map(|n| format!("AND node_id = '{}'", n))
            .unwrap_or_default();

        let sql = format!(
            r#"
            SELECT ts, dataflow_uuid, node_id, cpu_percent, memory_mb, disk_read_mb_s, disk_write_mb_s
            FROM metrics
            WHERE dataflow_uuid = '{}'
              AND ts >= timestamp '{}'
              AND ts < timestamp '{}'
              {}
            ORDER BY ts
            "#,
            dataflow_uuid,
            start.format("%Y-%m-%dT%H:%M:%S%.6fZ"),
            end.format("%Y-%m-%dT%H:%M:%S%.6fZ"),
            node_filter
        );

        let df = self.ctx.sql(&sql).await?;
        let batches = df.collect().await?;

        // Convert Arrow batches to MetricsSample
        let mut results = Vec::new();
        for batch in batches {
            // ... convert each row to MetricsSample
        }

        Ok(results)
    }

    /// Query traces by time range
    pub async fn query_traces(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
        limit: usize,
    ) -> Result<Vec<TraceSummary>, StorageError> {
        let sql = format!(
            r#"
            SELECT
                trace_id,
                MIN(name) as root_span,
                MIN(start_time) as start_time,
                SUM(duration_ms) as total_duration_ms,
                COUNT(*) as span_count
            FROM spans
            WHERE start_time >= timestamp '{}'
              AND start_time < timestamp '{}'
            GROUP BY trace_id
            ORDER BY start_time DESC
            LIMIT {}
            "#,
            start.format("%Y-%m-%dT%H:%M:%S%.6fZ"),
            end.format("%Y-%m-%dT%H:%M:%S%.6fZ"),
            limit
        );

        let df = self.ctx.sql(&sql).await?;
        let batches = df.collect().await?;

        // Convert to TraceSummary
        let mut results = Vec::new();
        for batch in batches {
            // ... convert each row
        }

        Ok(results)
    }

    /// Run retention cleanup - delete old parquet partition files
    pub async fn cleanup(&self, retention: &RetentionConfig) -> Result<(), StorageError> {
        let cutoff_metrics = chrono::Utc::now() - chrono::Duration::days(retention.metrics as i64);
        let cutoff_logs = chrono::Utc::now() - chrono::Duration::days(retention.logs as i64);
        let cutoff_traces = chrono::Utc::now() - chrono::Duration::days(retention.traces as i64);

        self.delete_partitions_before("metrics", cutoff_metrics)?;
        self.delete_partitions_before("logs", cutoff_logs)?;
        self.delete_partitions_before("spans", cutoff_traces)?;

        Ok(())
    }

    /// Delete partition files older than cutoff date
    fn delete_partitions_before(
        &self,
        table: &str,
        cutoff: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), StorageError> {
        let dir = self.base_path.join(table);
        if !dir.exists() {
            return Ok(());
        }

        let cutoff_str = cutoff.format("%Y-%m-%d").to_string();

        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let file_name = entry.file_name().to_string_lossy().to_string();

            // Parse date from filename (e.g., "2026-01-10.parquet")
            if let Some(date_str) = file_name.strip_suffix(".parquet") {
                if date_str < cutoff_str.as_str() {
                    std::fs::remove_file(entry.path())?;
                }
            }
        }

        Ok(())
    }
}

pub struct RetentionConfig {
    pub metrics: u32,  // Default: 7 days
    pub logs: u32,     // Default: 3 days
    pub traces: u32,   // Default: 1 day
}

impl Default for RetentionConfig {
    fn default() -> Self {
        Self {
            metrics: 7,
            logs: 3,
            traces: 1,
        }
    }
}
```

---

## OTLP Receiver

Receives traces from Dora daemons via OpenTelemetry Protocol:

```rust
// dora-studio-client/src/otlp_receiver.rs

use opentelemetry_proto::tonic::collector::trace::v1::{
    trace_service_server::{TraceService, TraceServiceServer},
    ExportTraceServiceRequest, ExportTraceServiceResponse,
};
use tonic::{Request, Response, Status};
use std::sync::Arc;

pub struct OtlpReceiver {
    storage: Arc<tokio::sync::Mutex<Storage>>,
}

impl OtlpReceiver {
    pub fn new(storage: Arc<tokio::sync::Mutex<Storage>>) -> Self {
        Self { storage }
    }

    /// Start gRPC server on given port
    pub async fn start(&self, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!("0.0.0.0:{}", port).parse()?;

        let service = TraceServiceServer::new(OtlpTraceService {
            storage: self.storage.clone(),
        });

        tonic::transport::Server::builder()
            .add_service(service)
            .serve(addr)
            .await?;

        Ok(())
    }
}

struct OtlpTraceService {
    storage: Arc<tokio::sync::Mutex<Storage>>,
}

#[tonic::async_trait]
impl TraceService for OtlpTraceService {
    async fn export(
        &self,
        request: Request<ExportTraceServiceRequest>,
    ) -> Result<Response<ExportTraceServiceResponse>, Status> {
        let req = request.into_inner();

        // Process resource spans
        for resource_span in req.resource_spans {
            let resource_attrs = resource_span.resource
                .map(|r| extract_attributes(&r.attributes))
                .unwrap_or_default();

            for scope_span in resource_span.scope_spans {
                for span in scope_span.spans {
                    let trace_span = TraceSpan {
                        trace_id: hex::encode(&span.trace_id),
                        span_id: hex::encode(&span.span_id),
                        parent_span_id: if span.parent_span_id.is_empty() {
                            None
                        } else {
                            Some(hex::encode(&span.parent_span_id))
                        },
                        name: span.name,
                        start_time: nanos_to_datetime(span.start_time_unix_nano),
                        end_time: Some(nanos_to_datetime(span.end_time_unix_nano)),
                        duration_ms: (span.end_time_unix_nano - span.start_time_unix_nano) as f64 / 1_000_000.0,
                        node_id: extract_node_id(&span.attributes),
                        attributes: Some(extract_attributes(&span.attributes)),
                    };

                    let storage = self.storage.lock().await;
                    if let Err(e) = storage.insert_span(&trace_span).await {
                        tracing::warn!("Failed to insert span: {}", e);
                    }
                }
            }
        }

        Ok(Response::new(ExportTraceServiceResponse {
            partial_success: None,
        }))
    }
}

fn extract_attributes(attrs: &[opentelemetry_proto::tonic::common::v1::KeyValue]) -> HashMap<String, String> {
    attrs.iter()
        .filter_map(|kv| {
            kv.value.as_ref().and_then(|v| {
                v.value.as_ref().map(|val| {
                    let value = match val {
                        opentelemetry_proto::tonic::common::v1::any_value::Value::StringValue(s) => s.clone(),
                        opentelemetry_proto::tonic::common::v1::any_value::Value::IntValue(i) => i.to_string(),
                        opentelemetry_proto::tonic::common::v1::any_value::Value::BoolValue(b) => b.to_string(),
                        _ => return None,
                    };
                    Some((kv.key.clone(), value))
                }).flatten()
            })
        })
        .collect()
}

fn extract_node_id(attrs: &[opentelemetry_proto::tonic::common::v1::KeyValue]) -> Option<String> {
    attrs.iter()
        .find(|kv| kv.key == "node_id" || kv.key == "dora.node_id")
        .and_then(|kv| {
            kv.value.as_ref().and_then(|v| {
                match &v.value {
                    Some(opentelemetry_proto::tonic::common::v1::any_value::Value::StringValue(s)) => Some(s.clone()),
                    _ => None,
                }
            })
        })
}
```

---

## AI Agent Layer

Dora Studio integrates AI-powered assistance via a bottom chat bar in each mini-app (Claude Code style).

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    AGENT COORDINATOR                        │
│  ┌─────────────┐  ┌─────────────┐  ┌───────────────────┐   │
│  │ Tool        │  │ Context     │  │ LLM Client        │   │
│  │ Registry    │  │ Manager     │  │ (Multi-provider)  │   │
│  └──────┬──────┘  └──────┬──────┘  └─────────┬─────────┘   │
│         └────────────────┴───────────────────┘              │
│                          │                                  │
│                    AGENT LOOP                               │
│         ┌────────────────┴────────────────┐                 │
│         │ 1. Get current app context      │                 │
│         │ 2. Send to LLM with app tools   │                 │
│         │ 3. Execute tool calls locally   │                 │
│         │ 4. Stream response to chat bar  │                 │
│         │ 5. Update app state             │                 │
│         └─────────────────────────────────┘                 │
└─────────────────────────────────────────────────────────────┘
```

### Agent Coordinator

```rust
// dora-studio-client/src/agent/mod.rs

use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AgentCoordinator {
    llm_client: Arc<dyn LlmClient>,
    tool_registry: ToolRegistry,
    context_manager: ContextManager,
}

impl AgentCoordinator {
    pub fn new(config: AgentConfig) -> Self {
        let llm_client: Arc<dyn LlmClient> = match config.provider {
            Provider::Claude => Arc::new(ClaudeClient::new(&config.api_key)),
            Provider::OpenAI => Arc::new(OpenAIClient::new(&config.api_key)),
            Provider::Ollama => Arc::new(OllamaClient::new(&config.endpoint)),
        };

        Self {
            llm_client,
            tool_registry: ToolRegistry::new(),
            context_manager: ContextManager::new(),
        }
    }

    /// Process user message and return AI response
    pub async fn process_message(
        &self,
        app_id: &str,
        message: &str,
        app_state: &AppState,
    ) -> Result<AgentResponse, AgentError> {
        // 1. Build context from app state
        let context = self.context_manager.build_context(app_id, app_state);

        // 2. Get tools for this app
        let tools = self.tool_registry.get_tools(app_id);

        // 3. Send to LLM
        let mut response = self.llm_client.chat(
            &context,
            message,
            &tools,
        ).await?;

        // 4. Execute tool calls
        while let Some(tool_call) = response.tool_calls.pop() {
            let result = self.execute_tool(&tool_call).await?;
            response = self.llm_client.continue_with_result(
                &context,
                &tool_call,
                &result,
            ).await?;
        }

        Ok(response)
    }

    async fn execute_tool(&self, tool_call: &ToolCall) -> Result<ToolResult, AgentError> {
        self.tool_registry.execute(tool_call).await
    }
}
```

### LLM Client Trait

```rust
// dora-studio-client/src/agent/llm_client.rs

#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Send chat message with tools
    async fn chat(
        &self,
        context: &Context,
        message: &str,
        tools: &[ToolDefinition],
    ) -> Result<AgentResponse, LlmError>;

    /// Continue conversation with tool result
    async fn continue_with_result(
        &self,
        context: &Context,
        tool_call: &ToolCall,
        result: &ToolResult,
    ) -> Result<AgentResponse, LlmError>;

    /// Stream response tokens
    async fn chat_stream(
        &self,
        context: &Context,
        message: &str,
        tools: &[ToolDefinition],
    ) -> Result<impl Stream<Item = StreamChunk>, LlmError>;
}

pub struct ClaudeClient {
    client: anthropic_sdk::Client,
    model: String,
}

impl ClaudeClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            client: anthropic_sdk::Client::new(api_key),
            model: "claude-sonnet-4-20250514".to_string(),
        }
    }
}

#[async_trait]
impl LlmClient for ClaudeClient {
    async fn chat(
        &self,
        context: &Context,
        message: &str,
        tools: &[ToolDefinition],
    ) -> Result<AgentResponse, LlmError> {
        let request = self.client.messages()
            .model(&self.model)
            .system(&context.system_prompt)
            .messages(context.history.clone())
            .user(message)
            .tools(tools.iter().map(|t| t.to_claude_tool()).collect())
            .max_tokens(4096);

        let response = request.send().await?;

        Ok(AgentResponse::from_claude_response(response))
    }

    // ... other implementations
}
```

### Tool Registry

```rust
// dora-studio-client/src/agent/tools.rs

use std::collections::HashMap;

pub struct ToolRegistry {
    tools: HashMap<String, Vec<Box<dyn Tool>>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut registry = Self { tools: HashMap::new() };

        // Register tools for each app
        registry.register("dataflow-manager", vec![
            Box::new(ListDataflowsTool),
            Box::new(StartDataflowTool),
            Box::new(StopDataflowTool),
            Box::new(GetNodeMetricsTool),
            Box::new(RestartDataflowTool),
        ]);

        registry.register("yaml-editor", vec![
            Box::new(ValidateYamlTool),
            Box::new(ExplainDataflowTool),
            Box::new(SuggestFixTool),
            Box::new(GenerateDataflowTool),
            Box::new(AddNodeTool),
        ]);

        registry.register("log-viewer", vec![
            Box::new(SearchLogsTool),
            Box::new(AnalyzeLogsTool),
            Box::new(FilterLogsTool),
            Box::new(ExportLogsTool),
        ]);

        registry.register("telemetry-dashboard", vec![
            Box::new(QueryMetricsTool),
            Box::new(FindBottleneckTool),
            Box::new(GetTraceTool),
            Box::new(AnalyzeLatencyTool),
        ]);

        registry
    }

    pub fn get_tools(&self, app_id: &str) -> Vec<ToolDefinition> {
        self.tools
            .get(app_id)
            .map(|tools| tools.iter().map(|t| t.definition()).collect())
            .unwrap_or_default()
    }

    pub async fn execute(&self, call: &ToolCall) -> Result<ToolResult, AgentError> {
        // Find and execute the tool
        for tools in self.tools.values() {
            for tool in tools {
                if tool.name() == call.name {
                    return tool.execute(&call.arguments).await;
                }
            }
        }
        Err(AgentError::ToolNotFound(call.name.clone()))
    }
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn definition(&self) -> ToolDefinition;
    async fn execute(&self, args: &serde_json::Value) -> Result<ToolResult, AgentError>;
}
```

### Example Tool Implementation

```rust
// apps/dataflow-manager/src/tools.rs

pub struct StartDataflowTool;

#[async_trait]
impl Tool for StartDataflowTool {
    fn name(&self) -> &str {
        "start_dataflow"
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "start_dataflow".to_string(),
            description: "Start a dataflow from a YAML file path".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "yaml_path": {
                        "type": "string",
                        "description": "Path to the dataflow YAML file"
                    },
                    "name": {
                        "type": "string",
                        "description": "Optional name for the dataflow"
                    }
                },
                "required": ["yaml_path"]
            }),
        }
    }

    async fn execute(&self, args: &serde_json::Value) -> Result<ToolResult, AgentError> {
        let yaml_path = args["yaml_path"].as_str()
            .ok_or(AgentError::InvalidArgument("yaml_path required"))?;

        let name = args["name"].as_str();

        // Call DoraClient to start dataflow
        let client = get_dora_client().await?;
        let uuid = client.start_dataflow(Path::new(yaml_path), name).await?;

        Ok(ToolResult {
            success: true,
            output: format!("Dataflow started with UUID: {}", uuid),
            data: Some(json!({ "uuid": uuid.to_string() })),
        })
    }
}
```

### Chat Bar Widget

```rust
// dora-studio-widgets/src/chat_bar.rs

use makepad_widgets::*;

live_design! {
    ChatBar = {{ChatBar}} {
        width: Fill, height: Fit,
        flow: Down,

        // Chat history (collapsible)
        chat_history = <View> {
            width: Fill, height: 120,
            visible: false,  // Expand when has messages

            <PortalList> {
                // Virtualized message list
            }
        }

        // Input bar (always visible)
        input_bar = <View> {
            width: Fill, height: 44,
            padding: 8,
            spacing: 8,
            align: { y: 0.5 },

            prompt_icon = <Label> {
                text: "💬"
            }

            input = <TextInput> {
                width: Fill,
                placeholder: "Ask AI..."
                on_return: SendMessage
            }

            send_button = <Button> {
                text: "↵"
                on_click: SendMessage
            }
        }

        // Status indicator
        status = <Label> {
            visible: false,
            text: "Thinking..."
        }
    }
}

#[derive(Live, Widget)]
pub struct ChatBar {
    #[walk] walk: Walk,
    #[layout] layout: Layout,

    #[live] chat_history: View,
    #[live] input_bar: View,
    #[live] status: Label,

    #[rust] messages: Vec<ChatMessage>,
    #[rust] agent: Option<Arc<AgentCoordinator>>,
    #[rust] app_id: String,
}

impl ChatBar {
    pub fn set_agent(&mut self, agent: Arc<AgentCoordinator>, app_id: &str) {
        self.agent = Some(agent);
        self.app_id = app_id.to_string();
    }

    fn send_message(&mut self, cx: &mut Cx) {
        let input = self.input_bar.text_input(id!(input));
        let message = input.text();

        if message.is_empty() {
            return;
        }

        // Add user message
        self.messages.push(ChatMessage::user(&message));
        input.set_text("");

        // Show thinking status
        self.status.set_visible(true);
        self.redraw(cx);

        // Send to agent (async)
        if let Some(agent) = &self.agent {
            let agent = agent.clone();
            let app_id = self.app_id.clone();
            let message = message.clone();

            cx.spawn(async move {
                let response = agent.process_message(&app_id, &message, &AppState::default()).await;
                // Send response back to UI thread via signal
            });
        }
    }
}

impl Widget for ChatBar {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        match event {
            Event::Actions(actions) => {
                if self.input_bar.button(id!(send_button)).clicked(actions) {
                    self.send_message(cx);
                }
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.draw_abs(cx, walk);
        DrawStep::done()
    }
}
```

### Context Management

```rust
// dora-studio-client/src/agent/context.rs

pub struct ContextManager {
    max_history: usize,
}

impl ContextManager {
    pub fn new() -> Self {
        Self { max_history: 10 }
    }

    pub fn build_context(&self, app_id: &str, state: &AppState) -> Context {
        let system_prompt = format!(
            r#"You are an AI assistant integrated into Dora Studio, a dashboard for managing Dora dataflows.

Current app: {}

Available context:
{}

You have access to tools to interact with the Dora system. Use them to help the user.
Be concise and actionable. Show tool results clearly."#,
            app_id,
            self.summarize_state(state)
        );

        Context {
            system_prompt,
            history: state.chat_history.iter()
                .rev()
                .take(self.max_history)
                .rev()
                .cloned()
                .collect(),
        }
    }

    fn summarize_state(&self, state: &AppState) -> String {
        let mut summary = String::new();

        if !state.dataflows.is_empty() {
            summary.push_str(&format!(
                "Active dataflows: {}\n",
                state.dataflows.iter()
                    .map(|d| format!("{} ({})", d.name, d.status))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        if !state.recent_errors.is_empty() {
            summary.push_str(&format!(
                "Recent errors: {}\n",
                state.recent_errors.len()
            ));
        }

        summary
    }
}
```

### Configuration

```rust
// dora-studio-client/src/agent/config.rs

#[derive(Clone, Debug, Deserialize)]
pub struct AgentConfig {
    pub provider: Provider,
    pub api_key: Option<String>,
    pub endpoint: Option<String>,
    pub model: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub enum Provider {
    Claude,
    OpenAI,
    Ollama,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            provider: Provider::Claude,
            api_key: std::env::var("ANTHROPIC_API_KEY").ok(),
            endpoint: None,
            model: None,
        }
    }
}
```

---

## Widget System

### Theme (theme.rs)

```rust
// dora-studio-widgets/src/theme.rs

use makepad_widgets::*;

// Font definitions (multi-language support)
live_design! {
    FONT_REGULAR = {
        font: { path: dep("crate://self/resources/fonts/Manrope-Regular.ttf") }
        font: { path: dep("crate://self/resources/fonts/LXGWWenKai-Regular.ttf") }
        font: { path: dep("crate://self/resources/fonts/NotoColorEmoji.ttf") }
    }

    FONT_MEDIUM = {
        font: { path: dep("crate://self/resources/fonts/Manrope-Medium.ttf") }
        font: { path: dep("crate://self/resources/fonts/LXGWWenKai-Regular.ttf") }
        font: { path: dep("crate://self/resources/fonts/NotoColorEmoji.ttf") }
    }

    FONT_SEMIBOLD = {
        font: { path: dep("crate://self/resources/fonts/Manrope-SemiBold.ttf") }
        font: { path: dep("crate://self/resources/fonts/LXGWWenKai-Bold.ttf") }
        font: { path: dep("crate://self/resources/fonts/NotoColorEmoji.ttf") }
    }

    FONT_BOLD = {
        font: { path: dep("crate://self/resources/fonts/Manrope-Bold.ttf") }
        font: { path: dep("crate://self/resources/fonts/LXGWWenKai-Bold.ttf") }
        font: { path: dep("crate://self/resources/fonts/NotoColorEmoji.ttf") }
    }
}

// Color palette - Light mode
pub const DARK_BG: Vec4 = vec4(0.96, 0.97, 0.98, 1.0);          // #f5f7fa
pub const PANEL_BG: Vec4 = vec4(1.0, 1.0, 1.0, 1.0);            // #ffffff
pub const ACCENT_BLUE: Vec4 = vec4(0.23, 0.51, 0.96, 1.0);      // #3b82f6
pub const ACCENT_GREEN: Vec4 = vec4(0.06, 0.72, 0.51, 1.0);     // #10b981
pub const ACCENT_RED: Vec4 = vec4(0.94, 0.27, 0.27, 1.0);       // #ef4444
pub const ACCENT_AMBER: Vec4 = vec4(0.96, 0.62, 0.04, 1.0);     // #f59e0b
pub const TEXT_PRIMARY: Vec4 = vec4(0.12, 0.16, 0.22, 1.0);     // #1f2937
pub const TEXT_SECONDARY: Vec4 = vec4(0.42, 0.45, 0.50, 1.0);   // #6b7280
pub const BORDER: Vec4 = vec4(0.90, 0.91, 0.92, 1.0);           // #e5e7eb
pub const HOVER_BG: Vec4 = vec4(0.95, 0.96, 0.97, 1.0);         // #f1f5f9

// Color palette - Dark mode
pub const DARK_BG_DARK: Vec4 = vec4(0.06, 0.09, 0.16, 1.0);     // #0f172a
pub const PANEL_BG_DARK: Vec4 = vec4(0.12, 0.16, 0.23, 1.0);    // #1f293b
pub const ACCENT_BLUE_DARK: Vec4 = vec4(0.38, 0.65, 0.98, 1.0); // #60a5fa
pub const TEXT_PRIMARY_DARK: Vec4 = vec4(0.95, 0.96, 0.97, 1.0);// #f1f5f9
pub const TEXT_SECONDARY_DARK: Vec4 = vec4(0.58, 0.64, 0.72, 1.0); // #94a3b8
pub const BORDER_DARK: Vec4 = vec4(0.20, 0.25, 0.33, 1.0);      // #334155

// Status colors
pub const STATUS_RUNNING: Vec4 = ACCENT_GREEN;
pub const STATUS_FINISHED: Vec4 = TEXT_SECONDARY;
pub const STATUS_FAILED: Vec4 = ACCENT_RED;

// Log level colors
pub const LOG_DEBUG: Vec4 = vec4(0.42, 0.45, 0.50, 1.0);        // Gray
pub const LOG_INFO: Vec4 = ACCENT_BLUE;
pub const LOG_WARN: Vec4 = ACCENT_AMBER;
pub const LOG_ERROR: Vec4 = ACCENT_RED;
```

### Time Series Chart Widget

```rust
// dora-studio-widgets/src/time_series_chart.rs

use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;
    use crate::theme::*;

    TimeSeriesChart = {{TimeSeriesChart}} {
        width: Fill, height: 200
        show_bg: true

        draw_bg: {
            instance dark_mode: 0.0
            fn pixel(self) -> vec4 {
                return mix((PANEL_BG), (PANEL_BG_DARK), self.dark_mode);
            }
        }

        // Chart area
        chart_area = <View> {
            width: Fill, height: Fill
            margin: { left: 50, right: 20, top: 20, bottom: 30 }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct TimeSeriesChart {
    #[deref]
    view: View,

    #[rust]
    series: Vec<ChartSeries>,

    #[rust]
    time_range: TimeRange,

    #[rust]
    y_axis_label: String,
}

pub struct ChartSeries {
    pub name: String,
    pub color: Vec4,
    pub data: Vec<DataPoint>,
}

pub struct DataPoint {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub value: f64,
}

pub struct TimeRange {
    pub start: chrono::DateTime<chrono::Utc>,
    pub end: chrono::DateTime<chrono::Utc>,
}

impl TimeSeriesChart {
    pub fn set_series(&mut self, series: Vec<ChartSeries>) {
        self.series = series;
    }

    pub fn set_time_range(&mut self, range: TimeRange) {
        self.time_range = range;
    }
}

impl Widget for TimeSeriesChart {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Draw background
        self.view.draw_walk(cx, scope, walk)?;

        // Get chart area bounds
        let rect = self.view.area().rect(cx);
        let margin = Margin { left: 50.0, right: 20.0, top: 20.0, bottom: 30.0 };
        let chart_rect = Rect {
            pos: dvec2(rect.pos.x + margin.left, rect.pos.y + margin.top),
            size: dvec2(
                rect.size.x - margin.left - margin.right,
                rect.size.y - margin.top - margin.bottom,
            ),
        };

        // Draw grid lines
        self.draw_grid(cx, &chart_rect);

        // Draw each series
        for series in &self.series {
            self.draw_series(cx, &chart_rect, series);
        }

        // Draw axes
        self.draw_axes(cx, &chart_rect);

        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        // Handle zoom/pan gestures
        // ...
    }
}

impl TimeSeriesChartRef {
    pub fn update_dark_mode(&self, cx: &mut Cx, dark_mode: f64) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.view.apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });
            inner.view.redraw(cx);
        }
    }
}
```

### Dataflow Graph Widget

```rust
// dora-studio-widgets/src/dataflow_graph.rs

use makepad_widgets::*;
use std::collections::HashMap;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use crate::theme::*;

    DataflowGraph = {{DataflowGraph}} {
        width: Fill, height: Fill
        show_bg: true

        draw_bg: {
            instance dark_mode: 0.0
            fn pixel(self) -> vec4 {
                return mix((DARK_BG), (DARK_BG_DARK), self.dark_mode);
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct DataflowGraph {
    #[deref]
    view: View,

    #[rust]
    nodes: Vec<GraphNode>,

    #[rust]
    edges: Vec<GraphEdge>,

    #[rust]
    node_positions: HashMap<String, DVec2>,

    #[rust]
    selected_node: Option<String>,

    #[rust]
    pan_offset: DVec2,

    #[rust]
    zoom: f64,
}

#[derive(Clone)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub node_type: NodeType,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

#[derive(Clone, Copy)]
pub enum NodeType {
    Source,      // No inputs (e.g., camera, timer)
    Processor,   // Has inputs and outputs
    Sink,        // No outputs (e.g., display, logger)
}

#[derive(Clone)]
pub struct GraphEdge {
    pub from_node: String,
    pub from_output: String,
    pub to_node: String,
    pub to_input: String,
}

impl DataflowGraph {
    /// Load graph from dora-core descriptor
    pub fn load_from_descriptor(&mut self, yaml: &str) -> Result<(), String> {
        // Parse YAML using dora-core
        let descriptor = dora_core::descriptor::Descriptor::parse(yaml)
            .map_err(|e| e.to_string())?;

        // Convert to graph nodes
        self.nodes.clear();
        self.edges.clear();

        for node in descriptor.nodes {
            let graph_node = GraphNode {
                id: node.id.to_string(),
                label: node.name.unwrap_or_else(|| node.id.to_string()),
                node_type: if node.inputs.is_empty() {
                    NodeType::Source
                } else if node.outputs.is_empty() {
                    NodeType::Sink
                } else {
                    NodeType::Processor
                },
                inputs: node.inputs.keys().map(|k| k.to_string()).collect(),
                outputs: node.outputs.iter().map(|o| o.to_string()).collect(),
            };
            self.nodes.push(graph_node);

            // Create edges from input mappings
            for (input_name, input) in &node.inputs {
                if let Some(mapping) = &input.mapping {
                    // Parse "source_node/output" format
                    if let Some((source_node, output)) = mapping.split_once('/') {
                        self.edges.push(GraphEdge {
                            from_node: source_node.to_string(),
                            from_output: output.to_string(),
                            to_node: node.id.to_string(),
                            to_input: input_name.to_string(),
                        });
                    }
                }
            }
        }

        // Calculate layout using dagre-style algorithm
        self.layout_graph();

        Ok(())
    }

    /// Calculate node positions using hierarchical layout
    fn layout_graph(&mut self) {
        // Simple top-to-bottom layout
        // TODO: Implement proper dagre algorithm

        let node_width = 120.0;
        let node_height = 60.0;
        let h_spacing = 40.0;
        let v_spacing = 80.0;

        // Group nodes by depth (topological order)
        let depths = self.calculate_node_depths();

        let mut level_counts: HashMap<usize, usize> = HashMap::new();

        for node in &self.nodes {
            let depth = depths.get(&node.id).copied().unwrap_or(0);
            let level_index = level_counts.entry(depth).or_insert(0);

            let x = *level_index as f64 * (node_width + h_spacing) + 50.0;
            let y = depth as f64 * (node_height + v_spacing) + 50.0;

            self.node_positions.insert(node.id.clone(), dvec2(x, y));
            *level_index += 1;
        }
    }

    fn calculate_node_depths(&self) -> HashMap<String, usize> {
        // BFS to calculate depth from sources
        let mut depths: HashMap<String, usize> = HashMap::new();
        let mut queue: Vec<String> = Vec::new();

        // Find source nodes (no incoming edges)
        for node in &self.nodes {
            let has_incoming = self.edges.iter().any(|e| e.to_node == node.id);
            if !has_incoming {
                depths.insert(node.id.clone(), 0);
                queue.push(node.id.clone());
            }
        }

        while let Some(node_id) = queue.pop() {
            let current_depth = depths.get(&node_id).copied().unwrap_or(0);

            for edge in &self.edges {
                if edge.from_node == node_id {
                    let next_depth = current_depth + 1;
                    let existing = depths.get(&edge.to_node).copied().unwrap_or(0);
                    if next_depth > existing {
                        depths.insert(edge.to_node.clone(), next_depth);
                        queue.push(edge.to_node.clone());
                    }
                }
            }
        }

        depths
    }
}

impl Widget for DataflowGraph {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)?;

        let rect = self.view.area().rect(cx);

        // Apply pan and zoom
        let transform_point = |p: DVec2| -> DVec2 {
            (p + self.pan_offset) * self.zoom
        };

        // Draw edges first (behind nodes)
        for edge in &self.edges {
            if let (Some(&from_pos), Some(&to_pos)) = (
                self.node_positions.get(&edge.from_node),
                self.node_positions.get(&edge.to_node),
            ) {
                let from = transform_point(from_pos + dvec2(60.0, 60.0)); // Bottom center
                let to = transform_point(to_pos + dvec2(60.0, 0.0));       // Top center

                // Draw bezier curve
                self.draw_edge(cx, from, to, &edge.from_output);
            }
        }

        // Draw nodes
        for node in &self.nodes {
            if let Some(&pos) = self.node_positions.get(&node.id) {
                let screen_pos = transform_point(pos);
                let is_selected = self.selected_node.as_ref() == Some(&node.id);
                self.draw_node(cx, screen_pos, node, is_selected);
            }
        }

        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        // Handle click to select node
        match event.hits(cx, self.view.area()) {
            Hit::FingerDown(fe) => {
                // Find clicked node
                for node in &self.nodes {
                    if let Some(&pos) = self.node_positions.get(&node.id) {
                        let screen_pos = (pos + self.pan_offset) * self.zoom;
                        let node_rect = Rect {
                            pos: screen_pos,
                            size: dvec2(120.0, 60.0) * self.zoom,
                        };
                        if node_rect.contains(fe.abs) {
                            self.selected_node = Some(node.id.clone());
                            cx.widget_action(
                                self.view.widget_uid(),
                                &scope.path,
                                DataflowGraphAction::NodeSelected(node.id.clone()),
                            );
                            self.view.redraw(cx);
                            return;
                        }
                    }
                }
                // Click on empty space deselects
                self.selected_node = None;
                self.view.redraw(cx);
            }
            Hit::FingerMove(fe) => {
                // Pan on drag
                self.pan_offset += fe.abs - fe.abs_start;
                self.view.redraw(cx);
            }
            _ => {}
        }
    }
}

#[derive(Clone, Debug)]
pub enum DataflowGraphAction {
    NodeSelected(String),
}
```

---

## Shell Architecture

### Main App Widget

```rust
// dora-studio-shell/src/app.rs

use makepad_widgets::*;
use dora_studio_widgets::AppRegistry;
use dora_studio_client::{DoraClient, Storage};

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use dora_studio_widgets::theme::*;

    App = {{App}} {
        ui: <Window> {
            window: { inner_size: vec2(1400, 900) }
            body = <View> {
                width: Fill, height: Fill
                flow: Down

                // Status bar (always visible)
                status_bar = <StatusBar> {
                    height: 32
                }

                // Main content
                main = <View> {
                    width: Fill, height: Fill
                    flow: Right

                    // Sidebar navigation
                    sidebar = <Sidebar> {
                        width: 200
                    }

                    // App content area (overlay)
                    content = <View> {
                        width: Fill, height: Fill
                        flow: Overlay

                        dataflow_manager_page = <DataflowManagerScreen> {
                            width: Fill, height: Fill
                            visible: true
                        }
                        yaml_editor_page = <YamlEditorScreen> {
                            width: Fill, height: Fill
                            visible: false
                        }
                        log_viewer_page = <LogViewerScreen> {
                            width: Fill, height: Fill
                            visible: false
                        }
                        telemetry_page = <TelemetryDashboardScreen> {
                            width: Fill, height: Fill
                            visible: false
                        }
                    }
                }
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct App {
    #[live]
    ui: WidgetRef,

    // Theme state
    #[rust]
    dark_mode: bool,
    #[rust]
    dark_mode_anim: f64,

    // Navigation state
    #[rust]
    active_app: AppId,

    // App registry
    #[rust]
    app_registry: AppRegistry,

    // Dora client (shared across apps)
    #[rust]
    client: Option<DoraClient>,

    // Storage (shared across apps)
    #[rust]
    storage: Option<Storage>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppId {
    DataflowManager,
    YamlEditor,
    LogViewer,
    TelemetryDashboard,
}

impl App {
    fn switch_app(&mut self, cx: &mut Cx, app: AppId) {
        self.active_app = app;

        // Toggle visibility
        self.ui.view(ids!(content.dataflow_manager_page)).apply_over(cx, live!{
            visible: (app == AppId::DataflowManager)
        });
        self.ui.view(ids!(content.yaml_editor_page)).apply_over(cx, live!{
            visible: (app == AppId::YamlEditor)
        });
        self.ui.view(ids!(content.log_viewer_page)).apply_over(cx, live!{
            visible: (app == AppId::LogViewer)
        });
        self.ui.view(ids!(content.telemetry_page)).apply_over(cx, live!{
            visible: (app == AppId::TelemetryDashboard)
        });

        self.ui.redraw(cx);
    }

    fn toggle_dark_mode(&mut self, cx: &mut Cx) {
        self.dark_mode = !self.dark_mode;
        self.dark_mode_anim = if self.dark_mode { 1.0 } else { 0.0 };

        // Propagate to all apps
        self.notify_dark_mode_change(cx, self.dark_mode_anim);
    }

    fn notify_dark_mode_change(&mut self, cx: &mut Cx, dark_mode: f64) {
        // Update shell widgets
        self.ui.status_bar(ids!(status_bar)).update_dark_mode(cx, dark_mode);
        self.ui.sidebar(ids!(sidebar)).update_dark_mode(cx, dark_mode);

        // Update all apps
        self.ui.dataflow_manager_screen(ids!(dataflow_manager_page))
            .update_dark_mode(cx, dark_mode);
        self.ui.yaml_editor_screen(ids!(yaml_editor_page))
            .update_dark_mode(cx, dark_mode);
        self.ui.log_viewer_screen(ids!(log_viewer_page))
            .update_dark_mode(cx, dark_mode);
        self.ui.telemetry_dashboard_screen(ids!(telemetry_page))
            .update_dark_mode(cx, dark_mode);
    }
}

impl Widget for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.ui.handle_event(cx, event, scope);

        let actions = match event {
            Event::Actions(actions) => actions.as_slice(),
            _ => return,
        };

        // Handle sidebar navigation
        if let Some(app) = self.ui.sidebar(ids!(sidebar)).app_selected(actions) {
            self.switch_app(cx, app);
        }

        // Handle status bar actions
        if self.ui.status_bar(ids!(status_bar)).dark_mode_toggled(actions) {
            self.toggle_dark_mode(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.ui.draw_walk(cx, scope, walk)
    }
}
```

---

## Best Practices

### App Development

1. **Implement DoraApp trait**: Standard registration and metadata
2. **Use shared widgets**: Theme consistency, reusable visualizations
3. **Export `update_dark_mode()`**: Allow shell to propagate theme changes
4. **Handle events properly**: Hover before actions early-return
5. **Use `apply_over()` for visibility**: More reliable than `set_visible()`

### Storage Access

1. **Share Storage instance**: Pass `Arc<Mutex<Storage>>` to apps that need historical data
2. **Batch inserts**: Accumulate metrics/logs before writing to reduce parquet file count
3. **Run cleanup periodically**: Call `storage.cleanup()` daily to remove old partition files
4. **Use partition pruning**: DataFusion automatically prunes partitions based on date filters

### Client Communication

1. **Share DoraClient**: Pass `Arc<Mutex<DoraClient>>` for coordinator access
2. **Handle disconnections**: Reconnect on connection loss
3. **Use async**: Don't block UI thread on network operations
4. **Cache state**: Store dataflow list locally, refresh on timer

### Performance

1. **Virtualize large lists**: Only render visible rows in LogTable
2. **Limit chart data points**: Downsample to ~1000 points for rendering
3. **Debounce graph layout**: Don't recalculate on every keystroke
4. **Use DataFusion for aggregation**: Push computation to query engine

---

## Troubleshooting

### Widget Not Rendering
- Check `live_design(cx)` is called in correct order
- Verify import paths in live_design macro
- Ensure `visible: true` is set

### Coordinator Connection Fails
- Check coordinator is running: `dora system check`
- Verify address/port configuration
- Check firewall rules

### Storage Errors
- Ensure `~/.dora/studio/` directory exists with write permissions
- Check disk space for parquet files
- Verify Arrow/Parquet crate versions are compatible

### OTLP Receiver Not Working
- Check port 4317 is available
- Verify Dora daemon has `DORA_OTLP_ENDPOINT` set
- Check trace export is enabled in dora

### Chart Not Updating
- Ensure `redraw(cx)` is called after data changes
- Check time range includes data points
- Verify data format matches chart expectations

---

## Related Documents

| Document | Description |
|----------|-------------|
| `PRD.md` | Product requirements, feature specifications |
| `README.md` | Quick start, project overview |
| `../mofa-studio/ARCHITECTURE.md` | Reference architecture for Makepad apps |
| `../dora/CLAUDE.md` | Dora framework architecture, coordinator protocol |

---

*Last Updated: January 2026*
*Status: Planning Phase*
