/* 
TEST_HEADER
 id = $Id: //info.ravenbrook.com/project/mps/version/1.114/test/function/0.c#1 $
 summary = test that the mps header file is accepted by the compiler
 language = c
 link = testlib.o
END_HEADER
*/

#include "mps.h"
#include "testlib.h"

int main(void)
{
 pass();
 return 0;
}

