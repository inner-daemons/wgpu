float2 ZeroValuefloat2() {
    return (float2)0;
}

[numthreads(1, 1, 1)]
void main()
{
    int2 x0_;
    float2 i1_;

    x0_ = int2(int(1), int(2));
    i1_ = ZeroValuefloat2();
    int _e9 = x0_.x;
    int _e11 = x0_.y;
    i1_ = ((_e9 < _e11) ? float2(0.0, 1.0) : float2(1.0, 0.0));
    return;
}
