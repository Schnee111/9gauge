# Visual Design Specification: AETER Monitor Archetype

## Project: 9Gauge
**Design Archetype:** AETER Monitor (`monitor.aeter.my.id`)  
**Visual Style:** Utilitarian Premium Glassmorphism (Calm, Precision-Engineered, Zero-Slop)  

---

## 1. Design Philosophy & Negative Constraints

### Non-Negotiable Rules
- ❌ **DILARANG KERAS Sci-Fi / Cyberpunk Slop:** No fake telemetry badges (`v2.4-PRO`, `GEO-02`), no cyan/magenta neon glows, no fake radar pulses, no military hazard stripes.
- ❌ **No Washed-Out Flat Dark Glass:** Glassmorphism must have tactile presence with a 68%–82% substrate opacity, crisp specular double-bezel, and clean refraction.
- ❌ **No Shifting Numbers:** All numerical values, token counts, timestamps, and currency amounts MUST use `JetBrains Mono` with `font-feature-settings: "tnum"` (tabular figures) so columns never wobble when counters update.

---

## 2. Design Tokens (CSS / Tailwind)

### 2.1 Color Palette
```css
:root {
  /* Ambient Void Substrates */
  --bg-app-dark: rgba(13, 15, 20, 0.82);
  --bg-app-light: rgba(255, 255, 255, 0.78);
  
  /* Frosted Glass Layers */
  --glass-card: rgba(255, 255, 255, 0.04);
  --glass-card-hover: rgba(255, 255, 255, 0.08);
  
  /* Double-Bezel Specular Highlights */
  --bezel-outer: 1px solid rgba(255, 255, 255, 0.12);
  --bezel-inner: inset 0 1px 0 rgba(255, 255, 255, 0.18);
  
  /* Status Inks (High Fidelity, Muted Tone) */
  --ink-healthy: #2e9e6b;   /* Emerald */
  --ink-warning: #d99a2b;   /* Amber */
  --ink-critical: #d4553f;  /* Rose */
  --ink-neutral: #717682;   /* Slate */
  
  /* Data Segment Accents */
  --token-prompt: #60a5fa;   /* Input Blue */
  --token-output: #a78bfa;   /* Completion Purple */
  --token-cache: #34d399;    /* Cache Hit Mint */
}
```

### 2.2 Typography Scale
- **Display & Interface:** `Plus Jakarta Sans`, font-weight 400, 500, 600.
  - Title / Header: `12px`, letter-spacing `0.05em`, uppercase, font-weight 600.
  - Subheaders / Labels: `11px`, font-weight 500, text-muted.
- **Data & Telemetry:** `JetBrains Mono`, font-feature-settings `"tnum"`.
  - Hero Counter: `22px`, font-weight 600, tabular-nums.
  - Provider Rows / Stats: `12px`, font-weight 500, tabular-nums.
  - Micro-Feed Log: `10.5px`, font-weight 400, tabular-nums.

---

## 3. Component Hierarchy (Popover Layout)

Width: `360px` | Fixed Height: `Auto` (Clamped to max 480px with subtle scroll).

```
┌─────────────────────────────────────────────────────────────┐
│ 1. HEADER & HOST BREADCRUMB                                 │
│    [✦ 9GAUGE  /  TELEMETRY]               [Localhost (20128) ▾] │
├─────────────────────────────────────────────────────────────┤
│ 2. HERO TOKEN METER                                         │
│    1,248,520 TOKENS CONSUMED TODAY            Est. $0.42    │
│    [ Prompt: 890k ] [ Output: 148k ] [ Cached: 210k (16%) ] │
│    [███████████████████████████████████░░░░░░░░░░░░░░░░░░░] │
│    Throughput: 42.8k tok/h · 3.2 req/m                      │
├─────────────────────────────────────────────────────────────┤
│ 3. UPSTREAM PROVIDER BREAKDOWN                              │
│    • Google AG (OAuth Pool)                                 │
│      820,400 tok (65.7%) · 48 req             [ ● Healthy ] │
│    • OpenRouter                                             │
│      284,100 tok (22.7%) · $0.38              [ $14.20 left]│
│    • Kiro (Amazon Q)                                        │
│      108,020 tok (8.6%) · 18 req              [ Resets 4h ] │
│    • DeepSeek                                               │
│      36,000 tok (2.9%) · ¥1.15               [ ¥28.50 left]│
├─────────────────────────────────────────────────────────────┤
│ 4. REAL-TIME ACTIVITY FEED (LAST 3 PULSES)                  │
│    14:52:10 · gpt-5.3-codex ──► 200 OK (1.2s · 4.2k tok)       │
│    14:51:44 · ag/gemini-3.8 ──► 200 OK (0.8s · 1.1k tok)       │
│    14:50:02 · openrouter    ──► 200 OK (2.4s · 8.9k tok)       │
├─────────────────────────────────────────────────────────────┤
│ 5. ACTION FOOTER & RTK BADGE                                │
│    ✦ RTK Saved: 185k tok (-14.8%)                           │
│    [ Dashboard ]          [ Copy URL ]             [ Quit ] │
└─────────────────────────────────────────────────────────────┘
```

---

## 4. Tactile Micro-Interactions
- **Glass Card Hover:** Subtle brightness increase (`var(--glass-card-hover)`) with `120ms ease-out` transition.
- **Live Dot Pulse:** The tray health dot features a continuous 3-second breathing keyframe animation on healthy states, turning solid amber on rate-limit warnings.
- **Copy Feedback:** Clicking `[ Copy URL ]` triggers an instant visual swap to `✓ Copied!` in Emerald ink for 1.5 seconds.
