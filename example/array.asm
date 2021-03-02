entry:
    .array state 10
    .start_frame

    push state.len
    loadi fp
    addi state
    jal fill_array
    addsp -2

    push state.len
    loadi fp
    addi state
    jal print_array
    addsp -2

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
    jal increment
    storef val

    loadf index
    addi 1
    storef index

    loadf index
    loadf array.len
    blt _loop ;  if index < len goto loop

    .end_frame
    ret

increment:
    .args val

    .start_frame

    loadf val
    addi 1
    storef val

    .end_frame
    ret

print_array:
    .args array array.len
    .locals index
    .start_frame

    push 0;
    storef index;

_loop:
    loadf index
    loadf array
    add
    load
    print

    loadf index
    addi 1
    storef index

    loadf index
    loadf array.len
    blt _loop

    .end_frame
    ret