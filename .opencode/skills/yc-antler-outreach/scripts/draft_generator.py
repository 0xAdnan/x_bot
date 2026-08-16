#!/usr/bin/env python3
"""
Anti-AI Outreach Draft Generator for YC & Antler Startups
Produces research-grounded, human-written Cold Emails and X DMs.

Strict Rules Enforced:
- Zero Em-Dashes (—) and Zero En-Dashes (–)
- Zero AI words (delve, elevate, leverage, seamless, robust, game-changer, revolutionary, landscape, etc.)
- Zero Emojis (no 🚀, 🔥, ✨, etc.)
- No Rule-of-Three Adjective lists
- Natural, practical, peer-to-peer founder register
"""

import re
from typing import Dict, Any

BANNED_WORDS = [
    "delve", "elevate", "unlock", "leverage", "seamless", "robust",
    "game-changer", "game changer", "revolutionary", "transformative",
    "landscape", "streamline", "navigate", "bespoke", "beacon", "testament",
    "in today's fast-paced", "hope this email finds you well", "happy to help",
    "great question", "cutting-edge", "state-of-the-art", "paradigm shift"
]

EMOJI_PATTERN = re.compile(
    r"[\U00010000-\U0010ffff]|[\u2600-\u27BF]|[\uD83C-\uDBFF\uDC00-\uDFFF]"
)

def clean_anti_ai_text(text: str) -> str:
    """Strip em-dashes, en-dashes, and emojis."""
    # Replace em-dashes and en-dashes with comma or period
    text = text.replace("—", ", ").replace("–", ", ").replace(" - ", ", ")
    # Strip emojis
    text = EMOJI_PATTERN.sub("", text)
    # Normalize double spaces
    text = re.sub(r" +", " ", text)
    return text.strip()

def validate_draft(text: str, max_chars: int = 0) -> list[str]:
    """Check for any violations of human-writing anti-AI rules and character limits."""
    violations = []
    if "—" in text or "–" in text:
        violations.append("Contains em-dash or en-dash")
    if EMOJI_PATTERN.search(text):
        violations.append("Contains emojis")
    lower = text.lower()
    for w in BANNED_WORDS:
        if w in lower:
            violations.append(f"Contains banned AI word: '{w}'")
    if max_chars > 0 and len(text) > max_chars:
        violations.append(f"Exceeds max character limit ({len(text)} > {max_chars})")
    return violations

def get_short_batch_code(batch: str) -> str:
    """Convert verbose batch strings to punchy 2-3 letter codes for mobile subject lines."""
    b = batch.lower()
    if "winter 2026" in b or "w26" in b: return "w26"
    if "summer 2025" in b or "s25" in b: return "s25"
    if "winter 2025" in b or "w25" in b: return "w25"
    if "fall 2024" in b or "f24" in b: return "f24"
    if "summer 2024" in b or "s24" in b: return "s24"
    if "winter 2024" in b or "w24" in b: return "w24"
    if "antler uk" in b: return "antler uk"
    if "antler singapore" in b or "antler sg" in b: return "antler sg"
    if "antler us" in b: return "antler us"
    if "antler germany" in b: return "antler germany"
    if "antler" in b: return "antler"
    if "yc" in b: return "yc"
    return "the batch"

def generate_outreach_drafts(startup: Dict[str, Any]) -> Dict[str, Any]:
    """
    Generate tailored cold email, concise 280-char X DM, and public mention tweet.
    Uses company name, founder name, product tagline/description, batch, and website.
    """
    name = startup.get("name", "there").strip()
    founders = startup.get("founders", [])
    founder_name = ""
    if founders:
        founder_name = founders[0].get("full_name", "").split()[0]
    elif startup.get("founder_name"):
        founder_name = startup.get("founder_name").split()[0]
    
    greeting_name = founder_name if founder_name else name
    batch = startup.get("batch", "recent batch").strip()
    website = startup.get("website", "").strip()
    desc = startup.get("one_liner") or startup.get("description") or "your product"
    desc_clean = desc.strip().rstrip(".")
    
    # 1. Cold Email Draft — Product-Tailored, Mobile-Safe 2-4 Word Subject Lines (< 35 chars)
    short_batch = get_short_batch_code(batch)
    clean_prod = name.lower()

    subject_options = [
        f"saw {clean_prod} in {short_batch}" if len(f"saw {clean_prod} in {short_batch}") <= 32 else f"saw {clean_prod}",
        f"{clean_prod} demo walkthrough" if len(f"{clean_prod} demo walkthrough") <= 32 else f"{clean_prod} demo",
        f"{greeting_name.lower()} / {clean_prod}" if len(f"{greeting_name.lower()} / {clean_prod}") <= 32 else f"{clean_prod} demo",
        f"idea for {clean_prod} demo" if len(f"idea for {clean_prod} demo") <= 32 else f"idea for {clean_prod}",
        f"{greeting_name.lower()}, quick question" if len(f"{greeting_name.lower()}, quick question") <= 32 else "quick question"
    ]
    email_subject = subject_options[0] # Default to high-converting trigger-specific subject line
    
    # Contextual Openers (NO icons, clean human text)
    if any(k in batch for k in ["Winter", "Summer", "Fall", "Spring", "W26", "S25", "W25", "S24", "W24"]):
        opener_gift = f"Hey {greeting_name},\n\nSaw {name} in the {batch} batch. {desc_clean.lower()} is super sharp."
        opener_consult = f"Hey {greeting_name},\n\nI saw {name} in the {batch} batch. {desc_clean.lower()} is a super sharp concept."
    elif "Antler" in batch:
        opener_gift = f"Hey {greeting_name},\n\nSaw {name} in {batch}. {desc_clean.lower()} is super sharp."
        opener_consult = f"Hey {greeting_name},\n\nI saw {name} in {batch}. {desc_clean.lower()} is a super sharp concept."
    else:
        opener_gift = f"Hey {greeting_name},\n\nChecked out {name}. {desc_clean.lower()} is super sharp."
        opener_consult = f"Hey {greeting_name},\n\nI checked out {name}. {desc_clean.lower()} is a super sharp concept."
        
    # Variant 1: Done-For-You Gift (~65 words, highest reply rate)
    email_body_gift = clean_anti_ai_text(f"""{opener_gift}

Recording launch videos or onboarding walkthroughs usually takes hours of retakes. I work on https://trypitch.co, which renders studio-quality narrated demo MP4s from text in minutes.

I put together a quick 30s video walkthrough of {name} for your launch. Mind if I send the video link over?

Best,
Adnan 
Co-Founder, Pitch
adnan@trypitch.co""")

    # Variant 2: 50-Word Quick Spear (Under 55 words)
    email_body_spear = clean_anti_ai_text(f"""Hey {greeting_name},

Saw {name} in {batch}. Love what you are building with {desc_clean.lower()}.

If you need a crisp 45s product demo or launch video for your landing page without spending days on screen recording, we automated the whole pipeline at https://trypitch.co.

Want me to spin up a quick walkthrough video for {name}?

Best,
Adnan 
Co-Founder, Pitch
adnan@trypitch.co""")

    # Variant 3: Founder Consultative (~75 words)
    email_body_consultative = clean_anti_ai_text(f"""{opener_consult}

prepping launch videos, demo day clips, or onboarding walkthroughs usually eats hours of manual recording and retakes.

i work on https://trypitch.co. you give it a plain written walkthrough or prompt and it renders a studio-quality, narrated demo mp4 in minutes, so you can iterate on videos whenever product features change.

happy to put together a demo walkthrough of {name} if you want one for your launch, or you can test it directly at https://trypitch.co. no pressure either way, best of luck with the build.

Best,
Adnan 
Co-Founder, Pitch
adnan@trypitch.co""")

    # Default to Done-For-You Gift variant
    email_body = email_body_gift

    # 2. X DM Draft (Strictly <= 280 characters)
    x_dm = f"hey {greeting_name.lower()}, congrats on {name} in {batch}. i work on @trypitchdotco. we turn written walkthroughs into studio demo videos in minutes, saves days during launch weeks. want me to make a quick 30s demo of {name} for your launch? or try it at trypitch.co"
    x_dm = clean_anti_ai_text(x_dm)
    
    if len(x_dm) > 280:
        # Compact fallback format
        x_dm = f"hey {greeting_name.lower()}, saw {name} in {batch}. i work on @trypitchdotco. we turn written walkthroughs into studio demo videos in minutes. want me to spin up a free 30s demo of {name} for your launch? or test it at trypitch.co"
        x_dm = clean_anti_ai_text(x_dm)

    # 3. Public X Mention / Tweet Hook (Strictly <= 280 characters, for when DMs are disabled)
    clean_handle = startup.get("primary_handle", "").replace("@", "")
    if clean_handle:
        handle_tag = f"@{clean_handle}"
    else:
        handle_tag = name

    public_tweet = f"congrats on shipping {handle_tag}. if you need a quick 45s launch video walkthrough for {name}, tag @trypitchdotco with your link and we will render one in minutes"
    public_tweet = clean_anti_ai_text(public_tweet)

    if len(public_tweet) > 280:
        public_tweet = f"congrats on shipping {handle_tag}. need a 45s launch video for {name}? tag @trypitchdotco with your link and we will render a demo in minutes"
        public_tweet = clean_anti_ai_text(public_tweet)

    # Check validations
    v_email = validate_draft(email_body)
    v_dm = validate_draft(x_dm, max_chars=280)
    v_tweet = validate_draft(public_tweet, max_chars=280)
    
    return {
        "email_subject": email_subject,
        "subject_options": subject_options,
        "email_body": email_body,
        "email_variants": {
            "gift": email_body_gift,
            "spear": email_body_spear,
            "consultative": email_body_consultative
        },
        "x_dm": x_dm,
        "public_tweet": public_tweet,
        "char_count_dm": len(x_dm),
        "char_count_tweet": len(public_tweet),
        "validation_email": v_email,
        "validation_dm": v_dm,
        "validation_tweet": v_tweet
    }

if __name__ == "__main__":
    test_startup = {
        "name": "Cardinal",
        "founders": [{"full_name": "Devi Jha"}],
        "batch": "Winter 2026",
        "website": "https://trycardinal.com",
        "one_liner": "Revenue Agents for GTM teams"
    }
    drafts = generate_outreach_drafts(test_startup)
    print("--- SUBJECT ---")
    print(drafts["email_subject"])
    print("\n--- EMAIL BODY ---")
    print(drafts["email_body"])
    print("\n--- X DM ---")
    print(drafts["x_dm"])
    print("\n--- VALIDATIONS ---")
    print("Email errors:", drafts["validation_email"])
    print("DM errors:", drafts["validation_dm"])
