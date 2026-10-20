# Validation — October 19, 2026

Validated on Linux x86_64, Rust 1.90.0, Node.js 22.22.2, Playwright 1.56.1.

| Check | Result |
|---|---|
| Rust formatting | Passed |
| Clippy, native all targets, warnings denied | Passed |
| Clippy, WASM target, warnings denied | Passed |
| Native simulation tests | 13 passed |
| Track/car/preset driving matrix | All 36 combinations finished three laps |
| Six-car AI grids | All cars finished on all 9 track/preset combinations |
| JavaScript syntax | Passed |
| Locked release WASM build | Passed; 430,307-byte WASM |
| Chromium 141 browser integration | 6 passed |
| Firefox 142 browser integration, headed under Xvfb | 6 passed |
| Staged whitespace and conflict check | Passed |

Browser coverage includes settings and records, keyboard and emulated standard gamepad input, disconnect fallback, pause/resume/reset/restart, complete races and solo trials, result screens, blocked storage, demo record protection, and 1080p captures. Garage, race, and results captures were visually reviewed. The release contains a local loader, local styles/scripts, and 37 dependency license directories; no external runtime script/style URLs.

Firefox reports no audio output devices on this machine. Its audio graph initializes and the game remains playable, but audible Firefox playback was not verified. Chromium reaches a running audio context. No physical controller or human listening test was available.

## 1080p software render samples

120 animation frames sampled after the countdown with a six-car race at 1920 × 1080. These are measurements of the CI-style software graphics environment, not an integrated GPU benchmark.

| Browser / renderer | Mean RAF rate | 95th-percentile frame time |
|---|---:|---:|
| Chromium / ANGLE SwiftShader Vulkan | 10.26 fps | 150 ms |
| Firefox / Mesa llvmpipe | 23.94 fps | 51.5 ms |

Smooth 1080p on representative integrated-GPU hardware remains unverified. Use hardware-accelerated WebGL for normal play. Browser tests use an accelerated demo driver; Rust tests exercise the same unaccelerated fixed-step simulation. Test captures/traces are generated under the ignored `test-results/` directory and replaced on subsequent browser runs.
