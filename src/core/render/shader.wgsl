struct UniformBuffer {
    view_proj: mat4x4<f32>,
    position: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: UniformBuffer;

@group(0) @binding(1)
var texture_atlas: texture_2d<f32>;

@group(0) @binding(2)
var texture_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec3<f32>,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.view_proj * vec4<f32>(model.position, 1.0);
    out.tex_coords = model.tex_coords;
    out.color = model.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Mix texture with vertex color for now
    let texture_color = textureSample(texture_atlas, texture_sampler, in.tex_coords);
    return texture_color * vec4<f32>(in.color, 1.0);
}