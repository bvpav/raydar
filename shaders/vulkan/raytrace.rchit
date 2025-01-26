#version 460
#extension GL_EXT_ray_tracing : require

#include "common.glsl"

layout(location = 0) rayPayloadInEXT HitRecord hit_record;

hitAttributeEXT vec2 attribs;

void main() {
    hit_record.is_hit = true;

    hit_record.hit_distance = gl_HitTEXT;
    hit_record.world_position = gl_WorldRayOriginEXT + gl_WorldRayDirectionEXT * gl_HitTEXT;
    hit_record.world_normal = vec3(1.0 - attribs.x - attribs.y, attribs.x, attribs.y);
}
