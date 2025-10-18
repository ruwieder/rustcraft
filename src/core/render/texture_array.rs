use anyhow::*;
use image::GenericImageView;

pub struct TextureArray {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub layers: u32,
}

impl TextureArray {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        textures_data: &[&[u8]],
        label: Option<&str>,
    ) -> Result<Self> {
        let mut textures = Vec::new();
        let mut dimensions = (0, 0);

        for (i, bytes) in textures_data.iter().enumerate() {
            let img = image::load_from_memory(bytes)?;
            if i == 0 {
                dimensions = img.dimensions();
            } else {
                assert_eq!(
                    dimensions,
                    img.dimensions(),
                    "All textures must have the same dimensions"
                );
            }
            textures.push(img.to_rgba8());
        }

        let mip_level_count = Self::calculate_mip_level_count(dimensions.0);

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: textures.len() as u32,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        for (i, rgba) in textures.iter().enumerate() {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: i as u32,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                rgba,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * dimensions.0),
                    rows_per_image: Some(dimensions.1),
                },
                wgpu::Extent3d {
                    width: dimensions.0,
                    height: dimensions.1,
                    depth_or_array_layers: 1,
                },
            );
        }

        Self::generate_mipmaps(
            device,
            queue,
            &texture,
            dimensions.0,
            textures.len() as u32,
            mip_level_count,
            &textures,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            base_mip_level: 0,
            mip_level_count: Some(mip_level_count),
            ..Default::default()
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: mip_level_count as f32,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
            layers: textures.len() as u32,
        })
    }

    fn calculate_mip_level_count(size: u32) -> u32 {
        (size as f32).log2().floor() as u32 + 1
    }

    fn generate_mipmaps(
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        size: u32,
        layers: u32,
        mip_level_count: u32,
        original_textures: &[image::RgbaImage],
    ) {
        for layer in 0..layers {
            let original_data = &original_textures[layer as usize];
            for mip_level in 1..mip_level_count {
                let mip_size = (size >> mip_level).max(1);
                let mip_data = Self::generate_mip_level(original_data, mip_level, size);

                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture,
                        mip_level,
                        origin: wgpu::Origin3d {
                            x: 0,
                            y: 0,
                            z: layer,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    &mip_data,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(4 * mip_size),
                        rows_per_image: Some(mip_size),
                    },
                    wgpu::Extent3d {
                        width: mip_size,
                        height: mip_size,
                        depth_or_array_layers: 1,
                    },
                );
            }
        }
    }

    fn generate_mip_level(original: &image::RgbaImage, mip_level: u32, base_size: u32) -> Vec<u8> {
        let mip_size = (base_size >> mip_level).max(1);
        let mut mip_data = Vec::with_capacity((mip_size * mip_size * 4) as usize);

        for y in 0..mip_size {
            for x in 0..mip_size {
                let src_x = (x * (1 << mip_level)).min(base_size - 1);
                let src_y = (y * (1 << mip_level)).min(base_size - 1);
                let pixel = original.get_pixel(src_x, src_y);
                mip_data.extend_from_slice(&[pixel[0], pixel[1], pixel[2], pixel[3]]);
            }
        }
        mip_data
    }
}
