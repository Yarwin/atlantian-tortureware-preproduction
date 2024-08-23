shader_type spatial; render_mode blend_mix, depth_draw_opaque, cull_back, diffuse_lambert, specular_disabled;
uniform vec4 albedo : source_color;
uniform sampler2D texture_albedo : source_color,filter_nearest_mipmap,repeat_enable;
uniform float point_size : hint_range(0,128);
uniform float roughness : hint_range(0,1);
uniform sampler2D texture_metallic : hint_default_white,filter_nearest_mipmap,repeat_enable;
uniform vec4 metallic_texture_channel;
uniform sampler2D texture_roughness : hint_roughness_r,filter_nearest_mipmap,repeat_enable;
uniform float specular;
uniform float metallic;
uniform vec3 uv1_scale;
uniform vec3 uv1_offset;
uniform vec3 uv2_scale;
uniform vec3 uv2_offset;

//your FOV value

uniform float viewmodel_fov = 52.0f;

void vertex() {
	UV= UV * uv1_scale.xy - uv1_offset.xy;

	//important bit starts here

	float onetanfov = 1.0f / tan(0.5f * (viewmodel_fov * PI / 180.0f));
	float aspect = VIEWPORT_SIZE.x / VIEWPORT_SIZE.y;
	// modify projection matrix
	PROJECTION_MATRIX[1][1] = -onetanfov;
	PROJECTION_MATRIX[0][0] = onetanfov / aspect;

	// draws the viewmodel over everything
	POSITION = PROJECTION_MATRIX * MODELVIEW_MATRIX * vec4(VERTEX.xyz, 1.0);
	POSITION.z = mix(POSITION.z, 0, 0.99);
}

void fragment() {
	DEPTH = FRAGCOORD.z * 0.7;

	//and ends here
	vec2 base_uv = UV;
	vec4 albedo_tex = texture(texture_albedo,base_uv);
	ALBEDO = albedo.rgb * albedo_tex.rgb;
	float metallic_tex = dot(texture(texture_metallic,base_uv),metallic_texture_channel);
	METALLIC = metallic_tex * metallic;
	vec4 roughness_texture_channel = vec4(1.0,0.0,0.0,0.0);
	float roughness_tex = dot(texture(texture_roughness,base_uv),roughness_texture_channel);
	ROUGHNESS = roughness_tex * roughness;
	SPECULAR = specular;
}