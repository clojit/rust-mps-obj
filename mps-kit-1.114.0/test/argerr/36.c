/* 
TEST_HEADER
 id = $Id: //info.ravenbrook.com/project/mps/version/1.114/test/argerr/36.c#1 $
 summary = wrong size_t to free (MV)
 language = c
 link = testlib.o
END_HEADER
*/

#include "testlib.h"
#include "mpscmv.h"
#include "arg.h"

void *stackpointer;

static void test(void)
{
 mps_arena_t arena;
 mps_pool_t pool;
 mps_thr_t thread;

 mps_addr_t a,b;

 cdie(mps_arena_create(&arena, mps_arena_class_vm(), mmqaArenaSIZE), "create arena");

 cdie(mps_thread_reg(&thread, arena), "register thread");

 cdie(
  mps_pool_create(
     &pool, arena, mps_class_mv(),
     (size_t) 4096, (size_t) 32, (size_t) 64*1024),
  "create pool");

 die(mps_alloc(&a, pool, 8),
     "alloc a");
 die(mps_alloc(&b, pool, 32),
     "alloc b");
 mps_free(pool, a, 9);
 mps_pool_destroy(pool);

}

int main(void)
{
 void *m;
 stackpointer=&m; /* hack to get stack pointer */

 easy_tramp(test);
 return 0;
}
