
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

