# slide-classifier

Train a lightweight vision model to classify video frames as presentation slides vs non-slides, using Qwen-VL as a teacher model and Oumi for fine-tuning.

## Setup

1. **LM Studio running**: Qwen-VL loaded at `http://localhost:1234/v1`. Test: `curl http://localhost:1234/v1/models`
2. **Python deps**: `pip install -r src/requirements.txt`
3. **Training data**: Check if `yt-sl` has collected enough labels:
   ```bash
   wc -l ~/Library/Application\ Support/yt-sl/training/labels.jsonl
   ```
   Need 500+ labels. If not enough, process more videos with `yt-sl.sh`.

## Pipeline

### Phase 1: Data (automatic)

Training data is collected automatically every time you run `yt-sl`. Each frame's classification (SLIDE / NOT_SLIDE) is appended to `~/Library/Application Support/yt-sl/training/labels.jsonl` in Oumi-compatible JSONL format.

To collect more data, just process more videos:
```bash
./yt-sl.sh https://youtu.be/VIDEO_ID
```

Or label frames directly with the teacher model:
```bash
python src/label.py --frames-dir <FRAMES_DIR> --output data/
```

### Phase 2: Train

Fine-tune SmolVLM-256M on the collected labels using Oumi:

```bash
oumi train -c src/train.yaml
```

**Tunable parameters in `src/train.yaml`:**
- `num_epochs` — more for small datasets, fewer for large (default: 3)
- `learning_rate` — try 1e-5 to 1e-4 (default: 5e-5)
- `lora_r` — rank 4-16 (default: 8)
- `model.model_name_or_path` — try `SmolVLM-2B` for better quality

**Hardware:**
- CPU: ~2-4 hours for 500 samples
- Apple Silicon MPS: ~30-60 min
- GPU 8GB+: ~10-15 min

### Phase 3: Evaluate

```bash
python src/infer.py --frames-dir <TEST_FRAMES_DIR> --model output/slide-classifier/ --output results.json
```

If accuracy is poor: add more training data (more videos), increase epochs, or try a larger base model.

## Autonomous experiment loop

LOOP:
1. Process a new video with `yt-sl.sh` (auto-collects labels)
2. Retrain: `oumi train -c src/train.yaml`
3. Evaluate on held-out frames
4. If accuracy improved, keep the checkpoint
5. If not, try different hyperparameters
6. Repeat

**Key insight:** The teacher (Qwen-VL) is slow but accurate. The student (SmolVLM) is fast but needs training. The goal is to transfer the teacher's knowledge into a model that runs instantly on CPU without any API calls.

## Files

```
src/label.py         — Qwen-VL teacher auto-labels frames
src/train.yaml       — Oumi training config (SmolVLM-256M + LoRA)
src/infer.py         — batch inference with fine-tuned model
src/requirements.txt — Python dependencies
data/                — generated training data (gitignored)
output/              — model checkpoints (gitignored)
```
