#version 460
#extension GL_EXT_ray_tracing : require

struct HitRecord {
  bool is_hit;
  // float hit_distance;
  // bool is_front_face;
  // vec3 world_position;
  // vec3 world_normal;
};

layout(location = 0) rayPayloadInEXT HitRecord hit_record;

void main() {
    hit_record.is_hit = true;
}
