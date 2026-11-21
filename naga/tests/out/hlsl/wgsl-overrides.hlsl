static const bool has_point_light = false;
static const float specular_param = 2.3;
static const float gain = 1.1;
static const float width = 0.0;
static const float depth = 2.3;
static const float height = 4.6;
static const float inferred_f32_ = 2.718;
static const uint auto_conversion = 0u;

static float gain_x_10_;
static float store_override;

[numthreads(1, 1, 1)]
void main()
{
    float t;
    bool x;
    float gain_x_100_;

    t = 23.0;
    x = false;
    gain_x_100_ = 0.0;
    gain_x_10_ = 11.0;
    store_override = 0.0;
    x = true;
    float _e4 = gain_x_10_;
    gain_x_100_ = (_e4 * 10.0);
    store_override = gain;
    return;
}
