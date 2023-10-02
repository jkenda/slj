format ELF64 executable

use64

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
    jl _fatal_error
}

; write(fd, buf, count)
macro read fd, buf, count {
    ; syscall(SYS_write, fd, buf, count)
    mov rax, SYS_read
    mov rdi, fd
    mov rsi, buf
    mov rdx, count
    syscall
    cmp rax, 0
    jl _fatal_error
}

macro NOOP {
    nop
}

macro JUMP addr {
    jmp addr
}

macro JMPD {
    pop rax
    jmp [rax]
}

macro JMPC addr {
    pop  rax
    test rax, 1
    jne  stack_0 + addr
}

macro PUSH data {
    push data
}

macro ALOC mem {
    if mem >= 0
        repeat mem
            push 0
        end repeat
    else
        repeat -mem
            pop qword [rsp-8]
        end repeat
    end if
}

macro POS {
    pop  rax
    cmp  rax, 0
    mov  rax, 0
    setg al
    push rax
}

macro ZERO {
    pop  rax
    cmp  rax, 0
    mov  rax, 0
    sete al
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

macro PC offset {
    mov rax, rip
    add rax, offset
    push rax
}

macro TOP addr {
    ; addroff = PC
    mov r8, rip
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
    ; count UTF-8 bytes
    mov rax, [rsp]
    mov rbx, 1
    repeat 3
        shr   rax, 8
        cmp   al,  0
        setne cl
        add   bl,  cl
    end repeat

    ; write(SYS_stdout, dst, len)
    write STDOUT, rsp, rbx
    ALOC -1
}

macro GETC {
    call _getc
    push rax
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

_getc:
    read STDIN, rsp, 1
    ; if buf[0] & 0b11100000 == 0b11000000 {
    mov rax, [rsp]
    mov cl, al
    and cl, 11100000b
    cmp cl, 11000000b
    je _read2
    ; else if buf[0] & 0b11110000 == 0b11100000 {
    mov cl, al
    and cl, 11110000b
    cmp cl, 11100000b
    je _read3
    ; else if buf[0] & 0b11111000 == 0b11110000 {
    mov cl, al
    and cl, 11111000b
    cmp cl, 11110000b
    je _read4

_read1:
    ret
_read2:
    shl  rax, 8
    read STDIN, rsp, 1
    or   rax, [rsp]
    ret
_read3:
    shl  rax, 16
    read STDIN, rsp, 2
    or   rax, [rsp]
    ret
_read4:
    shl  rax, 24
    read STDIN, rsp, 3
    or   rax, [rsp]
    ret

_fatal_error:
    exit rax

