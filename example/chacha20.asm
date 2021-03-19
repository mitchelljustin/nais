.define STDIN 1
.define STDOUT 1

.define NUL 0x00
.define SPACE 0x20

.define EXIT_OK 0

.define DELTA_MAX 10

main:
    .local str 64
    .local str.addr 1
    .local str.len 1
    .local delta 1
    .start_frame

    loadi fp
    addi str
    storef str.addr

    push .sizeof.str
    loadf str.addr
    push STDIN
    ecall .ecall.read
    addsp -1

    push .sizeof.str
    loadf str.addr
    push
    jal strlen
    storef str.len
    addsp -2

    push 0
    storef delta

    _print_loop:
        push 1
        loadf str.len
        loadf str.addr
        push
        jal str_add_all
        addsp -3

        loadf str.len
        loadf str.addr
        push STDOUT
        ecall .ecall.write
        addsp -1

        loadf delta
        addi 1
        storef delta

        loadf delta
        push DELTA_MAX
        blt _print_loop

    .end_frame
    push EXIT_OK
    ecall .ecall.exit

str_add_all:
    .param str.addr 1
    .param str.len 1
    .param delta 1
    .local ch 1
    .local i 1
    .start_frame

    push 0
    storef i

    _loop:
        loadf str.addr
        loadf i
        add
        load
        storef ch

        loadf ch
        push SPACE
        blt _next

        _modify:
        loadf ch
        loadf delta
        add
        storef ch

        loadf ch
        loadf str.addr
        loadf i
        add
        store

        _next:
        loadf i
        addi 1
        storef i

        loadf i
        loadf str.len
        blt _loop

    .end_frame
    ret

strlen:
    .param str.addr 1
    .param str.max_len 1
    .local str.len 1
    .start_frame

    push 0
    storef str.len

    _loop:
        loadf str.addr
        loadf str.len
        add
        load
        push NUL
        beq _end

        loadf str.len
        loadf str.max_len
        bge _end

        loadf str.len
        addi 1
        storef str.len

        jump _loop

    _end:
        loadf str.len
        storef retval
        .end_frame
        ret