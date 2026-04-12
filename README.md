# yt-sl

Extract presentation slides from video frames using vision LLMs.

A minimal Rust CLI + bash wrapper that downloads YouTube videos, extracts frames via scene detection, deduplicates using perceptual hashing with auto-detected slide regions, classifies and OCRs each frame using a local vision model (Qwen-VL), and generates a markdown report.

## Install

```bash
brew tap ragavsathish/yt-sl https://github.com/ragavsathish/yt-sl
brew install yt-sl
```

Or from source:

```bash
cargo install --path .
```

## Prerequisites

1. **yt-dlp** — `brew install yt-dlp`
2. **ffmpeg** — `brew install ffmpeg`
3. **Local vision model** — [LM Studio](https://lmstudio.ai/) with `qwen/qwen3-vl-8b` or similar, running on `localhost:1234`

## Quick Start

```bash
# One command — downloads, extracts frames, OCRs, generates report
./yt-sl.sh https://youtu.be/g0047beVND4
```

Output goes to `./output/<video-title>/report.md` with slides in `./output/<video-title>/slides/`.

## Usage

### Wrapper script (recommended)

```bash
# Full pipeline: download → frames → OCR → report
./yt-sl.sh https://youtu.be/VIDEO_ID

# With custom title
./yt-sl.sh https://youtu.be/VIDEO_ID --title "My Talk"

# Custom output directory
./yt-sl.sh https://youtu.be/VIDEO_ID --output ./my-slides
```

### Rust binary directly (pipe architecture)

```bash
# You handle download + extraction
yt-dlp -f best -o video.mp4 "https://youtube.com/watch?v=..."
ffmpeg -i video.mp4 -vf "select='gt(scene,0.2)',scale=1024:-1" -vsync vfr -q:v 2 frames/frame_%04d.jpg

# yt-sl does dedup + OCR + report
yt-sl --frames frames/ --output ./output --title "My Talk"
```

## Options (yt-sl binary)

```
  -f, --frames <DIR>          Directory containing frame images (jpg/png)
  -o, --output <DIR>          Output directory (default: ./output)
      --transcript <FILE>     Whisper verbose_json transcript file
      --title <TITLE>         Report title (default: Untitled)
      --url <URL>             Source URL (for report metadata)
  -i, --interval <SECS>       Frame interval for timestamp estimation (default: 5)
  -T, --threshold <0.0-1.0>   Hash similarity threshold (default: 0.90)
      --model <NAME>          Vision model (default: qwen/qwen3-vl-8b)
      --vision-api <URL>      API base URL (default: http://localhost:1234/v1)
      --concurrency <N>       Max concurrent API requests (default: 4)
```

## How it works

1. **Frame extraction** — Scene detection + fixed interval fallback via ffmpeg, scaled to 1024px
2. **Slide region detection** — Samples frames and asks the LLM to identify the projected slide bounding box (one-time, handles wide-shot recordings)
3. **Hash dedup** — Perceptual hash on the cropped slide region, comparing against all accepted frames
4. **Classify + OCR** — Vision LLM classifies each frame and extracts text in a single call
5. **Text dedup** — Removes slides with duplicate OCR text, keeps the version with more extracted text (cleaner capture)
6. **Fragment filter** — Drops frames with fewer than 5 words (speaker close-ups with blurry text)
7. **Report** — Generates markdown with slide images, extracted text, and matched transcript segments
8. **Training data** — Every classification is saved for fine-tuning a local classifier

## Directory layout

```
~/Library/Application Support/yt-sl/
  cache/
    videos/          # downloaded .mp4, .wav, transcript .json
    frames/          # extracted frames per video ID
  training/
    labels.jsonl     # auto-collected by yt-sl (Oumi-compatible)
```

## Training your own classifier

Every time you run `yt-sl`, Qwen-VL labels are automatically collected. After processing several videos (~500+ labels), fine-tune a lightweight model that runs without any API:

```bash
# Check how much data you've collected
wc -l ~/Library/Application\ Support/yt-sl/training/labels.jsonl

# Fine-tune SmolVLM-256M using Oumi (teacher-student distillation)
pip install -r classifier/src/requirements.txt
oumi train -c classifier/src/train.yaml

# Test the trained model
python classifier/src/infer.py --frames-dir frames/ --model output/slide-classifier/
```

See `classifier/program.md` for the full workflow.

## Project structure

```
src/main.rs          # Rust binary — dedup, vision OCR, text dedup, markdown generation
yt-sl.sh             # Bash wrapper — download, frame extraction, full pipeline
classifier/          # Teacher-student training pipeline (Oumi + Qwen-VL)
  program.md         # Agent instructions for autonomous training
  src/
    label.py         # Qwen-VL teacher auto-labels frames
    train.yaml       # Oumi fine-tuning config (SmolVLM-256M + LoRA)
    infer.py         # Batch inference with fine-tuned model
Formula/yt-sl.rb     # Homebrew formula
videos.yaml          # Registry of processed videos
```

## License

MIT
