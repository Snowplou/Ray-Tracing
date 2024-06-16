struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) screen_size: vec2<f32>,
    @location(1) screen_width: f32,
    @location(2) screen_height: f32,
    @location(3) camera_position: vec3<f32>,
    @location(4) camera_rotation: vec3<f32>,
};

struct HitInfo {
    did_hit: bool,
    distance: f32,
    position: vec3<f32>,
    normal: vec3<f32>,
};

@group(0) @binding(0) var<storage, read> sphere_data : array<array<f32, 7>, 6>;

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0), // Bottom Left
        vec2<f32>(1.0, -1.0),  // Bottom Right
        vec2<f32>(-1.0, 1.0),   // Top Left

        vec2<f32>(1.0, 1.0), // Top Right
        vec2<f32>(-1.0, 1.0), // Top Left
        vec2<f32>(1.0, -1.0) // Bottom Right
    );

    var screen_size: vec2<f32> = vec2<f32>(1200.0, 600.0);
    var fov: f32 = 60.0 * 3.14159 / 180.0;
    var aspect_ratio: f32 = screen_size.x / screen_size.y;
    var screen_width: f32 = tan(fov * 0.5) * 2.0;
    var screen_height: f32 = screen_width / aspect_ratio;
    var camera_position: vec3<f32> = vec3<f32>(90.0, 90.0, 130.0);
    var camera_rotation: vec3<f32> = vec3<f32>(-28, 30.0, 0.0);

    var out: VertexOutput;
    out.pos = vec4<f32>(positions[i], 0.0, 1.0);
    out.screen_size = screen_size;
    out.screen_width = screen_width;
    out.screen_height = screen_height;
    out.camera_position = camera_position;
    out.camera_rotation = camera_rotation;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    if(sphere_data[1][0] == -40.0){
        return vec4<f32>(0.0, 0.0, 1.0, 1.0);
    }
    
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}