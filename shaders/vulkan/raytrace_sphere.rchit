#version 460
#extension GL_EXT_ray_tracing : require

#include "common.glsl"

layout(set = 0, binding = 6) buffer _Sphere { Sphere[] spheres; };

layout(location = 0) rayPayloadInEXT HitRecord hit_record;

hitAttributeEXT vec2 attribs;

vec3 get_normal(uint index);

void main() {
    hit_record.is_hit = true;

    Sphere sphere = spheres[gl_InstanceCustomIndexEXT];

    hit_record.hit_distance = gl_HitTEXT;
    hit_record.world_position = gl_WorldRayOriginEXT + gl_WorldRayDirectionEXT * gl_HitTEXT;
    hit_record.world_normal = normalize(hit_record.world_position - sphere.center);
    hit_record.is_front_face = dot(hit_record.world_normal, gl_WorldRayDirectionEXT) <= 0.0;
    hit_record.world_normal *= float(hit_record.is_front_face) * 2.0 - 1.0;

    hit_record.material_index = gl_InstanceCustomIndexEXT;
}
