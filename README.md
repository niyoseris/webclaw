# 🦀 claWasm

**claWasm** - A self-evolving AI assistant that runs 100% in your browser via WebAssembly.

Inspired by [ZeroClaw](https://github.com/zero-claw) and [OpenClaw](https://github.com/open-claw), claWasm brings browser-native AI with the ability to create its own tools on-the-fly, conduct research, and generate PDFs with images - all while keeping your data private in the browser.

## ✨ Features

### Core
- 🏎️ **Browser-Native**: Runs entirely in WebAssembly - no server required
- 🔒 **100% Private**: API keys stay in your browser, conversations never leave your device
- ⚡ **Fast**: ~200KB WASM binary, instant startup
- 🌍 **Multi-Provider**: OpenAI, Anthropic, Ollama (Local & Cloud), Groq, Together AI

### Self-Evolving Tools 🧬
- **`create_tool`**: AI creates its own JavaScript tools on-the-fly
- **`list_custom_tools`**: View all created tools
- **`delete_tool`**: Remove tools when no longer needed
- Tools persist in localStorage and work immediately

### Research & Content
- **`research`**: Deep research with web search, URL fetching, and Reddit discussions
- **`image_search`**: Find images for reports and content
- **`create_pdf`**: Generate PDFs with embedded images
- **`web_search`**: DuckDuckGo search via proxy
- **`fetch_url`**: Extract content from any URL
- **`save_note` / `read_notes`**: Persistent note-taking

## 🚀 Quick Start

### One-Command Start

```bash
git clone https://github.com/niyoseris/claWasm.git
cd claWasm
./start.sh
```

Open http://localhost:5001 in your browser.

### Manual Build

### Prerequisites

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# WASM target
rustup target add wasm32-unknown-unknown

# wasm-bindgen
cargo install wasm-bindgen-cli
```

### Build & Run

```bash
# Clone
git clone https://github.com/niyoseris/claWasm.git
cd claWasm

# Build WASM
cargo build --target wasm32-unknown-unknown --release

# Generate JS bindings
wasm-bindgen --out-dir web/pkg --target web target/wasm32-unknown-unknown/release/clawasm.wasm

# Start proxy server (for CORS bypass)
cargo run --bin proxy --features proxy &

# Serve frontend
cd web && python3 -m http.server 5001
```

Open http://localhost:5001 in your browser.

## 🛠️ Tools Available

| Tool | Description |
|------|-------------|
| `web_search` | Search the web via DuckDuckGo |
| `reddit_search` | Search Reddit for discussions |
| `image_search` | Find images on the web |
| `research` | Deep research on any topic |
| `fetch_url` | Extract content from URLs |
| `create_pdf` | Generate PDFs with images |
| `download_file` | Download generated files |
| `save_note` / `read_notes` | Note management |
| `create_tool` | Create custom JavaScript tools |
| `list_custom_tools` | List custom tools |
| `delete_tool` | Delete custom tools |
| `get_current_time` | Current date/time |
| `calculate` | Math calculations |

## 🤖 Self-Evolving Tools Example

Ask the AI to create a tool:
```
"Create a tool called 'word_counter' that counts words in text"
```

The AI will generate and save the tool:
```json
{
  "name": "create_tool",
  "arguments": {
    "name": "word_counter",
    "description": "Count words in text",
    "parameters_schema": {"type": "object", "properties": {"text": {"type": "string"}}},
    "code": "return args.text.split(' ').filter(w => w.length > 0).length + ' words';"
  }
}
```

Now the AI can use `word_counter` anytime!

## 📊 PDF with Images

```json
{
  "name": "create_pdf",
  "arguments": {
    "title": "European Castles",
    "content": "The most beautiful castles in Europe...",
    "images": [
      {"url": "https://example.com/castle1.jpg", "caption": "Neuschwanstein"},
      {"url": "https://example.com/castle2.jpg", "caption": "Edinburgh Castle"}
    ]
  }
}
```

## 🔌 Supported Providers

| Provider | Default Model | Notes |
|----------|--------------|-------|
| OpenAI | gpt-4o-mini | Requires API key |
| Anthropic | claude-3-haiku-20240307 | Requires API key |
| Ollama (Local) | llama3.2 | Free, runs locally |
| Ollama Cloud | glm-5 | Requires API key |
| Groq | llama-3.1-70b-versatile | Requires API key |
| Together | meta-llama/Llama-3-70b-chat-hf | Requires API key |

## 🏗️ Architecture

```
claWasm/
├── src/
│   ├── lib.rs        # WASM bindings, tool parsing
│   ├── config.rs     # Configuration
│   ├── chat.rs       # Message handling
│   ├── providers.rs  # AI provider implementations
│   ├── tools.rs      # Tool definitions & execution
│   ├── memory.rs     # Memory system
│   └── security.rs   # Security manager
├── src/bin/
│   └── proxy.rs      # CORS proxy server
├── web/
│   ├── index.html    # Web UI with settings
│   └── pkg/          # Generated WASM/JS
└── Cargo.toml
```

## 📡 Proxy Server

For CORS bypass (Ollama Cloud, web search, etc.):

```bash
cargo run --bin proxy --features proxy
```

Runs on http://localhost:3000

## 🔌 JavaScript API

```javascript
import init, { ClaWasm } from './pkg/clawasm.js';

await init();
const assistant = new ClaWasm();

// Configure
assistant.setProvider('ollama_cloud', 'your-api-key');
assistant.setModel('glm-5:cloud');

// Chat
const response = await assistant.chat('Research AI trends and create a PDF');

// Tools
const tools = ClaWasm.getTools();
const result = await ClaWasm.executeTool('calculate', '{"expression": "2+2"}');

// History
const history = JSON.parse(assistant.getHistory());
assistant.clearHistory();
```

## 🆚 vs ZeroClaw

| Feature | claWasm | ZeroClaw |
|---------|---------|----------|
| Runtime | WASM (browser) | Native binary |
| Size | ~200KB | ~3.4MB |
| Server Required | No | Yes |
| Self-Evolving Tools | ✅ JavaScript | ❌ |
| PDF with Images | ✅ | ❌ |
| Memory System | Basic | Full vector search |
| Mattermost | ❌ | ✅ |

## 📄 License

MIT

## 🙏 Credits

Inspired by [ZeroClaw](https://github.com/zeroclaw-labs/zeroclaw)
