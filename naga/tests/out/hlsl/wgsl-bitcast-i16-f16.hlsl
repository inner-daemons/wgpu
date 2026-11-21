struct NagaConstants {
    int first_vertex;
    int first_instance;
    uint other;
};
ConstantBuffer<NagaConstants> _NagaConstants: register(b0, space1);

[numthreads(1, 1, 1)]
void main()
{
    half h;
    int16_t i;
    uint16_t u;
    half4 h4_;
    uint16_t4 u4_;

    h = 1.0h;
    i = int16_t(1);
    u = uint16_t(2);
    h4_ = half4(1.0h, 2.0h, 3.0h, 4.0h);
    u4_ = uint16_t4(uint16_t(1), uint16_t(2), uint16_t(3), uint16_t(4));
    half _e8 = h;
    i = asint16(_e8);
    half _e10 = h;
    u = asuint16(_e10);
    int16_t _e12 = i;
    h = asfloat16(_e12);
    uint16_t _e14 = u;
    h = asfloat16(_e14);
    half4 _e16 = h4_;
    u4_ = asuint16(_e16);
    uint16_t4 _e18 = u4_;
    h4_ = asfloat16(_e18);
    return;
}
