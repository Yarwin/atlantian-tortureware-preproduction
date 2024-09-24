shader_type canvas_item;
const float seed_variation = 0.0;
uniform sampler2D o79805_tex;
uniform float value: hint_range(0.0, 1.0, 0.01) = 1.0;


float dot2(vec2 x) {
	return dot(x, x);
}

float rand(vec2 x) {
    return fract(cos(mod(dot(x, vec2(13.9898, 8.141)), 3.14)) * 43758.5453);
}

vec2 rand2(vec2 x) {
    return fract(cos(mod(vec2(dot(x, vec2(13.9898, 8.141)),
						      dot(x, vec2(3.4562, 17.398))), vec2(3.14))) * 43758.5453);
}

vec3 rand3(vec2 x) {
    return fract(cos(mod(vec3(dot(x, vec2(13.9898, 8.141)),
							  dot(x, vec2(3.4562, 17.398)),
                              dot(x, vec2(13.254, 5.867))), vec3(3.14))) * 43758.5453);
}

vec3 rgb2hsv(vec3 c) {
	vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
	vec4 p = c.g < c.b ? vec4(c.bg, K.wz) : vec4(c.gb, K.xy);
	vec4 q = c.r < p.x ? vec4(p.xyw, c.r) : vec4(c.r, p.yzx);

	float d = q.x - min(q.w, q.y);
	float e = 1.0e-10;
	return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c) {
	vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
	vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
	return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

float param_rnd(float minimum, float maximum, float seed) {
	return minimum+(maximum-minimum)*rand(vec2(seed));
}

float param_rndi(float minimum, float maximum, float seed) {
	return floor(param_rnd(minimum, maximum + 1.0, seed));
}
vec2 get_from_tileset(float count, float seed, vec2 uv) {
	return clamp((uv+floor(rand2(vec2(seed))*count))/count, vec2(0.0), vec2(1.0));
}

vec2 custom_uv_transform(vec2 uv, vec2 cst_scale, float rnd_rotate, float rnd_scale, vec2 seed) {
	seed = rand2(seed);
	uv -= vec2(0.5);
	float angle = (seed.x * 2.0 - 1.0) * rnd_rotate;
	float ca = cos(angle);
	float sa = sin(angle);
	uv = vec2(ca*uv.x+sa*uv.y, -sa*uv.x+ca*uv.y);
	uv *= (seed.y-0.5)*2.0*rnd_scale+1.0;
	uv /= cst_scale;
	uv += vec2(0.5);
	return uv;
}
const float seed_o8745 = 0.000000000;
const float p_o8745_count = 12.000000000;
const float p_o8745_rings = 1.000000000;
const float p_o8745_scale_x = 1.000000000;
const float p_o8745_scale_y = 1.000000000;
const float p_o8745_radius = 0.000000000;
const float p_o8745_spiral = 0.000000000;
const float p_o8745_i_scale = 0.000000000;
const float p_o8745_rotate = 15.000000000;
const float p_o8745_scale = 0.000000000;
const float p_o8745_value = 0.000000000;
float o8745_input_in(vec2 uv, float _seed_variation_) {
vec4 o79805_0 = textureLod(o79805_tex, uv, 0.0);

return (dot((o79805_0).rgb, vec3(1.0))/3.0);
}
float o8745_input_mask(vec2 uv, float _seed_variation_) {

return 1.0;
}
vec4 splatter_o8745(vec2 uv, int count, int rings, inout vec3 instance_uv, vec2 seed, float _seed_variation_) {
	float c = 0.0;
	vec3 rc = vec3(0.0);
	vec3 rc1;
	seed = rand2(seed);
	for (int i = 0; i < count; ++i) {
		float a = -1.57079632679+6.28318530718*float(i)*p_o8745_rings/float(count);
		float rings_distance = ceil(float(i+1)*float(rings)/float(count))/float(rings);
		float spiral_distance = float(i+1)/float(count);
		vec2 pos = p_o8745_radius*mix(rings_distance, spiral_distance, p_o8745_spiral)*vec2(cos(a), sin(a));
		float mask = o8745_input_mask(fract(pos-vec2(0.5)), _seed_variation_);
		if (mask > 0.01) {
			vec2 pv = uv-0.5-pos;
			rc1 = rand3(seed);
			seed = rand2(seed);
			float angle = (seed.x * 2.0 - 1.0) * p_o8745_rotate * 0.01745329251 + (a+1.57079632679) * value;
			float ca = cos(angle);
			float sa = sin(angle);
			pv = vec2(ca*pv.x+sa*pv.y, -sa*pv.x+ca*pv.y);
			pv /= mix(1.0, float(i+1)/float(count+1), p_o8745_i_scale);
			pv /= vec2(p_o8745_scale_x, p_o8745_scale_y);
			pv *= (seed.y-0.5)*2.0*p_o8745_scale+1.0;
			pv += vec2(0.5);
			seed = rand2(seed);
			if (pv != clamp(pv, vec2(0.0), vec2(1.0))) {
				continue;
			}
			vec2 full_uv = pv;
			pv = get_from_tileset( 1.0, seed.x, pv);
			float c1 = o8745_input_in(pv, false ? seed.x : 0.0)*mask*(1.0-p_o8745_value*seed.x);
			c = max(c, c1);
			rc = mix(rc, rc1, step(c, c1));
			instance_uv = mix(instance_uv, vec3(full_uv, seed.x), step(c, c1));
		}
	}
	return vec4(rc, c);
}

void fragment() {
float _seed_variation_ = seed_variation;
vec3 o8745_0_instance_uv = vec3(0.0);
vec4 o8745_0_rch = splatter_o8745((UV), int(p_o8745_count), int(p_o8745_rings), o8745_0_instance_uv, vec2(float((seed_o8745+fract(_seed_variation_)))), _seed_variation_);float o8745_0_1_f = o8745_0_rch.a;
COLOR = vec4(o8745_0_1_f);
}
