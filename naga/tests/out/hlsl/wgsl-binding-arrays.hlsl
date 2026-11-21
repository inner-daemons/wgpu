struct UniformIndex {
    uint index;
};

struct FragmentIn {
    nointerpolation uint index : LOC0;
};

Texture2D<float4> texture_array_unbounded[10] : register(t0);
Texture2D<float4> texture_array_bounded[5] : register(t0, space1);
Texture2DArray<float4> texture_array_2darray[5] : register(t0, space2);
Texture2DMS<float4> texture_array_multisampled[5] : register(t0, space3);
Texture2D<float> texture_array_depth[5] : register(t0, space4);
RWTexture2D<float4> texture_array_storage[5] : register(u0, space5);
SamplerState nagaSamplerHeap[2048]: register(s0, space0);
SamplerComparisonState nagaComparisonSamplerHeap[2048]: register(s0, space1);
StructuredBuffer<uint> nagaGroup0SamplerIndexArray : register(t0, space255);
static const uint samp = 0;
static const uint samp_comp = 0;
cbuffer uni : register(b0, space8) { UniformIndex uni; }

struct FragmentInput_main {
    nointerpolation uint index : LOC0;
};

uint2 NagaDimensions2D(Texture2D<float4> tex)
{
    uint4 ret;
    tex.GetDimensions(0, ret.x, ret.y, ret.z);
    return ret.xy;
}

uint NagaNumLayers2DArray(Texture2DArray<float4> tex)
{
    uint4 ret;
    tex.GetDimensions(0, ret.x, ret.y, ret.z, ret.w);
    return ret.z;
}

uint NagaNumLevels2D(Texture2D<float4> tex)
{
    uint4 ret;
    tex.GetDimensions(0, ret.x, ret.y, ret.z);
    return ret.z;
}

uint NagaMSNumSamples2D(Texture2DMS<float4> tex)
{
    uint4 ret;
    tex.GetDimensions(ret.x, ret.y, ret.z);
    return ret.z;
}

float4 main(FragmentInput_main fragmentinput_main) : SV_Target0
{
    FragmentIn fragment_in = { fragmentinput_main.index };
    uint u1_;
    uint2 u2_;
    float v1_;
    float4 v4_;

    u1_ = 0u;
    u2_ = (0u).xx;
    v1_ = 0.0;
    v4_ = (0.0).xxxx;
    uint uniform_index = uni.index;
    uint non_uniform_index = fragment_in.index;
    float2 uv = (0.0).xx;
    int2 pix = (int(0)).xx;
    uint2 _e15 = u2_;
    u2_ = (_e15 + NagaDimensions2D(texture_array_unbounded[0]));
    uint2 _e20 = u2_;
    u2_ = (_e20 + NagaDimensions2D(texture_array_unbounded[uniform_index]));
    uint2 _e25 = u2_;
    u2_ = (_e25 + NagaDimensions2D(texture_array_unbounded[NonUniformResourceIndex(non_uniform_index)]));
    float4 _e30 = v4_;
    float4 _e35 = texture_array_bounded[0].Gather(nagaSamplerHeap[nagaGroup0SamplerIndexArray[samp + 0]], uv);
    v4_ = (_e30 + _e35);
    float4 _e37 = v4_;
    float4 _e42 = texture_array_bounded[uniform_index].Gather(nagaSamplerHeap[nagaGroup0SamplerIndexArray[samp + uniform_index]], uv);
    v4_ = (_e37 + _e42);
    float4 _e44 = v4_;
    float4 _e49 = texture_array_bounded[NonUniformResourceIndex(non_uniform_index)].Gather(nagaSamplerHeap[NonUniformResourceIndex(nagaGroup0SamplerIndexArray[samp + non_uniform_index])], uv);
    v4_ = (_e44 + _e49);
    float4 _e51 = v4_;
    float4 _e57 = texture_array_depth[0].GatherCmp(nagaComparisonSamplerHeap[nagaGroup0SamplerIndexArray[samp_comp + 0]], uv, 0.0);
    v4_ = (_e51 + _e57);
    float4 _e59 = v4_;
    float4 _e65 = texture_array_depth[uniform_index].GatherCmp(nagaComparisonSamplerHeap[nagaGroup0SamplerIndexArray[samp_comp + uniform_index]], uv, 0.0);
    v4_ = (_e59 + _e65);
    float4 _e67 = v4_;
    float4 _e73 = texture_array_depth[NonUniformResourceIndex(non_uniform_index)].GatherCmp(nagaComparisonSamplerHeap[NonUniformResourceIndex(nagaGroup0SamplerIndexArray[samp_comp + non_uniform_index])], uv, 0.0);
    v4_ = (_e67 + _e73);
    float4 _e75 = v4_;
    float4 _e79 = texture_array_unbounded[0].Load(int3(pix, int(0)));
    v4_ = (_e75 + _e79);
    float4 _e81 = v4_;
    float4 _e85 = texture_array_unbounded[uniform_index].Load(int3(pix, int(0)));
    v4_ = (_e81 + _e85);
    float4 _e87 = v4_;
    float4 _e91 = texture_array_unbounded[NonUniformResourceIndex(non_uniform_index)].Load(int3(pix, int(0)));
    v4_ = (_e87 + _e91);
    uint _e93 = u1_;
    u1_ = (_e93 + NagaNumLayers2DArray(texture_array_2darray[0]));
    uint _e98 = u1_;
    u1_ = (_e98 + NagaNumLayers2DArray(texture_array_2darray[uniform_index]));
    uint _e103 = u1_;
    u1_ = (_e103 + NagaNumLayers2DArray(texture_array_2darray[NonUniformResourceIndex(non_uniform_index)]));
    uint _e108 = u1_;
    u1_ = (_e108 + NagaNumLevels2D(texture_array_bounded[0]));
    uint _e113 = u1_;
    u1_ = (_e113 + NagaNumLevels2D(texture_array_bounded[uniform_index]));
    uint _e118 = u1_;
    u1_ = (_e118 + NagaNumLevels2D(texture_array_bounded[NonUniformResourceIndex(non_uniform_index)]));
    uint _e123 = u1_;
    u1_ = (_e123 + NagaMSNumSamples2D(texture_array_multisampled[0]));
    uint _e128 = u1_;
    u1_ = (_e128 + NagaMSNumSamples2D(texture_array_multisampled[uniform_index]));
    uint _e133 = u1_;
    u1_ = (_e133 + NagaMSNumSamples2D(texture_array_multisampled[NonUniformResourceIndex(non_uniform_index)]));
    float4 _e138 = v4_;
    float4 _e143 = texture_array_bounded[0].Sample(nagaSamplerHeap[nagaGroup0SamplerIndexArray[samp + 0]], uv);
    v4_ = (_e138 + _e143);
    float4 _e145 = v4_;
    float4 _e150 = texture_array_bounded[uniform_index].Sample(nagaSamplerHeap[nagaGroup0SamplerIndexArray[samp + uniform_index]], uv);
    v4_ = (_e145 + _e150);
    float4 _e152 = v4_;
    float4 _e157 = texture_array_bounded[NonUniformResourceIndex(non_uniform_index)].Sample(nagaSamplerHeap[NonUniformResourceIndex(nagaGroup0SamplerIndexArray[samp + non_uniform_index])], uv);
    v4_ = (_e152 + _e157);
    float4 _e159 = v4_;
    float4 _e165 = texture_array_bounded[0].SampleBias(nagaSamplerHeap[nagaGroup0SamplerIndexArray[samp + 0]], uv, 0.0);
    v4_ = (_e159 + _e165);
    float4 _e167 = v4_;
    float4 _e173 = texture_array_bounded[uniform_index].SampleBias(nagaSamplerHeap[nagaGroup0SamplerIndexArray[samp + uniform_index]], uv, 0.0);
    v4_ = (_e167 + _e173);
    float4 _e175 = v4_;
    float4 _e181 = texture_array_bounded[NonUniformResourceIndex(non_uniform_index)].SampleBias(nagaSamplerHeap[NonUniformResourceIndex(nagaGroup0SamplerIndexArray[samp + non_uniform_index])], uv, 0.0);
    v4_ = (_e175 + _e181);
    float _e183 = v1_;
    float _e189 = texture_array_depth[0].SampleCmp(nagaComparisonSamplerHeap[nagaGroup0SamplerIndexArray[samp_comp + 0]], uv, 0.0);
    v1_ = (_e183 + _e189);
    float _e191 = v1_;
    float _e197 = texture_array_depth[uniform_index].SampleCmp(nagaComparisonSamplerHeap[nagaGroup0SamplerIndexArray[samp_comp + uniform_index]], uv, 0.0);
    v1_ = (_e191 + _e197);
    float _e199 = v1_;
    float _e205 = texture_array_depth[NonUniformResourceIndex(non_uniform_index)].SampleCmp(nagaComparisonSamplerHeap[NonUniformResourceIndex(nagaGroup0SamplerIndexArray[samp_comp + non_uniform_index])], uv, 0.0);
    v1_ = (_e199 + _e205);
    float _e207 = v1_;
    float _e213 = texture_array_depth[0].SampleCmpLevelZero(nagaComparisonSamplerHeap[nagaGroup0SamplerIndexArray[samp_comp + 0]], uv, 0.0);
    v1_ = (_e207 + _e213);
    float _e215 = v1_;
    float _e221 = texture_array_depth[uniform_index].SampleCmpLevelZero(nagaComparisonSamplerHeap[nagaGroup0SamplerIndexArray[samp_comp + uniform_index]], uv, 0.0);
    v1_ = (_e215 + _e221);
    float _e223 = v1_;
    float _e229 = texture_array_depth[NonUniformResourceIndex(non_uniform_index)].SampleCmpLevelZero(nagaComparisonSamplerHeap[NonUniformResourceIndex(nagaGroup0SamplerIndexArray[samp_comp + non_uniform_index])], uv, 0.0);
    v1_ = (_e223 + _e229);
    float4 _e231 = v4_;
    float4 _e236 = texture_array_bounded[0].SampleGrad(nagaSamplerHeap[nagaGroup0SamplerIndexArray[samp + 0]], uv, uv, uv);
    v4_ = (_e231 + _e236);
    float4 _e238 = v4_;
    float4 _e243 = texture_array_bounded[uniform_index].SampleGrad(nagaSamplerHeap[nagaGroup0SamplerIndexArray[samp + uniform_index]], uv, uv, uv);
    v4_ = (_e238 + _e243);
    float4 _e245 = v4_;
    float4 _e250 = texture_array_bounded[NonUniformResourceIndex(non_uniform_index)].SampleGrad(nagaSamplerHeap[NonUniformResourceIndex(nagaGroup0SamplerIndexArray[samp + non_uniform_index])], uv, uv, uv);
    v4_ = (_e245 + _e250);
    float4 _e252 = v4_;
    float4 _e258 = texture_array_bounded[0].SampleLevel(nagaSamplerHeap[nagaGroup0SamplerIndexArray[samp + 0]], uv, 0.0);
    v4_ = (_e252 + _e258);
    float4 _e260 = v4_;
    float4 _e266 = texture_array_bounded[uniform_index].SampleLevel(nagaSamplerHeap[nagaGroup0SamplerIndexArray[samp + uniform_index]], uv, 0.0);
    v4_ = (_e260 + _e266);
    float4 _e268 = v4_;
    float4 _e274 = texture_array_bounded[NonUniformResourceIndex(non_uniform_index)].SampleLevel(nagaSamplerHeap[NonUniformResourceIndex(nagaGroup0SamplerIndexArray[samp + non_uniform_index])], uv, 0.0);
    v4_ = (_e268 + _e274);
    float4 _e278 = v4_;
    texture_array_storage[0][pix] = _e278;
    float4 _e281 = v4_;
    texture_array_storage[uniform_index][pix] = _e281;
    float4 _e284 = v4_;
    texture_array_storage[NonUniformResourceIndex(non_uniform_index)][pix] = _e284;
    uint2 _e285 = u2_;
    uint _e286 = u1_;
    float2 v2_ = float2((_e285 + (_e286).xx));
    float4 _e290 = v4_;
    float _e297 = v1_;
    return ((_e290 + float4(v2_.x, v2_.y, v2_.x, v2_.y)) + (_e297).xxxx);
}
