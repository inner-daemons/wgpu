#version 460
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

void _5()
{
    int _14 = 0;
    memoryBarrierBuffer();
    barrier();
    barrier();
    groupMemoryBarrier();
    barrier();
    do
    {
        _14 = 1;
        break;
    } while(false);
    switch (_14)
    {
        case 1:
        {
            _14 = 0;
            break;
        }
        case 2:
        {
            _14 = 1;
            break;
        }
        case 3:
        case 4:
        {
            _14 = 2;
            break;
        }
        case 5:
        {
            _14 = 3;
            break;
        }
        default:
        {
            _14 = 4;
            break;
        }
    }
    switch (0u)
    {
        case 0u:
        {
            break;
        }
        default:
        {
            break;
        }
    }
    switch (_14)
    {
        case 1:
        {
            _14 = 0;
            break;
        }
        case 2:
        {
            _14 = 1;
            break;
        }
        case 3:
        {
            _14 = 2;
            break;
        }
        case 4:
        {
            break;
        }
        default:
        {
            _14 = 3;
            break;
        }
    }
    switch (_14)
    {
        case 1:
        {
            _14 = 0;
            return;
        }
        case 2:
        {
            _14 = 1;
            return;
        }
        case 3:
        case 4:
        {
            _14 = 2;
            return;
        }
        case 5:
        case 6:
        {
            _14 = 3;
            return;
        }
        default:
        {
            _14 = 4;
            return;
        }
    }
}

void _50(int _49)
{
    do
    {
        break;
    } while(false);
}

void _56()
{
    switch (0)
    {
        case 0:
        {
            break;
        }
        default:
        {
            break;
        }
    }
}

void _62()
{
    switch (0u)
    {
        case 0u:
        {
            break;
        }
        default:
        {
            break;
        }
    }
    switch (0u)
    {
        case 0u:
        {
            return;
        }
        default:
        {
            return;
        }
    }
}

void _71()
{
    switch (0)
    {
        case 0:
        {
            return;
        }
        case 1:
        {
            return;
        }
        case 2:
        {
            return;
        }
        case 3:
        {
            return;
        }
        case 4:
        {
            return;
        }
        default:
        {
            return;
        }
    }
}

void _82(int _81)
{
    uvec2 _95 = uvec2(4294967295u);
    for (;;)
    {
        if (all(equal(uvec2(0u), _95)))
        {
            break;
        }
        _95 -= uvec2(uint(_95.y == 0u), 1u);
        switch (_81)
        {
            case 1:
            {
                continue;
            }
            default:
            {
                break;
            }
        }
        continue;
    }
}

void _113(int _110, int _111, int _112)
{
    uvec2 _120 = uvec2(4294967295u);
    uvec2 _142 = uvec2(4294967295u);
    uvec2 _162 = uvec2(4294967295u);
    for (;;)
    {
        if (all(equal(uvec2(0u), _120)))
        {
            break;
        }
        _120 -= uvec2(uint(_120.y == 0u), 1u);
        switch (_110)
        {
            case 1:
            {
                continue;
            }
            case 2:
            {
                switch (_111)
                {
                    case 1:
                    {
                        continue;
                    }
                    default:
                    {
                        for (;;)
                        {
                            if (all(equal(uvec2(0u), _142)))
                            {
                                break;
                            }
                            _142 -= uvec2(uint(_142.y == 0u), 1u);
                            switch (_112)
                            {
                                case 1:
                                {
                                    continue;
                                }
                                default:
                                {
                                    break;
                                }
                            }
                            continue;
                        }
                        break;
                    }
                }
                break;
            }
            default:
            {
                break;
            }
        }
        do
        {
            continue;
        } while(false);
        continue;
    }
    for (;;)
    {
        if (all(equal(uvec2(0u), _162)))
        {
            break;
        }
        _162 -= uvec2(uint(_162.y == 0u), 1u);
        switch (_111)
        {
            default:
            {
                do
                {
                    continue;
                } while(false);
                break;
            }
        }
        continue;
    }
}

void _182(int _178, int _179, int _180, int _181)
{
    uvec2 _190 = uvec2(4294967295u);
    uvec2 _208 = uvec2(4294967295u);
    int _184 = 0;
    for (;;)
    {
        if (all(equal(uvec2(0u), _190)))
        {
            break;
        }
        _190 -= uvec2(uint(_190.y == 0u), 1u);
        switch (_178)
        {
            case 1:
            {
                _184 = 1;
                break;
            }
            default:
            {
                break;
            }
        }
        continue;
    }
    for (;;)
    {
        if (all(equal(uvec2(0u), _208)))
        {
            break;
        }
        _208 -= uvec2(uint(_208.y == 0u), 1u);
        switch (_178)
        {
            case 1:
            {
                break;
            }
            case 2:
            {
                switch (_179)
                {
                    case 1:
                    {
                        continue;
                    }
                    default:
                    {
                        switch (_180)
                        {
                            case 1:
                            {
                                _184 = 2;
                                break;
                            }
                            default:
                            {
                                break;
                            }
                        }
                        break;
                    }
                }
                break;
            }
            default:
            {
                break;
            }
        }
        continue;
    }
}

void main()
{
    _5();
    _50(1);
    _56();
    _62();
    _71();
    _82(1);
    _113(1, 2, 3);
    _182(1, 2, 3, 4);
}

