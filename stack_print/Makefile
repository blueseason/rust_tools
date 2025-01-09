all: stack stack1 rust

CFLAGS = -O1 -g -fno-omit-frame-pointer -I/usr/include/libunwind/

stack: stack_print.c
	gcc $(CFLAGS) -o $@ $<
stack1: stack_print1.c
	gcc $(CFLAGS) -funwind-tables -o $@ $< -lunwind

rust: src/main.rs
	cargo build

run: all
	@echo ----- Stack Test -----
	@./stack
	@echo ----- Stack1 Test -----
	@./stack1
#	@echo ----- Rust Stack Test -----
#	cargo run
clean:
	rm -f stack stack1
	cargo clean
