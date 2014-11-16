/* 
TEST_HEADER
 id = $Id: //info.ravenbrook.com/project/mps/version/1.114/test/misc/2.c#1 $
 summary = access negative location.
 language = c
 link = testlib.o
 parameters = NUM=1
OUTPUT_SPEC
 memoryerror = true
END_HEADER
*/

#include "testlib.h"
#include "mpsavm.h"

void *stackpointer;


static void test(void)
{
 mps_arena_t arena;
 char *p;

 cdie(mps_arena_create(&arena, mps_arena_class_vm(), 64*1024uL*1024uL),
      "create arena");

 p = (char *)-NUM;
 *p = 0;
 comment("%p", *p);
}

int main(void)
{
 void *m;
 stackpointer=&m;

 easy_tramp(test);
 return 0;
}

