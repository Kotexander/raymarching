mod pipeline;
pub mod wgpu_context;

pub use nalgebra as na;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use wgpu_context::WgpuContext;

pub struct Camera {
    pub pos: na::Point3<f32>,
    pub fov: f32,
    pub rot: na::UnitQuaternion<f32>,
}
impl Camera {
    fn uniform(&self, aspect: f32) -> pipeline::CameraUniform {
        let fov = self.fov / 2.0;
        let mat = na::Matrix4::new_translation(&self.pos.coords)
            * self.rotation().to_homogeneous()
            * na::Matrix4::new_nonuniform_scaling(&na::vector![aspect * fov, fov, 1.0]);
        pipeline::CameraUniform { matrix: mat.into() }
    }
    pub fn rotation(&self) -> na::Rotation3<f32> {
        self.rot.to_rotation_matrix()
    }
}

fn create_texture(
    device: &wgpu::Device,
    size: (u32, u32),
    format: wgpu::TextureFormat,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Texture"),
        size: wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    })
}

fn scaled_texture_bindgroup_and_view(
    device: &wgpu::Device,
    texture_bindgroup_layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    format: wgpu::TextureFormat,
    size: (u32, u32),
    scale: f32,
) -> (wgpu::BindGroup, wgpu::TextureView) {
    let size = (
        ((size.0 as f32 * scale) as u32).max(1),
        ((size.1 as f32 * scale) as u32).max(1),
    );
    let texture = create_texture(&device, size, format);
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let texture_bindgroup =
        pipeline::texture_bindgroup(&device, &texture_bindgroup_layout, &texture_view, &sampler);

    (texture_bindgroup, texture_view)
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Controller {
    pub right: bool,
    pub left: bool,
    pub down: bool,
    pub up: bool,
    pub forwards: bool,
    pub backwards: bool,
    pub looking: bool,
}
struct RayMarcherRenderer {
    pipeline: wgpu::RenderPipeline,

    camera_bindgroup: pipeline::BindGroup<pipeline::CameraUniform>,

    camera: pipeline::CameraUniform,
    settings: pipeline::SettingsUniform,
    settings_bindgroup: pipeline::BindGroup<pipeline::SettingsUniform>,

    mesh: pipeline::Mesh<pipeline::Vertex>,
}
impl RayMarcherRenderer {
    fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        settings: pipeline::SettingsUniform,
        camera: pipeline::CameraUniform,
    ) -> Self {
        let camera_bindgroup_layout = pipeline::camera_bindgroup_layout(device);
        let settings_bindgroup_layout = pipeline::settings_bindgroup_layout(device);
        let camera_bindgroup = pipeline::camera_bindgroup(device, &camera_bindgroup_layout, camera);

        let settings_bindgroup =
            pipeline::settings_bindgroup(&device, &settings_bindgroup_layout, settings);
        let pipeline = pipeline::raymarcher_pipeline(
            &device,
            format,
            &camera_bindgroup_layout,
            &settings_bindgroup_layout,
        );
        let mesh = pipeline::new_fullscreen_quad(&device);

        Self {
            pipeline,
            camera,
            camera_bindgroup,
            settings,
            settings_bindgroup,
            mesh,
        }
    }
    fn update(&mut self, queue: &wgpu::Queue) {
        self.camera_bindgroup.update(queue, self.camera);
        self.settings_bindgroup.update(queue, self.settings);
    }
    fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.camera_bindgroup.bindgroup, &[]);
        render_pass.set_bind_group(1, &self.settings_bindgroup.bindgroup, &[]);
        self.mesh.draw(render_pass);
    }
}

struct FullscreenRenderer {
    pipeline: wgpu::RenderPipeline,

    texture_bindgroup_layout: wgpu::BindGroupLayout,
    texture_bindgroup: wgpu::BindGroup,

    texture_view: wgpu::TextureView,
    sampler: wgpu::Sampler,

    mesh: pipeline::Mesh<pipeline::Vertex>,
}
impl FullscreenRenderer {
    fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        size: (u32, u32),
        scale: f32,
    ) -> Self {
        let texture_bindgroup_layout = pipeline::texture_bindgroup_layout(device);

        let pipeline = pipeline::fullscreen_pipeline(device, format, &texture_bindgroup_layout);

        let address_mode = wgpu::AddressMode::ClampToEdge;
        let filter = wgpu::FilterMode::Nearest;
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Sampler"),
            address_mode_u: address_mode,
            address_mode_v: address_mode,
            address_mode_w: address_mode,
            mag_filter: filter,
            min_filter: filter,
            mipmap_filter: filter,
            ..Default::default()
        });

        let (texture_bindgroup, texture_view) = scaled_texture_bindgroup_and_view(
            device,
            &texture_bindgroup_layout,
            &sampler,
            format,
            size,
            scale,
        );

        let mesh = pipeline::new_fullscreen_quad(&device);

        Self {
            pipeline,
            texture_bindgroup_layout,
            texture_bindgroup,
            texture_view,
            sampler,
            mesh,
        }
    }
    fn resize(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        new_size: (u32, u32),
        scale: f32,
    ) {
        let (texture_bindgroup, texture_view) = scaled_texture_bindgroup_and_view(
            device,
            &self.texture_bindgroup_layout,
            &self.sampler,
            format,
            new_size,
            scale,
        );
        self.texture_bindgroup = texture_bindgroup;
        self.texture_view = texture_view;
    }
    fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.texture_bindgroup, &[]);
        self.mesh.draw(render_pass);
    }
}

pub struct RayMarcher<W>
where
    W: HasRawWindowHandle + HasRawDisplayHandle,
{
    pub wgpu_ctx: WgpuContext<W>,
    pub camera: Camera,
    pub controller: Controller,

    pub scale: f32,

    raymarcher_renderer: RayMarcherRenderer,
    fullscreen_renderer: FullscreenRenderer,
}
impl<W: HasRawWindowHandle + HasRawDisplayHandle> RayMarcher<W> {
    pub async fn new(window: W, size: (u32, u32), scale: f32) -> Self {
        let wgpu_ctx = WgpuContext::new(window, size).await;

        let camera = Camera {
            pos: na::point![0.0, 0.0, -3.0],
            fov: std::f32::consts::FRAC_PI_3,
            rot: na::UnitQuaternion::default(),
        };

        let controller = Controller::default();

        let aspect = size.0 as f32 / size.1 as f32;

        let settings = pipeline::SettingsUniform {
            sun_dir: na::Vector3::new(3.0, 3.0, -3.0).normalize().into(),
            ..Default::default()
        }
        .set_mandelbulb();

        let raymarcher_renderer = RayMarcherRenderer::new(
            &wgpu_ctx.device,
            wgpu_ctx.config.format,
            settings,
            camera.uniform(aspect),
        );
        let fullscreen_renderer =
            FullscreenRenderer::new(&wgpu_ctx.device, wgpu_ctx.config.format, size, scale);

        Self {
            wgpu_ctx,
            camera,
            controller,
            scale,
            raymarcher_renderer,
            fullscreen_renderer,
        }
    }
    pub fn switch_scene(&mut self) {
        match self.raymarcher_renderer.settings.scene {
            0 => {
                self.raymarcher_renderer.settings.set_mengersponge_mut();
            }
            1 => {
                self.raymarcher_renderer.settings.set_mandelbulb_mut();
            }
            _ => {
                self.raymarcher_renderer.settings.set_mandelbulb_mut();
            }
        }
    }

    pub fn update(&mut self, dt: f32) {
        let speed = 1.0;
        let mut dir = na::Vector3::<f32>::zeros();
        if self.controller.up {
            dir += na::Vector3::y();
        }
        if self.controller.down {
            dir -= na::Vector3::y();
        }
        if self.controller.right {
            dir += na::Vector3::x();
        }
        if self.controller.left {
            dir -= na::Vector3::x();
        }
        if self.controller.forwards {
            dir += na::Vector3::z();
        }
        if self.controller.backwards {
            dir -= na::Vector3::z();
        }
        if dir.magnitude_squared() != 0.0 {
            dir = dir.normalize();
            dir = self
                .camera
                .rotation()
                .to_homogeneous()
                .transform_vector(&dir);
            self.camera.pos += dir * speed * dt;
        }

        self.raymarcher_renderer.settings.time += dt;
    }
    fn aspect(&self) -> f32 {
        self.wgpu_ctx.config.width as f32 / self.wgpu_ctx.config.height as f32
        // self.wgpu_ctx.config.height as f32 / self.wgpu_ctx.config.width as f32
    }
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.raymarcher_renderer.camera = self.camera.uniform(self.aspect());
        self.raymarcher_renderer.update(&self.wgpu_ctx.queue);

        // get window's view
        let output = self.wgpu_ctx.surface.get_current_texture()?;
        let surface_view = output.texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Surface View"),
            ..Default::default()
        });

        let mut encoder =
            self.wgpu_ctx
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    // This is what @location(0) in the fragment shader targets
                    Some(wgpu::RenderPassColorAttachment {
                        // view: &surface_view,
                        view: &self.fullscreen_renderer.texture_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.1,
                                b: 0.1,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.raymarcher_renderer.render(&mut render_pass);
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    // This is what @location(0) in the fragment shader targets
                    Some(wgpu::RenderPassColorAttachment {
                        view: &surface_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.1,
                                b: 0.1,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.fullscreen_renderer.render(&mut render_pass);
        }

        // submit will accept anything that implements IntoIter
        self.wgpu_ctx
            .queue
            .submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        self.wgpu_ctx.resize(new_size);

        self.fullscreen_renderer.resize(
            &self.wgpu_ctx.device,
            self.wgpu_ctx.config.format,
            new_size,
            self.scale,
        );
    }
}
