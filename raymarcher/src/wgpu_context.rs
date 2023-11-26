use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

pub struct WgpuContext<W: HasRawDisplayHandle + HasRawWindowHandle> {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub surface: wgpu::Surface,
    pub window: W,
}
impl<W> WgpuContext<W>
where
    W: HasRawDisplayHandle + HasRawWindowHandle,
{
    pub async fn new(window: W, size: (u32, u32)) -> Self {
        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
            flags: wgpu::InstanceFlags::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::default(),
        });

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        log::info!("Surface capabilities: {:#?}", surface_caps);
        let format = surface_caps
            .formats
            .iter()
            .find(|format| !format.is_srgb())
            .unwrap_or_else(|| {
                log::error!("Could not find a non srgb format. Colors will be too bright!");
                &surface_caps.formats[0]
            });
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: *format,
            width: size.0,
            height: size.1,
            // present_mode: surface_caps.present_modes[0],
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        Self {
            window,
            surface,
            device,
            queue,
            config,
        }
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        if new_size.0 > 0 && new_size.1 > 0 {
            self.config.width = new_size.0;
            self.config.height = new_size.1;
            self.reconfigure_surface()
        }
    }
    pub fn reconfigure_surface(&self) {
        self.surface.configure(&self.device, &self.config);
    }
}
