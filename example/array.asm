.define stdout 1
.define start_val 16

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

    push start_val
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
    .local_array out 9 ; 8 hex chars + 1 newline
    .local_addrs out
    .locals index int hex_len
    .start_frame

    push 0
    storef index

    _loop:
        loadf ints ; &ints
        loadf index
        add ; &ints + index
        load ; *(&ints + index)
        storef int

        loadf out.addr
        loadf int
        push
        jal int_to_hex
        storef hex_len
        addsp -2

        push 0x0a ; newline
        loadi fp
        push out
        add
        loadf hex_len
        add
        store

        loadf hex_len
        addi 1 ; for the newline
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
    .args int buf ; buf must be at least len 8
    .locals char
    .return nchars

    .start_frame

    push 0
    storef nchars

    _loop:
        loadf int
        andi 0xf
        push
        jal int_to_hex_char
        storef char
        addsp -1

        loadf char
        loadf buf
        store

        loadf nchars
        addi 1
        storef nchars
        loadf int
        shr 4
        storef int

        loadf buf
        addi 1
        storef buf

        push 0
        loadf int
        bne _loop

_end:
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

reverse_array:
    .args arr arr.len
    .locals a b
    .start_frame



    .end_frame
    ret