use renderer_backend::pipeline_builder::PipelineBuilder;
mod renderer_backend;
use wgpu::{util::BufferInitDescriptor, BufferUsages};
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::EventLoopBuilder,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};

const SCREEN_SIZE: (u32, u32) = (1200, 600);

struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    window: &'a Window,
    render_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
}

impl<'a> State<'a> {
    async fn new(window: &'a Window) -> Self {
        let size = window.inner_size();

        let instance_descriptor = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        };
        let instance = wgpu::Instance::new(instance_descriptor);
        let surface = instance.create_surface(window).unwrap();

        let adapter_descriptor = wgpu::RequestAdapterOptionsBase {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        };
        let adapter = instance.request_adapter(&adapter_descriptor).await.unwrap();

        let device_descriptor = wgpu::DeviceDescriptor {
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            label: Some("Device"),
        };
        let (device, queue) = adapter
            .request_device(&device_descriptor, None)
            .await
            .unwrap();

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_capabilities.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Create bind group layout and bind group for sphere data
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("Sphere Bind Group Layout"),
        });

        // Pass bind group layout to pipeline builder
        let mut pipeline_builder = PipelineBuilder::new();
        pipeline_builder.set_shader_module("shaders/shader.wgsl", "vs_main", "fs_main");
        pipeline_builder.set_pixel_format(config.format);
        pipeline_builder.set_bind_group_layout(bind_group_layout);
        let render_pipeline = pipeline_builder.build_pipeline(&device);

        // Create a temporary bind group
        let temp_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Temporary Bind Group"),
            layout: &device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[],
                label: Some("Temporary Bind Group Layout"),
            }),
            entries: &[],
        });

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            bind_group: temp_bind_group,
        }
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let drawable = self.surface.get_current_texture()?;
        let image_view_descriptor = wgpu::TextureViewDescriptor::default();
        let image_view = drawable.texture.create_view(&image_view_descriptor);

        let command_encoder_descriptor = wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        };
        let mut command_encoder = self
            .device
            .create_command_encoder(&command_encoder_descriptor);
        let color_attachment = wgpu::RenderPassColorAttachment {
            view: &image_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.75,
                    g: 0.5,
                    b: 0.25,
                    a: 1.0,
                }),
                store: wgpu::StoreOp::Store,
            },
        };

        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(color_attachment)],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        };

        {
            let mut render_pass = command_encoder.begin_render_pass(&render_pass_descriptor);
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]); // Access using self
            render_pass.draw(0..3, 0..1); // Draw the first triangle
            render_pass.draw(3..6, 0..1); // Draw the second triangle
        }

        self.queue.submit(std::iter::once(command_encoder.finish()));

        drawable.present();

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
enum CustomEvent {
    Timer,
}

async fn run() {
    env_logger::init();

    // Create sphere data
    let sphere_centers: Vec<[f32; 3]> = vec![
        [20.0, 0.0, 0.0],
        [-20.0, 0.0, 0.0],
        [0.0, 20.0, 0.0],
        [0.0, -20.0, 0.0],
        [0.0, 0.0, 20.0],
        [0.0, 0.0, -20.0],
    ];
    let sphere_radii: Vec<f32> = vec![10.0; 6];

    let event_loop = EventLoopBuilder::<CustomEvent>::with_user_event()
        .build()
        .unwrap();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(SCREEN_SIZE.0, SCREEN_SIZE.1))
        .build(&event_loop)
        .unwrap();
    let event_loop_proxy = event_loop.create_proxy();

    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_millis(17));
        event_loop_proxy.send_event(CustomEvent::Timer).ok();
    });

    let mut state = State::new(&window).await;

    // Create storage buffer for sphere data
    let sphere_buffer_size =
        (sphere_centers.len() * std::mem::size_of::<[f32; 4]>()) as wgpu::BufferAddress;
    let sphere_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Sphere Buffer"),
        size: sphere_buffer_size,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // Combine center and radius into vec4 and copy to buffer
    let mut sphere_data: Vec<[f32; 4]> = Vec::new();
    for (i, center) in sphere_centers.iter().enumerate() {
        sphere_data.push([center[0], center[1], center[2], sphere_radii[i]]);
    }
    state
        .queue
        .write_buffer(&sphere_buffer, 0, bytemuck::cast_slice(&sphere_data));

    // Create bind group layout and bind group for sphere data
    let bind_group_layout =
        state
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("Sphere Bind Group Layout"),
            });
    state.bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Sphere Bind Group"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: sphere_buffer.as_entire_binding(),
        }],
    });

    // Pass bind group layout to pipeline builder
    let mut pipeline_builder = PipelineBuilder::new();
    pipeline_builder.set_shader_module("shaders/shader.wgsl", "vs_main", "fs_main");
    pipeline_builder.set_pixel_format(state.config.format);
    pipeline_builder.set_bind_group_layout(bind_group_layout);
    state.render_pipeline = pipeline_builder.build_pipeline(&state.device);

    event_loop
        .run(move |event, elwt| match event {
            Event::UserEvent(..) => {
                state.window.request_redraw();
            }

            Event::WindowEvent {
                window_id,
                ref event,
            } if window_id == state.window.id() => match event {
                WindowEvent::Resized(physical_size) => state.resize(*physical_size),

                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            physical_key: PhysicalKey::Code(KeyCode::Escape),
                            state: ElementState::Pressed,
                            repeat: false,
                            ..
                        },
                    ..
                } => {
                    println!("Goodbye see you!");
                    elwt.exit();
                }

                WindowEvent::RedrawRequested => match state.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                    Err(e) => eprintln!("{:?}", e),
                },

                _ => (),
            },

            _ => {}
        })
        .expect("Error!");
}

fn main() {
    pollster::block_on(run());
}
