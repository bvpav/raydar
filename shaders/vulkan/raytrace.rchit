#version 460
#extension GL_EXT_ray_tracing : require

#include "common.glsl"

layout(set = 0, binding = 3) buffer _VertexBuffer { float cube_vertex_data[]; };
layout(set = 0, binding = 4) buffer _IndexBuffer { uint cube_indices[]; };

layout(location = 0) rayPayloadInEXT HitRecord hit_record;

hitAttributeEXT vec2 attribs;

vec3 get_normal(uint index);

void main() {
    hit_record.is_hit = true;

    hit_record.hit_distance = gl_HitTEXT;
    hit_record.world_position = gl_WorldRayOriginEXT + gl_WorldRayDirectionEXT * gl_HitTEXT;
    // FIXME: We are only using the normal of the first vertex, not blending the normals of the other vertices in this triangle.
    //        Since we only have cubes, this is correct for now.
    hit_record.world_normal = get_normal(cube_indices[3 * gl_PrimitiveID]);
    hit_record.is_front_face = dot(hit_record.world_normal, gl_WorldRayDirectionEXT) <= 0.0;
    hit_record.world_normal *= float(hit_record.is_front_face) * 2.0 - 1.0;
}

vec3 get_normal(uint index) {
    const uint offset = 3;
    const uint stride = 6;
    float x = cube_vertex_data[offset + index * stride + 0];
    float y = cube_vertex_data[offset + index * stride + 1];
    float z = cube_vertex_data[offset + index * stride + 2];
    return vec3(x, y, z);
}
