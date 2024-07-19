shader_type spatial;

uniform vec3 fire_color: source_color;
uniform sampler2D fire_noise;
uniform sampler2D gradient;
uniform vec2 speed;

void fragment() {
	ALBEDO = fire_color;
	//ALBEDO = vec3(texture(gradient, vec2(UV.y, 0.5)).r);
	float alpha_strenght = texture(fire_noise, UV - (speed * TIME)).r - texture(gradient, vec2(UV.y, 0.5)).r;
	ALPHA = clamp(alpha_strenght, 0., 1.);	// Called for every pixel the material is visible on.
}
