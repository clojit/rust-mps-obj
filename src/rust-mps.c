#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <assert.h>

#include "mps.h"
#include "mpsavm.h"
#include "mpscamc.h"

#define ALIGNMENT sizeof(uint64_t)

uint16_t OBJ_FMT_TYPE_PAYLOAD = 0x0001;
uint16_t OBJ_FMT_TYPE_FORWARD = 0x0002;
uint16_t OBJ_FMT_TYPE_PADDING = 0x0004;

struct obj_header {
    uint16_t type;
    uint16_t _;
    uint32_t size;
} __attribute__((packed));

struct obj_forward {
    uint16_t type;
    uint16_t _;
    uint32_t size;
    mps_addr_t fwd;
} __attribute__((packed));

struct obj_stub {
    uint16_t type;
    uint16_t offset;
    uint32_t size;
    uint64_t info_type;
} __attribute__((packed));

static mps_res_t obj_scan(mps_ss_t ss, mps_addr_t base, mps_addr_t limit) {
  MPS_SCAN_BEGIN(ss) {
    while (base < limit) {
        struct obj_stub *obj = base;
        if (obj->type == OBJ_FMT_TYPE_PAYLOAD) {
            mps_addr_t ref_base =  (uint8_t*)base + obj->offset;
            mps_addr_t ref_limit = (uint8_t*)base + obj->size;
            for (mps_addr_t ref = ref_base; ref < ref_limit; ref++) {
                mps_res_t res = MPS_FIX12(ss, &ref);
                if (res != MPS_RES_OK) return res;
            }
        }
        base = (uint8_t*)base + obj->size;
    }
  } MPS_SCAN_END(ss);
  return MPS_RES_OK;
}

static mps_addr_t obj_skip(mps_addr_t base)
{
    struct obj_header *obj = base;
    base = (uint8_t*)base + obj->size;
    return base;
}

static mps_addr_t obj_isfwd(mps_addr_t addr)
{
    struct obj_forward *obj = addr;
    if (obj->type == OBJ_FMT_TYPE_FORWARD) {
        return obj->fwd;
    }

    return NULL;
}

static void obj_fwd(mps_addr_t old, mps_addr_t new)
{
    struct obj_forward *obj = old;
    mps_addr_t limit = obj_skip(old);
    uint32_t size = (uint32_t)((char *)limit - (char *)old);

    obj->type = OBJ_FMT_TYPE_FORWARD;
    obj->size = size;
    obj->fwd = new;
}

static void obj_pad(mps_addr_t addr, size_t size)
{
    struct obj_header *obj = addr;
    obj->type = OBJ_FMT_TYPE_PADDING;
    obj->size = size;
}

mps_res_t rust_mps_create_vm_area(mps_arena_t *arena_o, mps_thr_t *thr_o,
    size_t arenasize)
{
    mps_res_t res;

    MPS_ARGS_BEGIN(args) {
        MPS_ARGS_ADD(args, MPS_KEY_ARENA_SIZE, arenasize);
        res = mps_arena_create_k(arena_o, mps_arena_class_vm(),  args);
    } MPS_ARGS_END(args);

    if (res != MPS_RES_OK) return res;

    res = mps_thread_reg(thr_o, *arena_o);

    return res;
}

mps_res_t rust_mps_alloc_obj(mps_addr_t *addr_o, mps_ap_t ap, struct obj_stub *obj)
{
    assert(addr_o != NULL && *addr_o == NULL);
    // TODO: caller needs to make sure to root addr_o before calling this!
    return MPS_RES_FAIL;
}

mps_res_t rust_mps_create_obj_pool(mps_pool_t *pool_o, mps_ap_t *ap_o, mps_arena_t arena)
{
    mps_res_t res;
    mps_fmt_t obj_fmt;

    MPS_ARGS_BEGIN(args) {
        MPS_ARGS_ADD(args, MPS_KEY_FMT_ALIGN, ALIGNMENT);
        MPS_ARGS_ADD(args, MPS_KEY_FMT_SCAN, obj_scan);
        MPS_ARGS_ADD(args, MPS_KEY_FMT_SKIP, obj_skip);
        MPS_ARGS_ADD(args, MPS_KEY_FMT_FWD, obj_fwd);
        MPS_ARGS_ADD(args, MPS_KEY_FMT_ISFWD, obj_isfwd);
        MPS_ARGS_ADD(args, MPS_KEY_FMT_PAD, obj_pad);
        res = mps_fmt_create_k(&obj_fmt, arena, args);
    } MPS_ARGS_END(args);

    if (res != MPS_RES_OK) return res;

    MPS_ARGS_BEGIN(args) {
        MPS_ARGS_ADD(args, MPS_KEY_FORMAT, obj_fmt);
        res = mps_pool_create_k(pool_o, arena, mps_class_amc(), args);
    } MPS_ARGS_END(args);

    if (res != MPS_RES_OK) return res;

    res = mps_ap_create_k(ap_o, *pool_o, mps_args_none);

    return res;
}
