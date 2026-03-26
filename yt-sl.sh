#!/bin/bash
# yt-sl: Extract slides from a YouTube video using vision LLMs
#
# Usage:
#   ./yt-sl.sh https://youtu.be/g0047beVND4
#   ./yt-sl.sh https://youtu.be/g0047beVND4 --interval 3
#   ./yt-sl.sh https://youtu.be/g0047beVND4 --title "My Talk"

set -euo pipefail

URL="${1:?Usage: yt-sl.sh <youtube-url> [--interval N] [--title TITLE] [extra yt-sl flags...]}"
shift

CACHE="${HOME}/Library/Application Support/yt-sl/cache"
OUTPUT="./output"
INTERVAL=5
TITLE="Untitled"
EXTRA_ARGS=()

# Parse optional args
while [[ $# -gt 0 ]]; do
  case "$1" in
    --interval|-i) INTERVAL="$2"; shift 2 ;;
    --title) TITLE="$2"; shift 2 ;;
    --output|-o) OUTPUT="$2"; shift 2 ;;
    *) EXTRA_ARGS+=("$1"); shift ;;
  esac
done

# Get video ID and title
VIDEO_INFO=$(yt-dlp --print id --print title "$URL" 2>/dev/null || echo "")
VIDEO_ID=$(echo "$VIDEO_INFO" | head -1)
VIDEO_TITLE=$(echo "$VIDEO_INFO" | tail -1)

if [[ -z "$VIDEO_ID" ]]; then
  # Fallback: extract from URL
  VIDEO_ID=$(echo "$URL" | grep -oP '(?:v=|youtu\.be/)([a-zA-Z0-9_-]+)' | head -1 | sed 's/v=//;s/youtu\.be\///')
fi

if [[ -z "$VIDEO_ID" ]]; then
  echo "error: could not extract video ID from $URL" >&2
  exit 1
fi

# Use video title for output dir name, fallback to --title or ID
if [[ "$TITLE" == "Untitled" && -n "$VIDEO_TITLE" ]]; then
  TITLE="$VIDEO_TITLE"
fi
# Sanitize title for directory name: lowercase, replace spaces/special chars with hyphens
OUTPUT_NAME=$(echo "$TITLE" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]/-/g' | sed 's/--*/-/g' | sed 's/^-//;s/-$//')
if [[ -z "$OUTPUT_NAME" ]]; then
  OUTPUT_NAME="$VIDEO_ID"
fi

echo "[1/5] Video: $TITLE ($VIDEO_ID)"

VIDEOS_DIR="$CACHE/videos"
FRAMES_DIR="$CACHE/frames/$VIDEO_ID"
mkdir -p "$VIDEOS_DIR" "$FRAMES_DIR"

# Download video (cached)
VIDEO_PATH="$VIDEOS_DIR/$VIDEO_ID.mp4"
if [[ -f "$VIDEO_PATH" ]]; then
  echo "[2/5] Video cached: $VIDEO_PATH"
else
  echo "[2/5] Downloading video..."
  yt-dlp -f "bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best" \
    -o "$VIDEO_PATH" "$URL"
fi

# Extract frames using scene detection (skip if already done)
FRAME_COUNT=$(find "$FRAMES_DIR" -name "*.jpg" 2>/dev/null | wc -l | tr -d ' ')
if [[ "$FRAME_COUNT" -gt 0 ]]; then
  echo "[3/5] Frames cached: $FRAME_COUNT frames in $FRAMES_DIR"
else
  echo "[3/5] Extracting frames (scene detection + interval fallback)..."
  # Extract frames and capture their actual timestamps
  ffmpeg -i "$VIDEO_PATH" \
    -vf "select='gt(scene,0.2)+not(mod(n,25*$INTERVAL))',showinfo,scale=1024:-1" \
    -vsync vfr -q:v 2 \
    "$FRAMES_DIR/frame_%04d.jpg" 2>&1 | \
    grep "pts_time:" | sed 's/.*pts_time:\([0-9.]*\).*/\1/' > "$FRAMES_DIR/timestamps.txt"
  FRAME_COUNT=$(find "$FRAMES_DIR" -name "*.jpg" | wc -l | tr -d ' ')
  echo "  extracted $FRAME_COUNT frames"
fi

# Transcribe audio (optional, skip if whisper not available)
TRANSCRIPT_PATH="$VIDEOS_DIR/$VIDEO_ID.json"
TRANSCRIPT_ARGS=()
if [[ -f "$TRANSCRIPT_PATH" ]]; then
  echo "[4/5] Transcript cached: $TRANSCRIPT_PATH"
  TRANSCRIPT_ARGS=("--transcript" "$TRANSCRIPT_PATH")
else
  AUDIO_PATH="$VIDEOS_DIR/$VIDEO_ID.wav"
  if [[ ! -f "$AUDIO_PATH" ]]; then
    echo "[4/5] Extracting audio..."
    ffmpeg -i "$VIDEO_PATH" -vn -acodec pcm_s16le -ar 16000 -ac 1 -y \
      "$AUDIO_PATH" 2>/dev/null
  fi

  # Try whisper API
  if curl -s --max-time 5 http://localhost:1234/v1/models >/dev/null 2>&1; then
    echo "[4/5] Transcribing audio..."
    curl -s http://localhost:1234/v1/audio/transcriptions \
      -F file=@"$AUDIO_PATH" \
      -F model=whisper-1 \
      -F response_format=verbose_json \
      -o "$TRANSCRIPT_PATH" 2>/dev/null
    # Validate response has "text" field (not an error)
    if [[ -f "$TRANSCRIPT_PATH" ]] && grep -q '"text"' "$TRANSCRIPT_PATH" 2>/dev/null; then
      TRANSCRIPT_ARGS=("--transcript" "$TRANSCRIPT_PATH")
    else
      rm -f "$TRANSCRIPT_PATH"
      echo "  whisper failed, skipping transcription"
    fi
  else
    echo "[4/5] No whisper API available, skipping transcription"
  fi
fi

# Output with meaningful name
VIDEO_OUTPUT="$OUTPUT/$OUTPUT_NAME"

# Run yt-sl
echo "[5/5] Extracting slides..."
yt-sl --frames "$FRAMES_DIR" \
  --output "$VIDEO_OUTPUT" \
  --title "$TITLE" \
  --url "$URL" \
  --interval "$INTERVAL" \
  "${TRANSCRIPT_ARGS[@]+"${TRANSCRIPT_ARGS[@]}"}" \
  "${EXTRA_ARGS[@]+"${EXTRA_ARGS[@]}"}"

echo ""
echo "Done: $VIDEO_OUTPUT/report.md"
