#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings

#import bevy_pbr::pbr_types

#import bevy_pbr::utils
#import bevy_pbr::clustered_forward
#import bevy_pbr::lighting
#import bevy_pbr::shadows
#import bevy_pbr::pbr_functions

struct CustomMaterial {
    normals: array<vec3<f32>, 6>,
    // normals: vec4<f32>,
};

@group(1) @binding(0)
var<uniform> material: CustomMaterial;


#import bevy_pbr::mesh_functions
struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) blend_color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) frag_coord: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) blend_color: vec4<f32>,
    @location(2) world_normal: vec3<f32>,
};

// struct FragmentInput {
//     @location(0) blend_color: vec4<f32>,
// };

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.frag_coord = mesh_position_local_to_clip(mesh.model, vec4<f32>(vertex.position, 1.0));
    out.world_position = mesh.model * vec4<f32>(vertex.position, 1.0);
    out.blend_color = vertex.blend_color;
    out.world_normal = material.normals[(vertex.instance_index / 8u) % 8u];
    return out;
}

@fragment
fn fragment(
    in: VertexOutput
    // #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    // return vec4<f32>(1.0,0.0,0.0,1.0);
    var output_color: vec4<f32> = in.blend_color;
    var material = standard_material_new();
    material.base_color = in.blend_color;
    material.perceptual_roughness = 0.5;
    material.metallic = 0.02;
    material.reflectance = 0.5;
        // Prepare a 'processed' StandardMaterial by sampling all textures to resolve
        // the material members
    var pbr_input: PbrInput;

    pbr_input.material = material;

    var occlusion: f32 = 1.0;
    pbr_input.occlusion = occlusion;

    pbr_input.frag_coord = in.frag_coord;
    pbr_input.world_position = in.world_position;
    pbr_input.world_normal = prepare_world_normal(
        in.world_normal,
        (material.flags & STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u,
        false, // was in.is_front before
    );

    pbr_input.is_orthographic = view.projection[3].w == 1.0;

    pbr_input.N = apply_normal_mapping(
        material.flags,
        pbr_input.world_normal,
#ifdef VERTEX_TANGENTS
#ifdef STANDARDMATERIAL_NORMAL_MAP
        in.world_tangent,
#endif
#endif
#ifdef VERTEX_UVS
        // in.uv,
        vec2<f32>(1.0, 0.0)
#endif
    );
    pbr_input.V = calculate_view(in.world_position, pbr_input.is_orthographic);
    output_color = pbr(pbr_input);
    

// #ifdef TONEMAP_IN_SHADER
        output_color = tone_mapping(output_color);
// #endif
#ifdef DEBAND_DITHER
    var output_rgb = output_color.rgb;
    output_rgb = pow(output_rgb, vec3<f32>(1.0 / 2.2));
    output_rgb = output_rgb + screen_space_dither(in.frag_coord.xy);
    // This conversion back to linear space is required because our output texture format is
    // SRGB; the GPU will assume our output is linear and will apply an SRGB conversion.
    output_rgb = pow(output_rgb, vec3<f32>(2.2));
    output_color = vec4(output_rgb, output_color.a);
#endif
#ifdef PREMULTIPLY_ALPHA
        output_color = premultiply_alpha(material.flags, output_color);
#endif
    return output_color;
    // return vec4<f32>(1.0,0.0,0.0,1.0);
}