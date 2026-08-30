---
sidebar_position: 5
---

# Observability

GlueSQL can emit structured spans and events for query execution when the optional `tracing`
feature is enabled. The feature is disabled by default, so applications that do not use
observability do not compile the instrumentation or its dependency.

GlueSQL does not install a global subscriber when used as a library. Applications can connect the
emitted data to `tracing-subscriber`, OpenTelemetry, or `tracing-flame`. The GlueSQL CLI installs a
formatted subscriber automatically when it is built with the feature.

## CLI logging

Install or build the CLI with tracing enabled:

```sh
cargo install gluesql --features tracing
```

Set `RUST_LOG` to select the required detail:

```sh
RUST_LOG=gluesql=info gluesql
RUST_LOG=gluesql=debug gluesql
RUST_LOG=gluesql=trace gluesql
```

The levels have the following intended scope:

| Level | Data |
| --- | --- |
| `info` | Total query execution time |
| `debug` | Parse, translate, plan, statement execution, and selected access path |
| `trace` | Transaction, primary storage, and enabled backend call boundaries |

The CLI reports span close events, including busy and idle durations.

Optional CLI exporter features build on the same instrumentation:

| Feature | Output |
| --- | --- |
| `tracing` | Formatted span close events on standard error |
| `tracing-flame` | Formatted events and folded stack data |
| `opentelemetry` | Formatted events and OTLP traces over HTTP/Protobuf |

`tracing-flame` and `opentelemetry` both enable `tracing` and can be enabled together.

### Try tracing locally

From the repository root, build the CLI with tracing enabled:

```sh
cargo build -p gluesql-cli --features tracing
```

Start the CLI with its default in-memory storage and trace-level logging:

```sh
RUST_LOG=gluesql=trace \
./target/debug/gluesql-cli
```

Run these statements at the `gluesql>` prompt:

```sql
CREATE TABLE Items (
    id INTEGER PRIMARY KEY,
    name TEXT
);

INSERT INTO Items VALUES
    (1, 'apple'),
    (2, 'banana'),
    (3, 'cherry');

SELECT * FROM Items WHERE id = 1;
SELECT * FROM Items;
```

The query with the primary-key predicate emits `gluesql.storage.fetch_data` with
`storage.type="gluesql_memory_storage::MemoryStorage"`. The query without a predicate uses a full
scan and emits `gluesql.storage.scan_data`. The same generic spans are available for every storage
used through `Glue`; only `storage.type` changes.

Tracing output is written to standard error. Redirect it to a file while keeping query results in
the terminal:

```sh
RUST_LOG=gluesql=trace \
./target/debug/gluesql-cli 2> ~/gluesql-trace.log
```

Follow the trace from another terminal:

```sh
tail -f ~/gluesql-trace.log
```

## Library logging

Enable GlueSQL instrumentation and add a subscriber in the application:

```toml
[dependencies]
gluesql = { version = "0.19", features = ["tracing"] }
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

Install the subscriber once, before executing queries:

```rust
use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};

let filter = EnvFilter::try_from_default_env()
    .unwrap_or_else(|_| EnvFilter::new("gluesql=info"));

tracing_subscriber::fmt()
    .with_env_filter(filter)
    .with_span_events(FmtSpan::CLOSE)
    .init();
```

Libraries should not install their own global subscriber. If the host application already uses
`tracing`, enabling GlueSQL's feature is sufficient; the existing subscriber receives GlueSQL
spans under the `gluesql` target.

## Span hierarchy

The initial instrumentation follows the query execution pipeline:

```text
gluesql.execute
├── gluesql.parse
├── gluesql.translate
├── gluesql.plan
│   └── gluesql.storage.plan
│       └── gluesql.storage.fetch_schema
└── gluesql.execute_statement
    ├── gluesql.storage.begin
    ├── gluesql.storage.fetch_data
    ├── gluesql.storage.scan_data
    ├── gluesql.storage.scan_indexed_data
    ├── gluesql.storage.commit
    └── gluesql.storage.rollback
```

Every generic `gluesql.storage.*` span records `storage.type` using the concrete Rust storage type.
New storage implementations therefore receive identifiable trait-boundary spans through
`gluesql-core/tracing` without adding storage-specific instrumentation. Storage-specific spans are
only needed for internal operations that are not visible at the trait boundary.

GlueSQL routes its storage trait calls through explicit Core tracing functions. The generic spans
cover every storage trait method used by the planner and executor:

| Trait | Spans |
| --- | --- |
| `Planner` | `plan` |
| `Store` | `fetch_schema`, `fetch_all_schemas`, `fetch_data`, `scan_data`, `fetch_referencings` |
| `StoreMut` | `insert_schema`, `delete_schema`, `append_data`, `insert_data`, `delete_data` |
| `Index` / `IndexMut` | `scan_indexed_data`, `create_index`, `drop_index` |
| `AlterTable` | `rename_schema`, `rename_column`, `add_column`, `drop_column` |
| `Metadata` | `scan_table_meta` |
| `CustomFunction` / `CustomFunctionMut` | `fetch_function`, `fetch_all_functions`, `insert_function`, `delete_function` |
| `Transaction` | `begin`, `commit`, `rollback` |

Each name is prefixed with `gluesql.storage.`. Data mutation spans also record `row_count`, and
`gluesql.storage.begin` records `autocommit`.

### Adding tracing support to a storage

A storage implementation does not need a derive macro, wrapper type, or tracing code to receive
the generic spans above. Implement the relevant GlueSQL storage traits as usual. When a query runs
through `Glue` with `gluesql-core/tracing` enabled, Core records the trait-call boundary and the
concrete Rust type in `storage.type`.

For wrapper or composite storages, `storage.type` identifies the outer type passed to `Glue`. Add
an internal dispatch span only when the selected inner backend must also be visible.

Calls made directly to a storage trait method outside the GlueSQL planner and executor do not pass
through these Core boundaries. Add an optional `tracing` feature to the storage crate only when it
needs spans for those direct calls or for backend-specific work below the trait boundary. Storage
crates should not install a subscriber; the CLI or host application owns subscriber configuration.

Use backend-specific spans only when they add information that the generic boundary cannot expose.
For example, a storage can instrument iterator consumption, serialization, network requests, or
transaction flushes. Use the `gluesql.<storage>.<operation>` naming pattern and avoid recording SQL
text, keys, row values, or one span per row.

Access-path events use one of these stable values:

```text
primary_key
secondary_index
full_scan
```

`scan_data` and `scan_indexed_data` return lazy iterators. Their generic storage spans measure
iterator creation, not the complete scan; `gluesql.execute_statement` includes subsequent iterator
consumption.

### Backend-specific spans

Generic spans are the portable observability contract. A storage may optionally add child spans
for work below its trait boundary. RedbStorage is the first reference implementation: when its
`tracing` feature is enabled, it emits `gluesql.redb.*` spans for database operations and
`gluesql.redb.scan_rows{row_count=...}` for lazy iterator consumption. These Redb spans are an
example, not a requirement for other storages.

For `gluesql.redb.scan_rows`, the busy duration measures row reads and deserialization, the idle
duration covers time spent by the consumer between reads, and `row_count` records the number of
yielded items.

When `tracing` is enabled, eager execution boundaries also expose the number of items retained for
the operation. These fields can be aligned with RSS samples to identify which operation overlaps a
memory increase:

| Span | Field | Meaning |
| --- | --- | --- |
| `gluesql.validate.unique` | `scanned_rows` | Existing rows checked for a unique constraint |
| `gluesql.query.hash_join.build` | `buffered_rows` | Rows retained on the hash-build side |
| `gluesql.query.aggregate` | `buffered_groups` | Aggregate groups retained in memory |
| `gluesql.query.order_by` | `buffered_rows` | Rows collected before sorting |
| `gluesql.query.distinct` | `buffered_rows` | Rows collected before duplicate filtering |
| `gluesql.mutation.collect` | `buffered_rows` | UPDATE rows or DELETE keys collected before mutation |
| `gluesql.insert.collect` | `buffered_rows` | VALUES or query rows collected before insertion |
| `gluesql.result.materialize` | `buffered_rows` | SELECT rows collected for the returned payload |

The counts describe logical items rather than allocated bytes. Use the RSS counter for process
memory and these spans to locate the corresponding execution boundary.

## Resource benchmark profiles

A resource benchmark groups one SQL workload, query spans, and resource measurements under a
`gluesql.benchmark.run` span. The field names and Firefox Profiler representation below are shared
across storage implementations. The current runnable reference is in the RedbStorage crate; it is
not the definition of the generic storage tracing contract.

Execute the SQL file once and use its filename without the extension as `benchmark.name`. The
closing benchmark span uses these storage-independent fields:

| Field | Meaning |
| --- | --- |
| `process.memory.peak_bytes` | Peak resident set size of the benchmark process |
| `gluesql.database.size_bytes` | Storage-owned persistent data size after the workload, when measurable |
| `process.executable.size_bytes` | Benchmark executable file size |

At `debug` or `trace` level, a benchmark can also emit current RSS samples under the run span:

```text
gluesql.benchmark.memory_sample elapsed_ms=20 rss_bytes=18874368
```

### Run the RedbStorage reference

The reference example is available only when its `tracing` feature is enabled.

Run a SQL workload against a new Redb database path:

```sh
RUST_LOG=gluesql=trace \
cargo run --release \
  -p gluesql-redb-storage \
  --example resource_benchmark \
  --features tracing \
  -- /tmp/gluesql-benchmark.redb \
  storages/redb-storage/examples/resource_benchmark.sql
```

The default interval is 10 milliseconds. Set `GLUESQL_MEMORY_SAMPLE_MS` to use a different
positive interval:

```sh
GLUESQL_MEMORY_SAMPLE_MS=50 \
RUST_LOG=gluesql=debug \
cargo run --release \
  -p gluesql-redb-storage \
  --example resource_benchmark \
  --features tracing \
  -- /tmp/gluesql-benchmark.redb \
  storages/redb-storage/examples/resource_benchmark.sql
```

The example emits tracing data only; it does not select a graphing or storage format. Subscribers
can consume `elapsed_ms` and `rss_bytes` to produce a step chart and align it with the existing
query spans. At `info` level the sampler is not started, so only the final resource fields are
recorded. RSS samples include the memory and scheduling overhead of the sampler thread itself.
Current RSS sampling is supported on macOS and Linux.

### Firefox Profiler output

The optional `firefox-profile` feature keeps the formatted standard-error output and also writes
GlueSQL spans, events, and RSS samples directly in the Firefox Profiler processed-profile JSON
format. It uses a Rust library and does not require `protoc` or a separate trace converter.

Generate a profile from one workload:

```sh
GLUESQL_FIREFOX_PROFILE_PATH=~/gluesql-benchmark-profile.json \
GLUESQL_MEMORY_SAMPLE_MS=10 \
RUST_LOG=gluesql=debug \
cargo run --release \
  -p gluesql-redb-storage \
  --example resource_benchmark \
  --features firefox-profile \
  -- /tmp/gluesql-benchmark.redb \
  storages/redb-storage/examples/resource_benchmark.sql
```

`GLUESQL_FIREFOX_PROFILE_PATH` defaults to `gluesql-benchmark-profile.json`. Open
[Firefox Profiler](https://profiler.firefox.com/), select **Load a profile from file**, and choose
the generated JSON file. Select the `process_rss` counter track to inspect RSS over time. GlueSQL
spans and events appear as interval and instant markers on the same timeline. The profile remains
local unless it is explicitly uploaded or shared; GlueSQL does not require a visualization
service.

Peak RSS is a process-lifetime high-water mark, so run each workload in a separate process and use
a new database path when comparing results. Use the same build profile and target platform for
executable-size comparisons. Peak RSS measurement is currently supported on Unix platforms.

### Adding a benchmark for another storage

The Redb example is currently the reference implementation rather than a shared benchmark harness.
To add the same measurements to another storage crate, use these files as the starting point:

```text
storages/redb-storage/examples/resource_benchmark.rs
storages/redb-storage/examples/resource_benchmark/firefox_profile.rs
storages/redb-storage/examples/resource_benchmark.sql
storages/redb-storage/Cargo.toml
```

First add a `tracing` feature to the target storage crate. It must enable the optional `tracing`
dependency and `gluesql-core/tracing`. Register the example with `required-features = ["tracing"]`
so its tracing and RSS dependencies are not compiled for normal storage users:

```toml
[features]
tracing = ["dep:tracing", "gluesql-core/tracing"]

[dependencies]
tracing = { version = "0.1", optional = true }

[dev-dependencies]
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }

[target.'cfg(unix)'.dev-dependencies]
libc = "0.2"

[[example]]
name = "resource_benchmark"
required-features = ["tracing"]
```

Copy `resource_benchmark.rs` and adapt only the storage-specific parts:

1. Import and construct the target storage instead of `RedbStorage`.
2. Change `benchmark.storage` from `"redb"` to a stable storage name.
3. Adjust the command-line arguments required to create or connect to the storage.
4. Record the storage size only when it can be measured locally and consistently.
5. Add a representative SQL file and pass the complete document to `Glue::execute(&sql)`.

Keep the following names and behavior unchanged so profiles remain comparable across storage
implementations:

```text
gluesql.benchmark.run
gluesql.benchmark.memory_sample
benchmark.name
benchmark.storage
process.memory.peak_bytes
gluesql.database.size_bytes
process.executable.size_bytes
GLUESQL_MEMORY_SAMPLE_MS
```

Use the storage type to decide how `gluesql.database.size_bytes` is populated:

| Storage type | Size measurement |
| --- | --- |
| Single local file | Read the database file metadata after the workload finishes |
| Local directory | Sum only the files owned by that database after pending writes are flushed |
| In-memory | Leave `gluesql.database.size_bytes` empty |
| Remote service | Leave the local field empty; report a server-side metric separately if available |

The generic `gluesql.storage.*` spans are available through `gluesql-core/tracing` without adding
backend-specific instrumentation. Add internal spans such as `gluesql.<storage>.*` only when the
storage implementation has a concrete diagnostic boundary to expose. Avoid per-row spans; for a
lazy iterator, use one span covering iterator consumption and record the final row count.

Firefox Profiler support is optional. To include it, copy the `firefox_profile.rs` support module
without changing its marker and counter names, retain `GLUESQL_FIREFOX_PROFILE_PATH` as the output
setting, and add these optional dependencies:

```toml
[features]
firefox-profile = [
  "tracing",
  "dep:fxprof-processed-profile",
  "dep:serde_json",
]

[dependencies]
fxprof-processed-profile = { version = "0.8.1", optional = true }
serde_json = { version = "1", optional = true }
```

Validate the new example in both modes:

```sh
cargo run --release \
  -p <storage-package> \
  --example resource_benchmark \
  --features tracing \
  -- <storage-arguments> <workload.sql>

GLUESQL_FIREFOX_PROFILE_PATH=~/gluesql-benchmark-profile.json \
RUST_LOG=gluesql=debug \
cargo run --release \
  -p <storage-package> \
  --example resource_benchmark \
  --features firefox-profile \
  -- <storage-arguments> <workload.sql>
```

Confirm that the formatted trace contains `gluesql.benchmark.run`, the Firefox profile contains
GlueSQL markers and a `process_rss` counter, and a tracing-disabled build of the storage remains
unchanged. Run each comparison in a fresh process with an equivalent workload and build profile.

## OpenTelemetry

OpenTelemetry integration belongs to the host application rather than `gluesql-core`. Add the
OpenTelemetry crates that match the application's chosen transport:

### CLI OTLP export

Build the CLI with the OpenTelemetry exporter:

```sh
cargo build -p gluesql-cli --features opentelemetry
```

Set the standard OpenTelemetry environment variables and run the CLI:

```sh
OTEL_SERVICE_NAME=gluesql-cli \
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318 \
OTEL_EXPORTER_OTLP_PROTOCOL=http/protobuf \
RUST_LOG=gluesql=trace \
./target/debug/gluesql-cli
```

The CLI exports completed spans to `/v1/traces` and flushes pending batches when it exits. The
configured endpoint must accept OTLP over HTTP/Protobuf. A collector can forward those traces to
Jaeger, Grafana Tempo, or another compatible backend.

### Application integration

```sh
cargo add tracing-opentelemetry opentelemetry opentelemetry_sdk
cargo add opentelemetry-otlp --features grpc-tonic
```

Create an OTLP exporter and attach the OpenTelemetry layer to the application's subscriber:

```rust
use {
    opentelemetry::trace::TracerProvider as _,
    opentelemetry_sdk::trace::SdkTracerProvider,
    tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt},
};

let exporter = opentelemetry_otlp::SpanExporter::builder()
    .with_tonic()
    .build()?;
let provider = SdkTracerProvider::builder()
    .with_batch_exporter(exporter)
    .build();
let tracer = provider.tracer("gluesql");
let filter = EnvFilter::try_from_default_env()
    .unwrap_or_else(|_| EnvFilter::new("gluesql=info"));

tracing_subscriber::registry()
    .with(filter)
    .with(tracing_opentelemetry::layer().with_tracer(tracer))
    .init();

// Run GlueSQL queries here.

provider.shutdown()?;
```

Configure the collector endpoint with the standard OpenTelemetry environment variables:

```sh
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
```

The collector can forward traces to Jaeger, Grafana Tempo, or another OTLP-compatible backend. See
the
[`opentelemetry-otlp` exporter documentation](https://docs.rs/opentelemetry-otlp/latest/opentelemetry_otlp/)
and
[`tracing-opentelemetry` layer documentation](https://docs.rs/tracing-opentelemetry/latest/tracing_opentelemetry/)
for transport and SDK-specific configuration.

## Flamegraphs

`tracing-flame` can convert the same span hierarchy into folded stack data:

### CLI flamegraph

Build the CLI with the flame exporter:

```sh
cargo build -p gluesql-cli --features tracing-flame
```

Run a workload and choose the folded output path with `GLUESQL_FLAMEGRAPH_PATH`. The default path
is `tracing.folded`.

```sh
GLUESQL_FLAMEGRAPH_PATH=~/gluesql.folded \
RUST_LOG=gluesql=trace \
./target/debug/gluesql-cli
```

After exiting the CLI, generate an SVG with Inferno:

```sh
cargo install inferno
```

```sh
inferno-flamegraph < ~/gluesql.folded > ~/gluesql.svg
```

The CLI keeps empty samples out of the folded output so time waiting at the interactive prompt
does not dominate the graph.

### Application integration

```rust
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

let (flame_layer, _guard) = tracing_flame::FlameLayer::with_file("tracing.folded")?;
let filter = EnvFilter::try_from_default_env()
    .unwrap_or_else(|_| EnvFilter::new("gluesql=info"));

tracing_subscriber::registry()
    .with(filter)
    .with(flame_layer)
    .init();
```

Keep the returned guard alive until tracing has finished so buffered output is flushed. Generate
an SVG with Inferno:

```sh
inferno-flamegraph < tracing.folded > tracing.svg
```

`tracing-flame` measures elapsed time between instrumented span events; it is not a sampling CPU
profiler. Use `perf` or `cargo-flamegraph` when function-level CPU samples are required.

## Data handling

GlueSQL does not record the following values in its default instrumentation:

- SQL source text
- Bound parameters
- Row contents
- Full error messages

The default fields are limited to stable execution metadata such as the access path and
transaction mode. Applications that add their own fields or layers are responsible for applying
their data retention and access-control policies.
