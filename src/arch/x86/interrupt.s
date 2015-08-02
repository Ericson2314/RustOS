.section .text
.global no_op
.global unified_handler
.global register_all_callbacks

no_op:
  iret

.altmacro

.macro make_callback num
  callback_\num\():
.endm

.macro make_all_callbacks, num=50
.if \num+1
   make_callback %num
      pusha
      pushl $\num
      call unified_handler

      addl $4, %esp
      popa
      iret
  make_all_callbacks \num-1
.endif
.endm
make_all_callbacks

.macro push_callback num
  pushl $callback_\num\()
.endm

# args: &mut IDT
# the idea here is to use an as macro to fill in
# all of the interrupts
register_all_callbacks:
  pushl %ebp
  movl %esp, %ebp

  .macro make_register_all_callbacks, num=50
    .if \num+1
          push_callback %num # arg3 (fn) to add_entry
          pushl $\num # arg2 (index) to add_entry
          movl 8(%ebp), %eax
          pushl %eax # arg1 (&self) to add_entry
          call add_entry
          movl %ebp, %esp
      make_register_all_callbacks \num-1
    .endif
  .endm
  make_register_all_callbacks

  leave
  ret
