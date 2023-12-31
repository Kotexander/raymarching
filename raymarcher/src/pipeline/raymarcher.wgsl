@group(0) @binding(0)
var<uniform> camera: mat4x4<f32>;
@group(1) @binding(0)
var<uniform> settings: Settings;

const PI: f32 = 3.14159265358979323846264338327950288;

struct Settings {
  max_steps: i32,
  epsilon: f32,
  max_dist: f32,
  
  sun_size: f32,
  sun_dir: vec3<f32>,
  sun_sharpness: f32,
  
  alpha: f32,

  time: f32,
  scene: u32
}

fn distributionGGX(a: f32, n: vec3<f32>, h: vec3<f32>) -> f32 {
    let a2 = a * a;
    let ndoth = max(dot(n, h), 0.0);
    let ndoth2 = ndoth * ndoth;

    let num = a2;
    var den = ndoth2 * (a2 - 1.0) + 1.0;
    den = PI * den * den;
    return num / max(den, settings.epsilon);
}
fn geometrySchlickGGX(ndotv: f32, k: f32) -> f32 {
    let num = ndotv;
    let den = ndotv * (1.0 - k) + k;
	
    return num / max(den, settings.epsilon);
}
fn geometrySmith(n: vec3<f32>, v: vec3<f32>, l: vec3<f32>, k: f32) -> f32 {
    let ndotv = max(dot(n, v), 0.0);
    let ndotl = max(dot(n, l), 0.0);
    let ggx1 = geometrySchlickGGX(ndotv, k);
    let ggx2 = geometrySchlickGGX(ndotl, k);
	
    return ggx1 * ggx2;
}
fn fresnelSchlick(cosTheta: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (1.0 - f0) * pow(1.0 - cosTheta, 5.0);
}

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

fn rotateX(a: f32) -> mat3x3<f32>{
    let s = sin(a);
    let c = cos(a);
    return mat3x3(
        vec3<f32>(1.0, 0.0, 0.0),
        vec3<f32>(0.0,   c,  -s),
        vec3<f32>(0.0,   s,   c),
    );
}
fn rotateZ(a: f32) -> mat3x3<f32>{
    let s = sin(a);
    let c = cos(a);
    return mat3x3(
        vec3<f32>(  c,  -s, 0.0),
        vec3<f32>(  s,   c, 0.0),
        vec3<f32>(0.0, 0.0, 1.0),
    );
}
fn rotateY(a: f32) -> mat3x3<f32>{
    let s = sin(a);
    let c = cos(a);
    return mat3x3(
        vec3<f32>(  c, 0.0,   s),
        vec3<f32>(0.0, 1.0, 0.0),
        vec3<f32>( -s, 0.0,   c),
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
    return min(da, min(db,dc));
}

const K = 0.05235987755; // 2pi / 120

const MENGER_SPONGE_ITERATIONS = 5;
fn menger_sponge(p: vec3<f32>) -> f32 {
    var pr = p;

    var d = box_distance(p,vec3(1.0));
    var s = 1.0;
    for(var m = 0; m < MENGER_SPONGE_ITERATIONS; m++){

        let a = real_mod_vec3f32(pr * s, 2.0) - 1.0;
        s *= 3.0;
        let r = 1.0 - 3.0*abs(a);

        let c = cross_distance(r)/s;
        d = max(d,c);

        // let ra = sin(K * settings.time);
        // let rx = rotateX(ra);
        // let ry = rotateZ(ra);
        // let rz = rotateY(ra);
        // pr = pr*rx*ry*rz;
        pr += 0.1 * s;
    }
    return d;
}

const AMP = 4.5;
const MID = 5.5;
const ITERATIONS = 15;
fn mandelbulb(pos: vec3<f32>) -> f32 {
    let power = AMP * sin(K * settings.time) + MID;
	
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
		dr = pow(r, power - 1.0)*power*dr + 1.0;
		
		// scale and rotate the point
		var zr = pow(r,power);
		theta = theta*power;
		phi = phi*power;
		
		// convert back to cartesian coordinates
		z = zr*vec3<f32>(sin(theta)*cos(phi), sin(phi)*sin(theta), cos(theta));
		z+=pos;
	}
	return 0.5*log(r)*r/dr;
}

fn de(p: vec3<f32>) -> f32 {
    switch settings.scene {
        case 0u {
            return mandelbulb(p);
        }
        case 1u {
            return menger_sponge(p);
        }
        default {
            return sphere_distance(p);
        }
    }
}

fn calc_normal(p: vec3<f32>, d: f32) -> vec3<f32> {
    let x  = de(p);
    let dx = de(p + vec3<f32>(d, 0.0, 0.0)) - x;
    let dy = de(p + vec3<f32>(0.0, d, 0.0)) - x;
    let dz = de(p + vec3<f32>(0.0, 0.0, d)) - x;
    return normalize(vec3<f32>(dx, dy ,dz));
}

fn shadow(pos: vec3<f32>, dir: vec3<f32>) -> bool {
    var depth = de(pos + dir * settings.epsilon * 10.0);
    // var depth = 0.0;

    for (var i = 0; i < settings.max_steps; i++) {
        let dist = de(pos + dir * depth);
        depth += abs(dist);

        // hit something
        if dist < settings.epsilon {
            return true;
        }

        // skybox
        if depth >= settings.max_dist {
            return false;
        }
    }
    // skybox?
    return true;
}

fn run(pos: vec3<f32>, dir: vec3<f32>) -> vec3<f32> {
    let l = settings.sun_dir;

    var depth = 0.0;
    var i = 0;

    var p: vec3<f32>;
    for (; i < settings.max_steps; i++) {
        p = pos + dir * depth;
        let dist = de(p);
        
        if dist < settings.epsilon {
            // hit
            break;
        }
        depth += dist;
        
        // background color
        if depth >= settings.max_dist {
            var sun_spec = dot(dir, l) - 1.0 + settings.sun_size;
			sun_spec = min(exp(sun_spec * settings.sun_sharpness / settings.sun_size), 1.0);
			return vec3<f32>(0.02, 0.4, 0.6) + sun_spec;
        }
    }
        
    let n = calc_normal(p, settings.epsilon);
    let v = -dir;
    let h = normalize(l + v);

    var f = fresnelSchlick(dot(v, h), vec3<f32>(0.04));
    var light = 0.0;
    let in_shadow = shadow(p + n * settings.epsilon, l);
    if !in_shadow {
        light = dot(l, n) * 2.0;
    }
    else {
        f = vec3<f32>(0.0);
    }
    
    let ks = f;
    let kd = 1.0 - ks;

    let color = (n + 1.0) / 2.0;
    let lambert = color / PI;
    let d = distributionGGX(settings.alpha, n, h);
    let g = geometrySmith(n, v, l, pow(settings.alpha+1.0,2.0)/8.0);

    let num = g * d * f;
    let den = 4.0 * dot(v, n) * dot(l, n);

    let diffuse = kd * color;
    let specular = num/max(den, settings.epsilon);

    return (diffuse + specular) * max(light, 0.1);
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
