#![allow(dead_code)]

use std::{ffi::CStr, ptr::NonNull};

#[repr(i32)]
pub enum SensorType {
    Invalid = ndk_sys::ASENSOR_TYPE_INVALID,
    Accelerometer = ndk_sys::ASENSOR_TYPE_ACCELEROMETER,
    MagneticField = ndk_sys::ASENSOR_TYPE_MAGNETIC_FIELD,
    Gyroscope = ndk_sys::ASENSOR_TYPE_GYROSCOPE,
    Light = ndk_sys::ASENSOR_TYPE_LIGHT,
    Pressure = ndk_sys::ASENSOR_TYPE_PRESSURE,
    Proximity = ndk_sys::ASENSOR_TYPE_PROXIMITY,
    Gravity = ndk_sys::ASENSOR_TYPE_GRAVITY,
    LinearAcceleration = ndk_sys::ASENSOR_TYPE_LINEAR_ACCELERATION,
    RotationVector = ndk_sys::ASENSOR_TYPE_ROTATION_VECTOR,
    RelativeHumidity = ndk_sys::ASENSOR_TYPE_RELATIVE_HUMIDITY,
    AmbientTemperature = ndk_sys::ASENSOR_TYPE_AMBIENT_TEMPERATURE,
    MagneticFieldUncalibrated = ndk_sys::ASENSOR_TYPE_MAGNETIC_FIELD_UNCALIBRATED,
    GameRotationVector = ndk_sys::ASENSOR_TYPE_GAME_ROTATION_VECTOR,
    GyroscopeUncalibrated = ndk_sys::ASENSOR_TYPE_GYROSCOPE_UNCALIBRATED,
    SignificantMotion = ndk_sys::ASENSOR_TYPE_SIGNIFICANT_MOTION,
    StepDetector = ndk_sys::ASENSOR_TYPE_STEP_DETECTOR,
    StepCounter = ndk_sys::ASENSOR_TYPE_STEP_COUNTER,
    GeomagneticRotationVector = ndk_sys::ASENSOR_TYPE_GEOMAGNETIC_ROTATION_VECTOR,
    HeartRate = ndk_sys::ASENSOR_TYPE_HEART_RATE,
    Pose6DOF = ndk_sys::ASENSOR_TYPE_POSE_6DOF,
    StationaryDetect = ndk_sys::ASENSOR_TYPE_STATIONARY_DETECT,
    MotionDetect = ndk_sys::ASENSOR_TYPE_MOTION_DETECT,
    HeartBeat = ndk_sys::ASENSOR_TYPE_HEART_BEAT,
    DynamicSensorMeta = ndk_sys::ASENSOR_TYPE_DYNAMIC_SENSOR_META,
    AdditionalInfo = ndk_sys::ASENSOR_TYPE_ADDITIONAL_INFO,
    LowLatencyOffbodyDetect = ndk_sys::ASENSOR_TYPE_LOW_LATENCY_OFFBODY_DETECT,
    AccelerometerUncalibrated = ndk_sys::ASENSOR_TYPE_ACCELEROMETER_UNCALIBRATED,
    HingeAngle = ndk_sys::ASENSOR_TYPE_HINGE_ANGLE,
    HeadTracker = ndk_sys::ASENSOR_TYPE_HEAD_TRACKER,
    AccelerometerLimitedAxes = ndk_sys::ASENSOR_TYPE_ACCELEROMETER_LIMITED_AXES,
    GyroscopeLimitedAxes = ndk_sys::ASENSOR_TYPE_GYROSCOPE_LIMITED_AXES,
    AccelerometerLimitedAxesUncalibrated =
        ndk_sys::ASENSOR_TYPE_ACCELEROMETER_LIMITED_AXES_UNCALIBRATED,
    GyroscopeLimitedAxesUncalibrated = ndk_sys::ASENSOR_TYPE_GYROSCOPE_LIMITED_AXES_UNCALIBRATED,
    Heading = ndk_sys::ASENSOR_TYPE_HEADING,
}

pub struct Sensor {
    ptr: NonNull<ndk_sys::ASensor>,
}
impl Sensor {
    pub fn name(&self) -> &CStr {
        unsafe { CStr::from_ptr(ndk_sys::ASensor_getName(self.ptr.as_ptr())) }
    }
    pub fn min_delay(&self) -> std::time::Duration {
        unsafe {
            std::time::Duration::from_micros(ndk_sys::ASensor_getMinDelay(self.ptr.as_ptr()) as _)
        }
    }
    fn from_raw(ptr: NonNull<ndk_sys::ASensor>) -> Self {
        Self { ptr }
    }
}
pub struct SensorEvent {
    event: ndk_sys::ASensorEvent,
}
impl SensorEvent {
    pub fn gyro(&self) -> [f32; 3] {
        unsafe {
            self.event
                .__bindgen_anon_1
                .__bindgen_anon_1
                .gyro
                .__bindgen_anon_1
                .v
        }
    }
}

pub struct SensorManager {
    ptr: NonNull<ndk_sys::ASensorManager>,
}
impl SensorManager {
    pub fn new() -> Self {
        unsafe {
            let ptr = NonNull::new(ndk_sys::ASensorManager_getInstance()).unwrap();
            Self { ptr }
        }
    }
    pub fn default_sensor(&self, sensor_type: SensorType) -> Option<Sensor> {
        Some(Sensor::from_raw(NonNull::new(unsafe {
            ndk_sys::ASensorManager_getDefaultSensor(self.ptr.as_ptr(), sensor_type as _) as _
        })?))
    }
    pub fn sensor_list(&self) -> &[Sensor] {
        unsafe {
            let mut list: ndk_sys::ASensorList = std::mem::zeroed();
            let list_size = ndk_sys::ASensorManager_getSensorList(
                self.ptr.as_ptr(),
                std::ptr::addr_of_mut!(list),
            ) as usize;

            let list: *const Sensor = std::mem::transmute(list);
            std::slice::from_raw_parts(list, list_size)
        }
    }
    pub fn create_event_queue<'a>(&'a self, ident: i32) -> SensorEventQueue<'a> {
        let ptr = NonNull::new(unsafe {
            ndk_sys::ASensorManager_createEventQueue(
                self.ptr.as_ptr(),
                ndk_sys::ALooper_prepare(ndk_sys::ALOOPER_PREPARE_ALLOW_NON_CALLBACKS as _),
                ident,
                None,
                std::ptr::null_mut(),
            )
        })
        .unwrap();
        SensorEventQueue { ptr, manager: self }
    }
}

pub struct SensorEventQueue<'a> {
    ptr: NonNull<ndk_sys::ASensorEventQueue>,
    manager: &'a SensorManager,
}
impl<'a> SensorEventQueue<'a> {
    pub fn register_sensor(
        &self,
        sensor: &Sensor,
        sampling_period: std::time::Duration,
        max_batch_report_latency: std::time::Duration,
    ) -> bool {
        unsafe {
            ndk_sys::ASensorEventQueue_registerSensor(
                self.ptr.as_ptr(),
                sensor.ptr.as_ptr() as _,
                sampling_period.as_micros() as i32,
                max_batch_report_latency.as_micros() as i64,
            ) == 0
        }
    }
    pub fn enable_sensor(&self, sensor: &Sensor) -> bool {
        unsafe {
            ndk_sys::ASensorEventQueue_enableSensor(self.ptr.as_ptr(), sensor.ptr.as_ptr() as _)
                == 0
        }
    }
    pub fn disable_sensor(&self, sensor: &Sensor) -> bool {
        unsafe {
            ndk_sys::ASensorEventQueue_disableSensor(self.ptr.as_ptr(), sensor.ptr.as_ptr() as _)
                == 0
        }
    }
    pub fn has_events(&self) -> bool {
        // TODO: consider error
        unsafe { ndk_sys::ASensorEventQueue_hasEvents(self.ptr.as_ptr()) == 1 }
    }
    pub fn get_event(&self) -> Option<SensorEvent> {
        unsafe {
            let mut event: ndk_sys::ASensorEvent = std::mem::zeroed();
            let size = ndk_sys::ASensorEventQueue_getEvents(
                self.ptr.as_ptr(),
                std::ptr::addr_of_mut!(event),
                1,
            );
            if size == 1 {
                Some(SensorEvent { event })
            } else {
                None
            }
        }
    }
    pub fn set_event_rate(&self, sensor: &Sensor, rate: std::time::Duration) -> bool {
        unsafe {
            ndk_sys::ASensorEventQueue_setEventRate(
                self.ptr.as_ptr(),
                sensor.ptr.as_ptr(),
                rate.as_micros() as i32,
            ) == 0
        }
    }
}
impl<'a> Drop for SensorEventQueue<'a> {
    fn drop(&mut self) {
        unsafe {
            ndk_sys::ASensorManager_destroyEventQueue(self.manager.ptr.as_ptr(), self.ptr.as_ptr());
        }
    }
}
