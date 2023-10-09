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
    ; syscall(SYS_read, fd, buf, count)
    mov rax, SYS_read
    mov rdi, fd
    mov rsi, buf
    mov rdx, count
    syscall
    cmp rax, 0
    jl _fatal_error
}

macro aloc mem
{
    local rem

    if mem > 16
        rem = mem - (mem / 16 * 16)
        repeat rem
            push 0
        end repeat
        mov rax, mem / 16
        call _push
    else if mem < -32
        rem = -mem - (-mem / 16 * 16)
        repeat rem
            pop qword [rsp-8]
        end repeat
        mov rax, -mem / 16
        call _pop
    else if mem >= 0
        repeat mem
            push 0
        end repeat
    else
        repeat -mem
            pop qword [rsp-8]
        end repeat
    end if
}

macro getc
{
    call _getc
    push rax
}

; TODO: bufferred
macro putc
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

    write STDOUT, rsp, rbx
    pop qword [rsi - 8]
}

macro powf
{
    fld dword [rsp]
    fld dword [rsp + 8]
    fyl2x
    fld1
    fld st1
    fprem
    f2xm1
    faddp
    fscale
    fxch st1
    pop  qword [rsp - 8]
    fstp dword [rsp]
}

macro exit code
{
    mov rax, 60
    mov rdi, code
    syscall
}

format ELF64 executable
use64

segment readable executable

entry $
	mov r8, rsp
    ; addroff = SP
	mov r9, rsp
    finit

