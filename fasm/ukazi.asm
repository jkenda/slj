format ELF64 executable

use64

SYS_read  equ 0
SYS_write equ 1

STDIN  equ 0
STDOUT equ 1
STDERR equ 2

; write(fd, buf, count)
macro write fd, buf, count
{
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
macro read fd, buf, count
{
    ; syscall(SYS_write, fd, buf, count)
    mov rax, SYS_read
    mov rdi, fd
    mov rsi, buf
    mov rdx, count
    syscall
    cmp rax, 0
    jl _fatal_error
}

macro NOOP
{
    nop
}

macro JUMP label
{
    jmp label
}

macro CALL label
{
    call label
}

macro JMPD
{
    ret
}

macro JMPC label
{
    pop  rax
    test rax, 1
    jne label
}

macro PUSH data
{
    if data > 0xFFFF
        mov rax, data
        push rax
    else
        push data
    end if
}

macro ALOC mem
{
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

macro POS
{
    pop  rax
    cmp  rax, 0
    mov  rax, 0
    setg al
    push rax
}

macro ZERO
{
    pop  rax
    cmp  rax, 0
    mov  rax, 0
    sete al
    push rax
}

macro LOAD addr
{
    push qword [r8 - 8 - 8*addr]
}

macro LDOF addr
{
    push qword [r9 - 8 - 8*addr]
}

macro LDDY addr
{
    pop rax
    push qword [rax - 8 - 8*addr]
}

macro STOR addr
{
    pop qword [r8 - 8 - 8*addr]
}

macro STOF addr
{
    pop qword [r9 - 8 - 8*addr]
}

macro STDY addr
{
    ; save dynaddr to rax
    pop rax
    pop qword [rax - 8 - addr*8]
}

macro PC offset
{
}

macro TOP addr
{
    ; addroff = SP
    mov r9, rsp
}


macro SOFF
{
    ; addroff = stack.pop()
    pop r9
}

macro LOFF
{
    ; stack.push(addroff)
    push r9
}

macro PUTC
{
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

macro GETC
{
    call _getc
    push rax
}

macro ADDI
{
    pop rbx
    pop rax
    add rax, rbx
    push rax
}

macro SUBI
{
    pop rbx
    pop rax
    sub rax, rbx
    push rax
}

macro MULI
{
    pop  rax
    pop  rbx
    imul rax, rbx
    push rax
}

macro DIVI
{
    pop  rbx
    cdq
    pop  rax
    idiv ebx
    push rax
}

macro MODI
{
    pop  rbx
    cdq
    pop  rax
    idiv ebx
    push rdx
}

macro POWI
{
    mov rax, 1
    pop rcx
    pop rbx
    call _powi
    push rax
}

macro BOR 
{
    pop rbx
    pop rax
    or  rax, rbx
    push rax
}

macro BXOR
{
    pop rbx
    pop rax
    xor rax, rbx
    push rax
}

macro BAND
{
    pop rbx
    pop rax
    and rax, rbx
    push rax
}

macro BSLL
{
    pop rcx
    pop rax
    shl rax, cl
    push rax
}

macro BSLR
{
    pop rcx
    pop rax
    shr rax, cl
    push rax
}

macro ADDF
{
    _TODO
}

macro SUBF
{
    _TODO
}

macro MULF
{
    _TODO
}

macro DIVF
{
    _TODO
}

macro MODF
{
    _TODO
}

macro POWF
{
    _TODO
}

macro FTOI
{
    _TODO
}

macro ITOF
{
    _TODO
}

macro exit code
{
    mov rax, 60
    mov rdi, code
    syscall
}

segment readable writeable
; ptr to stack begin
stack_0 dq 0

segment readable executable

_powi:
    cmp rcx, 0
    je _powi_done
    imul rax, rbx
    dec rcx
    jmp _powi
_powi_done:
    ret

_getc:
    PUSH 0
    read STDIN, rsp, 1
    mov rax, [rsp]

    mov cl, al
    and cl, 11100000b
    cmp cl, 11000000b
    je _read2

    mov cl, al
    and cl, 11110000b
    cmp cl, 11100000b
    je _read3

    mov cl, al
    and cl, 11111000b
    cmp cl, 11110000b
    je _read4

_read1:
    ALOC -1
    ret
_read2:
    read STDIN, rsp, 1
    mov  rbx, [rsp]
    shl  rbx, 8
    or   rax, rbx
    ALOC -1
    ret
_read3:
    read STDIN, rsp, 2
    mov  rbx, [rsp]
    shl  rbx, 8
    or   rax, rbx
    ALOC -1
    ret
_read4:
    read STDIN, rsp, 3
    mov  rbx, [rsp]
    shl  rbx, 8
    or   rax, rbx
    ALOC -1
    ret

_fatal_error:
    exit rax

entry $
	mov r8, rsp
    ; addroff = SP
	mov r9, rsp

