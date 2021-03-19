.define start_val 0x0000
.define increment 0x0003

.define stdout 1

entry:
    .local ints 100
    .local ints.addr 1

    .start_frame

    loadi fp
    addi ints
    storef ints.addr

    push .sizeof.ints
    loadf ints.addr
    push
    jal fill_array
    addsp -3

    push .sizeof.ints
    loadf ints.addr
    push
    jal print_ints
    addsp -3

    .end_frame

    push 0
    ecall .ecall.exit

fill_array:
    .param array 1
    .param array.len 1
    .local index 1
    .local x 1
    .start_frame

    push 0
    storef index

    push start_val
    storef x

    _loop:
        loadf x
        loadf array
        loadf index
        add
        store

        loadf x
        addi increment
        storef x

        loadf index
        addi 1
        storef index

        loadf index
        loadf array.len
        blt _loop ;  if index < len goto loop

    .end_frame
    ret

print_ints:
    .param ints.addr 1
    .param ints.len 1
    .local index 1
    .local x 1
    .local nchars 1
    .local out 9 ; 8 hex chars + 1 newline
    .local out.addr 1
    .start_frame

    push 0
    storef index

    loadi fp
    addi out
    storef out.addr

    _loop:
        loadf ints.addr
        loadf index
        add ; &ints + index
        load ; ints[index]
        storef x

        loadf out.addr
        loadf x
        push
        jal int_to_hex
        storef nchars
        addsp -2

        push 0x0a ; newline
        loadi fp
        addi out
        loadf nchars
        add
        store

        loadf nchars
        addi 1 ; for the newline
        storef nchars

        loadf nchars
        loadf out.addr
        push stdout
        ecall .ecall.write
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
    .param x 1
    .param out.addr 1 ; buf must be at least len 8
    .local ch 1
    .local nchars 1
    .start_frame

    push 0
    storef nchars

    _loop:
        loadf x
        andi 0xf
        push
        jal int4_to_hex_char
        storef ch
        addsp -1

        loadf ch
        loadf out.addr
        loadf nchars
        add
        store

        loadf nchars
        addi 1
        storef nchars

        loadf x
        shr 4
        storef x

        push 0
        loadf x
        bne _loop

    loadf nchars
    loadf out.addr
    push
    jal reverse_array
    addsp -3

    loadf nchars
    storef retval

    .end_frame
    ret

int4_to_hex_char:
    .param x 1
    .start_frame

    loadf x
    push 0
    blt _err ; if x < 0 err

    loadf x
    push 10
    blt _0_to_9

    loadf x
    push 16
    blt _a_to_f

    jump _err

    _0_to_9:
        loadf x
        addi '0'
        storef retval
        jump _end

    _a_to_f:
        loadf x
        subi 10
        addi 'a'
        storef retval
        jump _end

    _err:
        push '?'
        storef retval
    _end:
        .end_frame
        ret

reverse_array:
    .param arr.addr 1
    .param arr.len 1
    .local temp 1
    .local i 1
    .local max 1
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