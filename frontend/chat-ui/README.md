# Ninja Gekko – Talk to Gordon UI

A modern conversational control surface for Ninja Gekko inspired by ChatGPT, Claude, and LobeChat.

## Getting Started

```bash
pnpm install
pnpm dev --filter ninja-gekko-chat-ui
```

The development server proxies all `/api` requests to `http://localhost:8787`, which is served by the Rust Axum layer (`src/web.rs`).

## Features

- Multi-panel layout (conversation, persona tuning, diagnostics, live intel)
- Persona sliders to toggle Gordon's tone/style/mood
- Action buttons for pausing trades, summoning swarms, requesting deep research
- File upload affordance for future MCP knowledge ingestion
- React Query polling for account snapshots, news, and diagnostics

## Build

```bash
pnpm build --filter ninja-gekko-chat-ui
```

The build output lives under `frontend/chat-ui/dist` and can be served by the Rust gateway or a static CDN.
