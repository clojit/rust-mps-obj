#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <assert.h>

#include "mps.h"
#include "mpsavm.h"
#include "mpscamc.h"

#define ALIGNMENT sizeof(uint64_t)

#define HEADER_SIZE sizeof(uint64_t)

#define ALIGN_WORD(size) \
  (((size) + ALIGNMENT - 1) & ~(ALIGNMENT - 1))

uint8_t OBJ_MPS_TYPE_PADDING = 0x00;
uint8_t OBJ_MPS_TYPE_FORWARD = 0x01;
uint8_t OBJ_MPS_TYPE_OBJECT  = 0x02;
uint8_t OBJ_MPS_TYPE_ARRAY   = 0x03;

struct obj_stub {
    uint8_t type;
    uint8_t _;
    uint16_t cljtype;
    uint32_t size; // incl. header
    mps_addr_t ref[];
} __attribute__((packed));

static mps_res_t obj_scan(mps_ss_t ss, mps_addr_t base, mps_addr_t limit) {
  MPS_SCAN_BEGIN(ss) {
    while (base < limit) {
        struct obj_stub *obj = base;
        // FIXME: we currently only scan objects, arrays are not supported yet
        if (obj->type == OBJ_MPS_TYPE_OBJECT) {
            mps_addr_t ref_base =  obj->ref;
            mps_addr_t ref_limit = (uint8_t*)ref_base + obj->size;
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
    struct obj_stub *obj = base;
    base = (uint8_t*)base + obj->size;
    return base;
}

static mps_addr_t obj_isfwd(mps_addr_t addr)
{
    struct obj_stub *obj = addr;
    if (obj->type == OBJ_MPS_TYPE_FORWARD) {
        return obj->ref[0];
    }

    return NULL;
}

static void obj_fwd(mps_addr_t old,
                    mps_addr_t new)
{
    struct obj_stub *obj = old;
    mps_addr_t limit = obj_skip(old);
    uint32_t size = (uint32_t)((uint8_t*)limit - (uint8_t*)old);

    obj->type = OBJ_MPS_TYPE_FORWARD;
    obj->size = size;
    obj->ref[0] = new;
}

static void obj_pad(mps_addr_t addr, size_t size)
{
    struct obj_stub *obj = addr;
    obj->type = OBJ_MPS_TYPE_PADDING;
    obj->size = size;
}

mps_res_t rust_mps_create_vm_area(mps_arena_t *arena_o,
                                  //mps_thr_t *thr_o,
                                  size_t arenasize)
{
    mps_res_t res;

    MPS_ARGS_BEGIN(args) {
        MPS_ARGS_ADD(args, MPS_KEY_ARENA_SIZE, arenasize);
        res = mps_arena_create_k(arena_o, mps_arena_class_vm(),  args);
    } MPS_ARGS_END(args);

    return res;
}

// caller needs to make sure to root addr_o before calling this!
// size is the size in bytes (excluding alignment)
mps_res_t rust_mps_alloc_obj(mps_addr_t *addr_o,
                             mps_ap_t ap,
                             uint32_t size,
                             uint16_t cljtype,
                             uint8_t mpstype)
{
    assert(addr_o != NULL);
    mps_res_t res;

    do {
        res = mps_reserve(addr_o, ap, size);
        if (res != MPS_RES_OK) return res;
        struct obj_stub *obj = *addr_o;

        obj->type = mpstype;
        obj->cljtype = cljtype;
        obj->size = size;

        // zero all fields
        memset(obj->ref, 0, size - HEADER_SIZE);
    } while (!mps_commit(ap, *addr_o, size));

    return res;
}

mps_res_t rust_mps_create_amc_pool(mps_pool_t *pool_o, mps_fmt_t *fmt_o, mps_arena_t arena)
{
    mps_res_t res;
    MPS_ARGS_BEGIN(args) {
        MPS_ARGS_ADD(args, MPS_KEY_FMT_ALIGN, ALIGNMENT);
        MPS_ARGS_ADD(args, MPS_KEY_FMT_SCAN, obj_scan);
        MPS_ARGS_ADD(args, MPS_KEY_FMT_SKIP, obj_skip);
        MPS_ARGS_ADD(args, MPS_KEY_FMT_FWD, obj_fwd);
        MPS_ARGS_ADD(args, MPS_KEY_FMT_ISFWD, obj_isfwd);
        MPS_ARGS_ADD(args, MPS_KEY_FMT_PAD, obj_pad);
        res = mps_fmt_create_k(fmt_o, arena, args);
    } MPS_ARGS_END(args);

    if (res != MPS_RES_OK) return res;

    MPS_ARGS_BEGIN(args) {
        MPS_ARGS_ADD(args, MPS_KEY_FORMAT, *fmt_o);
        res = mps_pool_create_k(pool_o, arena, mps_class_amc(), args);
    } MPS_ARGS_END(args);

    return res;
}

mps_res_t rust_mps_root_create_table(mps_root_t *root_o,
                                     mps_arena_t arena,
                                     mps_addr_t  *base,
                                     size_t count) {

  return mps_root_create_table_masked(root_o, arena,
                                     mps_rank_exact(),
                                     (mps_rm_t)0,
                                     base,
                                     count,
                                     (mps_word_t)0xFFFF000000000000);

}

mps_res_t rust_mps_create_ap(mps_ap_t *ap_o, mps_pool_t pool) {
    return mps_ap_create_k(ap_o, pool, mps_args_none);
}
