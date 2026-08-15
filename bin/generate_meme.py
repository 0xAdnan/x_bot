#!/usr/bin/env python3
"""
generate_meme.py — Automated high-res visual meme and chart generator for PITCH X bot.
Generates:
  1. polymarket_chart : Dark-mode Polymarket-style odds prediction card
  2. comparison_card  : Side-by-side / Top-Bottom "Manual recording vs Pitch" card
  3. progression_card : 4-stage escalating builder pain card
  4. dark_quote_card  : High-contrast dev quote card
"""

import sys
import os
import argparse
from PIL import Image, ImageDraw, ImageFont

ASSETS_DIR = "/home/adnan/x_bot/assets"
os.makedirs(ASSETS_DIR, exist_ok=True)

def get_font(size, bold=False):
    # Try system fonts
    font_paths = [
        "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf" if bold else "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/freefont/FreeSansBold.ttf" if bold else "/usr/share/fonts/truetype/freefont/FreeSans.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf" if bold else "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf"
    ]
    for path in font_paths:
        if os.path.exists(path):
            try:
                return ImageFont.truetype(path, size)
            except Exception:
                pass
    return ImageFont.load_default()

def wrap_text(text, font, max_width, draw):
    words = text.split()
    lines = []
    current_line = []
    for word in words:
        test_line = " ".join(current_line + [word])
        bbox = draw.textbbox((0, 0), test_line, font=font)
        w = bbox[2] - bbox[0]
        if w <= max_width:
            current_line.append(word)
        else:
            if current_line:
                lines.append(" ".join(current_line))
            current_line = [word]
    if current_line:
        lines.append(" ".join(current_line))
    return lines

def generate_polymarket_chart(
    question="Vibe coders spend more time recording screen demos than writing code",
    yes_pct=95,
    volume="$1,420,690 Vol",
    out_path="/tmp/meme_polymarket.png"
):
    width, height = 1200, 675
    img = Image.new("RGB", (width, height), color="#0F172A") # Slate 900
    draw = ImageDraw.Draw(img)

    # Gradient / Card border
    draw.rounded_rectangle([30, 30, width - 30, height - 30], radius=24, fill="#1E293B", outline="#334155", width=3)

    # Polymarket / Pitch Header
    font_badge = get_font(22, bold=True)
    draw.rounded_rectangle([60, 60, 240, 105], radius=12, fill="#0284C7")
    draw.text((80, 72), "POLYMARKET", fill="#FFFFFF", font=font_badge)

    font_live = get_font(20, bold=True)
    draw.text((270, 74), "• LIVE PREDICTION MARKET", fill="#38BDF8", font=font_live)

    font_vol = get_font(22, bold=True)
    draw.text((width - 260, 72), volume, fill="#94A3B8", font=font_vol)

    # Market Question
    font_q = get_font(38, bold=True)
    lines = wrap_text(question, font_q, width - 160, draw)
    y_text = 145
    for line in lines[:3]:
        draw.text((70, y_text), line, fill="#F8FAFC", font=font_q)
        y_text += 48

    # Odds Bar Container
    bar_y = y_text + 35
    bar_w = width - 140
    bar_h = 75

    draw.rounded_rectangle([70, bar_y, 70 + bar_w, bar_y + bar_h], radius=16, fill="#0F172A", outline="#475569", width=2)

    # Green YES segment
    yes_w = int(bar_w * (yes_pct / 100.0))
    draw.rounded_rectangle([70, bar_y, 70 + yes_w, bar_y + bar_h], radius=16, fill="#10B981")

    # Percentage Labels
    font_pct = get_font(36, bold=True)
    draw.text((95, bar_y + 16), f"YES {yes_pct}%", fill="#FFFFFF", font=font_pct)
    no_pct = 100 - yes_pct
    draw.text((70 + bar_w - 180, bar_y + 16), f"NO {no_pct}%", fill="#EF4444", font=font_pct)

    # Bottom Branding & Context
    font_sub = get_font(24, bold=False)
    draw.text((70, height - 90), "Resolution Source: AI Developer Workflow Census • @trypitchdotco", fill="#64748B", font=font_sub)

    font_brand = get_font(24, bold=True)
    draw.text((width - 240, height - 90), "trypitch.co", fill="#38BDF8", font=font_brand)

    img.save(out_path, "PNG", quality=95)
    print(f"[OK] Generated Polymarket Chart Meme: {out_path}")
    return out_path

def generate_comparison_card(
    top_text="Spending 4 hours re-recording on Screen Studio because your dog barked at 0:58",
    bottom_text="Rendering a flawless 60s narrated 1080p demo from your URL in 60 seconds with @trypitchdotco",
    out_path="/tmp/meme_comparison.png"
):
    width, height = 1200, 675
    img = Image.new("RGB", (width, height), color="#090D16")
    draw = ImageDraw.Draw(img)

    # Top Panel (Red / The Hard Way)
    draw.rounded_rectangle([40, 40, width - 40, 320], radius=20, fill="#1F1318", outline="#7F1D1D", width=2)
    draw.rounded_rectangle([65, 60, 160, 100], radius=8, fill="#DC2626")
    draw.text((80, 68), "MANUAL", fill="#FFFFFF", font=get_font(20, bold=True))

    font_text = get_font(30, bold=True)
    lines_top = wrap_text(top_text, font_text, width - 180, draw)
    y = 120
    for l in lines_top[:3]:
        draw.text((70, y), l, fill="#FECACA", font=font_text)
        y += 40

    # Bottom Panel (Emerald / The Pitch Way)
    draw.rounded_rectangle([40, 355, width - 40, 635], radius=20, fill="#0B1E19", outline="#065F46", width=2)
    draw.rounded_rectangle([65, 375, 230, 415], radius=8, fill="#059669")
    draw.text((80, 383), "TRYPITCH.CO", fill="#FFFFFF", font=get_font(20, bold=True))

    lines_bot = wrap_text(bottom_text, font_text, width - 180, draw)
    y = 435
    for l in lines_bot[:3]:
        draw.text((70, y), l, fill="#A7F3D0", font=font_text)
        y += 40

    img.save(out_path, "PNG", quality=95)
    print(f"[OK] Generated Comparison Meme: {out_path}")
    return out_path

def generate_progression_card(
    stages=None,
    out_path="/tmp/meme_progression.png"
):
    if stages is None:
        stages = [
            "Stage 1: \"I'll just record a quick 1-minute Loom for our launch\"",
            "Stage 2: Take 14... my microphone was on the wrong input",
            "Stage 3: Take 31... Slack notification popped up on the final second",
            "Stage 4: It's 3:30 AM, haven't shipped yet, still editing keyframes"
        ]

    width, height = 1200, 675
    img = Image.new("RGB", (width, height), color="#0F172A")
    draw = ImageDraw.Draw(img)

    # Title
    font_title = get_font(32, bold=True)
    draw.text((50, 40), "THE SCREEN RECORDING PIPELINE OF DOOM", fill="#F8FAFC", font=font_title)

    font_sub = get_font(20, bold=False)
    draw.text((50, 85), "Why founders lose 20+ hours every launch week • @trypitchdotco", fill="#94A3B8", font=font_sub)

    colors = ["#1E293B", "#334155", "#475569", "#7F1D1D"]
    text_colors = ["#F8FAFC", "#F1F5F9", "#E2E8F0", "#FECACA"]

    y_box = 135
    font_stage = get_font(24, bold=True)

    for i, stage in enumerate(stages[:4]):
        draw.rounded_rectangle([50, y_box, width - 50, y_box + 105], radius=14, fill=colors[i], outline="#64748B", width=1)
        lines = wrap_text(stage, font_stage, width - 140, draw)
        y_text = y_box + 22
        for l in lines[:2]:
            draw.text((80, y_text), l, fill=text_colors[i], font=font_stage)
            y_text += 32
        y_box += 125

    img.save(out_path, "PNG", quality=95)
    print(f"[OK] Generated Progression Meme: {out_path}")
    return out_path

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--template", choices=["polymarket", "comparison", "progression"], default="polymarket")
    parser.add_argument("--out", default="/tmp/meme.png")
    args = parser.parse_args()

    if args.template == "polymarket":
        generate_polymarket_chart(out_path=args.out)
    elif args.template == "comparison":
        generate_comparison_card(out_path=args.out)
    elif args.template == "progression":
        generate_progression_card(out_path=args.out)
