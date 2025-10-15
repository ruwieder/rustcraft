use wgpu::*;
use wgpu::util::DeviceExt;
use cgmath::{Point3, Vector2, Vector3};
use hashbrown::HashMap;

use crate::core::{chunk::{Chunk, CHUNK_SIZE}, mesh::Mesh, render::{camera::{Camera, UniformBuffer}, texture, vertex::Vertex}, world::world::{ChunkStorage, World}};

const SKYBOX: Color = Color{ r: 65.0 / 255.0, g: 200.0 / 255.0, b: 1.0, a: 1.0 };
const USE_GREEDY: bool = true;
const RENDER_LOGGING: bool = cfg!(debug_assertions);
const INITIAL_MESH_CAPACITY: usize = 1000;

pub struct Renderer {
    pub device: Device,
    pub queue: Queue,
    pub surface: Surface<'static>,
    pub config: SurfaceConfiguration,
    pub render_pipeline: RenderPipeline,
    pub uniform_buffer: Buffer,
    pub uniform_bind_group: BindGroup,
    pub camera: Camera,
    depth_texture: TextureView, // Store view instead of texture
    pub texture_bind_group: BindGroup,
    pub texture: texture::Texture,
    depth_texture_format: TextureFormat,
    mesh_cache: HashMap<(i64, i64, i64), GpuMesh>,
    dirty_meshes: Vec<(i64, i64, i64)>,
}

#[derive(Default)]
struct GpuMesh {
    vertex_buffer: Option<Buffer>,
    index_buffer: Option<Buffer>,
    index_count: u32,
    version: u32,
}

impl Renderer {
    pub async fn new(window: &'static winit::window::Window) -> Self {
        log::debug!("started renderer initialization...");
        let instance = Instance::new(&InstanceDescriptor {
            backends: Backends::PRIMARY,
            ..Default::default()
        });
        let surface = instance.create_surface(window).unwrap();
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();
        
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    required_features: Features::empty(),
                    required_limits: Limits::default(),
                    memory_hints: MemoryHints::default(),
                    experimental_features: ExperimentalFeatures::default(),
                    trace: Trace::default(),
                }
            )
            .await
            .unwrap();
        
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats[0];
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
        
        let depth_texture_format = TextureFormat::Depth24Plus;
        let depth_texture_view = Self::create_depth_texture(&device, &config, depth_texture_format);
        
        let uniform_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[UniformBuffer::default()]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        
        let uniform_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("uniform_bind_group_layout"),
        });
        
        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
            label: Some("uniform_bind_group"),
        });
        
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader"),
            source: ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        
        let diffuse_bytes = include_bytes!("../../../assets/textures/stone.png");
        let texture = texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "atlas.png").unwrap();
        let texture_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });
        
        let texture_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture.view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });
        
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Vertex::desc()],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: depth_texture_format,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let camera = Camera::new(
            Vector3::new(0.0, 0.0, 4.0),
            Vector2::new(0.0, 0.0),
            config.width as f32 / config.height as f32,
        );
        
        let uniform = camera.get_uniform();
        queue.write_buffer(&uniform_buffer, 0, bytemuck::cast_slice(&[uniform]));
        log::debug!("renderer initialized");
        Self {
            device,
            queue,
            surface,
            config,
            render_pipeline,
            uniform_buffer,
            uniform_bind_group,
            camera,
            depth_texture: depth_texture_view,
            texture_bind_group,
            texture,
            depth_texture_format,
            mesh_cache: HashMap::with_capacity(INITIAL_MESH_CAPACITY),
            dirty_meshes: Vec::new(),
        }
    }
    
    fn create_depth_texture(device: &Device, config: &SurfaceConfiguration, format: TextureFormat) -> TextureView {
        let depth_texture = device.create_texture(&TextureDescriptor {
            label: Some("Depth Texture"),
            size: Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        depth_texture.create_view(&TextureViewDescriptor::default())
    }
    
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }
        
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
        self.camera.aspect = self.config.width as f32 / self.config.height as f32;
        
        self.depth_texture = Self::create_depth_texture(&self.device, &self.config, self.depth_texture_format);
    }
    
    pub fn render(&mut self, world: &World) -> Result<(), SurfaceError> {
        if RENDER_LOGGING { log::trace!("started render..."); }
        self.process_dirty_meshes(world);
        
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&TextureViewDescriptor::default());
    
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        
        let frustum = &self.camera.frustum;

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: Operations {
                        load: LoadOp::Clear(SKYBOX),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth_texture,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
    
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_bind_group(1, &self.texture_bind_group, &[]);
            
            let mut count_rendered = 0;
            let mut total_triangles = 0;
            
            for (key, gpu_mesh) in &self.mesh_cache {
                if let Some(chunk) = world.chunks.get(key) {
                    if chunk.is_dirty {
                        continue;
                    }
                }
                
                // Frustum culling check
                let chunk_aabb_min = Point3::new(
                    key.0 as f32 * CHUNK_SIZE as f32,
                    key.1 as f32 * CHUNK_SIZE as f32,
                    key.2 as f32 * CHUNK_SIZE as f32,
                );
                let chunk_aabb_max = Point3::new(
                    (key.0 + 1) as f32 * CHUNK_SIZE as f32,
                    (key.1 + 1) as f32 * CHUNK_SIZE as f32,
                    (key.2 + 1) as f32 * CHUNK_SIZE as f32,
                );
                
                if !frustum.intersects_aabb(chunk_aabb_min, chunk_aabb_max) {
                    // count_culled += 1;
                    continue;
                }
                
                if let (Some(vertex_buffer), Some(index_buffer)) = (&gpu_mesh.vertex_buffer, &gpu_mesh.index_buffer) {
                    if gpu_mesh.index_count > 0 {
                        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                        render_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint32);
                        render_pass.draw_indexed(0..gpu_mesh.index_count, 0, 0..1);
                        count_rendered += 1;
                        total_triangles += gpu_mesh.index_count / 3;
                    }
                }
            }
            
            if RENDER_LOGGING { 
                log::trace!("rendered {} meshes ({} triangles)", count_rendered, total_triangles);
            }
        }
    
        self.queue.submit(std::iter::once(encoder.finish()));        
        output.present();
        if RENDER_LOGGING { log::trace!("render pass done"); }
        Ok(())
    }
    
    
    fn process_dirty_meshes(&mut self, world: &World) {
        if self.dirty_meshes.is_empty() {
            return;
        }
        
        let dirty_meshes = std::mem::take(&mut self.dirty_meshes);
        
        for key in dirty_meshes {
            if let Some(mesh) = world.meshes.get(&key) {
                self.update_gpu_mesh(key, mesh);
            }
        }
    }
    
    pub fn mark_mesh_dirty(&mut self, key: (i64, i64, i64)) {
        self.dirty_meshes.push(key);
    }
    
    fn update_gpu_mesh(&mut self, key: (i64, i64, i64), mesh: &Mesh) {
        let gpu_mesh = self.mesh_cache.entry(key).or_insert_with(GpuMesh::default);
        if mesh.is_dirty && !mesh.vertices.is_empty() && !mesh.indices.is_empty() {
            gpu_mesh.vertex_buffer = Some(self.device.create_buffer_init(
                &util::BufferInitDescriptor {
                    label: Some(&format!("Vertex Buffer {:?}", key)),
                    contents: bytemuck::cast_slice(&mesh.vertices),
                    usage: BufferUsages::VERTEX,
                }
            ));
            
            gpu_mesh.index_buffer = Some(self.device.create_buffer_init(
                &util::BufferInitDescriptor {
                    label: Some(&format!("Index Buffer {:?}", key)),
                    contents: bytemuck::cast_slice(&mesh.indices),
                    usage: BufferUsages::INDEX,
                }
            ));
            
            gpu_mesh.index_count = mesh.indices.len() as u32;
            gpu_mesh.version += 1;
            
            if RENDER_LOGGING {
                log::trace!("updated GPU mesh {:?} with {} indices", key, gpu_mesh.index_count);
            }
        }
    }

    pub fn update_camera(&mut self, dt: f64, movement: (f32, f32, f32), mouse_delta: (f32, f32)) {
        if movement.0 == 0.0 && movement.1 == 0.0 && movement.2 == 0.0 
          && mouse_delta.0 == 0.0 && mouse_delta.1 == 0.0 {
            return;
        }
        
        self.camera.update(dt, movement, mouse_delta);
        let uniform = self.camera.get_uniform();
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniform]));
    }
    
    pub fn on_mesh_updated(&mut self, key: (i64, i64, i64)) {
        self.mark_mesh_dirty(key);
    }
    
    pub fn cleanup_unused_meshes(&mut self, active_chunks: &HashMap<(i64, i64, i64), Chunk>) {
        self.mesh_cache.retain(|key, _| active_chunks.contains_key(key));
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        log::info!("dropping renderer...");
        let _ = self.device.poll(wgpu::PollType::Poll);
        log::info!("done");
    }
}