shader_type canvas_item;

#define TWO_PI 6.28318530718
uniform vec4 primary_color: source_color = vec4(1.0);
uniform vec4 unfilled_color: source_color = vec4(0.443);
uniform float value: hint_range(0.0, 1.0, 0.01) = 1.0;


float radial_progress(vec2 UV_, float VAL)
{
	vec2 uv = (UV_ * 2.0) - 1.0;
	vec2 origin = vec2(0.0, 0.0);

	float d = length(uv);
	float a = atan(uv.y - origin.y, uv.x - origin.x);
	float theta = (a < 0.0) ? (a + TWO_PI) / TWO_PI : a / TWO_PI;
	float progress = smoothstep(0., theta, VAL);

	return progress;
}


void fragment() {
	float filled = radial_progress(UV, value);
	COLOR = ((filled > 0.5) ? primary_color : unfilled_color) * texture(TEXTURE, UV);

}

//void light() {
	// Called for every pixel for every light affecting the CanvasItem.
	// Uncomment to replace the default light processing function with this one.
//}
