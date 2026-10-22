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

Use mouse or keyboard Tab/Enter for garage and pause menus. Connect a standard-mapped controller before or during a race; disconnecting returns to keyboard input. Browser security may require a button press before detecting a controller. Click or press a key once to enable audio. Sound level is available in pause/results; the header toggles mute. Switching browser tabs or windows pauses racing.

## Tracks, cars, and modes

- **Beach Loop:** Wide sandy turns and a launch ramp; a good starting point.
- **Rocky Coast:** Narrower elevated terrain, rock surfaces, and railings.
- **Lagoon Run:** A central lagoon, wooden bridges with more grip, tight turns, and a ramp.

**Sunrunner** balances grip and power. **Sand Flea** is light and responsive. **Riptide** accelerates hardest but needs careful throttle and braking. **Dune Hauler** is heavy, stable, and loses less traction off-road.

**Arcade** has stronger grip and steering stability. **Balanced** permits more drifting. **Realistic** reduces assistance and traction: brake before turning and respect momentum. This is simcade physics, not a full tire or damage simulator.

**Quick Race** is three laps against five AI drivers. **Time Trial** is three solo laps, displaying the current lap timer. Gates must be crossed in order and in the forward direction. The minimap shows every car. Reset preserves elapsed time and checkpoint order; pause stops the simulation clock.

**Easy / Normal / Hard** changes rival pace independently of driving feel. Normal is the default. Rivals brake for occupied lanes and hold a passing line before merging back; they use the same vehicle physics and no rubber-banding. A difficulty change during a race takes effect on the next start or restart.

**High / Low quality** controls decorative detail and effect budgets without changing physics. **Reduced motion** follows the system preference on first use, can be overridden, and removes camera shake, speed-driven zoom, and celebratory motion. Quality and reduced motion apply immediately. All settings persist when browser storage is available.

**Camera view** in garage setup and pause offers **Follow car** (default) or **Whole track**. Whole track keeps the entire circuit in a fixed, slightly angled overhead view, without panning, shake, or speed-driven zoom. It frames the circuit tightly around the HUD and hides the redundant minimap. Runoff uses the outer screen so cars remain visible until recovery. Cars appear smaller, especially in narrow windows; your halo still identifies your car. Switching applies immediately without an animated transition, persists when storage is available, and works independently of Reduced motion.

Best completed laps are saved by track/car/physics combination in the browser's local storage and shared across both modes. Settings persist too. No account or server is required; private mode or blocked storage still permits play without saving. Clearing site data clears records.

Results highlight a new personal best and the improvement over the record held when the race started. Existing records remain compatible: the showcase update preserves the drivable layout, checkpoints, vehicle handling, and record keys.

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

The 21 Rust tests cover all **36 track/car/preset combinations**, **108 complete six-car grids** across rival difficulties, 27 stationary-start avoidance configurations, blocked passing, side-by-side racing, difficulty/handling isolation, feedback/reset state, ordered and directional checkpoints, out-of-bounds gate rejection, lap/finish/standings rules, record keys, acceleration, steering profiles, braking/reverse, tire slip/handbrake, suspension launch/landing, collision momentum, render-rate independence, and stationary camera framing across tracks and aspect ratios. The 16 browser tests per browser cover garage persistence, legacy saves, quality/reduced-motion/camera settings, keyboard/gamepad input, disconnect fallback, sound initialization, pause/reset/restart, complete races and trials, storage errors, whole-track captures at desktop/narrow sizes, and 1080p captures. Isolated presentation tests inject telemetry to check lap/landing/PB messages, camera controls, and audio gains; actual race completion uses the simulation.

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
