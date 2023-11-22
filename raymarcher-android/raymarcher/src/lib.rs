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

const SCALE: u32 = 2;

#[no_mangle]
fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );

    let mut quit = false;
    let mut ray_marcher = None;
    let mut combining_accent = None;

    let mut touches_down = 0;
    let mut last_gyro = [0f32; 3];

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
        std::time::Duration::from_micros(1000 / 60 * 1000),
    );
    let mut timer = std::time::Instant::now();
    let mut dt = 0.0;
    while !quit {
        // info!("min delay: {} us", default_gyro.min_delay().as_micros());
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
                            let native_window = app.native_window();
                            if let Some(native_window) = native_window {
                                let size = (
                                    native_window.width() as u32 / SCALE,
                                    native_window.height() as u32 / SCALE,
                                );
                                ray_marcher = Some(pollster::block_on(
                                    raymarcher::RayMarcher::new(AndroidWindow(native_window), size),
                                ));
                                if let Some(ray_marcher) = &mut ray_marcher {
                                    ray_marcher.camera.pos =
                                        raymarcher::na::Point3::new(0.0, 0.0, 3.0);
                                    ray_marcher.camera.yaw = std::f32::consts::PI;
                                }
                            }
                        }
                        MainEvent::TerminateWindow { .. } => {
                            ray_marcher = None;
                            touches_down = 0;
                        }
                        MainEvent::WindowResized { .. } => {
                            if let Some(ray_marcher) = &mut ray_marcher {
                                let w = ray_marcher.wgpu_ctx.window.0.width() as u32 / SCALE;
                                let h = ray_marcher.wgpu_ctx.window.0.height() as u32 / SCALE;
                                ray_marcher.wgpu_ctx.resize((w, h));
                            }
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
                        InputEvent::MotionEvent(motion_event) => match motion_event.action() {
                            MotionAction::Up => {
                                let pointer = motion_event.pointer_index();
                                let pointer = motion_event.pointer_at_index(pointer);
                                let x = pointer.x();
                                let y = pointer.y();

                                if x < 200.0 && y < 200.0 {
                                    info!("Requesting to show keyboard");
                                    app.show_soft_input(true);
                                }

                                touches_down -= 1;
                            }
                            MotionAction::Down => {
                                touches_down += 1;
                            }
                            MotionAction::Cancel => {
                                touches_down = 0;
                            }
                            _ => {}
                        },
                        InputEvent::TextEvent(state) => {
                            info!("Input Method State: {state:?}");
                        }
                        _ => {}
                    }

                    // info!("Input Event: {event:?}");
                    InputStatus::Unhandled
                }) {
                    // info!("No more input available");
                    break;
                }
            },
            Err(err) => {
                log::error!("Failed to get input events iterator: {err:?}");
            }
        }

        while let Some(event) = sensor_event_queue.get_event() {
            // info!("HAS EVENTS GYRO: {:?}", event.gyro());
            last_gyro = event.gyro();
        }

        if let Some(ray_marcher) = &mut ray_marcher {
            // info!("Rendering...");

            if touches_down <= 0 {
                ray_marcher.camera.pitch -= last_gyro[0] * dt;
                ray_marcher.camera.yaw -= last_gyro[1] * dt;
                ray_marcher.camera.pitch = ray_marcher
                    .camera
                    .pitch
                    .clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);
            }

            ray_marcher.update(dt);
            match ray_marcher.render() {
                Ok(_) => {
                    // info!("Done Rendering");
                }
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => ray_marcher.wgpu_ctx.reconfigure_surface(),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => quit = true,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => log::error!("{:?}", e),
            }
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
