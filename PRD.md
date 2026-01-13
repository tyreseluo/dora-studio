# Dora Studio - Product Requirements Document

> A native desktop dashboard for the Dora dataflow framework

**Version**: 0.1.0 (Draft)
**Last Updated**: January 2026
**Status**: Planning
**Implementation**: 100% Rust (no C/C++ dependencies)

---

## 1. Executive Summary

### Vision

Dora Studio is a GPU-accelerated native desktop application that provides a unified visual interface for the Dora dataflow framework. It replaces command-line workflows with an intuitive dashboard for dataflow lifecycle management, real-time monitoring, and observability.

### Key Value Propositions

| Capability | CLI Equivalent | Dora Studio Advantage |
|------------|---------------|----------------------|
| Dataflow management | `dora list/start/stop` | Visual status, one-click actions, batch operations |
| Graph visualization | `dora graph --open` | Live editing, interactive node inspection |
| Log analysis | `dora logs -f` | Filtering, search, multi-dataflow aggregation |
| Performance monitoring | `dora top` | Time-series charts, historical trends, drill-down |
| Trace analysis | External Jaeger | Built-in, correlated with metrics and logs |

### Target Users

- **Developers**: Building and debugging dataflows locally, visualizing YAML graphs, iterating on node logic
- **Operators/SREs**: Monitoring production dataflows, incident response, capacity planning

### Design Principles

1. **Self-contained**: No external dependencies required (embedded storage, built-in OTLP receiver)
2. **CLI parity**: Every CLI capability accessible via UI
3. **Real-time first**: Live updates for metrics, logs, and dataflow state
4. **5-second rule**: Critical information visible immediately upon opening

---

## 2. Architecture Overview

### System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        DORA STUDIO                              │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────┐ │
│  │ Dataflow │  │  YAML    │  │   Log    │  │    Telemetry     │ │
│  │ Manager  │  │  Editor  │  │  Viewer  │  │    Dashboard     │ │
│  │   App    │  │   App    │  │   App    │  │       App        │ │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────────┬─────────┘ │
│       │             │             │                  │          │
├───────┴─────────────┴─────────────┴──────────────────┴──────────┤
│                     SHELL (Navigation, Theme, Coordination)     │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────────┐  ┌──────────────────┐  ┌────────────────┐ │
│  │  Shared Widgets  │  │   Dora Client    │  │  Embedded DB   │ │
│  │  (Charts, Graph) │  │   (Coordinator)  │  │  (DataFusion)  │ │
│  └──────────────────┘  └────────┬─────────┘  └───────┬────────┘ │
└─────────────────────────────────┼────────────────────┼──────────┘
                                  │                    │
                    ┌─────────────▼─────────────┐      │
                    │     Dora Infrastructure   │      │
                    │  ┌────────────────────┐   │      │
                    │  │    Coordinator     │◄──┼──────┘
                    │  │   (TCP :53290)     │   │
                    │  └────────┬───────────┘   │
                    │           │               │
                    │  ┌────────▼───────────┐   │
                    │  │      Daemon(s)     │   │
                    │  │  (Node Management) │   │
                    │  └────────────────────┘   │
                    └───────────────────────────┘
```

### Plugin Architecture (MofaApp-style)

Following mofa-studio patterns, each mini-app implements a standard trait:

```rust
pub trait DoraApp {
    fn info() -> AppInfo;           // Metadata: name, id, icon
    fn live_design(cx: &mut Cx);    // Widget registration
}

pub struct AppInfo {
    pub name: &'static str,         // "Dataflow Manager"
    pub id: &'static str,           // "dataflow-manager"
    pub icon: &'static str,         // SVG icon path
    pub description: &'static str,
}
```

### Integration Points

| Component | Protocol | Data Format |
|-----------|----------|-------------|
| Coordinator | TCP (port 53290) | JSON (ControlRequest/Reply) |
| Daemon metrics | Via Coordinator | NodeMetricsInfo struct |
| Logs | TCP subscription | LogMessage stream |
| Traces | OTLP gRPC (port 4317) | OpenTelemetry spans |
| YAML parsing | Local | dora-core descriptor |

---

## 3. Mini-App Specifications

### 3.1 Dataflow Manager App

**Purpose**: Lifecycle management for dataflows (replaces `dora list`, `dora start`, `dora stop`, `dora up`)

#### Features

| Feature | Priority | Description |
|---------|----------|-------------|
| Dataflow list | P0 | Table with UUID, Name, Status, Node Count, CPU%, Memory |
| Status badges | P0 | Visual indicators: Running (green), Finished (gray), Failed (red) |
| Start dataflow | P0 | File picker → parse YAML → start via coordinator |
| Stop dataflow | P0 | Graceful (with duration) or force stop |
| Destroy dataflow | P0 | Remove from coordinator tracking |
| Infrastructure status | P1 | Coordinator/Daemon health panel |
| `dora up` equivalent | P1 | One-click bootstrap coordinator + daemon |
| Node list expansion | P1 | Expand row to see per-node status, PID, metrics |
| Batch operations | P2 | Select multiple, stop all |
| Recent dataflows | P2 | Quick-access to recently run YAMLs |

#### UI Layout

```
┌──────────────────────────────────────────────────────────────┐
│ Dataflow Manager                            [Start ▼] [Refresh]│
├──────────────────────────────────────────────────────────────┤
│ Infrastructure: ● Coordinator (connected)  ● Daemon (1 active)│
├──────────────────────────────────────────────────────────────┤
│ ┌──────────────────────────────────────────────────────────┐ │
│ │ UUID          │ Name     │ Status  │ Nodes │ CPU │ Mem  │ │
│ ├───────────────┼──────────┼─────────┼───────┼─────┼──────┤ │
│ │ a1b2c3...     │ yolo-det │ ● Run   │ 4     │ 45% │ 2.1G │ │
│ │ d4e5f6...     │ voice-ch │ ● Run   │ 7     │ 12% │ 0.8G │ │
│ │ g7h8i9...     │ benchmark│ ○ Done  │ 2     │ -   │ -    │ │
│ └──────────────────────────────────────────────────────────┘ │
│                                                              │
│ Selected: yolo-det                    [Stop ▼] [Logs] [View] │
└──────────────────────────────────────────────────────────────┘
```

#### Data Requirements

From Coordinator:
- `ControlRequest::List` → dataflow list with status
- `ControlRequest::GetNodeInfo` → per-node metrics
- `ControlRequest::Start/Stop/Destroy` → lifecycle actions

---

### 3.2 YAML Editor + Graph Visualizer App

**Purpose**: Edit dataflow YAML with live graph preview (replaces `dora graph`, manual editing)

#### Features

| Feature | Priority | Description |
|---------|----------|-------------|
| YAML editor | P0 | Syntax highlighting, line numbers, error markers |
| Graph visualization | P0 | Node-edge diagram auto-generated from YAML |
| Live preview | P0 | Graph updates as YAML is edited |
| Validation | P0 | Real-time errors from dora-core descriptor parsing |
| Node inspector | P1 | Click node to see inputs/outputs/env/build |
| File operations | P1 | New, Open, Save, Save As |
| Auto-complete | P2 | Suggest node IDs, input references |
| Templates | P2 | Quick-start templates (Python dataflow, Rust dataflow) |
| Run from editor | P2 | Direct "Run" button to start dataflow |

#### UI Layout

```
┌──────────────────────────────────────────────────────────────┐
│ YAML Editor             [New] [Open] [Save] [Validate] [Run] │
├─────────────────────────────┬────────────────────────────────┤
│                             │                                │
│  1│ nodes:                  │    ┌──────────┐                │
│  2│   - id: camera          │    │  camera  │                │
│  3│     path: opencv-cap    │    └────┬─────┘                │
│  4│     inputs:             │         │ image                │
│  5│       tick: dora/ti...  │    ┌────▼─────┐                │
│  6│     outputs:            │    │   yolo   │                │
│  7│       - image           │    └────┬─────┘                │
│  8│                         │         │ bbox                 │
│  9│   - id: yolo            │    ┌────▼─────┐                │
│ 10│     path: dora-yolo     │    │   plot   │                │
│ 11│     inputs:             │    └──────────┘                │
│ 12│       image: camera/... │                                │
│                             │  [Selected: yolo]              │
│ ─────────────────────────── │  Inputs: image ← camera/image  │
│ ✓ Valid dataflow (3 nodes)  │  Outputs: bbox                 │
│                             │  Path: dora-yolo               │
└─────────────────────────────┴────────────────────────────────┘
```

#### Graph Rendering

- **Algorithm**: Dagre-style hierarchical layout (top-to-bottom data flow)
- **Node shapes**: Rounded rectangles with icon based on node type
- **Edge labels**: Output ID on each connection
- **Interactions**: Pan, zoom, click-to-select
- **Colors**: Match mofa-studio theme (light/dark mode)

#### Data Requirements

From dora-core library:
- `Descriptor::parse()` → parse YAML
- `validate::check()` → validation errors
- `visualize_as_mermaid()` → reference for graph structure

---

### 3.3 Log Viewer App

**Purpose**: Real-time log streaming with filtering (replaces `dora logs -f`)

#### Features

| Feature | Priority | Description |
|---------|----------|-------------|
| Live log stream | P0 | Real-time logs from coordinator subscription |
| Level filtering | P0 | Toggle: DEBUG, INFO, WARN, ERROR |
| Dataflow filter | P0 | Dropdown to select dataflow(s) |
| Node filter | P0 | Filter by specific node(s) |
| Text search | P0 | Regex or substring search |
| Level highlighting | P0 | Color-coded: ERROR=red, WARN=amber, INFO=blue |
| Timestamp display | P1 | Relative or absolute timestamps |
| Auto-scroll | P1 | Follow new logs, pause on scroll-up |
| Export | P1 | Download filtered logs as file |
| Log context | P2 | Click to see full structured log fields |
| Saved filters | P2 | Save and name filter presets |

#### UI Layout

```
┌──────────────────────────────────────────────────────────────┐
│ Log Viewer                                    [Export] [Clear]│
├──────────────────────────────────────────────────────────────┤
│ Dataflow: [All ▼]  Node: [All ▼]  Level: [■D ■I ■W ■E]      │
│ Search: [________________________] [.*] [Aa]                 │
├──────────────────────────────────────────────────────────────┤
│ 14:23:01.234 │ INFO  │ camera   │ Captured frame 1234        │
│ 14:23:01.245 │ DEBUG │ yolo     │ Processing batch...        │
│ 14:23:01.312 │ INFO  │ yolo     │ Detected 3 objects         │
│ 14:23:01.315 │ WARN  │ plot     │ Frame dropped (queue full) │
│ 14:23:01.400 │ ERROR │ camera   │ Device disconnected        │
│                                                              │
│ ──────────────────── [Auto-scroll: ON] ──────────────────── │
│ Showing 1,234 of 5,678 logs (filtered)                       │
└──────────────────────────────────────────────────────────────┘
```

#### Performance Considerations

- **Virtualization**: Only render visible log rows (important for 100K+ logs)
- **Buffering**: Ring buffer with configurable max size (default: 100K entries)
- **Indexing**: Build in-memory index for fast filtering

#### Data Requirements

From Coordinator:
- `ControlRequest::LogSubscribe` → streaming log messages
- `LogMessage` struct: timestamp, level, node_id, message, fields

---

### 3.4 Telemetry Dashboard App

**Purpose**: Full observability with metrics, traces, and analytics (replaces `dora top`, external Jaeger/Grafana)

#### Features

| Feature | Priority | Description |
|---------|----------|-------------|
| **Metrics View** | | |
| Time-series charts | P0 | CPU, Memory, Disk I/O per node |
| Aggregation | P0 | Sum/Avg across dataflow, node group |
| Time range selector | P0 | 5m, 15m, 1h, 6h, 24h, custom |
| Auto-refresh | P0 | Configurable interval (5s default) |
| Golden signals panel | P1 | Request rate, Error rate, Latency (p50/p95/p99) |
| **Traces View** | | |
| Trace list | P0 | Recent traces with duration, status |
| Trace detail | P0 | Waterfall/Gantt chart of spans |
| Span attributes | P0 | View input_id, output_id, custom attrs |
| Trace search | P1 | By trace_id, node, duration threshold |
| **Topic Stats View** | | |
| Topic list | P0 | All outputs with frequency (Hz) |
| Message rate chart | P1 | Sparkline per topic |
| Arrow schema viewer | P1 | Display data type info |
| Bandwidth stats | P1 | MB/s per topic |
| **Dashboard Customization** | | |
| Panel arrangement | P2 | Drag-and-drop layout |
| Saved dashboards | P2 | Persist custom layouts |

#### UI Layout - Metrics View

```
┌──────────────────────────────────────────────────────────────┐
│ Telemetry Dashboard       [Metrics] [Traces] [Topics]        │
├──────────────────────────────────────────────────────────────┤
│ Time Range: [Last 15 min ▼]  Dataflow: [yolo-detection ▼]   │
├──────────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────┐ ┌─────────────────────────┐ │
│ │ CPU Usage (%)               │ │ Memory Usage (MB)       │ │
│ │   ▂▃▅▇█▇▅▃▂▃▅▇█▇▅          │ │   ▂▂▃▃▄▅▅▆▆▇▇████       │ │
│ │ — camera  — yolo  — plot    │ │ — camera  — yolo        │ │
│ └─────────────────────────────┘ └─────────────────────────┘ │
│ ┌─────────────────────────────┐ ┌─────────────────────────┐ │
│ │ Disk Read (MB/s)            │ │ Disk Write (MB/s)       │ │
│ │   ▁▁▂▁▁▁▂▂▁▁▁▁▁▁▁          │ │   ▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁       │ │
│ └─────────────────────────────┘ └─────────────────────────┘ │
│                                                              │
│ Node Details                                                 │
│ ┌──────────────────────────────────────────────────────────┐│
│ │ Node   │ PID   │ CPU   │ Mem    │ Read   │ Write        ││
│ │ camera │ 12345 │ 23.4% │ 512 MB │ 0.1    │ 0.0          ││
│ │ yolo   │ 12346 │ 67.8% │ 1.2 GB │ 0.0    │ 0.0          ││
│ └──────────────────────────────────────────────────────────┘│
└──────────────────────────────────────────────────────────────┘
```

#### UI Layout - Traces View

```
┌──────────────────────────────────────────────────────────────┐
│ Telemetry Dashboard       [Metrics] [Traces] [Topics]        │
├──────────────────────────────────────────────────────────────┤
│ Search: [trace_id or node...] Duration > [100ms ▼]          │
├──────────────────────────────────────────────────────────────┤
│ Recent Traces                                                │
│ ┌──────────────────────────────────────────────────────────┐│
│ │ Trace ID     │ Root Span    │ Duration │ Spans │ Status  ││
│ │ abc123...    │ on_event     │ 45.2ms   │ 12    │ ✓ OK    ││
│ │ def456...    │ on_event     │ 234.1ms  │ 8     │ ✗ Error ││
│ └──────────────────────────────────────────────────────────┘│
│                                                              │
│ Trace Detail: abc123...                                      │
│ ┌──────────────────────────────────────────────────────────┐│
│ │ camera:on_event     ████████░░░░░░░░░░░░░░ 12ms         ││
│ │   └─ send_output            ██░░░░░░░░░░░░ 2ms          ││
│ │ yolo:on_event                   ████████████████ 30ms   ││
│ │   └─ inference                  ██████████████░░ 28ms   ││
│ │   └─ send_output                            ██░░ 1ms    ││
│ │ plot:on_event                                   ███ 3ms ││
│ └──────────────────────────────────────────────────────────┘│
└──────────────────────────────────────────────────────────────┘
```

#### Data Requirements

**Metrics** (from Coordinator polling):
- `NodeMetricsInfo`: pid, cpu_usage, memory_mb, disk_read/write

**Traces** (from built-in OTLP receiver):
- OpenTelemetry spans with parent_span_id, attributes
- Context propagation via `metadata.parameters["open_telemetry_context"]`

**Topics** (from Zenoh subscription when enabled):
- Message timestamps for frequency calculation
- Arrow type info from metadata

---

## 4. Embedded Storage Design

### Database Choice: Apache Arrow DataFusion + Parquet

**Rationale**:
- **100% Rust**: No C/C++ dependencies (unlike DuckDB which has C++ core)
- Columnar storage via Apache Arrow (zero-copy, high performance)
- Parquet files for efficient on-disk persistence with excellent compression
- SQL interface via DataFusion query engine
- Native integration with Arrow ecosystem

### Data Architecture

```
~/.dora/studio/
├── dataflows.parquet      # Dataflow metadata
├── nodes.parquet          # Node metadata
├── metrics/               # Partitioned by date
│   ├── 2026-01-10.parquet
│   ├── 2026-01-11.parquet
│   └── 2026-01-12.parquet
├── logs/                  # Partitioned by date
│   ├── 2026-01-10.parquet
│   └── ...
└── spans/                 # Partitioned by date
    ├── 2026-01-10.parquet
    └── ...
```

### Schema (Arrow/Parquet)

```rust
// Dataflow metadata schema
let dataflow_schema = Schema::new(vec![
    Field::new("uuid", DataType::Utf8, false),
    Field::new("name", DataType::Utf8, true),
    Field::new("status", DataType::Utf8, false),
    Field::new("created_at", DataType::Timestamp(TimeUnit::Microsecond, None), false),
    Field::new("finished_at", DataType::Timestamp(TimeUnit::Microsecond, None), true),
]);

// Metrics schema (time-series)
let metrics_schema = Schema::new(vec![
    Field::new("ts", DataType::Timestamp(TimeUnit::Microsecond, None), false),
    Field::new("dataflow_uuid", DataType::Utf8, false),
    Field::new("node_id", DataType::Utf8, false),
    Field::new("cpu_percent", DataType::Float32, true),
    Field::new("memory_mb", DataType::Float64, true),
    Field::new("disk_read_mb_s", DataType::Float64, true),
    Field::new("disk_write_mb_s", DataType::Float64, true),
]);

// Log messages schema
let logs_schema = Schema::new(vec![
    Field::new("ts", DataType::Timestamp(TimeUnit::Microsecond, None), false),
    Field::new("dataflow_uuid", DataType::Utf8, true),
    Field::new("node_id", DataType::Utf8, true),
    Field::new("level", DataType::Utf8, false),
    Field::new("target", DataType::Utf8, true),
    Field::new("message", DataType::Utf8, false),
    Field::new("fields", DataType::Utf8, true),  // JSON string
]);

// Trace spans schema
let spans_schema = Schema::new(vec![
    Field::new("trace_id", DataType::Utf8, false),
    Field::new("span_id", DataType::Utf8, false),
    Field::new("parent_span_id", DataType::Utf8, true),
    Field::new("name", DataType::Utf8, false),
    Field::new("start_time", DataType::Timestamp(TimeUnit::Microsecond, None), false),
    Field::new("end_time", DataType::Timestamp(TimeUnit::Microsecond, None), true),
    Field::new("duration_ms", DataType::Float64, true),
    Field::new("node_id", DataType::Utf8, true),
    Field::new("attributes", DataType::Utf8, true),  // JSON string
]);
```

### Query Examples (DataFusion SQL)

```rust
// Query metrics for time range
let df = ctx.sql("
    SELECT ts, node_id, cpu_percent, memory_mb
    FROM metrics
    WHERE dataflow_uuid = 'abc123'
      AND ts >= '2026-01-10T00:00:00Z'
      AND ts < '2026-01-11T00:00:00Z'
    ORDER BY ts
").await?;

// Aggregate traces by root span
let df = ctx.sql("
    SELECT
        trace_id,
        MIN(name) as root_span,
        MIN(start_time) as start_time,
        SUM(duration_ms) as total_duration_ms,
        COUNT(*) as span_count
    FROM spans
    WHERE start_time >= '2026-01-10T00:00:00Z'
    GROUP BY trace_id
    ORDER BY start_time DESC
    LIMIT 100
").await?;
```

### Retention Policy

```rust
// Delete old partitions (run periodically)
impl Storage {
    pub async fn cleanup(&self, retention: &RetentionConfig) -> Result<()> {
        let cutoff_metrics = Utc::now() - Duration::days(retention.metrics as i64);
        let cutoff_logs = Utc::now() - Duration::days(retention.logs as i64);
        let cutoff_traces = Utc::now() - Duration::days(retention.traces as i64);

        // Remove old parquet files based on partition date
        self.delete_partitions_before("metrics", cutoff_metrics)?;
        self.delete_partitions_before("logs", cutoff_logs)?;
        self.delete_partitions_before("spans", cutoff_traces)?;

        Ok(())
    }
}
```

Configurable via settings: `retention_metrics_days`, `retention_logs_days`, `retention_traces_days`

---

## 5. Shared Widget Library

### Core Visualization Widgets

| Widget | Purpose | Key Props |
|--------|---------|-----------|
| `TimeSeriesChart` | Line/area charts with time axis | data: Vec<Point>, time_range, series[] |
| `DataflowGraph` | Interactive node-edge diagram | nodes: Vec<Node>, edges: Vec<Edge> |
| `LogTable` | Virtualized log list | logs: Vec<LogEntry>, filters |
| `MetricCard` | Single value with trend | value, label, trend_direction |
| `StatusBadge` | State indicator | status: Running\|Stopped\|Error |
| `SpanTimeline` | Trace waterfall | spans: Vec<Span>, root_span_id |
| `YamlEditor` | Syntax-highlighted editor | content, on_change, errors[] |
| `SearchInput` | Filter input with options | placeholder, regex_mode, case_sensitive |

### Theme Integration

Reuse mofa-widgets theme system:
- `instance dark_mode: 0.0` shader variable
- `mix(LIGHT_COLOR, DARK_COLOR, self.dark_mode)` for all colors
- Centralized color palette in `theme.rs`

---

## 6. Dora Client Layer

### API Wrapper

```rust
pub struct DoraClient {
    coordinator_addr: SocketAddr,
    connection: Option<TcpStream>,
}

impl DoraClient {
    // Lifecycle
    pub async fn connect(&mut self) -> Result<()>;
    pub async fn disconnect(&mut self);

    // Dataflow management
    pub async fn list_dataflows(&self) -> Result<Vec<DataflowEntry>>;
    pub async fn start_dataflow(&self, path: &Path, name: Option<&str>) -> Result<Uuid>;
    pub async fn stop_dataflow(&self, id: Uuid, grace_duration: Option<Duration>) -> Result<()>;
    pub async fn destroy_dataflow(&self, id: Uuid) -> Result<()>;

    // Metrics
    pub async fn get_node_metrics(&self, dataflow_id: Uuid) -> Result<HashMap<NodeId, NodeMetrics>>;

    // Logs
    pub async fn subscribe_logs(&self, filter: LogFilter) -> Result<LogStream>;

    // Infrastructure
    pub async fn check_coordinator(&self) -> Result<bool>;
    pub async fn check_daemon(&self) -> Result<bool>;
    pub async fn start_daemon(&self) -> Result<()>;
}
```

### OTLP Receiver

Built-in gRPC server to receive traces:

```rust
pub struct OtlpReceiver {
    port: u16,  // Default: 4317
    storage: Arc<Storage>,
}

impl OtlpReceiver {
    pub async fn start(&self) -> Result<()>;
    // Implements opentelemetry-proto TraceService
}
```

---

## 7. Project Structure

```
dora-studio/
├── Cargo.toml                    # Workspace definition
├── PRD.md                        # This document
├── ARCHITECTURE.md               # Technical details
├── README.md                     # Quick start guide
│
├── dora-studio-shell/            # Binary: Main application
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs               # Entry point
│   │   ├── lib.rs                # SharedState
│   │   ├── app.rs                # Shell widget
│   │   └── widgets/              # Shell-specific widgets
│   └── resources/                # Fonts, icons
│
├── dora-studio-widgets/          # Library: Shared components
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── theme.rs              # Colors, fonts
│       ├── app_trait.rs          # DoraApp trait
│       ├── time_series_chart.rs
│       ├── dataflow_graph.rs
│       ├── log_table.rs
│       ├── span_timeline.rs
│       ├── yaml_editor.rs
│       └── ...
│
├── dora-studio-client/           # Library: Dora API client
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── client.rs             # DoraClient impl
│       ├── otlp_receiver.rs      # OTLP gRPC server
│       └── storage.rs            # DataFusion + Parquet storage
│
├── apps/
│   ├── dataflow-manager/         # App: Dataflow lifecycle
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       └── screen.rs
│   │
│   ├── yaml-editor/              # App: YAML + Graph
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── screen.rs
│   │       ├── editor.rs
│   │       └── graph_view.rs
│   │
│   ├── log-viewer/               # App: Log streaming
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       └── screen.rs
│   │
│   └── telemetry-dashboard/      # App: Metrics + Traces
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── screen.rs
│           ├── metrics_view.rs
│           ├── traces_view.rs
│           └── topics_view.rs
│
└── tests/                        # Integration tests
```

---

## 8. Development Phases

### Phase 1: Foundation
- [ ] Project scaffolding (Cargo workspace, dependencies)
- [ ] Shell with navigation (sidebar, app switching)
- [ ] Theme system (light/dark mode)
- [ ] DoraClient basic implementation (connect, list, start, stop)
- [ ] Dataflow Manager MVP (list, status, basic actions)

### Phase 2: Editor & Visualization
- [ ] YAML Editor with syntax highlighting
- [ ] Dataflow graph rendering (dagre layout)
- [ ] Live preview (edit YAML → update graph)
- [ ] Validation feedback integration

### Phase 3: Logging
- [ ] Log streaming subscription
- [ ] Filtering UI (level, node, search)
- [ ] Virtualized log table
- [ ] Export functionality

### Phase 4: Telemetry
- [ ] DataFusion + Parquet storage integration
- [ ] OTLP receiver implementation
- [ ] Time-series chart widget
- [ ] Metrics dashboard
- [ ] Trace waterfall view

### Phase 5: Polish
- [ ] Performance optimization
- [ ] Error handling and edge cases
- [ ] Documentation
- [ ] Release packaging

---

## 9. Success Metrics

| Metric | Target |
|--------|--------|
| Startup time | < 2 seconds |
| Metrics refresh latency | < 500ms |
| Log rendering (10K entries) | < 100ms |
| Graph rendering (50 nodes) | < 200ms |
| Memory usage (idle) | < 100 MB |
| Memory usage (active monitoring) | < 500 MB |

---

## 10. Dependencies

### Rust Crates (100% Rust - No C/C++ Dependencies)

| Crate | Version | Purpose |
|-------|---------|---------|
| makepad-widgets | git (custom fork) | GPU-accelerated UI |
| datafusion | 44+ | SQL query engine (pure Rust) |
| arrow | 53+ | Columnar memory format (pure Rust) |
| parquet | 53+ | Columnar file storage (pure Rust) |
| tokio | 1.x | Async runtime |
| tonic | 0.10+ | gRPC for OTLP |
| opentelemetry-proto | 0.4+ | OTLP message types |
| dora-core | 0.4.0 | Descriptor parsing |
| serde / serde_json | 1.x | Serialization |
| uuid | 1.x | UUID handling |

> **Note**: We deliberately avoid DuckDB (C++ core) to maintain a 100% Rust codebase. DataFusion + Arrow + Parquet provides equivalent functionality with pure Rust implementation.

### External References

- [Dora CLI source](../dora/binaries/cli/)
- [MoFA Studio architecture](../mofa-studio/ARCHITECTURE.md)
- [SigNoz](https://github.com/SigNoz/signoz) - Observability reference
- [Grafana best practices](https://grafana.com/docs/grafana/latest/visualizations/dashboards/build-dashboards/best-practices/)
- [AWS Operational Dashboards](https://aws.amazon.com/builders-library/building-dashboards-for-operational-visibility/)

---

## Appendix A: CLI Command Mapping

| CLI Command | Dora Studio Equivalent |
|-------------|----------------------|
| `dora up` | Dataflow Manager → Infrastructure panel → "Start" |
| `dora list` | Dataflow Manager → Main table |
| `dora start <yaml>` | Dataflow Manager → "Start" button → File picker |
| `dora stop <id>` | Dataflow Manager → Select row → "Stop" |
| `dora destroy` | Dataflow Manager → "Destroy All" |
| `dora logs -f` | Log Viewer → Auto-scroll enabled |
| `dora top` | Telemetry Dashboard → Metrics view |
| `dora topic hz` | Telemetry Dashboard → Topics view |
| `dora graph` | YAML Editor → Graph panel |
| `dora node list` | Dataflow Manager → Expand row |
| `dora system check` | Status bar (always visible) |

---

## Appendix B: Comparison with SigNoz

| Feature | SigNoz | Dora Studio |
|---------|--------|-------------|
| Metrics | PromQL queries | Node-centric (CPU/Mem/IO) |
| Traces | Distributed tracing | Dataflow-aware traces |
| Logs | ClickHouse-backed | Embedded DataFusion+Parquet |
| Dashboards | Custom dashboards | P2 (fixed layouts MVP) |
| Alerts | Threshold + anomaly | Future consideration |
| Storage | ClickHouse (external) | DataFusion+Parquet (embedded, pure Rust) |
| Deployment | Docker/K8s | Native desktop app |
| Use case | Generic observability | Dora-specific operations |

---

## Appendix C: UI/UX Design Principles

Based on industry best practices from Grafana, AWS, and SigNoz:

1. **5-second rule**: Critical information visible immediately upon opening
2. **RED method layout**: Rate left, Errors center, Duration right
3. **Hierarchical drill-down**: Overview → Dataflow → Node → Metric/Span
4. **Consistent time ranges**: All panels sync to same time window
5. **Dark mode first**: Match mofa-studio theme system
6. **Golden signals panel**: Top of dashboard, always visible
7. **Virtualized rendering**: Handle 100K+ logs efficiently
8. **Real-time updates**: Smooth transitions, no jarring refreshes
