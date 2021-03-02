entry:
    .array state 10
    .start_frame

    push state.len
    loadi fp
    addi state
    push ; retval
    jal fill_array
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
    push
    jal increment
    storef val
    addsp -1

    loadf index
    addi 1
    storef index

    loadf index
    loadf array.len
    blt _loop ;  if index < len goto loop

    .end_frame
    ret

increment:
    .return retval
    .args val

    .start_frame

    loadf val
    addi 1
    storef retval

    .end_frame
    ret
