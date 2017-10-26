#include "mps.h"
#include "mpsavm.h"
#include "mpscamc.h"

mps_res_t rust_mps_create_vm_area(mps_arena_t *arena_o,
                                  size_t arenasize)
{
    mps_res_t res;

    MPS_ARGS_BEGIN(args) {
        MPS_ARGS_ADD(args, MPS_KEY_ARENA_SIZE, arenasize);
        res = mps_arena_create_k(arena_o, mps_arena_class_vm(),  args);
    } MPS_ARGS_END(args);

    return res;
}
