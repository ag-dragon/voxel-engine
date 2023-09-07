struct GlobalUniform {
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0)
var<uniform> globals: GlobalUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) ao: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) ao: f32,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = globals.view_proj * vec4<f32>(model.position, 1.0);
    out.ao = model.ao;
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    /*
    var output: vec4<f32>;
    var tout = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    if in.ao > 0.0 {
        output = vec4<f32>(0.5, 0.5, 0.5, 1.0);
    } else {
        output = tout;
    }
    return output;
    */
    return mix(textureSample(t_diffuse, s_diffuse, in.tex_coords), vec4<f32>(0.0, 0.0, 0.0, 1.0), in.ao * 0.3);
}
