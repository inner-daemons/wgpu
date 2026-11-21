static const double k = 2.0L;

static double v;

double f(double x)
{
    double z;
    double w;

    z = 0.0L;
    w = -1.0L;
    double phony = v;
    double y = (30.0L + 400.0L);
    z = (y + 5.0L);
    return (((x + y) + k) + 5.0L);
}

[numthreads(1, 1, 1)]
void main()
{
    v = 1.0L;
    const double _e1 = f(6.0L);
    return;
}
