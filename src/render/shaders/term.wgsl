struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) position_offset: vec2<f32>,
    @location(3) atlas_uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = (in.atlas_uv + in.uv) * (16.0 / 256.0);
	let scale = vec3(1440.0 / 2560.0, 1.0, 1.0);
	let atlas_scale = vec3(16.0 / 256.0, 16.0 / 256.0, 1.0);
    out.clip_position = vec4<f32>((in.position + vec3(in.position_offset, 1.0)) * scale * atlas_scale, 1.0);
    return out;
}

@group(0) @binding(0)
var atlas: texture_2d<f32>;
@group(0) @binding(1)
var atlas_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(atlas, atlas_sampler, in.uv);
}
