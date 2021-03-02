.define stdout 1

entry:
    .local_array ints 20
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
    ecall callcode.exit

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
    .local_array out 2
    .local_addrs out
    .locals index int
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
        storef out
        addsp -1

        push 0x0a ; newline
        push out
        addi 1
        loadi fp
        add
        store

        push out.len
        loadf out.addr
        push stdout
        ecall callcode.write
        addsp -1 ; ignore write result for now

        loadf index
        addi 1
        storef index

        loadf index
        loadf ints.len
        blt _loop

    .end_frame
    ret

int_to_hex:
    .args int buf
    .locals index char

    .start_frame
    push 0
    storef index

    _loop:
        loadf int
        push
        jal int_to_hex_char
        storef char

        loadf char
        loadf buf
        store

        loadf buf
        addi 1
        storef buf


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