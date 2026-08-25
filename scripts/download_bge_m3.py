#!/usr/bin/env python3
"""
Download BGE-M3 (or other ONNX embedding models) from Hugging Face mirror for remdb-server.

Supported mirrors:
  - https://hf-mirror.com          (recommended in China, default)
  - https://huggingface.co         (official, fallback)

The script downloads the ONNX-format model files needed by remdb-server's
ONNX runtime embedding engine into the configured models_dir.

Usage:
  # Download BGE-M3 with default settings
  python download_bge_m3.py

  # Download a different model
  python download_bge_m3.py --model BAAI/bge-small-zh-v1.5

  # Specify a custom output directory
  python download_bge_m3.py --output-dir ./models

  # List available models (no download)
  python download_bge_m3.py --list

  # Download only specific files (skip ONNX model, just tokenizer)
  python download_bge_m3.py --files tokenizer.json config.json

Environment variables:
  HF_MIRROR        Override the mirror URL (default: https://hf-mirror.com)
  HF_TOKEN         Hugging Face token for gated models (optional)
"""

import argparse
import json
import os
import shutil
import sys
import tempfile
from pathlib import Path
from typing import Optional
from urllib.parse import urljoin

import requests
from tqdm import tqdm

# ── Defaults ────────────────────────────────────────────────────────────────

DEFAULT_MODEL = "BAAI/bge-m3"
DEFAULT_MIRROR = os.environ.get("HF_MIRROR", "https://hf-mirror.com")
DEFAULT_OUTPUT = "./models"

# ONNX models typically need these files for inference
ONNX_REQUIRED_FILES = [
    "model.onnx",          # ONNX model (may be model_quantized.onnx etc.)
    "tokenizer.json",      # HuggingFace tokenizer
    "config.json",         # Model config (max_length, etc.)
]

# Full set of files we'd download for a complete model
ONNX_ALL_FILES = [
    "model.onnx",
    "model_quantized.onnx",
    "tokenizer.json",
    "tokenizer_config.json",
    "special_tokens_map.json",
    "config.json",
    "added_tokens.json",
]


# ── Helpers ─────────────────────────────────────────────────────────────────


def model_url(mirror: str, model_id: str, filename: str) -> str:
    """Build the raw download URL for a file on the mirror."""
    return urljoin(mirror.rstrip("/") + "/", f"{model_id}/resolve/main/{filename}")


def api_url(mirror: str, model_id: str) -> str:
    """Build the API URL to list files for a model."""
    return urljoin(mirror.rstrip("/") + "/", f"api/models/{model_id}")


def list_model_files(mirror: str, model_id: str, token: Optional[str] = None) -> list[str]:
    """List all files available for a model via the HuggingFace API."""
    url = api_url(mirror, model_id)
    headers = {"Authorization": f"Bearer {token}"} if token else {}
    resp = requests.get(url, headers=headers, timeout=30)
    if resp.status_code == 404:
        print(f"❌ Model '{model_id}' not found on {mirror}")
        print(f"   Check the model name or try a different mirror.")
        sys.exit(1)
    if resp.status_code == 401:
        print(f"❌ Model '{model_id}' requires authentication on {mirror}")
        if not token:
            print(f"   Try: --token <hf-token>  or  export HF_TOKEN=...")
        else:
            print(f"   The token may be invalid or lack access to this model.")
        print(f"")
        # Fall back to listing known-common files if direct download works
        # by checking if config.json is accessible
        test_url = model_url(mirror, model_id, "config.json")
        tresp = requests.head(test_url, headers=headers, timeout=15)
        if tresp.status_code in (200, 302, 307):
            print(f"   ⚠  Falling back to direct file download (no file listing).")
            # Return a placeholder — we'll let download_file handle 404s per-file
            return ["config.json", "tokenizer.json"]
        sys.exit(1)
    resp.raise_for_status()
    data = resp.json()
    siblings = data.get("siblings", [])
    files = [s["rfilename"] for s in siblings if not s.get("rfilename", "").endswith("/")]
    return sorted(files)


def download_file(
    url: str,
    dest: Path,
    desc: str = "",
    token: Optional[str] = None,
    resume: bool = True,
) -> Path:
    """Download a single file with progress bar, supporting resumption."""
    headers = {"Authorization": f"Bearer {token}"} if token else {}

    # Determine existing size for resume
    existing_size = 0
    if resume and dest.exists():
        existing_size = dest.stat().st_size
        if existing_size > 0:
            headers["Range"] = f"bytes={existing_size}-"

    resp = requests.get(url, headers=headers, stream=True, timeout=60)
    if resp.status_code == 416:
        # Range not satisfiable → file is already complete
        print(f"  ✓ {desc} — already downloaded")
        return dest
    if resp.status_code == 404:
        raise FileNotFoundError(f"404: {url}")

    # Follow redirects manually for range support
    if resp.status_code in (301, 302, 303, 307, 308):
        redirect_url = resp.headers.get("Location", "")
        if redirect_url:
            return download_file(redirect_url, dest, desc, token, resume)

    if resp.status_code == 206:
        # Partial content — resume
        mode = "ab"
        total = int(resp.headers.get("Content-Range", "").split("/")[-1])
        downloaded = existing_size
        desc = f"{desc} (resume)"
    elif resp.status_code == 200:
        mode = "wb"
        total = int(resp.headers.get("Content-Length", 0))
        downloaded = 0
        if resume and existing_size > 0:
            # Server doesn't support range, restart
            dest.unlink(missing_ok=True)
    else:
        resp.raise_for_status()
        return dest  # unreachable

    dest.parent.mkdir(parents=True, exist_ok=True)

    with open(dest, mode) as f:
        with tqdm(
            total=total,
            unit="B",
            unit_scale=True,
            unit_divisor=1024,
            desc=desc.ljust(30),
            initial=downloaded,
            ascii=" ▏▎▍▌▋▊▉█",
        ) as pbar:
            for chunk in resp.iter_content(chunk_size=8192):
                if chunk:
                    f.write(chunk)
                    pbar.update(len(chunk))

    return dest


def get_model_config(mirror: str, model_id: str, token: Optional[str] = None) -> dict:
    """Fetch and parse config.json for the model."""
    url = model_url(mirror, model_id, "config.json")
    headers = {"Authorization": f"Bearer {token}"} if token else {}
    resp = requests.get(url, headers=headers, timeout=30)
    if resp.status_code == 200:
        return resp.json()
    return {}


def print_model_summary(config: dict, model_id: str = ""):
    """Print a human-readable summary of the model config."""
    model_type = config.get("model_type", "unknown")
    hidden_size = config.get("hidden_size", "?")
    max_length = config.get("max_position_embeddings",
                            config.get("max_length", "?"))
    dim = config.get("dim", hidden_size)
    print(f"  Type:             {model_type}")
    print(f"  Dimensions:       {dim}")
    print(f"  Max length:       {max_length}")
    print()

    # Warn if it doesn't look like an ONNX model
    if model_type != "onnx" and "architectures" in config:
        arches = config["architectures"]
        short = model_id.split("/")[-1] if "/" in model_id else model_id
        print(f"  ⚠  This appears to be a {arches} model (not ONNX format).")
        print(f"     For ONNX runtime, you may need a converted model.")
        print(f"     Try: --model Xenova/{short}")
        print()


# ── Commands ────────────────────────────────────────────────────────────────


def cmd_list(args: argparse.Namespace):
    """List files available for a model."""
    mirror = args.mirror or DEFAULT_MIRROR
    print(f"🔍 Listing files for {args.model} on {mirror}")
    print()
    files = list_model_files(mirror, args.model, args.token)
    config = get_model_config(mirror, args.model, args.token)
    print_model_summary(config, args.model)

    # Categorize files
    onnx_files = [f for f in files if f.endswith(".onnx")]
    tokenizer_files = [f for f in files if "tokenizer" in f]
    other_files = [f for f in files if f not in onnx_files + tokenizer_files]

    if onnx_files:
        print("📦 ONNX model files:")
        for f in onnx_files:
            size = get_file_size(mirror, args.model, f, args.token)
            print(f"    {f:40s}  {size}")
        print()

    if tokenizer_files:
        print("📝 Tokenizer files:")
        for f in tokenizer_files:
            print(f"    {f}")
        print()

    print("📁 All files:")
    for f in files:
        print(f"    {f}")
    print()
    print(f"Total: {len(files)} files")


def cmd_download(args: argparse.Namespace):
    """Download the model files."""
    mirror = args.mirror or DEFAULT_MIRROR
    output_dir = Path(args.output_dir)
    model_dir = output_dir / args.model.split("/")[-1]

    print(f"🌐 Mirror:     {mirror}")
    print(f"📦 Model:      {args.model}")
    print(f"📂 Output:     {model_dir}")
    print()

    # Fetch remote file list
    remote_files = list_model_files(mirror, args.model, args.token)
    config = get_model_config(mirror, args.model, args.token)
    print_model_summary(config, args.model)

    # Determine which files to download
    if args.files:
        # User-specified files
        files_to_download = [f for f in args.files if f in remote_files]
        not_found = [f for f in args.files if f not in remote_files]
        if not_found:
            print(f"⚠  Files not found in remote: {', '.join(not_found)}")
            if not files_to_download:
                sys.exit(1)
    else:
        # Auto-detect: prefer ONNX files, fall back to safetensors/pytorch
        onnx_files = sorted([f for f in remote_files if f.endswith(".onnx")])
        tokenizer_files = sorted(
            [f for f in remote_files if f in ONNX_ALL_FILES or "tokenizer" in f]
        )
        config_files = sorted(
            [f for f in remote_files if f in ("config.json", "added_tokens.json",
                                               "special_tokens_map.json",
                                               "tokenizer_config.json")]
        )

        if onnx_files:
            files_to_download = onnx_files + tokenizer_files + config_files
            print(f"✅ Found {len(onnx_files)} ONNX model file(s)")
        else:
            # No ONNX files — this model isn't in ONNX format
            print(f"⚠  No ONNX files found for this model.")
            print(f"   The model exists but may not be in ONNX format.")
            print()
            # Suggest ONNX community models
            model_name = args.model.split("/")[-1]
            print(f"   Try one of the following:")
            print(f"     python download_bge_m3.py --model Xenova/{model_name}")
            print(f"     python download_bge_m3.py --model onnx-community/{model_name}")
            yr = input("\n   Download anyway (pytorch/safetensors)? [y/N] ").strip().lower()
            if yr == "y":
                # Download all non-onnx files (excluding potential GB-sized safetensors)
                excluded_exts = {".safetensors", ".bin", ".pt", ".pth", ".gguf"}
                files_to_download = [
                    f for f in remote_files
                    if Path(f).suffix not in excluded_exts
                ]
            else:
                sys.exit(0)

    # Remove duplicates, preserve order
    seen = set()
    files_to_download = [f for f in files_to_download if not (f in seen or seen.add(f))]

    print(f"\n📥 Downloading {len(files_to_download)} file(s)...")
    print()

    # Download each file
    success = 0
    skipped = 0
    failed = 0

    for filename in files_to_download:
        url = model_url(mirror, args.model, filename)
        dest = model_dir / filename
        dest.parent.mkdir(parents=True, exist_ok=True)

        # Skip if already exists and has content
        if dest.exists() and dest.stat().st_size > 0 and not args.force:
            # Quick check: if remote is larger, re-download
            remote_size = get_file_size_raw(mirror, args.model, filename, args.token)
            if remote_size is not None and dest.stat().st_size >= remote_size:
                print(f"  ✓ {filename} — already exists")
                skipped += 1
                continue

        try:
            desc = filename.split("/")[-1] if "/" in filename else filename
            download_file(url, dest, desc=desc, token=args.token)
            success += 1
        except FileNotFoundError:
            print(f"  ✗ {filename} — not found on mirror")
            failed += 1
        except requests.RequestException as e:
            print(f"  ✗ {filename} — {e}")
            failed += 1

    # Summary
    print()
    print(f"{'=' * 50}")
    print(f"  ✅ Downloaded: {success}  ⏭  Skipped: {skipped}  ❌ Failed: {failed}")
    print(f"  📂 Location: {model_dir.resolve()}")

    if failed == 0 and success > 0:
        print()
        print(f"  Next step: configure remdb-server with:")
        print(f"    [milvus.embedding]")
        print(f'    default_model = "{args.model.split("/")[-1]}"')
        print(f'    models_dir = "{output_dir.resolve()}"')
        print(f'    hf_mirror = "{mirror}"')
        print()

    return model_dir


# ── Utility ─────────────────────────────────────────────────────────────────


def get_file_size_raw(mirror: str, model_id: str, filename: str, token: Optional[str] = None) -> Optional[int]:
    """Get remote file size via HEAD request."""
    url = model_url(mirror, model_id, filename)
    headers = {"Authorization": f"Bearer {token}"} if token else {}
    try:
        resp = requests.head(url, headers=headers, timeout=15, allow_redirects=True)
        if resp.status_code == 200:
            cl = resp.headers.get("Content-Length")
            return int(cl) if cl else None
    except requests.RequestException:
        pass
    return None


def get_file_size(mirror: str, model_id: str, filename: str, token: Optional[str] = None) -> str:
    """Get human-readable file size."""
    size = get_file_size_raw(mirror, model_id, filename, token)
    if size is None:
        return "?"
    for unit in ("B", "KB", "MB", "GB"):
        if size < 1024:
            return f"{size:.1f} {unit}"
        size /= 1024
    return f"{size:.1f} TB"


# ── CLI ─────────────────────────────────────────────────────────────────────


def main():
    parser = argparse.ArgumentParser(
        description="Download BGE-M3 / ONNX embedding models from Hugging Face mirror",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )

    parser.add_argument(
        "--model", "-m",
        default=DEFAULT_MODEL,
        help=f"Model ID on Hugging Face (default: {DEFAULT_MODEL})",
    )
    parser.add_argument(
        "--mirror",
        default=None,
        help=f"HF mirror URL (default: {DEFAULT_MIRROR}, or $HF_MIRROR)",
    )
    parser.add_argument(
        "--output-dir", "-o",
        default=DEFAULT_OUTPUT,
        help=f"Output directory for models (default: {DEFAULT_OUTPUT})",
    )
    parser.add_argument(
        "--files", "-f",
        nargs="+",
        help="Specific files to download (e.g., tokenizer.json config.json)",
    )
    parser.add_argument(
        "--force", "-F",
        action="store_true",
        help="Force re-download even if files exist",
    )
    parser.add_argument(
        "--token",
        default=None,
        help="Hugging Face token for gated models (or $HF_TOKEN)",
    )
    parser.add_argument(
        "--list", "-l",
        action="store_true",
        help="List available files for the model (no download)",
    )
    parser.add_argument(
        "--no-resume",
        action="store_true",
        help="Disable resume for interrupted downloads",
    )

    args = parser.parse_args()
    args.mirror = args.mirror or DEFAULT_MIRROR
    args.token = args.token or os.environ.get("HF_TOKEN")

    if args.list:
        cmd_list(args)
    else:
        cmd_download(args)


if __name__ == "__main__":
    main()