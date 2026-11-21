// language: metal3.0
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;
struct DefaultConstructible {
    template<typename T>
    operator T() && {
        return T {};
    }
};

struct UniformIndex {
    uint index;
};
template <typename T>
struct NagaArgumentBufferWrapper {
    T inner;
};
struct FragmentIn {
    uint index;
};

struct main_Input {
    uint index [[user(loc0), flat]];
};
struct main_Output {
    metal::float4 member [[color(0)]];
};
fragment main_Output main_(
  main_Input varyings [[stage_in]]
, constant NagaArgumentBufferWrapper<metal::texture2d<float, metal::access::sample>>* texture_array_unbounded [[buffer(0)]]
, constant NagaArgumentBufferWrapper<metal::texture2d<float, metal::access::sample>>* texture_array_bounded [[user(fake0)]]
, constant NagaArgumentBufferWrapper<metal::texture2d_array<float, metal::access::sample>>* texture_array_2darray [[user(fake0)]]
, constant NagaArgumentBufferWrapper<metal::texture2d_ms<float, metal::access::read>>* texture_array_multisampled [[user(fake0)]]
, constant NagaArgumentBufferWrapper<metal::depth2d<float, metal::access::sample>>* texture_array_depth [[user(fake0)]]
, constant NagaArgumentBufferWrapper<metal::texture2d<float, metal::access::write>>* texture_array_storage [[user(fake0)]]
, constant NagaArgumentBufferWrapper<metal::sampler>* samp [[user(fake0)]]
, constant NagaArgumentBufferWrapper<metal::sampler>* samp_comp [[user(fake0)]]
, constant UniformIndex& uni [[user(fake0)]]
) {
    const FragmentIn fragment_in = { varyings.index };
    uint u1_;
    metal::uint2 u2_;
    float v1_;
    metal::float4 v4_;
    u1_ = 0u;
    u2_ = metal::uint2(0u);
    v1_ = 0.0;
    v4_ = metal::float4(0.0);
    uint uniform_index = uni.index;
    uint non_uniform_index = fragment_in.index;
    metal::float2 uv = metal::float2(0.0);
    metal::int2 pix = metal::int2(0);
    metal::uint2 _e15 = u2_;
    u2_ = _e15 + metal::uint2(texture_array_unbounded[0].inner.get_width(), texture_array_unbounded[0].inner.get_height());
    metal::uint2 _e20 = u2_;
    u2_ = _e20 + metal::uint2(texture_array_unbounded[uniform_index].inner.get_width(), texture_array_unbounded[uniform_index].inner.get_height());
    metal::uint2 _e25 = u2_;
    u2_ = _e25 + metal::uint2(texture_array_unbounded[non_uniform_index].inner.get_width(), texture_array_unbounded[non_uniform_index].inner.get_height());
    metal::float4 _e30 = v4_;
    metal::float4 _e35 = texture_array_bounded[0].inner.gather(samp[0].inner, uv);
    v4_ = _e30 + _e35;
    metal::float4 _e37 = v4_;
    metal::float4 _e42 = texture_array_bounded[uniform_index].inner.gather(samp[uniform_index].inner, uv);
    v4_ = _e37 + _e42;
    metal::float4 _e44 = v4_;
    metal::float4 _e49 = texture_array_bounded[non_uniform_index].inner.gather(samp[non_uniform_index].inner, uv);
    v4_ = _e44 + _e49;
    metal::float4 _e51 = v4_;
    metal::float4 _e57 = texture_array_depth[0].inner.gather_compare(samp_comp[0].inner, uv, 0.0);
    v4_ = _e51 + _e57;
    metal::float4 _e59 = v4_;
    metal::float4 _e65 = texture_array_depth[uniform_index].inner.gather_compare(samp_comp[uniform_index].inner, uv, 0.0);
    v4_ = _e59 + _e65;
    metal::float4 _e67 = v4_;
    metal::float4 _e73 = texture_array_depth[non_uniform_index].inner.gather_compare(samp_comp[non_uniform_index].inner, uv, 0.0);
    v4_ = _e67 + _e73;
    metal::float4 _e75 = v4_;
    metal::float4 _e79 = (uint(0) < texture_array_unbounded[0].inner.get_num_mip_levels() && metal::all(metal::uint2(pix) < metal::uint2(texture_array_unbounded[0].inner.get_width(0), texture_array_unbounded[0].inner.get_height(0))) ? texture_array_unbounded[0].inner.read(metal::uint2(pix), 0): DefaultConstructible());
    v4_ = _e75 + _e79;
    metal::float4 _e81 = v4_;
    metal::float4 _e85 = (uint(0) < texture_array_unbounded[uniform_index].inner.get_num_mip_levels() && metal::all(metal::uint2(pix) < metal::uint2(texture_array_unbounded[uniform_index].inner.get_width(0), texture_array_unbounded[uniform_index].inner.get_height(0))) ? texture_array_unbounded[uniform_index].inner.read(metal::uint2(pix), 0): DefaultConstructible());
    v4_ = _e81 + _e85;
    metal::float4 _e87 = v4_;
    metal::float4 _e91 = (uint(0) < texture_array_unbounded[non_uniform_index].inner.get_num_mip_levels() && metal::all(metal::uint2(pix) < metal::uint2(texture_array_unbounded[non_uniform_index].inner.get_width(0), texture_array_unbounded[non_uniform_index].inner.get_height(0))) ? texture_array_unbounded[non_uniform_index].inner.read(metal::uint2(pix), 0): DefaultConstructible());
    v4_ = _e87 + _e91;
    uint _e93 = u1_;
    u1_ = _e93 + texture_array_2darray[0].inner.get_array_size();
    uint _e98 = u1_;
    u1_ = _e98 + texture_array_2darray[uniform_index].inner.get_array_size();
    uint _e103 = u1_;
    u1_ = _e103 + texture_array_2darray[non_uniform_index].inner.get_array_size();
    uint _e108 = u1_;
    u1_ = _e108 + texture_array_bounded[0].inner.get_num_mip_levels();
    uint _e113 = u1_;
    u1_ = _e113 + texture_array_bounded[uniform_index].inner.get_num_mip_levels();
    uint _e118 = u1_;
    u1_ = _e118 + texture_array_bounded[non_uniform_index].inner.get_num_mip_levels();
    uint _e123 = u1_;
    u1_ = _e123 + texture_array_multisampled[0].inner.get_num_samples();
    uint _e128 = u1_;
    u1_ = _e128 + texture_array_multisampled[uniform_index].inner.get_num_samples();
    uint _e133 = u1_;
    u1_ = _e133 + texture_array_multisampled[non_uniform_index].inner.get_num_samples();
    metal::float4 _e138 = v4_;
    metal::float4 _e143 = texture_array_bounded[0].inner.sample(samp[0].inner, uv);
    v4_ = _e138 + _e143;
    metal::float4 _e145 = v4_;
    metal::float4 _e150 = texture_array_bounded[uniform_index].inner.sample(samp[uniform_index].inner, uv);
    v4_ = _e145 + _e150;
    metal::float4 _e152 = v4_;
    metal::float4 _e157 = texture_array_bounded[non_uniform_index].inner.sample(samp[non_uniform_index].inner, uv);
    v4_ = _e152 + _e157;
    metal::float4 _e159 = v4_;
    metal::float4 _e165 = texture_array_bounded[0].inner.sample(samp[0].inner, uv, metal::bias(0.0));
    v4_ = _e159 + _e165;
    metal::float4 _e167 = v4_;
    metal::float4 _e173 = texture_array_bounded[uniform_index].inner.sample(samp[uniform_index].inner, uv, metal::bias(0.0));
    v4_ = _e167 + _e173;
    metal::float4 _e175 = v4_;
    metal::float4 _e181 = texture_array_bounded[non_uniform_index].inner.sample(samp[non_uniform_index].inner, uv, metal::bias(0.0));
    v4_ = _e175 + _e181;
    float _e183 = v1_;
    float _e189 = texture_array_depth[0].inner.sample_compare(samp_comp[0].inner, uv, 0.0);
    v1_ = _e183 + _e189;
    float _e191 = v1_;
    float _e197 = texture_array_depth[uniform_index].inner.sample_compare(samp_comp[uniform_index].inner, uv, 0.0);
    v1_ = _e191 + _e197;
    float _e199 = v1_;
    float _e205 = texture_array_depth[non_uniform_index].inner.sample_compare(samp_comp[non_uniform_index].inner, uv, 0.0);
    v1_ = _e199 + _e205;
    float _e207 = v1_;
    float _e213 = texture_array_depth[0].inner.sample_compare(samp_comp[0].inner, uv, 0.0);
    v1_ = _e207 + _e213;
    float _e215 = v1_;
    float _e221 = texture_array_depth[uniform_index].inner.sample_compare(samp_comp[uniform_index].inner, uv, 0.0);
    v1_ = _e215 + _e221;
    float _e223 = v1_;
    float _e229 = texture_array_depth[non_uniform_index].inner.sample_compare(samp_comp[non_uniform_index].inner, uv, 0.0);
    v1_ = _e223 + _e229;
    metal::float4 _e231 = v4_;
    metal::float4 _e236 = texture_array_bounded[0].inner.sample(samp[0].inner, uv, metal::gradient2d(uv, uv));
    v4_ = _e231 + _e236;
    metal::float4 _e238 = v4_;
    metal::float4 _e243 = texture_array_bounded[uniform_index].inner.sample(samp[uniform_index].inner, uv, metal::gradient2d(uv, uv));
    v4_ = _e238 + _e243;
    metal::float4 _e245 = v4_;
    metal::float4 _e250 = texture_array_bounded[non_uniform_index].inner.sample(samp[non_uniform_index].inner, uv, metal::gradient2d(uv, uv));
    v4_ = _e245 + _e250;
    metal::float4 _e252 = v4_;
    metal::float4 _e258 = texture_array_bounded[0].inner.sample(samp[0].inner, uv, metal::level(0.0));
    v4_ = _e252 + _e258;
    metal::float4 _e260 = v4_;
    metal::float4 _e266 = texture_array_bounded[uniform_index].inner.sample(samp[uniform_index].inner, uv, metal::level(0.0));
    v4_ = _e260 + _e266;
    metal::float4 _e268 = v4_;
    metal::float4 _e274 = texture_array_bounded[non_uniform_index].inner.sample(samp[non_uniform_index].inner, uv, metal::level(0.0));
    v4_ = _e268 + _e274;
    metal::float4 _e278 = v4_;
    texture_array_storage[0].inner.write(_e278, metal::uint2(pix));
    metal::float4 _e281 = v4_;
    texture_array_storage[uniform_index].inner.write(_e281, metal::uint2(pix));
    metal::float4 _e284 = v4_;
    texture_array_storage[non_uniform_index].inner.write(_e284, metal::uint2(pix));
    metal::uint2 _e285 = u2_;
    uint _e286 = u1_;
    metal::float2 v2_ = static_cast<metal::float2>(_e285 + metal::uint2(_e286));
    metal::float4 _e290 = v4_;
    float _e297 = v1_;
    return main_Output { (_e290 + metal::float4(v2_.x, v2_.y, v2_.x, v2_.y)) + metal::float4(_e297) };
}
