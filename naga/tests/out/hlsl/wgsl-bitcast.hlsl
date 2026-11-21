[numthreads(1, 1, 1)]
void main()
{
    int2 i2_;
    int3 i3_;
    int4 i4_;
    uint2 u2_;
    uint3 u3_;
    uint4 u4_;
    float2 f2_;
    float3 f3_;
    float4 f4_;

    i2_ = (int(0)).xx;
    i3_ = (int(0)).xxx;
    i4_ = (int(0)).xxxx;
    u2_ = (0u).xx;
    u3_ = (0u).xxx;
    u4_ = (0u).xxxx;
    f2_ = (0.0).xx;
    f3_ = (0.0).xxx;
    f4_ = (0.0).xxxx;
    int2 _e9 = i2_;
    u2_ = asuint(_e9);
    int3 _e11 = i3_;
    u3_ = asuint(_e11);
    int4 _e13 = i4_;
    u4_ = asuint(_e13);
    uint2 _e15 = u2_;
    i2_ = asint(_e15);
    uint3 _e17 = u3_;
    i3_ = asint(_e17);
    uint4 _e19 = u4_;
    i4_ = asint(_e19);
    int2 _e21 = i2_;
    f2_ = asfloat(_e21);
    int3 _e23 = i3_;
    f3_ = asfloat(_e23);
    int4 _e25 = i4_;
    f4_ = asfloat(_e25);
    return;
}
