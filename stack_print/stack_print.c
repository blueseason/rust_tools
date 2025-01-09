#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>

// Compile with -fno-omit-frame-pointer
/*
 * 用户地址空间范围：0x0000000000000000 到 0x00007FFFFFFFFFFF（128 TiB）。
 * 内核地址空间范围：0xFFFF800000000000 到 0xFFFFFFFFFFFFFFFF（128 TiB）。
 */
#define X86_USER_MAX_VA 0x00007FFFFFFFFFFF

void print_stack_trace_fp_chain() {
  printf("=== Stack trace from fp chain ===\n");

  uintptr_t *fp;
  asm("movq %%rbp, %0" : "=r"(fp));
  //  asm("mv %0, fp" : "=r"(fp) : :); riscv

  // When should this stop?
  while (fp && fp < (uintptr_t *)X86_USER_MAX_VA) {
    printf("Return address: 0x%016" PRIxPTR "\n", fp[-1]);
    printf("Old stack pointer: 0x%016" PRIxPTR "\n", fp[-2]);
    printf("\n");

    fp = (uintptr_t *)fp[-2];
  }
  printf("=== End ===\n\n");
}

int main() { print_stack_trace_fp_chain(); }
