#version 460
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

void main()
{
    ivec2 _18 = ivec2(1, 2);
    vec2 _20 = vec2(0.0);
    _20 = mix(vec2(1.0, 0.0), vec2(0.0, 1.0), bvec2(_18.x < _18.y));
}

