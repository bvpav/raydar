struct HitRecord {
    bool is_hit;
    float hit_distance;
    bool is_front_face;
    vec3 world_position;
    vec3 world_normal;
    uint material_index;
};

struct Sphere {
    vec3 center;
    float radius;
};
