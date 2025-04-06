# 🛰️ Rust Example: Ping Monitor

This project is part of a series of example applications written in Rust to showcase practical patterns in building
async, concurrent systems with structured state management and a REST API.

The **Ping Monitor** demonstrates how to:

- Spawn and manage background tasks (e.g., ICMP ping probes)
- Aggregate results in a global application state
- Expose control and monitoring interfaces via a RESTful API

---

## 🧭 Purpose

This repository serves as a hands-on example for learning:

- Asynchronous task orchestration in Rust using `tokio`
- Shared state management using `Arc` and `Mutex`/`RwLock`
- Inter-thread communication using `mpsc` and `broadcast` channels.
- REST API implementation using `axum`
- Handling and reporting real-time data (e.g., latency, timeouts)

It is not a production-grade tool but rather an educational resource for developers interested in idiomatic async Rust.

---

## 🔧 Features

- Create and manage ICMP ping targets dynamically
- Background probes run with configurable intervals
- Store and retrieve ping results per target
- REST API to manage and inspect probe states

---

## 🚀 Getting Started

### Prerequisites

- Rust (latest stable recommended)
- `cargo` package manager
- `curl` (for testing endpoints)

### Run the Server

```bash
cargo run
```

The server will start on `http://localhost:3000`.

---

## 🧪 API Usage Examples

### List all targets

```bash
curl -v http://localhost:3000/targets
```

### Create a new probe target

```bash
curl -H 'content-type: application/json' \
     -d '{"addr":"8.8.8.8"}' \
     http://localhost:3000/targets
```

### Get probe results for a specific target

```bash
curl -v http://localhost:3000/targets/<uuid>/results
```

Replace `<uuid>` with the actual ID returned from the create call.

---

## 🤝 Contribution

Contributions are very welcome! Here's how to get started:

1. Fork this repository
2. Create a feature branch: `git checkout -b my-feature`
3. Commit your changes: `git commit -am 'Add new feature'`
4. Push to your branch: `git push origin my-feature`
5. Open a Pull Request

Please follow idiomatic Rust style (`rustfmt`, Clippy) and try to keep PRs focused.

---

## 📄 License

This project is open source and available under the MIT License.

---

## 🙌 Acknowledgements

Inspired by real-world monitoring systems and written to demonstrate clean Rust code structure and async patterns.

```