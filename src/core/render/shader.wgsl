struct CameraUniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>, // Better name than 'position'
}

@group(0) @binding(0)
var<uniform> camera: CameraUniforms;

@group(1) @binding(0)
var texture_atlas: texture_2d<f32>;

@group(1) @binding(1)
var texture_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    out.tex_coords = model.tex_coords;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // return vec4<f32>(in.tex_coords, 0.0, 1.0); // UVs
    // return textureSample(texture_atlas, texture_sampler, in.tex_coords); // texture sampling
    return textureSample(texture_atlas, texture_sampler, in.tex_coords) * vec4<f32>(in.tex_coords, 0.0, 1.0);
    
}