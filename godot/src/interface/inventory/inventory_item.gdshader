shader_type canvas_item;
uniform vec4 shine_color: source_color = vec4(1.0);
uniform float shine : hint_range(0.0, 1.0, 0.1) = 0.0;
uniform float shine_speed : hint_range(0.0, 5.0, 0.1) = 0.5;


void fragment() {
	vec4 color = texture(TEXTURE, UV);
	float shine_strenght = smoothstep(
		0.0,
		1.0,
		0.1 + 0.9 * abs(sin(TIME * shine_speed))
	);

	COLOR.rgb = mix(COLOR.rgb, shine_color.rgb, shine * shine_strenght * shine_color.a);
}
