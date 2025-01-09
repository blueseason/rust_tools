#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>

#define UNW_LOCAL_ONLY
#include <libunwind.h>

// Compile with -funwind-tables -lunwind
void print_stack_trace_libunwind() {
  printf("=== Stack trace from libunwind ===\n");

  unw_cursor_t cursor;
  unw_context_t uc;
  unw_word_t pc, sp;

  unw_getcontext(&uc);
  unw_init_local(&cursor, &uc);

  while (unw_step(&cursor) > 0) {
    unw_get_reg(&cursor, UNW_REG_IP, &pc);
    unw_get_reg(&cursor, UNW_REG_SP, &sp);

    printf("Program counter: 0x%016" PRIxPTR "\n", (uintptr_t)pc);
    printf("Stack pointer: 0x%016" PRIxPTR "\n", (uintptr_t)sp);
    printf("\n");
  }
  printf("=== End ===\n\n");
}

int main() { print_stack_trace_libunwind(); }
