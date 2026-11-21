#version 460
layout(local_size_x = 2, local_size_y = 3, local_size_z = 1) in;

const float _103[2] = float[](0.0, 0.0);
const int _152[9] = int[](1, 2, 3, 4, 5, 6, 7, 8, 9);

void _34()
{
    ivec4 _37 = ivec4(4, 3, 2, 1);
}

void _41()
{
    int _42 = 2;
}

void _46()
{
    int _48 = 6;
}

void _51()
{
    int _55 = 30;
    int _56 = 0;
    int _57 = 0;
    int _58 = 70;
    ivec4 _59 = ivec4(0);
    _56 = _55;
    _57 = _56;
    _59 = ivec4(_55, _56, _57, _58);
}

void _69()
{
    ivec4 _72 = ivec4(-4);
}

void _75()
{
    ivec4 _76 = ivec4(-4);
}

uint _80(int _79)
{
    switch (_79)
    {
        case 0:
        {
            return 10u;
        }
        case 1:
        {
            return 20u;
        }
        case 2:
        {
            return 30u;
        }
        default:
        {
            return 0u;
        }
    }
    return 0u;
}

void _94()
{
    vec4 _98 = vec4(2.0, 1.0, 1.0, 1.0);
}

void _102()
{
}

void _108()
{
    ivec3 _112 = ivec3(1);
    ivec3 _114 = ivec3(0, 1, 2);
    ivec3 _115 = ivec3(1, 0, 2);
}

void _118()
{
    bool _119 = false;
    bool _121 = true;
    bool _122 = false;
    bool _123 = true;
    bool _124 = false;
    bool _125 = true;
    bool _126 = false;
    bool _127 = true;
}

void _130()
{
    int _135 = 4;
    uint _136 = 4u;
    int _138 = 12;
    uint _139 = 12u;
    int _140 = 70;
    uint _141 = 70u;
    int _142 = -4;
}

void _146(uint _145)
{
    float _154 = 1.0;
    uint _156 = 1u;
    int _157 = 0;
    int _158 = 0;
    _157 = _152[_145];
    _158 = ivec4(1, 2, 3, 4)[_145];
}

void main()
{
    _34();
    _41();
    _46();
    _51();
    _69();
    _75();
    _94();
    _102();
    _108();
    _118();
    _130();
    _102();
    _146(1u);
}

