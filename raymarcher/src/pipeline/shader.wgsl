@group(0) @binding(0)
var<uniform> camera: mat4x4<f32>;

fn real_mod_f32(dividend: f32, divisor: f32) -> f32 {
    let q = floor(dividend / divisor);
    let m = dividend - divisor * q;
    return m; 
}
fn real_mod_vec3f32(dividend: vec3<f32>, divisor: f32) -> vec3<f32> {
    return vec3<f32>(
        real_mod_f32(dividend.x, divisor),
        real_mod_f32(dividend.y, divisor),
        real_mod_f32(dividend.z, divisor),
    );
}

fn sphere_distance(pos: vec3<f32>) -> f32 {
    return length(pos) - 1.0;
}
fn tetrahedron_distance(p: vec3<f32>) -> f32 {
    return (max(abs(p.x+p.y)-p.z, abs(p.x-p.y)+p.z) - 1.0)/sqrt(3.0);
}
fn box_distance(p: vec3<f32>, b: vec3<f32> ) -> f32 {
    let q = abs(p) - b;
    return length(max(q,vec3<f32>(0.0))) + min(max(q.x,max(q.y,q.z)),0.0);
}
const LEN: f32 = 3.0;
// const LEN: f32 = 1.0e30;
fn cross_distance(p: vec3<f32>) -> f32 {    
    let da = box_distance(p.xyz,vec3(LEN,1.0,1.0));
    let db = box_distance(p.yzx,vec3(1.0,LEN,1.0));
    let dc = box_distance(p.zxy,vec3(1.0,1.0,LEN));
    return min(da,min(db,dc));
}

const MENGER_SPONGE_ITERATIONS = 3;
fn menger_sponge(p: vec3<f32>) -> f32 {
    var d = box_distance(p,vec3(1.0));
    var s = 1.0;
    for(var m = 0; m < MENGER_SPONGE_ITERATIONS; m++){
        let a = real_mod_vec3f32(p * s, 2.0) - 1.0;
        s *= 3.0;
        let r = 1.0 - 3.0*abs(a);

        let c = cross_distance(r)/s;
        d = max(d,c);
    }
    return d;
}

// mandelbrot bulb
const MAX_STEPS = 100;
const EPSILON = 0.001;

// everything else
// const MAX_STEPS = 500;
// const EPSILON = 0.00001;

const MAX_DIST = 10.0;

const ITERATIONS = 50;
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

fn de(p: vec3<f32>) -> f32 {
    // return sphere_distance(p);
    // return tetrahedron_distance(p);
    // return box_distance(p);
    // return cross_distance(p;
    return mandelbulb(p);
    //return menger_sponge(p);

}
fn calc_normal(p: vec3<f32>, d: f32) -> vec3<f32> {
    let x  = de(p);
    let dx = de(p + vec3<f32>(d, 0.0, 0.0)) - x;
    let dy = de(p + vec3<f32>(0.0, d, 0.0)) - x;
    let dz = de(p + vec3<f32>(0.0, 0.0, d)) - x;
    return normalize(vec3<f32>(dx, dy ,dz));
}

fn shadow(pos: vec3<f32>, dir: vec3<f32>) -> bool {
    var depth = de(pos + dir * EPSILON * 10.0);
    // var depth = 0.0;

    for (var i = 0; i < MAX_STEPS; i++) {
        let dist = de(pos + dir * depth);
        depth += abs(dist);

        // hit something
        if dist < EPSILON {
            return true;
        }

        // skybox
        if depth >= MAX_DIST {
            return false;
        }
    }
    // skybox?
    return false;
}

fn run(pos: vec3<f32>, dir: vec3<f32>) -> vec3<f32> {
    let light_dir = normalize(vec3<f32>(1.5, 1.0, -1.0));

    var depth = 0.0;
    var normal: vec3<f32>;
    var in_shadow = false;

    var i = 0;
    for (; i < MAX_STEPS; i++) {
        let p = pos + dir * depth;
        let dist = de(p);
        
        if dist < EPSILON {
            normal = calc_normal(p, EPSILON);
            in_shadow = shadow(p + normal * EPSILON, light_dir);
            break;
        }
        depth += dist;
        
        // background color
        if depth >= MAX_DIST {
            return vec3<f32>(0.01, 0.2, 0.3);
            // return vec3<f32>(0.01, 0.01, 0.01);
        }
    }
    var s: vec3<f32>;
    if in_shadow {
        s = vec3<f32>(0.01);
    }
    else {
        s = vec3<f32>(max(0.01, dot(normal,light_dir)));
    }
    

    let c = (normal + 1.0) / 2.0;
    let o = (1.0-f32(i) / f32(MAX_STEPS)); 
    return c * o * s;
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
