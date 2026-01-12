# YouTube Video Slide Extractor

A robust Rust-based CLI tool that automatically extracts unique slides from YouTube presentation videos, performs OCR to retrieve text, and generates a structured Markdown report including slide images and a visual timeline.

## 🚀 Features

- **Automated Pipeline**: URL validation → Video download → Frame extraction → Perceptual deduplication → OCR → Markdown generation.
- **Intelligent Deduplication**: Uses perceptual hashing and Hamming distance to group similar frames and extract only unique slides.
- **Multi-Language OCR**: Powered by Tesseract, allowing text extraction in multiple languages (e.g., English, German, Spanish).
- **Session Recovery**: Stage-based checkpointing allows resuming interrupted extractions from where they left off.
- **Professional Reports**: Generates a `report.md` with video metadata, slide images, OCR text, and a **Mermaid.js timeline diagram**.
- **Real-Time Progress**: Visual feedback using multi-progress bars for every stage of the process.
- **Resource Management**: Automatically cleans up large temporary video and frame files after processing.

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

Clone the repository and build the project using Cargo:

```bash
git clone https://github.com/your-repo/yt-sl-extractor.git
cd yt-sl-extractor/poc
cargo build --release
```

The binary will be available at `./target/release/poc`.

## 📖 Usage

### Basic Command

```bash
cargo run -p poc -- --url "https://www.youtube.com/watch?v=dQw4w9WgXcQ" --output "./output"
```

### Advanced Options

```bash
Options:
  -u, --url <URL>               The YouTube video URL to process
  -i, --interval <SECONDS>       Frame extraction interval in seconds (default: 5.0)
  -t, --threshold <THRESHOLD>   Slide similarity threshold (0.0 - 1.0, default: 0.85)
  -o, --output <DIR>            Output directory (default: ".")
  -l, --languages <LANGS>       OCR languages, comma-separated (e.g., "eng,deu", default: "eng")
  -s, --timestamps              Include timestamps in the output filename
  -m, --memory-threshold <MB>   Memory threshold in MB (default: 500)
  -h, --help                    Print help
  -V, --version                 Print version
```

### Example: High-Precision Extraction with Multiple Languages

```bash
cargo run -p poc -- \
  --url "https://www.youtube.com/watch?v=..." \
  --interval 2.0 \
  --threshold 0.95 \
  --languages eng,spa \
  --output "./lecture_notes"
```

## 📂 Project Structure

- `poc/src/contexts/video`: YouTube interaction and downloading.
- `poc/src/contexts/frame`: FFmpeg integration and perceptual hashing.
- `poc/src/contexts/dedup`: Similarity logic and slide selection.
- `poc/src/contexts/ocr`: Tesseract integration and confidence parsing.
- `poc/src/contexts/document`: Markdown report generation with Tera templates.
- `poc/src/contexts/session`: Orchestration logic and state persistence.

## 🧪 Testing

The project includes a comprehensive test suite with 230+ tests.

```bash
cargo test -p poc
```

## 📝 License

This project is licensed under the MIT License.
