#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn coast_crate_version() -> u32 {
    1
}

#[cfg(target_arch = "wasm32")]
extern "C" {
    fn bb_command() -> i32;
    fn bb_option(index: i32) -> f32;
    fn bb_gamepad(index: i32) -> f32;
    fn bb_read(key: i32) -> f32;
    fn bb_store(key: i32, value: f32);
    fn bb_event(kind: i32, strength: f32);
    fn bb_telemetry(
        state: i32,
        speed: f32,
        lap: i32,
        position: i32,
        elapsed: f32,
        best: f32,
        countdown: f32,
        finish: f32,
        slip: f32,
        impact: f32,
        grounded: i32,
    );
}
pub fn command() -> i32 {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        bb_command()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        0
    }
}
pub fn option(index: i32) -> f32 {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        bb_option(index)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = index;
        0.
    }
}
pub fn pad(index: i32) -> f32 {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        bb_gamepad(index)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = index;
        0.
    }
}
pub fn read(key: i32) -> f32 {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        bb_read(key)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = key;
        -1.
    }
}
pub fn store(key: i32, value: f32) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        bb_store(key, value)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (key, value);
    }
}
pub fn event(kind: i32, strength: f32) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        bb_event(kind, strength)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (kind, strength);
    }
}
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub struct Telemetry {
    pub state: i32,
    pub speed: f32,
    pub lap: i32,
    pub position: i32,
    pub elapsed: f32,
    pub best: f32,
    pub countdown: f32,
    pub finish: f32,
    pub slip: f32,
    pub impact: f32,
    pub grounded: bool,
}
pub fn telemetry(t: Telemetry) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        bb_telemetry(
            t.state,
            t.speed,
            t.lap,
            t.position,
            t.elapsed,
            t.best,
            t.countdown,
            t.finish,
            t.slip,
            t.impact,
            i32::from(t.grounded),
        );
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = t;
    }
}
