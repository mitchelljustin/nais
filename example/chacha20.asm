.define STDIN 1
.define STDOUT 1
.define STDERR 2

.define OK 0

.define ROUNDS 20

main:
    .local errcode 1
    .local key 8
    .local key.addr 1
    .local nonce 2
    .local nonce.addr 1
    .local state 16
    .local state.addr 1
    .local orig_state 16
    .local orig_state.addr 1
    .local i 1
    .local pos 1
    .start_frame

    ; initialization
    loadi fp
    addi key
    storef key.addr

    loadi fp
    addi nonce
    storef nonce.addr

    loadi fp
    addi state
    storef state.addr

    loadi fp
    addi orig_state
    storef orig_state.addr

    push 0
    storef pos

    ; read state data
    push .L.PROMPT.KEY.len
    loadi pc
    addi 2, PROMPT.KEY
    push STDERR
    ecall .ecall.write
    storef errcode
    loadf errcode
    push OK
    bne _err

    push .sizeof.key
    loadf key.addr
    push
    jal read_stdin_packed
    storef errcode
    addsp -2
    loadf errcode
    push OK
    bne _err

    push .L.PROMPT.NONCE.len
    loadi pc
    addi 2, PROMPT.NONCE
    push STDERR
    ecall .ecall.write
    storef errcode
    loadf errcode
    push OK
    bne _err

    push .sizeof.nonce
    loadf nonce.addr
    push
    jal read_stdin_packed
    storef errcode
    addsp -2
    loadf errcode
    push OK
    bne _err

    ; initialize state
    loadr STATE.CONST, 0
    storef state, 0
    loadr STATE.CONST, 1
    storef state, 1
    loadr STATE.CONST, 2
    storef state, 2
    loadr STATE.CONST, 3
    storef state, 3

    loadf key, 0
    storef state, 4
    loadf key, 1
    storef state, 5
    loadf key, 2
    storef state, 6
    loadf key, 3
    storef state, 7
    loadf key, 4
    storef state, 8
    loadf key, 5
    storef state, 9
    loadf key, 6
    storef state, 10
    loadf key, 7
    storef state, 11

    ; Position
    push 0 ; pos < 2^31-1
    storef state, 12
    loadf pos
    storef state, 13

    loadf nonce, 0
    storef state, 14
    loadf nonce, 1
    storef state, 15
    ; done initializing state

    ; copy state to orig_state
    push .sizeof.state
    loadf state.addr
    loadf orig_state.addr
    push
    jal memcpy
    addsp -4

    ; perform rounds
    push 0
    storef i
    _rounds_loop:
        loadf state.addr
        push
        jal double_round
        addsp -2

        loadf i
        addi 2
        storef i

        loadf i
        push ROUNDS
        blt _rounds_loop

    push 0
    storef i
    _add_loop:
        loadf orig_state.addr
        loadf i
        add
        load

        loadf state.addr
        loadf i
        add
        load

        add

        loadf state.addr
        loadf i
        add
        store

        loadf i
        addi 1
        storef i

        loadf i
        push .sizeof.state
        blt _add_loop

    push .sizeof.state
    loadf state.addr
    push STDOUT
    ecall .ecall.write
    storef errcode

    push OK
    loadf errcode
    bne _err

    .end_frame
    push OK
    ecall .ecall.exit

    _err:
    loadf errcode
    ecall .ecall.exit

read_stdin_packed:
    .param out.addr 1
    .param out.len 1
    .local buf 128
    .local buf.addr 1
    .local buf.len 1
    .local word 1
    .local i 1
    .start_frame

    loadf out.len
    muli 4
    storef buf.len

    loadf out.len
    loadf buf.len
    bgt _too_long

    loadi fp
    addi buf
    storef buf.addr

    loadf buf.len
    loadf buf.addr
    push STDIN
    ecall .ecall.read
    storef retval

    loadf retval
    push OK
    bne _end

    ; for (i = 0; i < sizeof(buf); i += 4)
    push 0
    storef i
    _loop:
        push 0
        storef word

        ; word |= (buf[i] << 24);
        loadf buf.addr
        loadf i
        add
        load
        shli 24
        loadf word
        or
        storef word

        ; word |= (buf[i+1] << 16);
        loadf buf.addr
        loadf i
        addi 1
        add
        load
        shli 16
        loadf word
        or
        storef word

        ; word |= (buf[i+2] << 8);
        loadf buf.addr
        loadf i
        addi 2
        add
        load
        shli 8
        loadf word
        or
        storef word

        ; word |= buf[i+3];
        loadf buf.addr
        loadf i
        addi 3
        add
        load
        loadf word
        or
        storef word

        ; out[i / 4] = word;
        loadf word
        loadf out.addr
        loadf i
        divi 4
        add
        store

        loadf i
        addi 4
        storef i

        loadf i
        loadf buf.len
        blt _loop
    jump _end
    _too_long:
        push -3
        storef retval
    _end:
        .end_frame
        ret

double_round:
    .param state.addr 1
    .start_frame

    ; ROWS
    loadf state.addr
    addi 0
    loadf state.addr
    addi 4
    loadf state.addr
    addi 8
    loadf state.addr
    addi 12
    push
    jal quarter_round
    addsp -5

    loadf state.addr
    addi 1
    loadf state.addr
    addi 5
    loadf state.addr
    addi 9
    loadf state.addr
    addi 13
    push
    jal quarter_round
    addsp -5

    loadf state.addr
    addi 2
    loadf state.addr
    addi 6
    loadf state.addr
    addi 10
    loadf state.addr
    addi 14
    push
    jal quarter_round
    addsp -5

    loadf state.addr
    addi 3
    loadf state.addr
    addi 7
    loadf state.addr
    addi 11
    loadf state.addr
    addi 15
    push
    jal quarter_round
    addsp -5

    ; DIAGONALS

    loadf state.addr
    addi 0
    loadf state.addr
    addi 5
    loadf state.addr
    addi 10
    loadf state.addr
    addi 15
    push
    jal quarter_round
    addsp -5

    loadf state.addr
    addi 1
    loadf state.addr
    addi 6
    loadf state.addr
    addi 11
    loadf state.addr
    addi 12
    push
    jal quarter_round
    addsp -5

    loadf state.addr
    addi 2
    loadf state.addr
    addi 7
    loadf state.addr
    addi 8
    loadf state.addr
    addi 13
    push
    jal quarter_round
    addsp -5

    loadf state.addr
    addi 3
    loadf state.addr
    addi 4
    loadf state.addr
    addi 9
    loadf state.addr
    addi 14
    push
    jal quarter_round
    addsp -5

    .end_frame
    ret

quarter_round: ;(i32*,i32*,i32*,i32*)
    .param a 1
    .param b 1
    .param c 1
    .param d 1
    .start_frame

    push 16
    loadf d
    loadf b
    loadf a
    push
    jal qqround
    addsp -5

    push 12
    loadf b
    loadf d
    loadf c
    push
    jal qqround
    addsp -5

    push 8
    loadf d
    loadf b
    loadf a
    push
    jal qqround
    addsp -5

    push 7
    loadf b
    loadf d
    loadf c
    push
    jal qqround
    addsp -5

    .end_frame
    ret

qqround: ;(i32*,i32*,i32*,i32)
    .param x 1
    .param y 1
    .param z 1
    .param rotamt 1
    .start_frame

    ; *x += *y
    loadf x
    load
    loadf y
    load
    add
    loadf x
    store

    ; *z ^= *x
    loadf x
    load
    loadf z
    load
    xor
    loadf z
    store

    ; *z = (*z << rotamt) | (*z >> rotamt)
    loadf z
    load
    loadf rotamt
    shl
    loadf z
    load
    loadf rotamt
    push 32
    sub
    shr
    or
    loadf z
    store

    .end_frame
    ret

memcpy:
    .param dst.addr 1
    .param src.addr 1
    .param len 1
    .local i 1
    .start_frame

    push 0
    storef i
    _loop:
        loadf src.addr
        loadf i
        add
        load

        loadf dst.addr
        loadf i
        add
        store

        loadf i
        addi 1
        storef i

        loadf i
        loadf len
        blt _loop

    .end_frame
    ret

PROMPT.KEY:
    .string "KEY>" 0x20 ; space

PROMPT.NONCE:
    .string "NONCE>" 0x20

STATE.CONST:
    .word "expa"
    .word "nd" 0x20 "3"
    .word "2-by"
    .word "te" 0x20 "k"
