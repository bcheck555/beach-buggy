# Beach Buggy · Coast Club

A top-down 3D island racer written in Rust and compiled to WebAssembly. Three tracks, four cars, three physics presets, five AI rivals, solo time trials, and synthesized sound. Everything is unlocked. Beach Loop is the coastal showcase, with sculpted shores, shallow water, beach landmarks, and a marked jump; all tracks offer following and whole-track cameras, a visible player marker, and reactive driving effects.

## Run locally

Requires Node.js 22+ and a WebGL-enabled desktop browser. Initial setup downloads Rust 1.90.0 and dependencies; the built game makes no external network requests.

```sh
cd /home/checkb/repos/github/beach-buggy
npm ci
npm run setup
npm run build
npm run dev
```

Open **http://localhost:4173**. Choose a track, car, driving feel, mode, and rival difficulty; click **LET’S RIDE**. `npm run dev` serves the latest build; rebuild after source changes. Change the local port with `PORT=8080 npm run dev`.

`npm run setup` installs Rust automatically on Linux x64. On other platforms, install [rustup](https://rustup.rs/) first, then run the same command. Build scripts locate Cargo in `~/.cargo/bin`; for direct Cargo commands, add that directory to your PATH. Native Rust is used for simulation tests; the playable interface targets the browser.

## Controls

| Action | Keyboard | Standard gamepad |
|---|---|---|
| Accelerate | W / ↑ | Right trigger |
| Brake, then reverse | S / ↓ | Left trigger |
| Steer | A/D / ←/→ | Left stick |
| Handbrake | Space | A / bottom face button |
| Reset to last checkpoint | R | Y / top face button |
| Pause / resume | Escape | Start / Menu |
| Whole-track zoom in / out | + or = / − (also numeric keypad) | — |
| Fit entire track | 0 (also numeric keypad) | — |

Use mouse or keyboard Tab/Enter for garage and pause menus. Connect a standard-mapped controller before or during a race; disconnecting returns to keyboard input. Browser security may require a button press before detecting a controller. Click or press a key once to enable audio. Sound level is available in pause/results; the header toggles mute. Switching browser tabs or windows pauses racing.

## Tracks, cars, and modes

- **Beach Loop:** Wide sandy turns and a launch ramp; a good starting point.
- **Rocky Coast:** Narrower elevated terrain, rock surfaces, and railings, with sculpted shores and rounded coastal rocks.
- **Volcano Run:** The skill course: linked curves, two braking hairpins, a gradual outer climb, and an elevated crossing above a tunnel through a volcano. A modest jump sits on the long outer straight, with room to land before the next bend; airtime varies by car and speed. Brake before the marked corners; runoff gives room to recover. The volcano opens a cutaway while you drive beneath it so your car and halo stay visible in either camera view. Tunnel walls and elevated barriers remain active at both scenery settings. The crater and lava are scenery, without damage.

**Sunrunner** balances grip and power. **Sand Flea** is light and responsive. **Riptide** accelerates hardest but needs careful throttle and braking. **Dune Hauler** is heavy, stable, and loses less traction off-road.

**Arcade** has stronger grip and steering stability. **Balanced** permits more drifting. **Realistic** reduces assistance and traction: brake before turning and respect momentum. This is simcade physics, not a full tire or damage simulator.

**Quick Race** is three laps against five AI drivers. **Time Trial** is three solo laps, displaying the current lap timer. Gates must be crossed in order and in the forward direction. The minimap shows every car. Reset preserves elapsed time and checkpoint order; pause stops the simulation clock.

**Easy / Normal / Hard** changes rival pace independently of driving feel. Normal is the default. Rivals brake for occupied lanes and hold a passing line before merging back; they use the same vehicle physics and no rubber-banding. A difficulty change during a race takes effect on the next start or restart.

**High / Low quality** controls decorative detail and effect budgets without changing physics. **Reduced motion** follows the system preference on first use, can be overridden, and removes camera shake, speed-driven zoom, and celebratory motion. Quality and reduced motion apply immediately. All settings persist when browser storage is available.

**Camera view** in garage setup and pause offers **Follow car** (default) or **Whole track**. Whole track centers the circuit in the browser viewport in a fixed, slightly angled overhead view, without panning, shake, or speed-driven zoom. Use **+ / −** or the HUD/pause buttons to zoom from **50% to 200%**, in 10-point steps. **0 / Fit** restores 100%, showing the complete circuit and recovery area with a small border. Zooming in may crop track edges; your halo still identifies your car. Each track remembers its own zoom across restarts and sessions when storage is available. Resizing refits the viewport while retaining your multiplier. Controls work during racing or pause in Whole track mode; browser Ctrl/Command zoom shortcuts remain available. Camera and zoom changes apply immediately without an animated transition and work independently of Reduced motion. Follow car ignores manual zoom, and Whole track hides the redundant minimap.

Best completed laps are saved by track/car/physics combination in the browser's local storage and shared across both modes. Settings persist too. No account or server is required; private mode or blocked storage still permits play without saving. Clearing site data clears records.

Results highlight a new personal best and the improvement over the record held when the race started. Beach Loop and Rocky Coast retain their records. Volcano Run uses fresh record keys because its layout replaces Lagoon Run; old Lagoon Run records remain stored but are not used for the new course. Existing car, driving feel, camera, and track selections remain compatible.

## Development and validation

```sh
npm run check
npm run build
npx playwright install chromium firefox
npm run test:chromium
npm run test:firefox -- --headed
```

On a displayless Linux machine with Mesa and Xvfb, use:

```sh
xvfb-run -a npm run test:firefox -- --headed
```

Firefox's headless WebGL support depends on the host drivers; Xvfb provides the display used in validation. On hosts without an audio output device, the Firefox test verifies graph initialization and graceful operation rather than audible output. Chromium tests require a running audio context. Browser tests emulate a standard controller; physical controller hardware still benefits from a local check.

`npm run check` runs 30 simulation/camera tests and five renderer tests. They cover all **36 track/car/preset combinations**, **108 complete six-car grids** across rival difficulties, traffic behavior, feedback/reset state, ordered checkpoints, standings, records, braking, suspension, collisions, and camera framing/zoom. Volcano regressions additionally check crossing levels, recovery, tunnel boundaries, full braking approaches/exits, the gradual bridge climb, bounded jump height and airtime across approach lines and timings, stable road-height queries near the ramp edges, terrain clearance, cutaway clipping, and geometry budgets. Browser tests cover garage persistence, legacy saves, quality/reduced-motion/camera settings, keyboard/gamepad input, sound, pause/reset/restart, complete races and trials, record isolation, per-track zoom, narrow HUD layout, and renderer captures. Isolated presentation tests inject telemetry to check messages, controls, and audio gains; race completion and renderer projection checks use the simulation.

`?autodrive=1` enables an accelerated demo driver for browser race-completion tests. It drives through the same simulation and checkpoints and **never saves personal records**. It is intentionally absent from the regular UI.

Playwright writes screenshots, traces, and render samples under `test-results/`. The 1080p sample reports renderer, average RAF rate, and 95th-percentile frame time. Software-rendered CI numbers do not establish integrated-GPU performance; smooth 1080p on representative hardware remains a hardware validation target.

## Code layout

- `src/sim.rs`: Dependency-free simulation logic, vehicle/track data, AI, collisions, checkpoints, race timing, and tests. Fixed 60 Hz stepping with accumulated render time.
- `src/main.rs`, `src/render.rs`, `src/camera.rs`: Macroquad game loop, interpolated positions/headings/heights, following camera with bounded look-ahead, cached whole-track framing, cached scenery/car geometry, reusable mesh buffers, batched effects, and minimap. Particle budgets are 384 on High and 160 on Low.
- `src/bridge.rs`, `web/bridge.js`: Typed numeric WASM imports for controller input, browser audio, telemetry, and storage. HTML/CSS provide accessible garage, HUD, and pause/results controls.
- `scripts/`: Toolchain setup, locked release build, local static server, and verification.

Simulation coordinates use X/Z on screen with height as Y; the standalone simulation represents the ground plane with `V2`. Track geometry comes from sampled closed Catmull–Rom splines with terrain heights and ordered gates. Suspension follows terrain through a spring response, releasing the vehicle into gravity at ramp exits; lateral grip, yaw limits, and weight-transfer pose vary by car and preset. Railings and cars apply velocity impulses; severe off-track excursions recover at the last gate. The renderer interpolates previous/current simulation state and uses face colors for inexpensive lighting.

To add content, extend the car specs or track controls in the simulation and update the matching garage labels/options. Keep the bridge indices and record-key allocation in sync if expanding beyond the current content counts.

## Static distribution

Publish the contents of `dist/` to any static HTTP host with `.wasm` served as `application/wasm`. No backend, COOP/COEP headers, CDN, third-party assets, or API keys are required. The build vendors the loader from the exact locked Miniquad crate. Ship the generated `licenses/` directory and `THIRD_PARTY_NOTICES.txt` alongside the game.

Original game code, procedural geometry, UI art, and audio are MIT licensed. [Macroquad](https://github.com/not-fl3/macroquad), [Miniquad](https://github.com/not-fl3/miniquad), and their dependencies retain their respective licenses, included by the build.
