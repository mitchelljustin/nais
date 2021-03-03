.define start_val 0xf000
.define increment 0x0020


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
    ecall ecall.exit

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
        addi increment
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
    .args ints.addr ints.len
    .locals index int nchars
    .local_array out 9 ; 8 hex chars + 1 newline
    .local_addrs out
    .start_frame

    push 0
    storef index

    _loop:
        loadf ints.addr ; &ints
        loadf index
        add ; &ints + index
        load ; *(&ints + index)
        storef int

        loadf out.addr
        loadf int
        push
        jal int_to_hex
        storef nchars
        addsp -2

        push 0x0a ; newline
        loadi fp
        push out
        add
        loadf nchars
        add
        store

        loadf nchars
        addi 1 ; for the newline
        loadf out.addr
        push stdout
        ecall ecall.write
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
    .args int out.addr ; buf must be at least len 8
    .return nchars
    .locals char
    .start_frame

    push 0
    storef nchars

    _loop:
        loadf int
        andi 0xf
        push
        jal int4_to_hex_char
        storef char
        addsp -1

        loadf char
        loadf out.addr
        loadf nchars
        add
        store

        loadf nchars
        addi 1
        storef nchars

        loadf int
        shr 4
        storef int

        push 0
        loadf int
        bne _loop

    loadf nchars
    loadf out.addr
    push
    jal reverse_array
    addsp -3

    .end_frame
    ret

int4_to_hex_char:
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
    .args arr.addr arr.len
    .locals temp i max
    .start_frame

    loadf arr.len
    divi 2
    storef max

    push 0
    storef i

    _loop:
        loadf arr.addr
        loadf i
        add
        load
        storef temp

        loadf arr.addr
        loadf i
        loadf arr.len
        subi 1
        sub
        add
        load

        loadf arr.addr
        loadf i
        add
        store ; arr[i] = TOP

        loadf temp
        loadf arr.addr
        loadf i
        loadf arr.len
        subi 1
        sub
        add
        store ; arr[len - i - 1] = temp

        loadf i
        addi 1
        storef i

        loadf i
        loadf max
        blt _loop

    .end_frame
    ret