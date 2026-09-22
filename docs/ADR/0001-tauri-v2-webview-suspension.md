# ADR 0001: Selection of Tauri v2 with Webview Suspension over Slint & Native GUI Split

## Status
Accepted

## Context
9Gauge requires a cross-platform (macOS, Windows, Linux) system tray HUD companion that runs 24/7 in the background with minimal resource consumption (<18MB RAM idle, 0% CPU) while delivering the sophisticated frosted-glass visual standard of the AETER Monitor Archetype (`monitor.aeter.my.id`).

Three architectural paths were evaluated:
1. **Tauri v2 (Rust + Suspended Webview)**
2. **Slint / Egui (Pure Rust Native Retained/Immediate Mode GUI)**
3. **Platform Split (Swift/AppKit for macOS + Rust/Win32 for Windows)**

## Decision
We choose **Tauri v2 with Webview Suspension Governance**.

## Rationale & Trade-Offs

### Why Not Pure Rust Native (Slint / Egui)?
- **Sub-Tree Glass Incompatibility:** Slint's software/skia renderer can apply OS-level window acrylic/vibrancy, but cannot render nested CSS `backdrop-filter: blur(16px)` inside card containers without custom GLSL shaders.
- **Maintenance Cost:** Recreating complex responsive data layouts, segmented progress bars, and tabular micro-typography in Slint markup requires significant reinventing of primitives already perfected in HTML/CSS.
- **Memory Reality:** While Slint achieves ~12–20MB active RAM, Tauri v2 with Webview Suspension achieves ~14–18MB idle RAM by decoupling and sleeping the Webview when the window is hidden.

### Why Not Platform Split (Swift + Win32)?
- Triples the code maintenance surface. Any telemetry schema updates from 9Router would require updates across Swift, C++/Win32, and Linux GTK.

### Tauri v2 Suspension Optimization
To counteract WebKit/Chromium background compositing power draw:
- **macOS:** Explicitly unmounts or disables rendering loop when hidden (`visible: false`), preventing 120Hz ProMotion GPU recompositing (~620mW -> <50mW).
- **Windows:** Invokes `ICoreWebView2_3::TrySuspend` when the popover is dismissed, trimming working set RAM to ~15MB.
