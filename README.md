# Rustcraft

A high-performance, multithreaded voxel engine built in Rust with real-time terrain generation and rendering. Inspired by Minecraft, this project demonstrates advanced graphics programming, efficient chunk management, and modern game engine architecture.

## 🚀 Features

### Core Engine
- **Multithreaded Chunk Processing**: Utilizes Rayon for parallel terrain generation and mesh building
- **High-Performance Rendering**: Built on wgpu (WebGPU implementation) for modern graphics API support

### Graphics & Rendering
- **Frustum Culling**: Optimizes rendering by only processing visible chunks
- **Greedy Meshing**: Advanced mesh generation that reduces vertex count significantly
- **Dynamic LOD**: Automatic mesh updates and cache management

### World Generation
- **Procedural Terrain**: Noise-based terrain generation using `fastnoise-lite` and `noise` crates
- **Infinite World**: Dynamic chunk loading and unloading based on camera position
- **Smart Chunk Management**: Background loading with prioritization
- **Block Types**: Support for multiple materials (stone, dirt, grass, air, etc...)

### Performance Optimizations
- **Mesh Caching**: GPU mesh caching with version tracking
- **Dirty Flag System**: Only updates modified chunks
- **Background Processing**: Non-blocking asset loading

## 🛠️ Technical Architecture

### Key Systems

#### Rendering Pipeline
- **WGSL Shaders**: Custom shaders for vertex transformation and texture sampling
- **Uniform Buffers**: Efficient camera data updates
- **Depth Buffering**: 24-bit depth testing for proper occlusion

#### Mesh Generation
- **Greedy Meshing**: Combines adjacent faces to reduce triangle count
- **Face Culling**: Only generates visible faces (non-transparent neighbors)
- **Vertex Compression**: Compact vertex format with position, normal, and texture coordinates

## 📋 Requirements

### System Requirements
- **Rust**: 2024 edition or later
- **GPU**: Vulkan, Metal, or DX12 compatible graphics card
- **RAM**: 4GB minimum, 8GB recommended
- **OS**: Windows, macOS, or Linux

### Main dependencies
- **wgpu**: Modern graphics API abstraction
- **winit**: Cross-platform window creation and input
- **cgmath**: Linear algebra for 3D math
- **rayon**: Data parallelism
- **fastnoise-lite**: Procedural noise generation

## 🏗️ Building & Running

### Building
```bash
# Debug build (for development)
cargo build

# Release build (for performance)
cargo build --release

# Maximum optimization build
cargo build --profile most_release
```

### Running
```bash
# Run with default settings
cargo run

# Run with performance profiling
cargo run --profile profiling

# Run with detailed logging
RUST_LOG=debug cargo run
```

### Benchmarks
```bash
# Run terrain generation benchmarks
cargo bench --bench terrain_generation

# Run chunk meshing benchmarks  
cargo bench --bench chunk_meshing

# Run combined generation and meshing benchmarks
cargo bench --bench generation_and_meshing
```

## 🎮 Controls

| Action | Key | Description |
|--------|-----|-------------|
| **Move Forward** | `W` | Move camera forward |
| **Move Backward** | `S` | Move camera backward |
| **Strafe Left** | `A` | Move camera left |
| **Strafe Right** | `D` | Move camera right |
| **Move Up** | `Space` | Move camera upward |
| **Move Down** | `Left Shift` | Move camera downward |
| **Mouse Look** | `Left Click + Drag` | Rotate camera view |
| **Exit** | `Escape` | Close the application |

## 🔧 Advanced Usage

### Custom Terrain Generation
Modify `world/terrain_generator.rs` to implement custom terrain algorithms:

```rust
impl TerrainGenerator {
    pub fn generate_chunk(&self, position: Vector3<i64>, seed: u32) -> Chunk {
        // Custom noise functions and terrain shaping
    }
}
```

### Adding New Block Types
1. Add texture to `assets/textures/`
2. Update texture array initialization in `renderer.rs`
3. Define block properties in `core/block.rs`

### Shader Modifications
Edit `core/render/shader.wgsl` for custom rendering effects:

```wgsl
@vertex
fn vs_main(vertex: Vertex) -> VertexOutput {
    // Custom vertex transformations
}

@fragment  
fn fs_main(fragment: Fragment) -> @location(0) vec4<f32> {
    // Custom fragment shading
}
```

## 📊 Performance Profiling

The project includes multiple profiling configurations:

```toml
[profile.profiling]
inherits = "release"
debug = true
strip = "none"

[profile.most_release]  
inherits = "release"
strip = true
lto = "fat"
panic = "abort"
codegen-units = 1
```

Use `cargo build --profile profiling` for debug symbols with release optimizations.

## 🤝 Contributing

While this is primarily a learning project, contributions are welcome:

1. Fork the repository
2. Create a feature branch
3. Implement your changes with benchmarks
4. Submit a pull request with performance impact analysis

### Areas for Improvement
- **Networking**: Multiplayer support
- **Lighting**: Dynamic lighting system
- **Physics**: Collision detection and response
- **UI**: In-game menu and HUD systems
- **Audio**: Spatial audio system

## 📝 License

This project is available under the MIT License. See LICENSE file for details.

---

**Note**: This is an educational project focused on graphics programming and game engine architecture. It's not affiliated with or endorsed by Minecraft/Mojang.