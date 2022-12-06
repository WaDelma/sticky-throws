struct CustomMaterial {
    color: vec4<f32>,
};

@group(1) @binding(0)
var<uniform> material: CustomMaterial;
@group(1) @binding(1)
var<uniform> sticky: i32;
@group(1) @binding(2)
var base_color_texture: texture_2d<f32>;
@group(1) @binding(3)
var base_color_sampler: sampler;

fn make_kernel(tex: texture_2d<f32>, s: sampler, coord: vec2<f32>) -> array<vec4<f32>, 9> {
    let w = 1.0 / 50.;
    let h = 1.0 / 50.;

    var n: array<vec4<f32>,9>;
    n[0] = textureSample(tex, s, coord + vec2(-w, -h));
    n[1] = textureSample(tex, s, coord + vec2(0., -h));
    n[2] = textureSample(tex, s, coord + vec2(w, -h));
    n[3] = textureSample(tex, s, coord + vec2(-w, 0.));
    n[4] = textureSample(tex, s, coord);
    n[5] = textureSample(tex, s, coord + vec2(w, 0.));
    n[6] = textureSample(tex, s, coord + vec2(-w, h));
    n[7] = textureSample(tex, s, coord + vec2(0., h));
    n[8] = textureSample(tex, s, coord + vec2(w, h));
    return n;
}

@fragment
fn fragment(
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {
    // TODO: IDE doesn't know the type of uv. Maybe because of the import?
    let uv: vec2<f32> = uv;

    let n = make_kernel(base_color_texture, base_color_sampler, uv);
    let sobel_edge_h = n[2] + (2.0 * n[5]) + n[8] - (n[0] + (2.0 * n[3]) + n[6]);
    let sobel_edge_v = n[0] + (2.0 * n[1]) + n[2] - (n[6] + (2.0 * n[7]) + n[8]);
    let sobel = sqrt((sobel_edge_h * sobel_edge_h) + (sobel_edge_v * sobel_edge_v));
    // NOTE: This has to be outside of the if because of LOD: https://github.com/gfx-rs/wgpu-rs/issues/912
    let col = textureSample(base_color_texture, base_color_sampler, uv);
    if sticky == 1 && length(sobel.a) > 2.5 {
        return material.color;
    } else {
        return col;
    }
    // return vec4(col.rgb * sobel.rgb, 1.);//vec4(1.0 - sobel.rgb, 1.0);
    // let col = textureSample(base_color_texture, base_color_sampler, uv);
    // if (distance(uv, vec2(0.5, 0.5)) > 0.5) {
    //     return material.color;
    // } else {
    //     return col;
    // }
}