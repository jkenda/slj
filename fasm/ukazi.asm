format ELF64 executable

SYS_read  equ 0
SYS_write equ 1

STDIN  equ 0
STDOUT equ 1
STDERR equ 2

; write(fd, buf, count)
macro write fd, buf, count {
    ; syscall(SYS_write, fd, buf, count)
    mov rax, SYS_write
    mov rdi, fd
    mov rsi, buf
    mov rdx, count
    syscall
    cmp rax, 0
    jl .fatal_error
}

macro PUSH data {
    push qword data
}

macro JUMP addr {
    jmp stack_0 + addr
}

macro JMPC addr {
    pop  rax
    test rax, 1
    jne  stack_0 + addr
}

macro PC offset {
    push rax + offset
}

macro ALOC mem {
    if mem >= 0
        repeat mem
            push qword 0
        end repeat
    else
        repeat -mem
            pop qword [rsp-8]
        end repeat
    end if
}

macro ZERO {
    pop  rax
    cmp  rax, 0
    sete rax
    push rax
}

macro LOAD addr {
    push [stack_0 + addr]
}

macro LDOF addr {
    ; addroff stored in r8
    mov  rax, r8
    add  rax, stack_0
    push rax + addr
}

macro LDDY addr {
    pop  rax
    push rax + addr
}

macro STOR addr {
    pop stack_0 + addr
}

macro STOF addr {
    ; addroff stored in r8
    mov rax, r8
    add rax stack_0
    pop rax + addr
}

macro STDY addr {
    ; save dynaddr to rax
    pop rax
    pop rax + addr
}

macro TOP addr {
    ; addroff = PC
    mov rip, r8
}


macro SOFF {
    ; addroff = stack.pop()
    pop r8
}

macro LOFF {
    ; stack.push(addroff)
    push r8
}

macro PUTC {
    ; write(SYS_stdout, stack.pop(), 8)
    write STDOUT, rsp, 8
    ALOC -1
}

macro GETC {
    _TODO
}

macro ADDI {
    pop rbx
    pop rax
    add rax, rbx
    push rax
}

macro SUBI {
    pop rbx
    pop rax
    sub rax, rbx
    push rax
}

macro MULI {
    pop  rax
    pop  rcx
    mul  rcx
    push rax
}

macro DIVI {
    pop  rbx
    cdq
    pop  rax
    idiv ebx
    push rax
}

macro MODI {
    pop  rbx
    cdq
    pop  rax
    idiv ebx
    push rdx
}

macro POWI {
    pop rbx
    pop rax
    _TODO
    push rax
}

macro BOR  {
    pop rbx
    pop rax
    or  rax, rbx
    push rax
}

macro BXOR {
    pop rbx
    pop rax
    xor rax, rbx
    push rax
}

macro BAND {
    pop rbx
    pop rax
    xor rax, rbx
    push rax
}

macro BSLL {
    pop rbx
    pop rax
    sal rax, rbx
    push rax
}

macro BSLD {
    pop rbx
    pop rax
    sar rax, rbx
    push rax
}

macro ADDF {
    pop rbx
    pop rax
    fadd rax, rbx
    push rax
}

macro SUBF {
    pop rbx
    pop rax
    fsub rax, rbx
    push rax
}

macro MULF {
    pop rbx
    pop rax
    fmul rax, rbx
    push rax
}

macro DIVF {
    pop rbx
    pop rax
    fdiv rax, rbx
    push rax
}

macro MODF {
    pop rbx
    pop rax
    fdiv rax, rbx
    push rax
}

macro POWF {
    _TODO
}

macro FTOI {
    _TODO
}

macro ITOF {
    _TODO
}

macro exit code {
    mov rax, 60
    mov rdi, code
    syscall
}

segment readable writeable
; ptr to stack begin
stack_0 dq 0

segment readable executable

.fatal_error:
    exit rax

entry $
    mov [stack_0], rsp
    mov r8, rsp

    PUSH 48
    PUSH 3
    ADDI
    PUSH 48
    PUSH 1
    ADDI
    PUTC
    PUTC
    ; 48 + 1 = 49 -> '1'
    ; 48 + 3 = 51 -> '3'

    PUSH 10
    PUTC
    ; '\n'

    PUSH 58
    PUSH 10
    SUBI
    PUTC
    PUSH 10
    PUTC
    ; 58 - 10 = 48 -> '0'

    PUSH 15
    PUSH 4
    MULI
    PUTC
    PUSH 10
    PUTC
    ; 15 * 4 = 60 -> '<'

    PUSH 100
    PUSH 2
    DIVI
    PUTC
    PUSH 10
    PUTC
    ; 100 / 2 = 50 -> '2'

    PUSH 553
    PUSH 100
    MODI
    PUTC
    PUSH 10
    PUTC
    ; 553 % 100 = 53 -> '5'

    exit 0

