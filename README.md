# 🪐 Jupiter

> Minimalistic AI-driven matchmaking where agents talk so humans don't have to (until it matters).

Jupiter is a modern matchmaking platform that leverages LLMs to simulate agent-to-agent interactions. Instead of swiping, you chat with your personal AI companion. Your agent learns your personality, values, and preferences, then negotiates with other agents to find genuinely compatible matches.

---

## ✨ Features

- **Personal AI Agent**: A conversational companion that builds your profile over time through natural dialogue.
- **Agent Matchmaking**: Your agent evaluates potential matches by talking to their agents, analyzing compatibility scores, and filtering out deal-breakers.
- **Privacy-First**: Your raw chat history remains private; only the synthesized "Agent Knowledge" is shared with potential matches.
- **Real-time DMs**: Skip the small talk. Once agents confirm a match, jump into a direct conversation with a high compatibility foundation.

## 🚀 Quick Start

### Prerequisites
- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) (for the frontend)
- An OpenAI-compatible LLM endpoint (Ollama, vLLM, or OpenAI)

### Backend
```bash
# Set up environment
cp .env.example .env

# Run migrations & start
cargo run
```

### Frontend
```bash
cd client
npm install
npm run dev
```

## 🛠️ Tech Stack

- **Backend**: Rust, Actix-web, SQLite (Rusqlite)
- **Frontend**: React, Vite, TypeScript, Lucide Icons
- **AI**: Reqwest + Serde for OpenAI-compatible API integration

---

## 📄 License
MIT © 2026 Jupiter Team
