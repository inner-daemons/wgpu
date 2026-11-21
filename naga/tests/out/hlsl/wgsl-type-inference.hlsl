int4 ZeroValueint4() {
    return (int4)0;
}

static const uint g1_ = 1u;
static const float g3_ = 1.0;
static const int4 g4_ = ZeroValueint4();
static const int4 g5_ = (int(1)).xxxx;
static const float2x2 g6_ = float2x2(float2(0.0, 0.0), float2(0.0, 0.0));

[numthreads(1, 1, 1)]
void main()
{
    int g0x;
    float g2x;
    float2x2 g7x;
    int c0x;
    uint c1x;
    float c2x;
    float c3x;
    int4 c4x;
    int4 c5x;
    float2x2 c6x;
    float2x2 c7x;
    int l0x;
    uint l1x;
    float l2x;
    float l3x;
    int4 l4x;
    int v0_;
    uint v1_;
    float v2_;
    float v3_;
    int4 v4_;
    int4 v5_;
    float2x2 v6_;
    float2x2 v7_;

    g0x = int(1);
    g2x = 1.0;
    g7x = float2x2(float2(1.0, 1.0), float2(1.0, 1.0));
    c0x = int(1);
    c1x = 1u;
    c2x = 1.0;
    c3x = 1.0;
    c4x = ZeroValueint4();
    c5x = (int(1)).xxxx;
    c6x = float2x2(float2(0.0, 0.0), float2(0.0, 0.0));
    c7x = float2x2(float2(1.0, 1.0), float2(1.0, 1.0));
    l0x = int(0);
    l1x = 0u;
    l2x = 0.0;
    l3x = 0.0;
    l4x = ZeroValueint4();
    v0_ = int(1);
    v1_ = 1u;
    v2_ = 1.0;
    v3_ = 1.0;
    v4_ = ZeroValueint4();
    v5_ = (int(1)).xxxx;
    v6_ = float2x2(float2(0.0, 0.0), float2(0.0, 0.0));
    v7_ = float2x2(float2(1.0, 1.0), float2(1.0, 1.0));
    int4 l5_ = (int(1)).xxxx;
    float2x2 l6_ = float2x2(float2(0.0, 0.0), float2(0.0, 0.0));
    float2x2 l7_ = float2x2(float2(1.0, 1.0), float2(1.0, 1.0));
    l0x = int(1);
    l1x = 1u;
    l2x = 1.0;
    l3x = 1.0;
    l4x = ZeroValueint4();
    return;
}
