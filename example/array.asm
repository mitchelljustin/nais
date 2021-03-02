.define stdout 1

entry:
    .stack_array ints 16
    .local_addrs ints

    .start_frame

    push ints.len
    loadf ints.addr
    push
    jal fill_array
    addsp -3

    push ints.len
    loadf ints.addr
    push
    jal print_ints
    addsp -3

    .end_frame

    push 0
    ecall exit

fill_array:
    .args array array.len
    .locals index val

    .start_frame

    push 0
    storef index

    push 0
    storef val

    _loop:
        loadf val
        loadf array
        loadf index
        add
        store

        loadf val
        addi 1
        storef val

        loadf index
        addi 1
        storef index

        loadf index
        loadf array.len
        blt _loop ;  if index < len goto loop

    .end_frame
    ret

print_ints:
    .args ints ints.len
    .locals index int char
    .local_addrs char
    .start_frame

    push 0
    storef index

    _loop:
        loadf ints ; &ints
        loadf index
        add ; &ints + index
        load ; *(&ints + index)
        storef int

        loadf int
        push
        jal int_to_hex_char
        storef char
        addsp -1

        push char.len
        loadf char.addr
        push stdout
        ecall write
        addsp -1 ; ignore write result for now

        loadf index
        addi 1
        storef index

        loadf index
        loadf ints.len
        blt _loop

    push 0x0a
    storef char

    push char.len
    loadf char.addr
    push stdout
    ecall write
    addsp -1

    .end_frame
    ret

int_to_hex_char:
    .args int
    .return char

    .start_frame

    loadf int
    push 0
    blt _err ; if int < 0 err

    loadf int
    push 10
    blt _0_to_9

    loadf int
    push 16
    blt _a_to_f

    jump _err

    _0_to_9:
        loadf int
        addi '0'
        storef char
        jump _end

    _a_to_f:
        loadf int
        subi 10
        addi 'a'
        storef char
        jump _end

    _err:
        push '?'
        storef char

_end:
    .end_frame
    ret