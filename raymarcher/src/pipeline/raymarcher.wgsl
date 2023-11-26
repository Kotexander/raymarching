@group(0) @binding(0)
var<uniform> camera: mat4x4<f32>;

const PI: f32 = 3.14159265358979323846264338327950288;


const MAX_STEPS = 100;
// const EPSILON = 0.001;

// const MAX_STEPS = 500;
const EPSILON = 0.00001;
const MAX_DIST = 10.0;

fn distributionGGX(a: f32, n: vec3<f32>, h: vec3<f32>) -> f32 {
    let a2 = a * a;
    let ndoth = max(dot(n, h), 0.0);
    let ndoth2 = ndoth * ndoth;

    let num = a2;
    var den = ndoth2 * (a2 - 1.0) + 1.0;
    den = PI * den * den;
    return num / max(den, EPSILON);
}
fn geometrySchlickGGX(ndotv: f32, k: f32) -> f32 {
    let num = ndotv;
    let den = ndotv * (1.0 - k) + k;
	
    return num / max(den, EPSILON);
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

const MENGER_SPONGE_ITERATIONS = 5;
fn menger_sponge(p: vec3<f32>) -> f32 {
    let a = PI/f32(MENGER_SPONGE_ITERATIONS);
    let rx = rotateX(a);
    let ry = rotateZ(a);
    let rz = rotateY(a);
    var pr = p;

    var d = box_distance(p,vec3(1.0));
    var s = 1.0;
    for(var m = 0; m < MENGER_SPONGE_ITERATIONS; m++){
        let a = real_mod_vec3f32(pr * s, 2.0) - 1.0;
        s *= 3.0;
        let r = 1.0 - 3.0*abs(a);

        let c = cross_distance(r)/s;
        d = max(d,c);

        pr = pr*rx*ry*rz;
    }
    return d;
}

const ITERATIONS = 10;
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
    // return mandelbulb(p);
    return menger_sponge(p);
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
    return true;
}

fn run(pos: vec3<f32>, dir: vec3<f32>) -> vec3<f32> {
    let l = normalize(vec3<f32>(3.0, 3.0, -3.0));

    var depth = 0.0;
    var i = 0;

    var p: vec3<f32>;
    for (; i < MAX_STEPS; i++) {
        p = pos + dir * depth;
        let dist = de(p);
        
        if dist < EPSILON {
            // hit
            break;
        }
        depth += dist;
        
        // background color
        if depth >= MAX_DIST {
            let sun_size = 0.005;
            let sun_sharpness = 2.0;
            var sun_spec = dot(dir, l) - 1.0 + sun_size;
			sun_spec = min(exp(sun_spec * sun_sharpness / sun_size), 1.0);
			return vec3<f32>(0.02, 0.4, 0.6) + sun_spec;
        }
    }
    let a = 0.1;
        
    let n = calc_normal(p, EPSILON);
    let v = -dir;
    let h = normalize(l + v);

    var f = fresnelSchlick(dot(v, h), vec3<f32>(0.04));
    var light = 0.0;
    let in_shadow = shadow(p + n * EPSILON, l);
    if !in_shadow {
        light = dot(l, n) * 2.0;
    }
    else {
        f = vec3<f32>(0.0);
    }

    // let f = fresnelSchlick(dot(v, n), vec3<f32>(0.04));
    
    let ks = f;
    let kd = 1.0 - ks;

    let color = (n + 1.0) / 2.0;
    let lambert = color / PI;
    let d = distributionGGX(a, n, h);
    let g = geometrySmith(n, v, l, pow(a+1.0,2.0)/8.0);

    let num = g * d * f;
    let den = 4.0 * dot(v, n) * dot(l, n);

    let diffuse = kd * color;
    let specular = num/max(den, EPSILON);

    return (diffuse + specular) * max(light, 0.1);
    // return vec3<f32>(f);
    // return diffuse;
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
