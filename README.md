# YouTube Video Slide Extractor

A robust Rust-based CLI tool that automatically extracts unique slides from YouTube presentation videos, performs OCR to retrieve text, and generates a structured Markdown report including slide images and a visual timeline.

## 🚀 Features

- **Automated Pipeline**: URL validation → Video download → Frame extraction → Perceptual deduplication → **AI Verification** → OCR → Markdown generation.
- **Intelligent Deduplication**: Uses perceptual hashing and Hamming distance to group similar frames and extract only unique slides.
- **AI-Powered Verification**: Integrates with OpenAI-compatible Vision APIs (e.g., LM Studio, GPT-4o) to filter out non-slide content like speaker views or audience shots.
- **Multi-Language OCR**: Powered by Tesseract, allowing text extraction in multiple languages (e.g., English, German, Spanish).
- **Session Recovery**: Stage-based checkpointing allows resuming interrupted extractions from where they left off.
- **Professional Dual Reports**:
    - `report.md`: Full detailed report with all slides and AI warnings for potential non-slides.
    - `report_cleaned.md`: A concise report containing only the frames verified as slides.
- **Interactive Cleanup**: Prompts to delete tagged non-presentation slide images at the end of the process to save storage.
- **Real-Time Progress**: Visual feedback using multi-progress bars for every stage of the process.
- **Resource Management**: Automatically cleans up large temporary video and frame files after processing.
- **Smart Caching**: Caches downloaded videos and extracted audio to `/tmp/yt-sl-cache` to skip redundant steps in future runs, saving bandwidth and processing time.

## 🛠 Prerequisites

This tool relies on several external dependencies that must be available in your system `PATH`:

1.  **Rust**: [Install Rust](https://rustup.rs/) (1.70+)
2.  **yt-dlp**: Required for downloading YouTube videos.
    - `brew install yt-dlp` (macOS)
    - `sudo wget https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -O /usr/local/bin/yt-dlp && sudo chmod a+rx /usr/local/bin/yt-dlp` (Linux)
3.  **FFmpeg**: Required for frame extraction.
    - `brew install ffmpeg` (macOS)
    - `sudo apt install ffmpeg` (Linux)
4.  **Tesseract OCR**: Required for text extraction.
    - `brew install tesseract` (macOS)
    - `sudo apt install tesseract-ocr` (Linux)
    - *Note: Ensure you have the necessary language data files installed (e.g., `tessdata/eng.traineddata`).*

## 📦 Installation

### Option 1: Quick Install (Recommended)

Run the following command in your terminal:

```bash
curl -sSL https://raw.githubusercontent.com/ragavsathish/yt-sl/main/install.sh | bash
```

### Option 2: Build from Source

Clone the repository and build the project using Cargo:

```bash
git clone https://github.com/ragavsathish/yt-sl.git
cd yt-sl
cargo build --release
```

The binary will be available at `./target/release/yt-sl-extractor`.

## 📖 Usage

### Using the installed binary

```bash
yt-sl-extractor --url "https://www.youtube.com/watch?v=UF4krOvbeaw" --output "./output"
```

### Using Cargo

```bash
cargo run -p yt-sl-extractor -- --url "https://www.youtube.com/watch?v=UF4krOvbeaw" --output "./output"
```

### Advanced Options

```text
Options:
  -u, --url <URL>               The YouTube video URL to process
  -i, --interval <SECONDS>      Frame extraction interval (default: 5.0, max: 60.0)
  -t, --threshold <THRESHOLD>   Slide similarity threshold (0.0 - 1.0, default: 0.85)
  -o, --output <DIR>            Output directory (default: ".")
  -l, --languages <LANGS>       OCR languages, comma-separated (default: "eng")
  -s, --timestamps              Include timestamps in the output filename
  -m, --memory-threshold <MB>   Memory threshold for frame processing (default: 500)
      --llm-verify              Enable AI-powered slide verification
      --llm-api-base <URL>      OpenAI-compatible API base (default: http://localhost:1234/v1)
      --llm-api-key <KEY>       API key (optional for local servers)
      --llm-model <NAME>        Vision model name (default: qwen/qwen3-vl-8b)
  -h, --help                    Print help
  -V, --version                 Print version
```

### Example: AI-Verified Extraction with LM Studio

```bash
yt-sl-extractor --url "https://www.youtube.com/watch?v=..." \
  --llm-verify \
  --llm-api-base "http://localhost:1234/v1" \
  --llm-model "qwen/qwen3-vl-8b"
```

### Example: High-Precision Extraction with Multiple Languages

```bash
cargo run -p yt-sl-extractor -- \
  --url "https://www.youtube.com/watch?v=..." \
  --interval 2.0 \
  --threshold 0.95 \
  --languages eng,spa \
  --output "./lecture_notes"
```

## 🧠 AI Slide Verification

The tool supports an optional AI verification step to distinguish between actual presentation slides and other content (like the speaker's face or audience shots). This uses any **OpenAI-compatible Vision API**.

For a completely private and local experience, we recommend using **[LM Studio](https://lmstudio.ai/)** with a vision-capable model like `qwen/qwen3-vl-8b` or `llava`.

1. Start the LM Studio Local Server (usually at `http://localhost:1234`).
2. Run the extractor with the `--llm-verify` flag.
3. The tool will flag potential non-slides in the final report and offer to delete the corresponding images to keep your workspace clean.

## 📂 Project Structure

- `poc/src/contexts/video`: YouTube interaction and downloading.
- `poc/src/contexts/frame`: FFmpeg integration and perceptual hashing.
- `poc/src/contexts/dedup`: Similarity logic and slide selection.
- `poc/src/contexts/ocr`: Tesseract integration and confidence parsing.
- `poc/src/contexts/document`: Markdown report generation with Tera templates.
- `poc/src/contexts/session`: Orchestration logic and state persistence.

## 🧪 Testing

The project includes a comprehensive test suite with 230+ tests, covering unit logic, integration, and end-to-end pipelines.

### Run All Tests
```bash
cargo test -p yt-sl-extractor
```

### Run Unit Tests Only
To run fast unit tests (skipping slow integration tests):
```bash
cargo test --lib --bins
```

### Run End-to-End (E2E) Tests
These tests run the full pipeline and require external dependencies (yt-dlp, ffmpeg, tesseract) to be installed.
```bash
# Run the full pipeline test
cargo test --test e2e_pipeline

# Run LLM integration test (requires API keys/server)
cargo test --test e2e_llm
```

## 📝 License

This project is licensed under the MIT License.
