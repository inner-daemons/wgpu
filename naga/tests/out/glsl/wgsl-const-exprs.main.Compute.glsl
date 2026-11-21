#version 310 es

precision highp float;
precision highp int;

layout(local_size_x = 2, local_size_y = 3, local_size_z = 1) in;

const uint TWO = 2u;
const int THREE = 3;
const bool TRUE = true;
const bool FALSE = false;
const int FOUR = 4;
const int TEXTURE_KIND_REGULAR = 0;
const int TEXTURE_KIND_WARP = 1;
const int TEXTURE_KIND_SKY = 2;
const int FOUR_ALIAS = 4;
const int TEST_CONSTANT_ADDITION = 8;
const int TEST_CONSTANT_ALIAS_ADDITION = 8;
const float PI = 3.141;
const float phi_sun = 6.282;
const vec4 DIV = vec4(0.44444445, 0.0, 0.0, 0.0);
const vec2 add_vec = vec2(4.0, 5.0);
const bvec2 compare_vec = bvec2(true, false);


void swizzle_of_compose() {
    ivec4 out_;
    out_ = ivec4(4, 3, 2, 1);
    return;
}

void index_of_compose() {
    int out_1;
    out_1 = 2;
    return;
}

void compose_three_deep() {
    int out_2;
    out_2 = 6;
    return;
}

void non_constant_initializers() {
    int w;
    int x;
    int y;
    int z;
    ivec4 out_3;
    w = 30;
    x = 0;
    y = 0;
    z = 70;
    out_3 = ivec4(0);
    int _e2 = w;
    x = _e2;
    int _e4 = x;
    y = _e4;
    int _e8 = w;
    int _e9 = x;
    int _e10 = y;
    int _e11 = z;
    out_3 = ivec4(_e8, _e9, _e10, _e11);
    return;
}

void splat_of_constant() {
    ivec4 out_4;
    out_4 = ivec4(-4, -4, -4, -4);
    return;
}

void compose_of_constant() {
    ivec4 out_5;
    out_5 = ivec4(-4, -4, -4, -4);
    return;
}

uint map_texture_kind(int texture_kind) {
    switch(texture_kind) {
        case 0: {
            return 10u;
        }
        case 1: {
            return 20u;
        }
        case 2: {
            return 30u;
        }
        default: {
            return 0u;
        }
    }
}

void compose_of_splat() {
    vec4 x_1;
    x_1 = vec4(2.0, 1.0, 1.0, 1.0);
    return;
}

void test_local_const() {
    float arr[2];
    arr = float[2](0.0, 0.0);
    return;
}

void compose_vector_zero_val_binop() {
    ivec3 a;
    ivec3 b;
    ivec3 c;
    a = ivec3(1, 1, 1);
    b = ivec3(0, 1, 2);
    c = ivec3(1, 0, 2);
    return;
}

void relational() {
    bool scalar_any_false;
    bool scalar_any_true;
    bool scalar_all_false;
    bool scalar_all_true;
    bool vec_any_false;
    bool vec_any_true;
    bool vec_all_false;
    bool vec_all_true;
    scalar_any_false = false;
    scalar_any_true = true;
    scalar_all_false = false;
    scalar_all_true = true;
    vec_any_false = false;
    vec_any_true = true;
    vec_all_false = false;
    vec_all_true = true;
    return;
}

void packed_dot_product() {
    int signed_four;
    uint unsigned_four;
    int signed_twelve;
    uint unsigned_twelve;
    int signed_seventy;
    uint unsigned_seventy;
    int minus_four;
    signed_four = 4;
    unsigned_four = 4u;
    signed_twelve = 12;
    unsigned_twelve = 12u;
    signed_seventy = 70;
    unsigned_seventy = 70u;
    minus_four = -4;
    return;
}

void abstract_access(uint i) {
    float a_1;
    uint b_1;
    int c_1;
    int d;
    a_1 = 1.0;
    b_1 = 1u;
    c_1 = 0;
    d = 0;
    c_1 = int[9](1, 2, 3, 4, 5, 6, 7, 8, 9)[i];
    d = ivec4(1, 2, 3, 4)[i];
    return;
}

void main() {
    swizzle_of_compose();
    index_of_compose();
    compose_three_deep();
    non_constant_initializers();
    splat_of_constant();
    compose_of_constant();
    uint _e1 = map_texture_kind(1);
    compose_of_splat();
    test_local_const();
    compose_vector_zero_val_binop();
    relational();
    packed_dot_product();
    test_local_const();
    abstract_access(1u);
    return;
}

