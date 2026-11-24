void control_flow()
{
    int pos = (int)0;

    DeviceMemoryBarrierWithGroupSync();
    GroupMemoryBarrierWithGroupSync();
    DeviceMemoryBarrierWithGroupSync();
    do {
        pos = int(1);
    } while(false);
    int _e3 = pos;
    switch(_e3) {
        case 1: {
            pos = int(0);
            break;
        }
        case 2: {
            pos = int(1);
            break;
        }
        case 3:
        case 4: {
            pos = int(2);
            break;
        }
        case 5: {
            pos = int(3);
            break;
        }
        default:
        case 6: {
            pos = int(4);
            break;
        }
    }
    switch(0u) {
        case 0u: {
            break;
        }
        default: {
            break;
        }
    }
    int _e10 = pos;
    switch(_e10) {
        case 1: {
            pos = int(0);
            break;
        }
        case 2: {
            pos = int(1);
            break;
        }
        case 3: {
            pos = int(2);
            break;
        }
        case 4: {
            break;
        }
        default: {
            pos = int(3);
            break;
        }
    }
    int _e15 = pos;
    switch(_e15) {
        case 1: {
            pos = int(0);
            return;
        }
        case 2: {
            pos = int(1);
            return;
        }
        case 3:
        case 4: {
            pos = int(2);
            return;
        }
        case 5:
        case 6: {
            pos = int(3);
            return;
        }
        default: {
            pos = int(4);
            return;
        }
    }
}

void switch_default_break(int i)
{
    do {
        break;
    } while(false);
}

void switch_case_break()
{
    switch(int(0)) {
        case 0: {
            break;
        }
        default: {
            break;
        }
    }
    return;
}

void switch_selector_type_conversion()
{
    switch(0u) {
        case 0u: {
            break;
        }
        default: {
            break;
        }
    }
    switch(0u) {
        case 0u: {
            return;
        }
        default: {
            return;
        }
    }
}

void switch_const_expr_case_selectors()
{
    switch(int(0)) {
        case 0: {
            return;
        }
        case 1: {
            return;
        }
        case 2: {
            return;
        }
        case 3: {
            return;
        }
        case 4: {
            return;
        }
        default: {
            return;
        }
    }
}

void loop_switch_continue(int x)
{
    uint2 loop_bound = (4294967295u).xx;

    while(true) {
        {
            uint2 _e6 = loop_bound;
            loop_bound = (_e6 - uint2(((_e6.y == 0u) ? 1u : 0u), 1u));
            if (all((_e6 == (0u).xx))) {
                break;
            }
        }
        bool should_continue = false;
        switch(x) {
            case 1: {
                should_continue = true;
                break;
            }
            default: {
                break;
            }
        }
        if (should_continue) {
            continue;
        }
    }
    return;
}

void loop_switch_continue_nesting(int x_1, int y, int z)
{
    uint2 loop_bound_1 = (4294967295u).xx;
    uint2 loop_bound_2 = (4294967295u).xx;
    uint2 loop_bound_3 = (4294967295u).xx;

    while(true) {
        {
            uint2 _e23 = loop_bound_1;
            loop_bound_1 = (_e23 - uint2(((_e23.y == 0u) ? 1u : 0u), 1u));
            if (all((_e23 == (0u).xx))) {
                break;
            }
        }
        bool should_continue_1 = false;
        switch(x_1) {
            case 1: {
                should_continue_1 = true;
                break;
            }
            case 2: {
                switch(y) {
                    case 1: {
                        should_continue_1 = true;
                        break;
                    }
                    default: {
                        loop_bound_2 = (4294967295u).xx;
                        while(true) {
                            {
                                uint2 _e11 = loop_bound_2;
                                loop_bound_2 = (_e11 - uint2(((_e11.y == 0u) ? 1u : 0u), 1u));
                                if (all((_e11 == (0u).xx))) {
                                    break;
                                }
                            }
                            bool should_continue_2 = false;
                            switch(z) {
                                case 1: {
                                    should_continue_2 = true;
                                    break;
                                }
                                default: {
                                    break;
                                }
                            }
                            if (should_continue_2) {
                                continue;
                            }
                        }
                        break;
                    }
                }
                if (should_continue_1) {
                    break;
                }
                break;
            }
            default: {
                break;
            }
        }
        if (should_continue_1) {
            continue;
        }
        bool should_continue_3 = false;
        do {
            should_continue_3 = true;
            break;
        } while(false);
        if (should_continue_3) {
            continue;
        }
    }
    while(true) {
        {
            uint2 _e35 = loop_bound_3;
            loop_bound_3 = (_e35 - uint2(((_e35.y == 0u) ? 1u : 0u), 1u));
            if (all((_e35 == (0u).xx))) {
                break;
            }
        }
        bool should_continue_4 = false;
        do {
            do {
                should_continue_4 = true;
                break;
            } while(false);
            if (should_continue_4) {
                break;
            }
        } while(false);
        if (should_continue_4) {
            continue;
        }
    }
    return;
}

void loop_switch_omit_continue_variable_checks(int x_2, int y_1, int z_1, int w)
{
    int pos_1 = int(0);
    uint2 loop_bound_4 = (4294967295u).xx;
    uint2 loop_bound_5 = (4294967295u).xx;

    while(true) {
        {
            uint2 _e13 = loop_bound_4;
            loop_bound_4 = (_e13 - uint2(((_e13.y == 0u) ? 1u : 0u), 1u));
            if (all((_e13 == (0u).xx))) {
                break;
            }
        }
        bool should_continue_5 = false;
        switch(x_2) {
            case 1: {
                pos_1 = int(1);
                break;
            }
            default: {
                break;
            }
        }
    }
    while(true) {
        {
            uint2 _e25 = loop_bound_5;
            loop_bound_5 = (_e25 - uint2(((_e25.y == 0u) ? 1u : 0u), 1u));
            if (all((_e25 == (0u).xx))) {
                break;
            }
        }
        bool should_continue_6 = false;
        switch(x_2) {
            case 1: {
                break;
            }
            case 2: {
                switch(y_1) {
                    case 1: {
                        should_continue_6 = true;
                        break;
                    }
                    default: {
                        switch(z_1) {
                            case 1: {
                                pos_1 = int(2);
                                break;
                            }
                            default: {
                                break;
                            }
                        }
                        break;
                    }
                }
                if (should_continue_6) {
                    break;
                }
                break;
            }
            default: {
                break;
            }
        }
        if (should_continue_6) {
            continue;
        }
    }
    return;
}

[numthreads(1, 1, 1)]
void main()
{
    control_flow();
    switch_default_break(int(1));
    switch_case_break();
    switch_selector_type_conversion();
    switch_const_expr_case_selectors();
    loop_switch_continue(int(1));
    loop_switch_continue_nesting(int(1), int(2), int(3));
    loop_switch_omit_continue_variable_checks(int(1), int(2), int(3), int(4));
    return;
}
