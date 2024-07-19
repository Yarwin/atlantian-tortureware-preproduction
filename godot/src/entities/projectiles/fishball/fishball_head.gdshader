shader_type spatial;
uniform vec3 base_color: source_color;
uniform vec3 fresnel_color: source_color;

vec3 fresnel_glow(float amount, float intensity, vec3 f_color, vec3 normal, vec3 view)
{
	return pow((1.0 - dot(normalize(normal), normalize(view))), amount) * f_color * intensity;
}

void fragment() {
	ALBEDO = base_color + fresnel_glow(4.0, 4.5, fresnel_color, NORMAL, VIEW);
	//ALPHA = 1. - fresnel(5.0, 4.5, NORMAL, VIEW);
	// Called for every pixel the material is visible on.
}

//void light() {
	// Called for every pixel for every light affecting the material.
	// Uncomment to replace the default light processing function with this one.
//}
