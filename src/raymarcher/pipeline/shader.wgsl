@group(0) @binding(0)
var<uniform> camera: mat4x4<f32>;
@group(1) @binding(0)
var<uniform> sphere: Sphere;

struct Sphere {
    pos: vec3<f32>,
    rad: f32,
}
fn sphere_distance(sphere: Sphere, pos: vec3<f32>) -> f32 {
    return length(sphere.pos - pos) - sphere.rad;
}


const MAX_STEPS = 10;
const EPSILON = 0.01;
const MAX_DIST = 100.0;
fn run(pos: vec3<f32>, dir: vec3<f32>) -> vec3<f32> {
    var depth = 0.0;
    for (var i = 0; i < MAX_STEPS; i++) {
        let dist = sphere_distance(sphere, pos + dir * depth);
        if dist < EPSILON {
            return vec3<f32>(0.0, 0.0, 1.0);
        }
        depth += dist;
        if depth >= MAX_DIST {
            break;
        }
    }
    return vec3<f32>(0.0, 0.0, 0.0);
}

struct VertexIn {
    @location(0) position: vec2<f32>
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>
};

// makes a square
@vertex
fn vs_main(
    in: VertexIn
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.position, 0.0, 1.0);
    out.uv = in.position;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let pos = (camera * vec4<f32>(0.0, 0.0, 0.0, 1.0)).xyz;
    let dir = normalize((camera * vec4<f32>(in.uv.x, in.uv.y, 1.0, 1.0)).xyz);
    return vec4<f32>(run(pos, dir), 1.0);
    // return vec4<f32>(in.uv.x, in.uv.y, 0.0, 1.0);
}