#version 460
#extension GL_KHR_cooperative_matrix : require
#extension GL_KHR_memory_scope_semantics : require
layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

layout(set = 0, binding = 0, std430) buffer _19_18
{
    float _m0[];
} _18;

coopmat<float, gl_ScopeSubgroup, 8u, 8u, gl_MatrixUseA> _14;
coopmat<float, gl_ScopeSubgroup, 8u, 8u, gl_MatrixUseB> _16;

void main()
{
    coopmat<float, gl_ScopeSubgroup, 8u, 8u, gl_MatrixUseAccumulator> _29 = coopmat<float, gl_ScopeSubgroup, 8u, 8u, gl_MatrixUseAccumulator>(0.0);
    coopmat<float, gl_ScopeSubgroup, 8u, 8u, gl_MatrixUseAccumulator> _31 = coopmat<float, gl_ScopeSubgroup, 8u, 8u, gl_MatrixUseAccumulator>(0.0);
    _14 = coopmat<float, gl_ScopeSubgroup, 8u, 8u, gl_MatrixUseA>(0.0);
    _16 = coopmat<float, gl_ScopeSubgroup, 8u, 8u, gl_MatrixUseB>(0.0);
    coopmat<float, gl_ScopeSubgroup, 8u, 8u, gl_MatrixUseAccumulator> _36;
    coopMatLoad(_36, _18._m0, 4u, 8u, gl_CooperativeMatrixLayoutColumnMajor);
    _29 = _36;
    _31 = coopMatMulAdd(_14, _16, _29, 0);
    coopMatStore(_31, _18._m0, 0u, 8u, gl_CooperativeMatrixLayoutColumnMajor);
    _29 = _31;
}

