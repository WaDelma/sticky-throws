struct TilingMaterial {
    dims: vec4<f32>,
};

@group(1) @binding(0)
var<uniform> material: TilingMaterial;
@group(1) @binding(1)
var base_color_texture: texture_2d<f32>;
@group(1) @binding(2)
var base_color_sampler: sampler;

@fragment
fn fragment(
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {
    // TODO: IDE doesn't know the type of uv. Maybe because of the import?
    let dims = textureDimensions(base_color_texture);
    let dms = material.dims.xy * 0.05;
    let uv: vec2<f32> = uv * dms;
    return textureSample(base_color_texture, base_color_sampler, uv);
    // return vec4(col.rgb * sobel.rgb, 1.);//vec4(1.0 - sobel.rgb, 1.0);
    // let col = textureSample(base_color_texture, base_color_sampler, uv);
    // if (distance(uv, vec2(0.5, 0.5)) > 0.5) {
    //     return material.color;
    // } else {
    //     return col;
    // }
}