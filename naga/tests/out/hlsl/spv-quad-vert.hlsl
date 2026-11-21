struct gl_PerVertex {
    float4 gl_Position : SV_Position;
    float gl_PointSize;
    float gl_ClipDistance[1];
    float gl_CullDistance[1];
    int _end_pad_0;
};

struct VertexOutput {
    float2 member : LOC0;
    float4 gl_Position : SV_Position;
};

static float2 v_uv;
static float2 a_uv_1;
static gl_PerVertex unnamed;
static float2 a_pos_1;

struct VertexOutput_main {
    float2 member : LOC0;
    float4 gl_Position : SV_Position;
};

void main_1()
{
    float2 _e6 = a_uv_1;
    v_uv = _e6;
    float2 _e7 = a_pos_1;
    unnamed.gl_Position = float4(_e7.x, _e7.y, 0.0, 1.0);
    return;
}

VertexOutput ConstructVertexOutput(float2 arg0, float4 arg1) {
    VertexOutput ret = (VertexOutput)0;
    ret.member = arg0;
    ret.gl_Position = arg1;
    return ret;
}

gl_PerVertex Constructgl_PerVertex(float4 arg0, float arg1, float arg2[1], float arg3[1]) {
    gl_PerVertex ret = (gl_PerVertex)0;
    ret.gl_Position = arg0;
    ret.gl_PointSize = arg1;
    ret.gl_ClipDistance = arg2;
    ret.gl_CullDistance = arg3;
    return ret;
}

float2 ZeroValuefloat2() {
    return (float2)0;
}

typedef float ret_ZeroValuearray1_float_[1];
ret_ZeroValuearray1_float_ ZeroValuearray1_float_() {
    return (float[1])0;
}

VertexOutput_main main(float2 a_uv : LOC1, float2 a_pos : LOC0)
{
    v_uv = ZeroValuefloat2();
    a_uv_1 = ZeroValuefloat2();
    unnamed = Constructgl_PerVertex(float4(0.0, 0.0, 0.0, 1.0), 1.0, ZeroValuearray1_float_(), ZeroValuearray1_float_());
    a_pos_1 = ZeroValuefloat2();
    a_uv_1 = a_uv;
    a_pos_1 = a_pos;
    main_1();
    float2 _e7 = v_uv;
    float4 _e8 = unnamed.gl_Position;
    const VertexOutput vertexoutput = ConstructVertexOutput(_e7, _e8);
    const VertexOutput_main vertexoutput_1 = { vertexoutput.member, vertexoutput.gl_Position };
    return vertexoutput_1;
}
