struct NagaConstants {
    int first_vertex;
    int first_instance;
    uint other;
};
ConstantBuffer<NagaConstants> _NagaConstants: register(b0, space1);

int naga_bitcast_half2_to_int(half2 value) {
    uint16_t2 bits = asuint16(value);
    return asint((uint(bits.x) | (uint(bits.y) << 16)));
}

uint naga_bitcast_half2_to_uint(half2 value) {
    uint16_t2 bits = asuint16(value);
    return (uint(bits.x) | (uint(bits.y) << 16));
}

float naga_bitcast_half2_to_float(half2 value) {
    uint16_t2 bits = asuint16(value);
    return asfloat((uint(bits.x) | (uint(bits.y) << 16)));
}

int2 naga_bitcast_half4_to_int2(half4 value) {
    uint16_t4 bits = asuint16(value);
    return asint(uint2((uint(bits.x) | (uint(bits.y) << 16)), (uint(bits.z) | (uint(bits.w) << 16))));
}

uint2 naga_bitcast_half4_to_uint2(half4 value) {
    uint16_t4 bits = asuint16(value);
    return uint2((uint(bits.x) | (uint(bits.y) << 16)), (uint(bits.z) | (uint(bits.w) << 16)));
}

float2 naga_bitcast_half4_to_float2(half4 value) {
    uint16_t4 bits = asuint16(value);
    return asfloat(uint2((uint(bits.x) | (uint(bits.y) << 16)), (uint(bits.z) | (uint(bits.w) << 16))));
}

half2 naga_bitcast_int_to_half2(int value) {
    uint bits = asuint(value);
    return asfloat16(uint16_t2(uint16_t(bits & 0xFFFFu), uint16_t(bits >> 16)));
}

half2 naga_bitcast_uint_to_half2(uint value) {
    uint bits = value;
    return asfloat16(uint16_t2(uint16_t(bits & 0xFFFFu), uint16_t(bits >> 16)));
}

half2 naga_bitcast_float_to_half2(float value) {
    uint bits = asuint(value);
    return asfloat16(uint16_t2(uint16_t(bits & 0xFFFFu), uint16_t(bits >> 16)));
}

half4 naga_bitcast_int2_to_half4(int2 value) {
    uint2 bits = asuint(value);
    return asfloat16(uint16_t4(uint16_t(bits.x & 0xFFFFu), uint16_t(bits.x >> 16), uint16_t(bits.y & 0xFFFFu), uint16_t(bits.y >> 16)));
}

half4 naga_bitcast_uint2_to_half4(uint2 value) {
    uint2 bits = value;
    return asfloat16(uint16_t4(uint16_t(bits.x & 0xFFFFu), uint16_t(bits.x >> 16), uint16_t(bits.y & 0xFFFFu), uint16_t(bits.y >> 16)));
}

half4 naga_bitcast_float2_to_half4(float2 value) {
    uint2 bits = asuint(value);
    return asfloat16(uint16_t4(uint16_t(bits.x & 0xFFFFu), uint16_t(bits.x >> 16), uint16_t(bits.y & 0xFFFFu), uint16_t(bits.y >> 16)));
}

[numthreads(1, 1, 1)]
void main()
{
    half2 h2_;
    half4 h4_;
    int i1_;
    uint u1_;
    float f1_;
    int2 i2_;
    uint2 u2_;
    float2 f2_;

    h2_ = half2(1.0h, 2.0h);
    h4_ = half4(1.0h, 2.0h, 3.0h, 4.0h);
    i1_ = int(0);
    u1_ = 0u;
    f1_ = 0.0;
    i2_ = (int(0)).xx;
    u2_ = (0u).xx;
    f2_ = (0.0).xx;
    half2 _e11 = h2_;
    i1_ = naga_bitcast_half2_to_int(_e11);
    half2 _e13 = h2_;
    u1_ = naga_bitcast_half2_to_uint(_e13);
    half2 _e15 = h2_;
    f1_ = naga_bitcast_half2_to_float(_e15);
    half4 _e17 = h4_;
    i2_ = naga_bitcast_half4_to_int2(_e17);
    half4 _e19 = h4_;
    u2_ = naga_bitcast_half4_to_uint2(_e19);
    half4 _e21 = h4_;
    f2_ = naga_bitcast_half4_to_float2(_e21);
    int _e23 = i1_;
    h2_ = naga_bitcast_int_to_half2(_e23);
    uint _e25 = u1_;
    h2_ = naga_bitcast_uint_to_half2(_e25);
    float _e27 = f1_;
    h2_ = naga_bitcast_float_to_half2(_e27);
    int2 _e29 = i2_;
    h4_ = naga_bitcast_int2_to_half4(_e29);
    uint2 _e31 = u2_;
    h4_ = naga_bitcast_uint2_to_half4(_e31);
    float2 _e33 = f2_;
    h4_ = naga_bitcast_float2_to_half4(_e33);
    half2 _e35 = h2_;
    h2_ = _e35;
    uint _e37 = u1_;
    u1_ = _e37;
    u1_ = 4294967295u;
    f1_ = asfloat(int(1));
    i1_ = asint(1.0);
    h2_ = naga_bitcast_int_to_half2(int(1));
    h2_ = half2(1.0h, 2.0h);
    h4_ = naga_bitcast_int2_to_half4(int2(int(1), int(2)));
    u2_ = uint2(4294967295u, 1u);
    return;
}
