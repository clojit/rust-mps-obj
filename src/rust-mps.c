#include <stdio.h>
#include <stdlib.h>
#include <inttypes.h>
#include <assert.h>
#include <string.h>

#include "mps.h"
#include "mpsavm.h"
#include "mpscamc.h"

#define WORD_SIZE   sizeof(uint64_t)
#define ALIGNMENT   WORD_SIZE
#define HEADER_SIZE WORD_SIZE

#define ALIGN_WORD(size) \
  (((size) + ALIGNMENT - 1) & ~(ALIGNMENT - 1))

uint8_t OBJ_MPS_TYPE_PADDING = 0x00;
uint8_t OBJ_MPS_TYPE_FORWARD = 0x01;
uint8_t OBJ_MPS_TYPE_OBJECT  = 0x02;
uint8_t OBJ_MPS_TYPE_ARRAY   = 0x03;

static const char *OBJ_MPS_TYPE_NAMES[] = {
    "Padding", "Forward", "Object", "Array"
};

#define ARRAY_LEN(array)    (sizeof(array) / sizeof(array[0]))

#define VAL_BITS (48u)
#define TAG_MASK (0xFFFFul << VAL_BITS)

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
// size is the size in bytes (including header)
mps_res_t rust_mps_alloc_obj(mps_addr_t *addr_o,
                             mps_ap_t ap,
                             uint32_t size,
                             uint16_t cljtype,
                             uint8_t mpstype)
{
    assert(addr_o != NULL);
    assert(size > HEADER_SIZE);
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
                                     (mps_word_t)TAG_MASK);

}

mps_res_t rust_mps_create_ap(mps_ap_t *ap_o, mps_pool_t pool) {
    return mps_ap_create_k(ap_o, pool, mps_args_none);
}

static void rust_mps_debug_print_formatted_object(mps_addr_t addr,
                                            mps_fmt_t fmt,
                                            mps_pool_t pool,
                                            void *p, size_t s) {
    assert(p == fmt);
    struct obj_stub *obj = addr;
    assert(obj->type < ARRAY_LEN(OBJ_MPS_TYPE_NAMES));

    const char *mps_type = OBJ_MPS_TYPE_NAMES[obj->type];
    fprintf(stderr, "%s(0x%012"PRIxPTR") [%"PRIu32" bytes] ", mps_type, (uintptr_t)addr, obj->size);
    if (obj->type == OBJ_MPS_TYPE_OBJECT || obj->type == OBJ_MPS_TYPE_ARRAY) {
        fprintf(stderr, "[type: %"PRIu16"]", obj->cljtype);
    }
    fprintf(stderr, "\n");

    if (obj->type == OBJ_MPS_TYPE_OBJECT) {
        size_t count = (obj->size - HEADER_SIZE) / WORD_SIZE;
        for (size_t i=0; i<count; i++) {
            uint16_t tag = ((uintptr_t)obj->ref[i] & TAG_MASK) >> VAL_BITS;
            uint64_t val = (uintptr_t)obj->ref[i] & ~TAG_MASK;

            fprintf(stderr, "  0x%04"PRIx16":%012"PRIx64"\n", tag, val);
        }
    }
}

void rust_mps_debug_print_reachable(mps_arena_t arena, mps_fmt_t fmt) {
    fprintf(stderr, "==== Walking Reachable Objects ====\n");
    mps_arena_formatted_objects_walk(arena, rust_mps_debug_print_formatted_object, fmt, 0);
}
