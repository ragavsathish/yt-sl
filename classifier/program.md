# slide-classifier

Train a lightweight vision model to classify video frames as presentation slides vs non-slides, using Qwen-VL as a teacher model and Oumi for fine-tuning.

## Setup

To set up a new training run, work with the user to:

1. **Check prerequisites**: Verify that LM Studio is running with Qwen-VL loaded at `http://localhost:1234/v1`. Test with: `curl http://localhost:1234/v1/models`.
2. **Check Python env**: Verify `uv` or `pip` is available. Install deps: `pip install -r src/requirements.txt`.
3. **Check for frames**: The user should have a directory of video frames (extracted via ffmpeg). If not, help them extract: `ffmpeg -i video.mp4 -vf fps=1/5 -q:v 2 frames/frame_%04d.jpg`.
4. **Confirm and go**: Confirm setup looks good, then kick off the pipeline.

## The Pipeline

There are three phases. Run them in order.

### Phase 1: Teacher Labeling

Use Qwen-VL (the teacher) to auto-label all frames:

```bash
python src/label.py --frames-dir <FRAMES_DIR> --output data/
```

This sends each frame to the local Qwen-VL API, classifies it as SLIDE or NOT_SLIDE, and writes `data/train.jsonl` and `data/eval.jsonl`.

**What to check:**
- The label distribution should be reasonable (e.g. 10-30% SLIDE for a typical presentation video).
- If everything is NOT_SLIDE, the teacher model might not be loaded or the frames might not contain slides.
- If everything is SLIDE, the threshold might be too loose or the video has no speaker shots.

**Scaling up:** For better results, label frames from multiple different videos. More diversity = better generalization. Run `label.py` multiple times with `--output data/` and it will append.

### Phase 2: Student Training

Fine-tune SmolVLM-256M on the teacher labels using Oumi:

```bash
oumi train -c src/train.yaml
```

**What you CAN modify in `src/train.yaml`:**
- `num_epochs` — more epochs for small datasets, fewer for large
- `learning_rate` — try 1e-5 to 1e-4
- `lora_r` — rank 4-16, higher = more capacity but slower
- `per_device_train_batch_size` — increase if you have GPU memory
- `model.model_name_or_path` — try SmolVLM-2B for better quality

**What you CANNOT modify:**
- The data format (JSONL with input/output/image fields)
- The evaluation metric (Oumi handles this)

**The goal: get the highest classification accuracy.** Since the task is binary (SLIDE vs NOT_SLIDE), even a tiny model should reach >90% accuracy quickly.

**Hardware expectations:**
- CPU: ~2-4 hours for 500 samples, 3 epochs
- Apple Silicon MPS: ~30-60 min
- GPU 8GB+: ~10-15 min

### Phase 3: Evaluation

Test the fine-tuned model on the eval set:

```bash
python src/infer.py --frames-dir <TEST_FRAMES_DIR> --model output/slide-classifier/ --output results.json
```

Check the results. If accuracy is poor:
- Add more training data (label more videos)
- Increase epochs or lora_r
- Try a larger base model (SmolVLM-2B)

## Using the trained model

Once trained, the model replaces the Qwen-VL API calls in `yt-sl`:

```bash
# Batch classify frames (no API needed)
python src/infer.py --frames-dir frames/ --model output/slide-classifier/ --output labels.json

# Or serve as OpenAI-compatible API for yt-sl
# (use vLLM or SGLang to serve the checkpoint)
```

## The experiment loop

If you want to iterate and improve the classifier autonomously:

LOOP:

1. Label frames from a new video using the teacher
2. Add to training data
3. Retrain with updated data
4. Evaluate on held-out frames
5. If accuracy improved, keep the checkpoint
6. If not, revert and try different hyperparameters
7. Repeat with more videos for better generalization

**Key insight:** The teacher (Qwen-VL) is slow but accurate. The student (SmolVLM) is fast but needs training. The goal is to transfer the teacher's knowledge into a model that runs instantly on CPU without any API calls.

## Files

```
src/label.py         — Qwen-VL teacher auto-labels frames (the only file that calls an API)
src/train.yaml       — Oumi training config (modify hyperparameters here)
src/infer.py         — batch inference with fine-tuned model (no API)
src/requirements.txt — Python dependencies
data/                — generated training data (gitignored)
output/              — model checkpoints (gitignored)
```
