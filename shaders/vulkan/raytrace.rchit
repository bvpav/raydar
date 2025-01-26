#version 460
#extension GL_EXT_ray_tracing : require

#include "common.glsl"

struct Vertex {
    vec3 position;
    vec3 normal;
};

layout(set = 0, binding = 3) buffer _VertexBuffer { Vertex cube_vertices[]; };
layout(set = 0, binding = 4) buffer _IndexBuffer { uint cube_indices[]; };

layout(location = 0) rayPayloadInEXT HitRecord hit_record;

hitAttributeEXT vec2 attribs;

vec3 get_normal();

void main() {
    hit_record.is_hit = true;

    hit_record.hit_distance = gl_HitTEXT;
    hit_record.world_position = gl_WorldRayOriginEXT + gl_WorldRayDirectionEXT * gl_HitTEXT;
    hit_record.world_normal = get_normal();
}

vec3 get_normal() {
    // Since it's a cube, we can just use the first vertex's normal
    Vertex v0 = cube_vertices[cube_indices[3 * gl_PrimitiveID]];
    return v0.normal;
}
