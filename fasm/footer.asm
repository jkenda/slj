
    exit 0

_push:
    pop rbx
_push_loop:
    repeat 16
        push 0
    end repeat
    dec  rax
    cmp  rax, 0
    jne  _push_loop
    push rbx
    ret

_pop:
    pop rbx
_pop_loop:
    repeat 16
        pop qword [rsp - 8]
    end repeat
    dec  rax
    cmp  rax, 0
    jne  _pop_loop
    push rbx
    ret

_powi:
    cmp rcx, 0
    je _powi_done
    imul rax, rbx
    dec rcx
    jmp _powi
_powi_done:
    ret

_putc:
    ; d = a
    mov r10, rax

    ; move 1st char to the buffer
    mov rbx, [stdout_buf.ptr]
    mov [rbx], al
    mov rdx, 1

    ; move next ones only if they're non-zero
    mov rcx, 0
    offs = 1
    repeat 3
        shr   rax, 8
        cmp   al, 0
        setne cl
        mov   [rbx + offs], al
        add   rdx, rcx
        offs = offs + 1
    end repeat

    ; increment len and pointer by how many chars were added
    add [stdout_buf.ptr], rdx
    add [stdout_buf.len], rdx

    ; output buffer on newline
    cmp r10, 10
    je _flush

    ; output buffer if full
    cmp [stdout_buf.len], stdout_buf.cap - 4
    jge _flush

    ret


_flush:
    ; write stdout
    write STDOUT, stdout_buf.data, [stdout_buf.len]
    ; reset buffer
    mov [stdout_buf.ptr], stdout_buf.data
    mov [stdout_buf.len], 0
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
    pop qword [rsp - 8]
    ret
_read2:
    read STDIN, rsp, 1
    mov  rbx, [rsp]
    shl  rbx, 8
    or   rax, rbx
    pop  qword [rsp - 8]
    ret
_read3:
    read STDIN, rsp, 2
    mov  rbx, [rsp]
    shl  rbx, 8
    or   rax, rbx
    pop  qword [rsp - 8]
    ret
_read4:
    read STDIN, rsp, 3
    mov  rbx, [rsp]
    shl  rbx, 8
    or   rax, rbx
    pop  qword [rsp - 8]
    ret

_fatal_error:
    exit rax

segment readable writeable

struc vec cap
{
    .data rb cap
    .len  dq 0
    .ptr  dq .data
    .cap  = cap
}

stdout_buf vec 512
