# Validation — October 19, 2026

## Tighter whole-track framing — October 21, 2026

- Reduced circuit padding and reserved HUD bands to enlarge the road. Fit the recovery envelope separately to the full viewport, preserving off-track car visibility.
- Regression reproduced before the fix: Beach Loop occupied only 69.3% of usable height at 1920×1080. Tests now require at least 85% road coverage along one usable dimension in landscape/square, or 83% in portrait where recovery visibility can limit zoom.
- All three tracks tested at seven viewport sizes, including 375×812 and 320×568. Padded circuits stay inside HUD-safe bounds; projected car-body corners remain on screen immediately before recovery. Stationary camera checks still pass.
- `npm run check` passed all 21 Rust tests, formatting, native/WASM Clippy and JavaScript syntax. After the recovery refinement, both camera tests and native/WASM Clippy passed again. Release build passed.
- Final targeted browser tests: Chromium 3/3 and Firefox headed under Xvfb 3/3 passed, including renderer stability, reset, switching, and desktop/narrow captures. A trial using headed Chromium reported resource-404 console errors; the documented headless Chromium configuration passed.
- Desktop/narrow screenshots inspected in both browsers. An independent reviewer using the inherited model found the initial narrow-screen runoff clipping; the full-viewport constraint and regression checks resolve that finding.

## Whole-track camera — October 21, 2026

- `npm run check`: passed formatting, native/WASM Clippy, JavaScript syntax checks, and all 21 Rust tests. Camera tests cover all three padded circuits at five viewport sizes and unchanged framing during vehicle motion/reset.
- `npm run build`: passed; static release rebuilt with the camera option.
- Chromium: full 16-test suite passed; all six camera regressions passed again after final mobile layout and renderer assertions.
- Firefox: full 16-test suite passed headed under Xvfb. Headless Firefox run was stopped after failing to initialize the game; headed execution is the documented environment fallback.
- Actual WASM/WebGL tests confirm the 3D projection stays identical while driving/resetting, changes in Follow car mode, and restores exactly in Whole track mode on all three tracks. Settings tests cover defaults, invalid/legacy values, synchronization, persistence, independent reduced motion, and blocked storage.
- All three circuits were visually inspected in Chromium at 1440×900 and 600×900; Firefox desktop/narrow samples were also inspected. Full circuits and player halos remain visible without HUD overlap. Expanded mobile garage layout was checked at widths 320, 390, and 600px; a discovered overlap was fixed with document flow and covered by a regression assertion.
- Two agents using the inherited session model handled browser settings/tests and independent review. Review findings (mobile overlap and missing renderer integration assertions) were resolved before committing.

The earlier validation below describes the pre-camera release. Existing hardware-performance and physical-controller limitations still apply.

Validated on Linux x86_64, Rust 1.90.0, Node.js 22.22.2, Playwright 1.56.1.

| Check | Result |
|---|---|
| Rust formatting | Passed |
| Clippy, native all targets, warnings denied | Passed |
| Clippy, WASM target, warnings denied | Passed |
| Native simulation tests | 19 passed |
| Track/car/preset driving matrix | All 36 combinations finished three laps |
| Six-car AI grids | All cars finished on all 108 track/car/preset/difficulty combinations |
| Stationary-start avoidance | 27 track/preset/difficulty combinations; player displacement below 0.1 m |
| Traffic behavior | Blocked-pass braking, passing commitment, and side-by-side separation passed |
| Difficulty and feedback | Player handling unchanged; throttle/load/surface/landing/reset checks passed |
| JavaScript syntax | Passed |
| Locked release WASM build | Passed; 453,040-byte WASM |
| Chromium 141 browser integration | 10 passed; garage/render checks repeated after visual refinement |
| Firefox 142 browser integration, headed under Xvfb | 10 passed; final renderer check repeated |
| Staged whitespace and conflict check | Passed |

Browser coverage includes settings and records, keyboard and emulated standard gamepad input, disconnect fallback, pause/resume/reset/restart, complete races and solo trials, result screens, blocked storage, demo record protection, and 1080p captures. New coverage checks legacy settings/records, difficulty/quality persistence, system and explicit reduced-motion preferences, lap/final-lap cues, first records and personal-best deltas, bounded audio gains, and distinct landing/collision sounds. Presentation/audio tests inject telemetry into the browser adapter; full race/trial completion tests run the simulation.

Garage, expanded settings, race, pause, jump, and results captures were visually reviewed. A keyboard-driven browser check kept the player stationary after GO, then exercised acceleration and steering without console errors. The accelerated demo produced a real airborne state for the jump review; the final capture confirms a continuous ground marker across the ramp, with no browser errors. No human competitiveness study or physical controller test was available. The release contains a local loader, local styles/scripts, and 37 dependency license directories; no external runtime script/style URLs.

Firefox reports no audio output devices on this machine. Its audio graph initializes and the game remains playable, but audible Firefox playback was not verified. Chromium reaches a running audio context. No physical controller or human listening test was available.

## 1080p software render samples

120 animation frames sampled after the countdown with a six-car race at 1920 × 1080. These are short samples of the CI-style software graphics environment, not an integrated GPU benchmark.

| Browser / renderer | Quality | Mean RAF rate | 95th-percentile frame time |
|---|---|---:|---:|
| Chromium / ANGLE SwiftShader Vulkan | High | 11.93 fps | 150 ms |
| Chromium / ANGLE SwiftShader Vulkan | Low | 12.51 fps | 133.4 ms |
| Firefox / Mesa llvmpipe | High | 18.82 fps | 67.62 ms |

The initial version's recorded High-equivalent samples were Chromium 10.26 fps / 150 ms and Firefox 23.94 fps / 51.5 ms. The final Firefox software sample is slower than that baseline; all software samples remain well below 60 fps. Low reduces the detail/effect budget. Cached car geometry and batched, bounded particles remove repeated geometry construction, but these short samples do not establish a hardware speedup.

The 60 fps at 1080p target on a representative integrated-GPU laptop remains unverified. Use hardware-accelerated WebGL for normal play. Audible Firefox output and human driving feel still require local checks. Browser completion tests use an accelerated demo driver; Rust tests exercise the same unaccelerated fixed-step simulation. Test captures/traces are generated under the ignored `test-results/` directory; the final Chromium and Firefox captures use separate subdirectories.
