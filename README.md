# Dora Studio

> A GPU-accelerated native desktop dashboard for the Dora dataflow framework
>
> **100% Rust implementation** - No C/C++ dependencies

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org)

Dora Studio provides a unified visual interface for managing, monitoring, and debugging Dora dataflows. Built with [Makepad](https://github.com/makepad/makepad) for GPU-accelerated rendering and following the architectural patterns established by [MoFA Studio](../mofa-studio/).

## Vision

Replace command-line workflows with an intuitive dashboard that supports the full dataflow lifecycle:

| Capability | CLI Equivalent | Dora Studio |
|------------|---------------|-------------|
| Dataflow management | `dora list/start/stop` | Visual status, one-click actions |
| Graph visualization | `dora graph` | Live editing, interactive inspection |
| Log analysis | `dora logs -f` | Filtering, search, aggregation |
| Performance monitoring | `dora top` | Time-series charts, historical trends |
| Trace analysis | External Jaeger | Built-in, correlated with metrics |

## Mini-Apps

Dora Studio is composed of four mini-apps, each focusing on a specific workflow:

### 1. Dataflow Manager
Lifecycle management for dataflows - start, stop, monitor status, view node metrics.

### 2. YAML Editor + Graph Visualizer
Edit dataflow YAML with live graph preview, validation feedback, and node inspection.

### 3. Log Viewer
Real-time log streaming with filtering by dataflow, node, level, and text search.

### 4. Telemetry Dashboard
Full observability with metrics charts, distributed traces, and topic statistics.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        DORA STUDIO                              │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────┐ │
│  │ Dataflow │  │  YAML    │  │   Log    │  │    Telemetry     │ │
│  │ Manager  │  │  Editor  │  │  Viewer  │  │    Dashboard     │ │
│  └──────────┘  └──────────┘  └──────────┘  └──────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                     SHELL + SHARED WIDGETS                      │
├─────────────────────────────────────────────────────────────────┤
│         DORA CLIENT          │      EMBEDDED DB (DataFusion)    │
└──────────────────────────────┴──────────────────────────────────┘
                    │
                    ▼
          Dora Coordinator + Daemon
```

Key design decisions:
- **100% Rust**: Pure Rust implementation with no C/C++ dependencies
- **Self-contained**: Embedded DataFusion + Parquet storage, built-in OTLP receiver (no external dependencies)
- **Plugin system**: Apps implement `DoraApp` trait (following mofa-studio patterns)
- **Dark mode first**: GPU-accelerated theme with smooth transitions

## Status

**Planning** - See [PRD.md](PRD.md) for detailed requirements.

## Documentation

- **[PRD.md](PRD.md)** - Product Requirements Document (comprehensive)
- **[../dora/CLAUDE.md](../dora/CLAUDE.md)** - Dora framework architecture
- **[../mofa-studio/ARCHITECTURE.md](../mofa-studio/ARCHITECTURE.md)** - UI patterns reference

## Technology Stack (100% Rust)

All dependencies are pure Rust with no C/C++ components:

- **[Rust](https://www.rust-lang.org/)** - Systems programming language
- **[Makepad](https://github.com/makepad/makepad)** - GPU-accelerated UI framework (pure Rust)
- **[Apache Arrow DataFusion](https://github.com/apache/datafusion)** - SQL query engine (pure Rust)
- **[Apache Arrow](https://arrow.apache.org/)** - Columnar memory format (pure Rust)
- **[Parquet](https://github.com/apache/arrow-rs/tree/master/parquet)** - Columnar file storage (pure Rust)
- **[Tokio](https://tokio.rs/)** - Async runtime (pure Rust)
- **[Tonic](https://github.com/hyperium/tonic)** - gRPC for OTLP receiver (pure Rust)

> **Note**: We deliberately avoid DuckDB (which has a C++ core) to maintain a 100% Rust codebase.

## Related Projects

- **[Dora](../dora/)** - The dataflow framework this dashboard manages
- **[MoFA Studio](../mofa-studio/)** - Reference implementation for Makepad app architecture
- **[SigNoz](https://github.com/SigNoz/signoz)** - Inspiration for observability features

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.
