# yt-sl

Extract presentation slides from video frames using vision LLMs.

A minimal Rust CLI that takes a directory of video frames, deduplicates them via perceptual hashing, classifies and OCRs each unique frame using a local vision model (Qwen-VL), and generates a markdown report.

Does **not** call yt-dlp or ffmpeg — you handle extraction yourself, then pipe the frames in.

## Install

```bash
brew tap ragavsathish/yt-sl
brew install yt-sl
```

Or from source:

```bash
cargo install --path .
```

## Prerequisites

1. **yt-dlp** — `brew install yt-dlp`
2. **ffmpeg** — `brew install ffmpeg`
3. **Local vision model** — [LM Studio](https://lmstudio.ai/) with `qwen/qwen3-vl-8b` or similar

## Usage

```bash
CACHE=~/Library/Application\ Support/yt-sl/cache
VIDEO_ID="dQw4w9WgXcQ"

# 1. Download video (cached, survives reboots)
mkdir -p "$CACHE/videos" "$CACHE/frames/$VIDEO_ID"
yt-dlp -f best -o "$CACHE/videos/$VIDEO_ID.mp4" "https://youtube.com/watch?v=$VIDEO_ID"

# 2. Extract frames
ffmpeg -i "$CACHE/videos/$VIDEO_ID.mp4" -vf fps=1/5 -q:v 2 "$CACHE/frames/$VIDEO_ID/frame_%04d.jpg"

# 3. (Optional) Transcribe audio
ffmpeg -i "$CACHE/videos/$VIDEO_ID.mp4" -vn -acodec pcm_s16le -ar 16000 -ac 1 "$CACHE/videos/$VIDEO_ID.wav"
curl http://localhost:1234/v1/audio/transcriptions \
  -F file=@"$CACHE/videos/$VIDEO_ID.wav" -F model=whisper-1 \
  -F response_format=verbose_json > "$CACHE/videos/$VIDEO_ID.json"

# 4. Extract slides
yt-sl --frames "$CACHE/frames/$VIDEO_ID" --output ./output --title "My Talk" \
  --transcript "$CACHE/videos/$VIDEO_ID.json"
```

### Recommended directory layout

```
~/Library/Application Support/yt-sl/
  cache/
    videos/          # downloaded .mp4, .wav, transcript .json
    frames/          # extracted frames per video ID
      g0047beVND4/
      dQw4w9WgXcQ/
  training/
    labels.jsonl     # auto-collected by yt-sl (for fine-tuning)
```

## Options

```
Options:
  -f, --frames <DIR>          Directory containing frame images (jpg/png)
  -o, --output <DIR>          Output directory (default: ./output)
      --transcript <FILE>     Whisper verbose_json transcript file
      --title <TITLE>         Report title (default: Untitled)
      --url <URL>             Source URL (for report metadata)
  -i, --interval <SECS>       Frame interval used during extraction (default: 5)
  -T, --threshold <0.0-1.0>   Hash similarity threshold (default: 0.85)
      --model <NAME>          Vision model (default: qwen/qwen3-vl-8b)
      --vision-api <URL>      API base URL (default: http://localhost:1234/v1)
      --concurrency <N>       Max concurrent API requests (default: 4)
  -h, --help                  Print help
```

## How it works

1. **Dedup** — Computes average perceptual hash (8x8 grayscale) for each frame, keeps only frames that differ beyond the similarity threshold
2. **Classify + OCR** — Sends each unique frame to a vision LLM with a single prompt that both classifies (slide vs not-slide) and extracts text
3. **Report** — Generates a markdown file with slide images, extracted text, and matched transcript segments
4. **Training data** — Every classification is saved to `~/.local/share/yt-sl/training/labels.jsonl` for fine-tuning a local classifier

## Training your own classifier

Every time you run `yt-sl`, Qwen-VL labels are automatically collected. After processing several videos, use them to fine-tune a lightweight model that runs without any API:

```bash
# Check how much data you've collected
wc -l ~/Library/Application\ Support/yt-sl/training/labels.jsonl

# Fine-tune SmolVLM-256M using Oumi
pip install -r classifier/src/requirements.txt
oumi train -c classifier/src/train.yaml

# Test the trained model
python classifier/src/infer.py --frames-dir frames/ --model output/slide-classifier/
```

See `classifier/program.md` for the full workflow.

## License

MIT
