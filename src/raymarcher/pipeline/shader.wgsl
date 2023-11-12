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

const MAX_STEPS = 250;
const EPSILON = 0.0001;
const MAX_DIST = 100.0;

const ITERATIONS = 15;
const POWER = 8.0;
fn mandelbulb(pos: vec3<f32>) -> f32 {
	var z = pos;
	var dr = 1.0;
	var r = 0.0;
	for (var i = 0; i < ITERATIONS ; i++) {
		r = length(z);
		if (r>2.0) { 
            break;
        }
		
		// convert to polar coordinates
		var theta = acos(z.z/r);
		var phi = atan2(z.y,z.x);
		dr = pow(r, POWER - 1.0)*POWER*dr + 1.0;
		
		// scale and rotate the point
		var zr = pow(r,POWER);
		theta = theta*POWER;
		phi = phi*POWER;
		
		// convert back to cartesian coordinates
		z = zr*vec3<f32>(sin(theta)*cos(phi), sin(phi)*sin(theta), cos(theta));
		z+=pos;
	}
	return 0.5*log(r)*r/dr;
}

fn run(pos: vec3<f32>, dir: vec3<f32>) -> vec3<f32> {
    var depth = 0.0;
    var i = 0;
    for (; i < MAX_STEPS; i++) {
        let dist = mandelbulb(pos + dir * depth);
        // let dist = sphere_distance(sphere, pos+dir*depth);
        if dist < EPSILON {
            break;
        }
        depth += dist;
        
        // background color
        if depth >= MAX_DIST {
            return vec3<f32>(0.1, 0.2, 0.3);
        }
    }
    return vec3<f32>(1.0, 1.0, 1.0) * (1.0-f32(i) / f32(MAX_STEPS));
}

struct VertexIn {
    @location(0) position: vec2<f32>
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>
};

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
    let dir = normalize(
        (camera * vec4<f32>(in.uv.x, in.uv.y, 1.0, 1.0)).xyz - pos
        );
    return vec4<f32>(run(pos, dir), 1.0);
}