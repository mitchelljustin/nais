.define ROW_SIZE 1024

main:
    push
    jal sum_rows
    ecall .cc.exit

sum_rows:
    .local path 64
    .local path.addr 1
    .local path.len 1
    .local fd 1
    .local row 64
    .local sums.addr 1
    .start_frame

    loadi fp
    addi path
    storef path.addr

    push .L.PROMPT.PATH.len
    loadi pc
    addi 2, PROMPT.PATH
    push .fd.stdout
    ecall .cc.write
    storef retval
    loadf retval
    push 0
    blt _end

    push .sizeof.path
    loadf path.addr
    push .fd.stdin
    ecall .cc.read
    storef path.len
    loadf path.len
    storef retval
    loadf retval
    push 0
    blt _end

    loadf path.len
    subi 1 ;; chop newline
    loadf path.addr
    ecall .cc.open
    storef fd
    loadf fd
    storef retval
    loadf retval
    push 0
    blt _end

    push .sizeof.row
    loadi fp
    addi row
    push .fd.stdin
    ecall .cc.read
    storef retval
    loadf retval
    push 0
    blt _end

    _ok:
    push 0
    storef retval
    _end:
    .end_frame
    ret

PROMPT.PATH:
    .string "Enter" 0x20 "path:" 0x20