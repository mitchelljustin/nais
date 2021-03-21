.define ROW_SIZE 1024

.define NOT_FOUND -1
.define NEWLINE 0x0a

main:
    push
    jal sum_rows
    ecall .cc.exit

sum_rows:
    .local path 64
    .local path.addr 1
    .local path.len 1
    .local fd 1
    .local buf 64
    .local buf.len 1
    .local buf.addr 1
    .local row 64
    .local row.addr 1
    .local columns.addr 1
    .local columns.len 1
    .local sums.addr 1
    .local newline_idx 1
    .local i 0
    .start_frame

    loadi fp
    addi path
    storef path.addr

    loadi fp
    addi row
    storef row.addr

    loadi fp
    addi buf
    storef buf.addr

    jump _preset_path
    _user_path:
        push .sizeof.path
        loadf path.addr
        push
        jal read_path_from_stdin
        storef path.len
        loadf path.len
        storef retval
        loadf retval
        push 0
        blt _end
        loadf path.len
        subi 1 ; strip newline
        storef path.len
        jump _path_done
    _preset_path:
        push .L.PRESET_PATH.len
        storef path.len

        loadf path.len
        loadi pc
        addi 2, PRESET_PATH
        loadf path.addr
        push
        jal memcpy
        addsp -4
    _path_done:
    loadf path.len
    loadf path.addr
    ecall .cc.open
    storef fd
    loadf fd
    storef retval
    loadf retval
    push 0
    blt _end

    push .sizeof.buf
    loadf buf.addr
    loadf fd
    ecall .cc.read
    storef buf.len
    loadf buf.len
    storef retval
    loadf retval
    push 0
    blt _end

    push .sizeof.buf
    loadf buf.addr
    push NEWLINE
    push
    jal index_of
    storef newline_idx
    addsp -3

    loadf newline_idx
    loadf buf.addr
    push
    jal num_columns
    storef columns.len
    addsp -2

    loadf columns.len
    muli 2
    ; [ptr0 len0 ptr1 len1 .. ptr(n) len(n)]
    ecall .cc.malloc
    storef columns.addr

    loadf columns.addr
    loadf newline_idx
    loadf buf.addr
    push
    jal load_row
    addsp -4
    ebreak

    _ok:
    push 0
    storef retval
    _end:
    .end_frame
    ret

load_row:
    .param src.addr 1
    .param src.len 1
    .param row.addr 1 ; row addr must have size >= (2 * ncols)
    .local col.addr 1
    .local col.len 1
    .local i 1
    .start_frame

    push 0
    loadf src.len
    beq _end

    loadf src.addr
    storef col.addr

    push 0
    storef i
    _loop:
        loadf col.addr
        loadf row.addr
        store

        loadf i
        loadf src.len
        sub ; TOP = src.len - i
        loadf col.addr
        push ','
        push
        jal index_of
        storef col.len
        addsp -3

        loadf col.len
        push NOT_FOUND
        bne _comma_found

        _comma_not_found:
        loadf i
        loadf src.len
        sub ; TOP = src.len - i
        storef col.len

        _comma_found:
        loadf col.len
        loadf row.addr
        addi 1
        store

        loadf row.addr
        addi 2
        storef row.addr

        loadf col.addr
        loadf col.len
        add 1 ; +1 for the comma
        storef col.addr

        loadf i
        loadf col.len
        add 1 ; +1 for the comma
        storef i

        loadf i
        loadf src.len
        blt _loop
    _end:
    .end_frame
    ret

index_of:
    .param chr 1
    .param str.addr 1
    .param str.len 1
    .local i 1
    .start_frame

    push 0
    storef i
    _loop:
        ; if str[i] == chr { goto _found }
        loadf str.addr
        loadf i
        add
        load
        loadf chr
        beq _found

        loadf i
        addi 1
        storef i

        ; if i >= str.len { goto _not_found } else { goto _loop }
        loadf i
        loadf str.len
        bge _not_found
        jump _loop
    _not_found:
    push NOT_FOUND
    storef retval
    jump _end
    _found:
    loadf i
    storef retval
    _end:
    .end_frame
    ret

num_columns:
    .param str.addr 1
    .param str.len 1
    .local n 1
    .local i 1
    .start_frame

    push 0
    loadf str.len
    beq _empty

    push 1
    storef n

    push 0
    storef i
    _loop:
        loadf str.addr
        loadf i
        add
        load
        push ','
        bne _next
        loadf n
        addi 1
        storef n

        _next:
        loadf i
        addi 1
        storef i

        loadf i
        loadf str.len
        blt _loop
    loadf n
    storef retval
    jump _end
    _empty:
    push 0
    storef retval
    _end:
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

read_path_from_stdin:
    .param path.addr 1
    .param path.len 1
    .start_frame

    push .L.PROMPT.PATH.len
    loadi pc
    addi 2, PROMPT.PATH
    push .fd.stdout
    ecall .cc.write
    storef retval
    loadf retval
    push 0
    blt _end

    loadf path.len
    loadf path.addr
    push .fd.stdin
    ecall .cc.read
    storef retval

    _end:
    .end_frame
    ret

PROMPT.PATH:
    .string "Enter" 0x20 "path:" 0x20

PRESET_PATH:
    .string "example/d.csv"