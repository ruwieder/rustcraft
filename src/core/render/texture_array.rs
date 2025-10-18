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
                assert_eq!(dimensions, img.dimensions(), "All textures must have the same dimensions");
            }
            textures.push(img.to_rgba8());
        };
        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: textures.len() as u32,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
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
                        z: i as u32, // layer index
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
        };
        
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        Ok(Self {
            texture,
            view,
            sampler,
            layers: textures.len() as u32,
        })
    }
    
    // pub fn from_bytes(device: &wgpu::Device, queue: &wgpu::Queue, bytes: &[u8], label: &str) -> Result<Self> {
    //     let img = image::load_from_memory(bytes)?;
    //     Self::from_image(device, queue, &img, Some(label))
    // }

    // pub fn from_image(
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     img: &image::DynamicImage,
    //     label: Option<&str>,
    // ) -> Result<Self> {
    //     let rgba = img.to_rgba8();
    //     let dimensions = img.dimensions();

    //     let size = wgpu::Extent3d {
    //         width: dimensions.0,
    //         height: dimensions.1,
    //         depth_or_array_layers: 1,
    //     };

    //     let texture = device.create_texture(&wgpu::TextureDescriptor {
    //         label,
    //         size,
    //         mip_level_count: 1,
    //         sample_count: 1,
    //         dimension: wgpu::TextureDimension::D2,
    //         format: wgpu::TextureFormat::Rgba8UnormSrgb,
    //         usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    //         view_formats: &[],
    //     });
    //     queue.write_texture(
    //         wgpu::TexelCopyTextureInfo {
    //             texture: &texture,
    //             mip_level: 0,
    //             origin: wgpu::Origin3d::ZERO,
    //             aspect: wgpu::TextureAspect::All,
    //         },
    //         &rgba,
    //         wgpu::TexelCopyBufferLayout {
    //             offset: 0,
    //             bytes_per_row: Some(4 * dimensions.0),
    //             rows_per_image: Some(dimensions.1),
    //         },
    //         size,
    //     );

    //     let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    //     let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
    //         address_mode_u: wgpu::AddressMode::Repeat,
    //         address_mode_v: wgpu::AddressMode::Repeat,
    //         address_mode_w: wgpu::AddressMode::Repeat,
    //         mag_filter: wgpu::FilterMode::Nearest,
    //         min_filter: wgpu::FilterMode::Nearest,
    //         mipmap_filter: wgpu::FilterMode::Nearest,
    //         ..Default::default()
    //     });

    //     Ok(Self { texture, view, sampler })
    // }
}