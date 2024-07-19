shader_type spatial;
render_mode cull_disabled;
uniform sampler2D trail;
uniform sampler2D gradient;
uniform sampler2D trail_texture;

void fragment() {
	ALBEDO = texture(trail_texture, UV).rgb;
	vec2 time_mod = vec2(-2.2, 0.1) * TIME;
	float trail_alpha = texture(trail, UV + time_mod).r - texture(gradient, vec2(UV.y, 0.25)).r;
	ALPHA = clamp(trail_alpha, 0., 1.) * texture(trail_texture, UV).a;
}
