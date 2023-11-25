use android_activity::{
    input::{InputEvent, KeyAction, KeyEvent, KeyMapChar, MotionAction},
    AndroidApp, InputStatus, MainEvent, PollEvent,
};
use log::info;
use raw_window_handle::{
    AndroidDisplayHandle, HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle,
    RawWindowHandle,
};

mod sensor;
use raymarcher::RayMarcher;
use sensor::*;

pub struct AndroidWindow(ndk::native_window::NativeWindow);
impl AndroidWindow {
    pub fn new(window: ndk::native_window::NativeWindow) -> Self {
        Self(window)
    }
}
unsafe impl HasRawDisplayHandle for AndroidWindow {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        RawDisplayHandle::Android(AndroidDisplayHandle::empty())
    }
}
unsafe impl HasRawWindowHandle for AndroidWindow {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.0.raw_window_handle()
    }
}

struct Pointer {
    id: i32,
    pre: raymarcher::na::Point2<f32>,
    now: raymarcher::na::Point2<f32>,
}
impl Pointer {
    fn new(id: i32, pos: raymarcher::na::Point2<f32>) -> Self {
        Self {
            id,
            pre: pos,
            now: pos,
        }
    }
    fn update(&mut self) {
        self.pre = self.now;
    }
}

const GYRO_PERIOD: f32 = 1.0 / 60.0;

struct App {
    ray_marcher: Option<RayMarcher<AndroidWindow>>,
    pointer: Option<Pointer>,
    num_pointers: usize,
}
impl App {
    fn new() -> Self {
        Self {
            ray_marcher: None,
            pointer: None,
            num_pointers: 0,
        }
    }
    fn init_window(&mut self, native_window: ndk::native_window::NativeWindow) {
        let size = (native_window.width() as u32, native_window.height() as u32);
        let ray_marcher = pollster::block_on(raymarcher::RayMarcher::new(
            AndroidWindow(native_window),
            size,
            1.0 / 4.0,
        ));
        self.ray_marcher = Some(ray_marcher);
    }
    fn window_exists(&self) -> bool {
        self.ray_marcher.is_some()
    }
    fn resize(&mut self) {
        if let Some(rm) = &mut self.ray_marcher {
            let w = rm.wgpu_ctx.window.0.width() as u32;
            let h = rm.wgpu_ctx.window.0.height() as u32;
            rm.wgpu_ctx.resize((w, h));
        } else {
            log::error!("Attempted window resize when no window exists!");
        }
    }
    fn gyro(&mut self, gyro: [f32; 3]) {
        if let Some(rm) = &mut self.ray_marcher {
            if self.pointer.is_none() {
                rm.camera.pitch -= gyro[0] * GYRO_PERIOD;
                rm.camera.yaw -= gyro[1] * GYRO_PERIOD;
            }
        } else {
            log::error!("Attempted gyro action when no window exists!");
        }
    }
    fn update(&mut self, dt: f32) {
        if let Some(rm) = &mut self.ray_marcher {
            if let Some(pointer) = &mut self.pointer {
                let sensativity = 0.001;
                let d = (pointer.pre - pointer.now) * sensativity;
                rm.camera.pitch += d.y;
                rm.camera.yaw += d.x;
                pointer.update();
            } else {
                if self.num_pointers == 2 || self.num_pointers == 3 {
                    let speed = 0.5;
                    let v = rm
                        .camera
                        .rotation()
                        .to_homogeneous()
                        .transform_vector(&raymarcher::na::Vector3::z_axis());
                    let d = v * speed * dt;
                    if self.num_pointers == 2 {
                        rm.camera.pos += d;
                    }
                    if self.num_pointers == 3 {
                        rm.camera.pos -= d;
                    }
                }
            }

            rm.update(dt);
        } else {
            log::error!("Attempted unpdate when to window exists!");
        }
    }
    fn render(&self) {
        if let Some(rm) = &self.ray_marcher {
            match rm.render() {
                Ok(_) => {}
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => rm.wgpu_ctx.reconfigure_surface(),
                // The system is out of memory, we should probably quit
                // Err(wgpu::SurfaceError::OutOfMemory) => quit = true,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => log::error!("{:?}", e),
            }
        } else {
            log::error!("Attempted rendering when no window exists!");
        }
    }
}

#[no_mangle]
fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );

    let mut quit = false;
    let mut combining_accent = None;

    let mut rm_app = App::new();

    // info!("Sensor list:");
    // for sensor in sensor_manager.sensor_list() {
    // info!("Sensor: {:?}", sensor.get_name());
    // }
    // info!("Default Gyro Sensor: {:?}", default_gyro.get_name());

    let sensor_manager = SensorManager::new();
    let sensor_event_queue = sensor_manager.create_event_queue(5);

    // sensor_event_queue.register_sensor(
    //     &default_gyro,
    //     std::time::Duration::from_millis(100),
    //     std::time::Duration::from_millis(100),
    // );
    let default_gyro = sensor_manager
        .default_sensor(SensorType::Gyroscope)
        .unwrap();
    sensor_event_queue.enable_sensor(&default_gyro);
    sensor_event_queue.set_event_rate(
        &default_gyro,
        std::time::Duration::from_secs_f32(GYRO_PERIOD),
    );
    let mut timer = std::time::Instant::now();
    let mut dt = 0.0;
    while !quit {
        app.poll_events(Some(std::time::Duration::ZERO), |event| {
            match event {
                PollEvent::Wake => {}
                PollEvent::Timeout => {}
                PollEvent::Main(main_event) => {
                    match main_event {
                        // MainEvent::SaveState { saver, .. } => {
                        // saver.store("foo://bar".as_bytes());
                        // }
                        MainEvent::Pause => {}
                        // MainEvent::Resume { loader, .. } => {
                        // if let Some(state) = loader.load() {
                        //     if let Ok(uri) = String::from_utf8(state) {
                        //         info!("Resumed with saved state = {uri:#?}");
                        //     }
                        // }
                        // }
                        MainEvent::InitWindow { .. } => {
                            if let Some(native_window) = app.native_window() {
                                rm_app.init_window(native_window);
                            }
                        }
                        MainEvent::TerminateWindow { .. } => {
                            rm_app.ray_marcher = None;
                        }
                        MainEvent::WindowResized { .. } => {
                            rm_app.resize();
                        }
                        MainEvent::RedrawNeeded { .. } => {}
                        MainEvent::InputAvailable { .. } => {}
                        MainEvent::ConfigChanged { .. } => {
                            info!("Config Changed: {:#?}", app.config());
                        }
                        MainEvent::LowMemory => {}

                        MainEvent::Destroy => {
                            quit = true;
                        }
                        _ => { /* ... */ }
                    }
                }
                _ => {}
            }
        });
        match app.input_events_iter() {
            Ok(mut iter) => loop {
                if !iter.next(|event| {
                    match event {
                        InputEvent::KeyEvent(key_event) => {
                            info!("key action = {:?}", key_event.action());
                            let combined_key_char = character_map_and_combine_key(
                                &app,
                                key_event,
                                &mut combining_accent,
                            );
                            info!("KeyEvent: combined key: {combined_key_char:?}");
                        }
                        InputEvent::MotionEvent(motion_event) => {
                            rm_app.num_pointers = motion_event.pointer_count();
                            let action = motion_event.action();

                            match action {
                                MotionAction::Up
                                | MotionAction::PointerUp
                                | MotionAction::Cancel => {
                                    rm_app.pointer = None;
                                    rm_app.num_pointers -= 1;
                                }
                                _ => {
                                    if rm_app.num_pointers == 1 {
                                        let p = motion_event.pointer_at_index(0);
                                        let pos = raymarcher::na::Point2::new(p.x(), p.y());
                                        match action {
                                            MotionAction::Down | MotionAction::PointerDown => {
                                                rm_app.pointer =
                                                    Some(Pointer::new(p.pointer_id(), pos));
                                            }
                                            MotionAction::Move => {
                                                if let Some(pointer) = &mut rm_app.pointer {
                                                    if pointer.id == p.pointer_id() {
                                                        pointer.now = pos;
                                                    } else {
                                                        *pointer =
                                                            Pointer::new(p.pointer_id(), pos);
                                                    }
                                                } else {
                                                    rm_app.pointer =
                                                        Some(Pointer::new(p.pointer_id(), pos));
                                                }
                                            }
                                            _ => {}
                                        }
                                    } else {
                                        rm_app.pointer = None;
                                    }
                                }
                            }
                        }
                        InputEvent::TextEvent(state) => {
                            info!("Input Method State: {state:?}");
                        }
                        _ => {}
                    }

                    InputStatus::Unhandled
                }) {
                    break;
                }
            },
            Err(err) => {
                log::error!("Failed to get input events iterator: {err:?}");
            }
        }

        while let Some(event) = sensor_event_queue.get_event() {
            if rm_app.window_exists() {
                let gyro = event.gyro();
                rm_app.gyro(gyro);
            }
        }
        if rm_app.window_exists() {
            rm_app.update(dt);
            rm_app.render();
        }

        dt = timer.elapsed().as_secs_f32();
        timer = std::time::Instant::now();
    }
    sensor_event_queue.disable_sensor(&default_gyro);
}

/// Tries to map the `key_event` to a `KeyMapChar` containing a unicode character or dead key accent
///
/// This shows how to take a `KeyEvent` and look up its corresponding `KeyCharacterMap` and
/// use that to try and map the `key_code` + `meta_state` to a unicode character or a
/// dead key that be combined with the next key press.
fn character_map_and_combine_key(
    app: &AndroidApp,
    key_event: &KeyEvent,
    combining_accent: &mut Option<char>,
) -> Option<KeyMapChar> {
    let device_id = key_event.device_id();

    let key_map = match app.device_key_character_map(device_id) {
        Ok(key_map) => key_map,
        Err(err) => {
            log::error!("Failed to look up `KeyCharacterMap` for device {device_id}: {err:?}");
            return None;
        }
    };

    match key_map.get(key_event.key_code(), key_event.meta_state()) {
        Ok(KeyMapChar::Unicode(unicode)) => {
            // Only do dead key combining on key down
            if key_event.action() == KeyAction::Down {
                let combined_unicode = if let Some(accent) = combining_accent {
                    match key_map.get_dead_char(*accent, unicode) {
                        Ok(Some(key)) => {
                            info!("KeyEvent: Combined '{unicode}' with accent '{accent}' to give '{key}'");
                            Some(key)
                        }
                        Ok(None) => None,
                        Err(err) => {
                            log::error!("KeyEvent: Failed to combine 'dead key' accent '{accent}' with '{unicode}': {err:?}");
                            None
                        }
                    }
                } else {
                    info!("KeyEvent: Pressed '{unicode}'");
                    Some(unicode)
                };
                *combining_accent = None;
                combined_unicode.map(|unicode| KeyMapChar::Unicode(unicode))
            } else {
                Some(KeyMapChar::Unicode(unicode))
            }
        }
        Ok(KeyMapChar::CombiningAccent(accent)) => {
            if key_event.action() == KeyAction::Down {
                info!("KeyEvent: Pressed 'dead key' combining accent '{accent}'");
                *combining_accent = Some(accent);
            }
            Some(KeyMapChar::CombiningAccent(accent))
        }
        Ok(KeyMapChar::None) => {
            // Leave any combining_accent state in tact (seems to match how other
            // Android apps work)
            info!("KeyEvent: Pressed non-unicode key");
            None
        }
        Err(err) => {
            log::error!("KeyEvent: Failed to get key map character: {err:?}");
            *combining_accent = None;
            None
        }
    }
}

// /// Post a NOP frame to the window
// ///
// /// Since this is a bare minimum test app we don't depend
// /// on any GPU graphics APIs but we do need to at least
// /// convince Android that we're drawing something and are
// /// responsive, otherwise it will stop delivering input
// /// events to us.
// fn dummy_render(native_window: &ndk::native_window::NativeWindow) {
//     unsafe {
//         let mut buf: ndk_sys::ANativeWindow_Buffer = std::mem::zeroed();
//         let mut rect: ndk_sys::ARect = std::mem::zeroed();
//         ndk_sys::ANativeWindow_lock(
//             native_window.ptr().as_ptr() as _,
//             &mut buf as _,
//             &mut rect as _,
//         );
//         // Note: we don't try and touch the buffer since that
//         // also requires us to handle various buffer formats
//         ndk_sys::ANativeWindow_unlockAndPost(native_window.ptr().as_ptr() as _);
//     }
// }
