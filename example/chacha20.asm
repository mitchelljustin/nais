.define STDIN 1
.define STDOUT 1

.define EXIT_OK 0

.define ROUNDS 20

entry:
    jump main

.word C_0         0x65787061 ;expa
.word C_1         0x6e642033 ;nd 3
.word C_2         0x322d6279 ;2-by
.word C_3         0x7465206b ;te k
.word NONCE_0     0x01234567
.word NONCE_1     0x89abcdef

main:
    .local key 8
    .local key.addr 1
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
    addi state
    storef state.addr

    loadi fp
    addi orig_state
    storef orig_state.addr

    push 0
    storef pos

    ; read key from STDIN
    push .sizeof.key
    loadf key.addr
    push STDIN
    ecall .ecall.read
    addsp -1

    ; initialize state
    loadi C_0
    storef state, 0
    loadi C_1
    storef state, 1
    loadi C_2
    storef state, 2
    loadi C_3
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

    loadi NONCE_0
    storef state, 14
    loadi NONCE_1
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

        push .sizeof.state
        loadf i
        blt _add_loop



    .end_frame
    push EXIT_OK
    ecall .ecall.exit


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