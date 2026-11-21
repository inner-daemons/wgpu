#version 310 es

precision highp float;
precision highp int;

layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

ivec2 xvipaiai_1;

uvec2 xvupaiai_1;

vec2 xvfpaiai_1;

vec2 xvfpafaf_1;

vec2 xvfpaiaf_1;

uvec2 xvupuai_2;

uvec2 xvupaiu_2;

uvec2 xvuuai_2;

uvec2 xvuaiu_2;

ivec2 xvip_1;

uvec2 xvup_1;

vec2 xvfp_1;

mat2x2 xmfp_1;

mat2x2 xmfpaiaiaiai_1;

mat2x2 xmfpafaiaiai_1;

mat2x2 xmfpaiafaiai_1;

mat2x2 xmfpaiaiafai_1;

mat2x2 xmfpaiaiaiaf_1;

ivec2 xvispai_1;

vec2 xvfspaf_1;

ivec2 xvis_ai_1;

uvec2 xvus_ai_1;

vec2 xvfs_ai_1;

vec2 xvfs_af_1;

float xafafaf_1[2];

float xafaiai_1[2];

int xaipaiai_1[2];

uint xaupaiai[2];

float xafpaiaf_1[2];

float xafpafai_1[2];

float xafpafaf_1[2];

ivec3 xavipai_1[1];

vec3 xavfpai_1[1];

vec3 xavfpaf_1[1];

ivec2 xvisai_1;

uvec2 xvusai_1;

vec2 xvfsai_1;

vec2 xvfsaf_1;

ivec2 ivispai;

vec2 ivfspaf;

ivec2 ivis_ai;

uvec2 ivus_ai;

vec2 ivfs_ai;

vec2 ivfs_af;

float iafafaf[2];

float iafaiai[2];

int iaipaiai_1[2];

float iafpafaf_1[2];

float iafpaiaf_1[2];

float iafpafai_1[2];

ivec3 iavipai[1];

ivec3 iavfpai[1];

vec3 iavfpaf[1];


void globals() {
    ivec2 phony = xvipaiai_1;
    uvec2 phony_1 = xvupaiai_1;
    vec2 phony_2 = xvfpaiai_1;
    vec2 phony_3 = xvfpafaf_1;
    vec2 phony_4 = xvfpaiaf_1;
    uvec2 phony_5 = xvupuai_2;
    uvec2 phony_6 = xvupaiu_2;
    uvec2 phony_7 = xvuuai_2;
    uvec2 phony_8 = xvuaiu_2;
    ivec2 phony_9 = xvip_1;
    uvec2 phony_10 = xvup_1;
    vec2 phony_11 = xvfp_1;
    mat2x2 phony_12 = xmfp_1;
    mat2x2 phony_13 = xmfpaiaiaiai_1;
    mat2x2 phony_14 = xmfpafaiaiai_1;
    mat2x2 phony_15 = xmfpaiafaiai_1;
    mat2x2 phony_16 = xmfpaiaiafai_1;
    mat2x2 phony_17 = xmfpaiaiaiaf_1;
    ivec2 phony_18 = xvispai_1;
    vec2 phony_19 = xvfspaf_1;
    ivec2 phony_20 = xvis_ai_1;
    uvec2 phony_21 = xvus_ai_1;
    vec2 phony_22 = xvfs_ai_1;
    vec2 phony_23 = xvfs_af_1;
    float phony_24[2] = xafafaf_1;
    float phony_25[2] = xafaiai_1;
    int phony_26[2] = xaipaiai_1;
    uint phony_27[2] = xaupaiai;
    float phony_28[2] = xafpaiaf_1;
    float phony_29[2] = xafpafai_1;
    float phony_30[2] = xafpafaf_1;
    ivec3 phony_31[1] = xavipai_1;
    vec3 phony_32[1] = xavfpai_1;
    vec3 phony_33[1] = xavfpaf_1;
    ivec2 phony_34 = xvisai_1;
    uvec2 phony_35 = xvusai_1;
    vec2 phony_36 = xvfsai_1;
    vec2 phony_37 = xvfsaf_1;
    ivec2 phony_38 = ivispai;
    vec2 phony_39 = ivfspaf;
    ivec2 phony_40 = ivis_ai;
    uvec2 phony_41 = ivus_ai;
    vec2 phony_42 = ivfs_ai;
    vec2 phony_43 = ivfs_af;
    float phony_44[2] = iafafaf;
    float phony_45[2] = iafaiai;
    int phony_46[2] = iaipaiai_1;
    float phony_47[2] = iafpafaf_1;
    float phony_48[2] = iafpaiaf_1;
    float phony_49[2] = iafpafai_1;
    ivec3 phony_50[1] = iavipai;
    ivec3 phony_51[1] = iavfpai;
    vec3 phony_52[1] = iavfpaf;
    return;
}

void all_constant_arguments() {
    ivec2 xvipaiai;
    uvec2 xvupaiai;
    vec2 xvfpaiai;
    vec2 xvfpafaf;
    vec2 xvfpaiaf;
    uvec2 xvupuai;
    uvec2 xvupaiu;
    uvec2 xvuuai;
    uvec2 xvuaiu;
    ivec2 xvip;
    uvec2 xvup;
    vec2 xvfp;
    mat2x2 xmfp;
    mat2x2 xmfpaiaiaiai;
    mat2x2 xmfpafaiaiai;
    mat2x2 xmfpaiafaiai;
    mat2x2 xmfpaiaiafai;
    mat2x2 xmfpaiaiaiaf;
    mat2x2 xmfp_faiaiai;
    mat2x2 xmfpai_faiai;
    mat2x2 xmfpaiai_fai;
    mat2x2 xmfpaiaiai_f;
    ivec2 xvispai;
    vec2 xvfspaf;
    ivec2 xvis_ai;
    uvec2 xvus_ai;
    vec2 xvfs_ai;
    vec2 xvfs_af;
    float xafafaf[2];
    float xaf_faf[2];
    float xafaf_f[2];
    float xafaiai[2];
    int xai_iai[2];
    int xaiai_i[2];
    int xaipaiai[2];
    float xafpaiai[2];
    float xafpaiaf[2];
    float xafpafai[2];
    float xafpafaf[2];
    ivec3 xavipai[1];
    vec3 xavfpai[1];
    vec3 xavfpaf[1];
    ivec2 xvisai;
    uvec2 xvusai;
    vec2 xvfsai;
    vec2 xvfsaf;
    int iaipaiai[2];
    float iafpaiaf[2];
    float iafpafai[2];
    float iafpafaf[2];
    xvipaiai = ivec2(42, 43);
    xvupaiai = uvec2(44u, 45u);
    xvfpaiai = vec2(46.0, 47.0);
    xvfpafaf = vec2(48.0, 49.0);
    xvfpaiaf = vec2(48.0, 49.0);
    xvupuai = uvec2(42u, 43u);
    xvupaiu = uvec2(42u, 43u);
    xvuuai = uvec2(42u, 43u);
    xvuaiu = uvec2(42u, 43u);
    xvip = ivec2(0, 0);
    xvup = uvec2(0u, 0u);
    xvfp = vec2(0.0, 0.0);
    xmfp = mat2x2(vec2(0.0, 0.0), vec2(0.0, 0.0));
    xmfpaiaiaiai = mat2x2(vec2(1.0, 2.0), vec2(3.0, 4.0));
    xmfpafaiaiai = mat2x2(vec2(1.0, 2.0), vec2(3.0, 4.0));
    xmfpaiafaiai = mat2x2(vec2(1.0, 2.0), vec2(3.0, 4.0));
    xmfpaiaiafai = mat2x2(vec2(1.0, 2.0), vec2(3.0, 4.0));
    xmfpaiaiaiaf = mat2x2(vec2(1.0, 2.0), vec2(3.0, 4.0));
    xmfp_faiaiai = mat2x2(vec2(1.0, 2.0), vec2(3.0, 4.0));
    xmfpai_faiai = mat2x2(vec2(1.0, 2.0), vec2(3.0, 4.0));
    xmfpaiai_fai = mat2x2(vec2(1.0, 2.0), vec2(3.0, 4.0));
    xmfpaiaiai_f = mat2x2(vec2(1.0, 2.0), vec2(3.0, 4.0));
    xvispai = ivec2(1);
    xvfspaf = vec2(1.0);
    xvis_ai = ivec2(1);
    xvus_ai = uvec2(1u);
    xvfs_ai = vec2(1.0);
    xvfs_af = vec2(1.0);
    xafafaf = float[2](1.0, 2.0);
    xaf_faf = float[2](1.0, 2.0);
    xafaf_f = float[2](1.0, 2.0);
    xafaiai = float[2](1.0, 2.0);
    xai_iai = int[2](1, 2);
    xaiai_i = int[2](1, 2);
    xaipaiai = int[2](1, 2);
    xafpaiai = float[2](1.0, 2.0);
    xafpaiaf = float[2](1.0, 2.0);
    xafpafai = float[2](1.0, 2.0);
    xafpafaf = float[2](1.0, 2.0);
    xavipai = ivec3[1](ivec3(1));
    xavfpai = vec3[1](vec3(1.0));
    xavfpaf = vec3[1](vec3(1.0));
    xvisai = ivec2(1);
    xvusai = uvec2(1u);
    xvfsai = vec2(1.0);
    xvfsaf = vec2(1.0);
    iaipaiai = int[2](1, 2);
    iafpaiaf = float[2](1.0, 2.0);
    iafpafai = float[2](1.0, 2.0);
    iafpafaf = float[2](1.0, 2.0);
    xvipaiai = ivec2(42, 43);
    xvupaiai = uvec2(44u, 45u);
    xvfpaiai = vec2(46.0, 47.0);
    xvfpafaf = vec2(48.0, 49.0);
    xvfpaiaf = vec2(48.0, 49.0);
    xvupuai = uvec2(42u, 43u);
    xvupaiu = uvec2(42u, 43u);
    xvuuai = uvec2(42u, 43u);
    xvuaiu = uvec2(42u, 43u);
    xvip = ivec2(0, 0);
    xvup = uvec2(0u, 0u);
    xvfp = vec2(0.0, 0.0);
    xmfp = mat2x2(vec2(0.0, 0.0), vec2(0.0, 0.0));
    xmfpaiaiaiai = mat2x2(vec2(1.0, 2.0), vec2(3.0, 4.0));
    xmfpafaiaiai = mat2x2(vec2(1.0, 2.0), vec2(3.0, 4.0));
    xmfpaiafaiai = mat2x2(vec2(1.0, 2.0), vec2(3.0, 4.0));
    xmfpaiaiafai = mat2x2(vec2(1.0, 2.0), vec2(3.0, 4.0));
    xmfpaiaiaiaf = mat2x2(vec2(1.0, 2.0), vec2(3.0, 4.0));
    xmfp_faiaiai = mat2x2(vec2(1.0, 2.0), vec2(3.0, 4.0));
    xmfpai_faiai = mat2x2(vec2(1.0, 2.0), vec2(3.0, 4.0));
    xmfpaiai_fai = mat2x2(vec2(1.0, 2.0), vec2(3.0, 4.0));
    xmfpaiaiai_f = mat2x2(vec2(1.0, 2.0), vec2(3.0, 4.0));
    xvispai = ivec2(1);
    xvfspaf = vec2(1.0);
    xvis_ai = ivec2(1);
    xvus_ai = uvec2(1u);
    xvfs_ai = vec2(1.0);
    xvfs_af = vec2(1.0);
    xafafaf = float[2](1.0, 2.0);
    xaf_faf = float[2](1.0, 2.0);
    xafaf_f = float[2](1.0, 2.0);
    xafaiai = float[2](1.0, 2.0);
    xai_iai = int[2](1, 2);
    xaiai_i = int[2](1, 2);
    xaipaiai = int[2](1, 2);
    xafpaiai = float[2](1.0, 2.0);
    xafpaiaf = float[2](1.0, 2.0);
    xafpafai = float[2](1.0, 2.0);
    xafpafaf = float[2](1.0, 2.0);
    xavipai = ivec3[1](ivec3(1));
    xavfpai = vec3[1](vec3(1.0));
    xavfpaf = vec3[1](vec3(1.0));
    xvisai = ivec2(1);
    xvusai = uvec2(1u);
    xvfsai = vec2(1.0);
    xvfsaf = vec2(1.0);
    iaipaiai = int[2](1, 2);
    iafpaiaf = float[2](1.0, 2.0);
    iafpafai = float[2](1.0, 2.0);
    iafpafaf = float[2](1.0, 2.0);
    return;
}

void mixed_constant_and_runtime_arguments() {
    uint u;
    int i;
    float f;
    uvec2 xvupuai_1;
    uvec2 xvupaiu_1;
    vec2 xvfpfai;
    vec2 xvfpfaf;
    uvec2 xvuuai_1;
    uvec2 xvuaiu_1;
    mat2x2 xmfp_faiaiai_1;
    mat2x2 xmfpai_faiai_1;
    mat2x2 xmfpaiai_fai_1;
    mat2x2 xmfpaiaiai_f_1;
    float xaf_faf_1[2];
    float xafaf_f_1[2];
    float xaf_fai[2];
    float xafai_f[2];
    int xai_iai_1[2];
    int xaiai_i_1[2];
    float xafp_faf[2];
    float xafpaf_f[2];
    float xafp_fai[2];
    float xafpai_f[2];
    int xaip_iai[2];
    int xaipai_i[2];
    ivec2 xvisi;
    uvec2 xvusu;
    vec2 xvfsf;
    u = 0u;
    i = 0;
    f = 0.0;
    xvupuai_1 = uvec2(0u);
    xvupaiu_1 = uvec2(0u);
    xvfpfai = vec2(0.0);
    xvfpfaf = vec2(0.0);
    xvuuai_1 = uvec2(0u);
    xvuaiu_1 = uvec2(0u);
    xmfp_faiaiai_1 = mat2x2(0.0);
    xmfpai_faiai_1 = mat2x2(0.0);
    xmfpaiai_fai_1 = mat2x2(0.0);
    xmfpaiaiai_f_1 = mat2x2(0.0);
    xaf_faf_1 = float[2](0.0, 0.0);
    xafaf_f_1 = float[2](0.0, 0.0);
    xaf_fai = float[2](0.0, 0.0);
    xafai_f = float[2](0.0, 0.0);
    xai_iai_1 = int[2](0, 0);
    xaiai_i_1 = int[2](0, 0);
    xafp_faf = float[2](0.0, 0.0);
    xafpaf_f = float[2](0.0, 0.0);
    xafp_fai = float[2](0.0, 0.0);
    xafpai_f = float[2](0.0, 0.0);
    xaip_iai = int[2](0, 0);
    xaipai_i = int[2](0, 0);
    xvisi = ivec2(0);
    xvusu = uvec2(0u);
    xvfsf = vec2(0.0);
    uint _e3 = u;
    xvupuai_1 = uvec2(_e3, 43u);
    uint _e7 = u;
    xvupaiu_1 = uvec2(42u, _e7);
    float _e11 = f;
    xvfpfai = vec2(_e11, 47.0);
    float _e15 = f;
    xvfpfaf = vec2(_e15, 49.0);
    uint _e19 = u;
    xvuuai_1 = uvec2(_e19, 43u);
    uint _e23 = u;
    xvuaiu_1 = uvec2(42u, _e23);
    float _e27 = f;
    xmfp_faiaiai_1 = mat2x2(vec2(_e27, 2.0), vec2(3.0, 4.0));
    float _e35 = f;
    xmfpai_faiai_1 = mat2x2(vec2(1.0, _e35), vec2(3.0, 4.0));
    float _e43 = f;
    xmfpaiai_fai_1 = mat2x2(vec2(1.0, 2.0), vec2(_e43, 4.0));
    float _e51 = f;
    xmfpaiaiai_f_1 = mat2x2(vec2(1.0, 2.0), vec2(3.0, _e51));
    float _e59 = f;
    xaf_faf_1 = float[2](_e59, 2.0);
    float _e63 = f;
    xafaf_f_1 = float[2](1.0, _e63);
    float _e67 = f;
    xaf_fai = float[2](_e67, 2.0);
    float _e71 = f;
    xafai_f = float[2](1.0, _e71);
    int _e75 = i;
    xai_iai_1 = int[2](_e75, 2);
    int _e79 = i;
    xaiai_i_1 = int[2](1, _e79);
    float _e83 = f;
    xafp_faf = float[2](_e83, 2.0);
    float _e87 = f;
    xafpaf_f = float[2](1.0, _e87);
    float _e91 = f;
    xafp_fai = float[2](_e91, 2.0);
    float _e95 = f;
    xafpai_f = float[2](1.0, _e95);
    int _e99 = i;
    xaip_iai = int[2](_e99, 2);
    int _e103 = i;
    xaipai_i = int[2](1, _e103);
    int _e107 = i;
    xvisi = ivec2(_e107);
    uint _e110 = u;
    xvusu = uvec2(_e110);
    float _e113 = f;
    xvfsf = vec2(_e113);
    uint _e116 = u;
    xvupuai_1 = uvec2(_e116, 43u);
    uint _e119 = u;
    xvupaiu_1 = uvec2(42u, _e119);
    uint _e122 = u;
    xvuuai_1 = uvec2(_e122, 43u);
    uint _e125 = u;
    xvuaiu_1 = uvec2(42u, _e125);
    float _e128 = f;
    xmfp_faiaiai_1 = mat2x2(vec2(_e128, 2.0), vec2(3.0, 4.0));
    float _e135 = f;
    xmfpai_faiai_1 = mat2x2(vec2(1.0, _e135), vec2(3.0, 4.0));
    float _e142 = f;
    xmfpaiai_fai_1 = mat2x2(vec2(1.0, 2.0), vec2(_e142, 4.0));
    float _e149 = f;
    xmfpaiaiai_f_1 = mat2x2(vec2(1.0, 2.0), vec2(3.0, _e149));
    float _e156 = f;
    xaf_faf_1 = float[2](_e156, 2.0);
    float _e159 = f;
    xafaf_f_1 = float[2](1.0, _e159);
    float _e162 = f;
    xaf_fai = float[2](_e162, 2.0);
    float _e165 = f;
    xafai_f = float[2](1.0, _e165);
    int _e168 = i;
    xai_iai_1 = int[2](_e168, 2);
    int _e171 = i;
    xaiai_i_1 = int[2](1, _e171);
    float _e174 = f;
    xafp_faf = float[2](_e174, 2.0);
    float _e177 = f;
    xafpaf_f = float[2](1.0, _e177);
    float _e180 = f;
    xafp_fai = float[2](_e180, 2.0);
    float _e183 = f;
    xafpai_f = float[2](1.0, _e183);
    int _e186 = i;
    xaip_iai = int[2](_e186, 2);
    int _e189 = i;
    xaipai_i = int[2](1, _e189);
    int _e192 = i;
    xvisi = ivec2(_e192);
    uint _e194 = u;
    xvusu = uvec2(_e194);
    float _e196 = f;
    xvfsf = vec2(_e196);
    return;
}

void main() {
    xvipaiai_1 = ivec2(42, 43);
    xvupaiai_1 = uvec2(44u, 45u);
    xvfpaiai_1 = vec2(46.0, 47.0);
    xvfpafaf_1 = vec2(48.0, 49.0);
    xvfpaiaf_1 = vec2(48.0, 49.0);
    xvupuai_2 = uvec2(42u, 43u);
    xvupaiu_2 = uvec2(42u, 43u);
    xvuuai_2 = uvec2(42u, 43u);
    xvuaiu_2 = uvec2(42u, 43u);
    xvip_1 = ivec2(0, 0);
    xvup_1 = uvec2(0u, 0u);
    xvfp_1 = vec2(0.0, 0.0);
    xmfp_1 = mat2x2(vec2(0.0, 0.0), vec2(0.0, 0.0));
    xmfpaiaiaiai_1 = mat2x2(vec2(1.0, 2.0), vec2(3.0, 4.0));
    xmfpafaiaiai_1 = mat2x2(vec2(1.0, 2.0), vec2(3.0, 4.0));
    xmfpaiafaiai_1 = mat2x2(vec2(1.0, 2.0), vec2(3.0, 4.0));
    xmfpaiaiafai_1 = mat2x2(vec2(1.0, 2.0), vec2(3.0, 4.0));
    xmfpaiaiaiaf_1 = mat2x2(vec2(1.0, 2.0), vec2(3.0, 4.0));
    xvispai_1 = ivec2(1);
    xvfspaf_1 = vec2(1.0);
    xvis_ai_1 = ivec2(1);
    xvus_ai_1 = uvec2(1u);
    xvfs_ai_1 = vec2(1.0);
    xvfs_af_1 = vec2(1.0);
    xafafaf_1 = float[2](1.0, 2.0);
    xafaiai_1 = float[2](1.0, 2.0);
    xaipaiai_1 = int[2](1, 2);
    xaupaiai = uint[2](1u, 2u);
    xafpaiaf_1 = float[2](1.0, 2.0);
    xafpafai_1 = float[2](1.0, 2.0);
    xafpafaf_1 = float[2](1.0, 2.0);
    xavipai_1 = ivec3[1](ivec3(1));
    xavfpai_1 = vec3[1](vec3(1.0));
    xavfpaf_1 = vec3[1](vec3(1.0));
    xvisai_1 = ivec2(1);
    xvusai_1 = uvec2(1u);
    xvfsai_1 = vec2(1.0);
    xvfsaf_1 = vec2(1.0);
    ivispai = ivec2(1);
    ivfspaf = vec2(1.0);
    ivis_ai = ivec2(1);
    ivus_ai = uvec2(1u);
    ivfs_ai = vec2(1.0);
    ivfs_af = vec2(1.0);
    iafafaf = float[2](1.0, 2.0);
    iafaiai = float[2](1.0, 2.0);
    iaipaiai_1 = int[2](1, 2);
    iafpafaf_1 = float[2](1.0, 2.0);
    iafpaiaf_1 = float[2](1.0, 2.0);
    iafpafai_1 = float[2](1.0, 2.0);
    iavipai = ivec3[1](ivec3(1));
    iavfpai = ivec3[1](ivec3(1));
    iavfpaf = vec3[1](vec3(1.0));
    globals();
    all_constant_arguments();
    mixed_constant_and_runtime_arguments();
    return;
}

