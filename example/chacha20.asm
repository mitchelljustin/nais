.define STDIN 1
.define STDOUT 1

.define EXIT_OK 0

.define ROUNDS 20

.define C_0         0x65787061 ;expa
.define C_1         0x6e642033 ;nd 3
.define C_2         0x322d6279 ;2-by
.define C_3         0x7465206b ;te k
.define NONCE_0     0x01234567
.define NONCE_1     0x89abcdef

main:
    .local key 8
    .local key.addr 1
    .local state 16
    .local state.addr 1
    .local i 1
    .start_frame

    loadi fp
    addi key
    storef key.addr

    loadi fp
    addi state
    storef state.addr

    ; read key from STDIN
    push .sizeof.key
    loadf key.addr
    push STDIN
    ecall .ecall.read
    addsp -1

    _build_state:
    push C_0
    storef state, 0
    push C_1
    storef state, 1
    push C_2
    storef state, 2
    push C_3
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
    push 0
    storef state, 12
    push 0
    storef state, 13

    push NONCE_0
    storef state, 14
    push NONCE_1
    storef state, 15

    ebreak

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

    .end_frame
    push EXIT_OK
    ecall .ecall.exit


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