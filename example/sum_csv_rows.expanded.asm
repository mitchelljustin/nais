.define NOT_FOUND -1
.define NEWLINE 0x0a

main:
    push
    jal sum_rows
    ecall .cc.exit

sum_rows:
    .local path 64
    .addr_of path
    .local path.len 1
    .local fd 1
    .local buf 64
    .addr_of buf
    .local buf.len 1
    .local buf.ptr 1
    .local row 64
    .addr_of row
    .local col.addr 1
    .local columns.addr 1
    .local ncols 1
    .local sums.addr 1
    .local line_len 1
    .local i 1
    .local x 1
    .start_frame
; BEGIN {{
    loadi fp
    loadi sp
    storei fp
    addsp 206
    loadi fp
    addi path
    storef path.addr
    loadi fp
    addi buf
    storef buf.addr
    loadi fp
    addi row
    storef row.addr
; }} END

    jump _preset_path
    _user_path:
        .call read_path_from_stdin lf:path.addr p:.sizeof.path ret:path.len
; BEGIN {{
    push .sizeof.path
    loadf path.addr
    push
    jal read_path_from_stdin
    storef path.len
    addsp -2
; }} END
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
    .call env.open lf:path.addr lf:path.len ret:fd
; BEGIN {{
    loadf path.len
    loadf path.addr
    ecall .cc.open
    storef fd
; }} END

    loadf fd
    storef retval
    loadf retval
    push 0
    blt _end

    .call env.read lf:fd lf:buf.addr p:.sizeof.buf ret:buf.len
; BEGIN {{
    push .sizeof.buf
    loadf buf.addr
    loadf fd
    ecall .cc.read
    storef buf.len
; }} END

    loadf buf.len
    storef retval
    loadf retval
    push 0
    blt _end

    .call index_of p:NEWLINE lf:buf.addr p:.sizeof.buf ret:line_len
; BEGIN {{
    push .sizeof.buf
    loadf buf.addr
    push NEWLINE
    push
    jal index_of
    storef line_len
    addsp -3
; }} END

    .call num_columns lf:buf.addr lf:line_len ret:ncols
; BEGIN {{
    loadf line_len
    loadf buf.addr
    push
    jal num_columns
    storef ncols
    addsp -2
; }} END

    loadf ncols
    muli 2
    ; [ptr0 len0 ptr1 len1 .. ptr(n) len(n)]
    ecall .cc.malloc
    storef columns.addr

    .call read_row lf:buf.addr lf:line_len lf:columns.addr
; BEGIN {{
    loadf columns.addr
    loadf line_len
    loadf buf.addr
    push
    jal read_row
    addsp -4
; }} END

    .call env.malloc lf:ncols ret:sums.addr
; BEGIN {{
    loadf ncols
    ecall .cc.malloc
    storef sums.addr
; }} END

    .for i 0 to ncols ; init sums loop
; BEGIN {{
    push 0
    storef i
    _loop.0:
; }} END
        push 0
        loadf sums.addr
        loadf i
        add
        store
    .end_for
; BEGIN {{
    loadf i
    addi 1
    storef i
    loadf i
    loadf ncols
    blt _loop.0
; }} END
    loadf buf.addr
    storef buf.ptr
    _sum_loop:
        loadf buf.ptr
        loadf line_len
        add 1 ; for the newline
        storef buf.ptr

        loadf line_len
        loadf buf.len
        sub 1
        storef buf.len

        .call index_of p:NEWLINE lf:buf.ptr lf:buf.len ret:line_len
; BEGIN {{
    loadf buf.len
    loadf buf.ptr
    push NEWLINE
    push
    jal index_of
    storef line_len
    addsp -3
; }} END

        loadf line_len
        push NOT_FOUND
        beq _sum_done

        .call read_row lf:buf.ptr lf:line_len lf:row.addr
; BEGIN {{
    loadf row.addr
    loadf line_len
    loadf buf.ptr
    push
    jal read_row
    addsp -4
; }} END

        .for i 0 to ncols ; row sum loop
; BEGIN {{
    push 0
    storef i
    _loop.1:
; }} END
            ; x = dec_to_int(row[i*2], row[i*2+1])
            loadf row.addr
            loadf i
            muli 2
            add
            storef col.addr

            loadf col.addr
            load 1 ; len
            loadf col.addr
            load 0 ; ptr
            push
            jal dec_to_int
            storef x
            addsp -2

            ; sums[i] = sums[i] + x
            loadf sums.addr
            loadf i
            add
            load
            loadf x
            add
            loadf sums.addr
            loadf i
            add
            store
        .end_for
; BEGIN {{
    loadf i
    addi 1
    storef i
    loadf i
    loadf ncols
    blt _loop.1
; }} END
        jump _sum_loop
    _sum_done:
    .for i 0 to ncols ; print loop
; BEGIN {{
    push 0
    storef i
    _loop.2:
; }} END
        loadf columns.addr
        loadf i
        muli 2
        add
        storef col.addr

        loadf col.addr
        load 1 ; len
        loadf col.addr
        load 0 ; ptr
        loadf path.addr
        push
        jal memcpy
        addsp -4

        loadf col.addr
        load 1 ; len
        storef path.len

        push '='
        loadf path.addr
        loadf path.len
        add
        store
        loadf path.len
        addi 1
        storef path.len

        ; path.len += int_to_dec(sums[i], &path[path.len]);
        loadf path.addr
        loadf path.len
        add
        loadf sums.addr
        loadf i
        add
        load
        push
        jal int_to_dec
        loadf path.len
        add
        storef path.len
        addsp -2

        push NEWLINE
        loadf path.addr
        loadf path.len
        add
        store
        loadf path.len
        addi 1
        storef path.len

        .call env.write p:.fd.stdout lf:path.addr lf:path.len
; BEGIN {{
    loadf path.len
    loadf path.addr
    push .fd.stdout
    ecall .cc.write
    addsp -1
; }} END
    .end_for
; BEGIN {{
    loadf i
    addi 1
    storef i
    loadf i
    loadf ncols
    blt _loop.2
; }} END
    _ok:
    push 0
    storef retval
    _end:
    .end_frame
; BEGIN {{
    addsp -206
    storei fp
; }} END
    ret

dec_to_int:
    .param decimal.addr 1
    .param decimal.len 1
    .local i 1
    .local digit 1
    .start_frame
; BEGIN {{
    loadi fp
    loadi sp
    storei fp
    addsp 2
; }} END

    push 0
    storef retval

    .for i 0 to decimal.len
; BEGIN {{
    push 0
    storef i
    _loop.0:
; }} END
        loadf retval
        muli 10
        storef retval

        ; digit = decimal[i] - '0';
        loadf decimal.addr
        loadf i
        add
        load
        subi '0'
        storef digit

        loadf digit
        loadf retval
        add
        storef retval
    .end_for
; BEGIN {{
    loadf i
    addi 1
    storef i
    loadf i
    loadf decimal.len
    blt _loop.0
; }} END

    .end_frame
; BEGIN {{
    addsp -2
    storei fp
; }} END
    ret

int_to_dec:
    .param x 1
    .param dst.addr 1 ; at least size 11
    .local i 1
    .start_frame
; BEGIN {{
    loadi fp
    loadi sp
    storei fp
    addsp 1
; }} END

    push 0
    storef i
    _loop:
        ; dst[i] = (x % 10) + '0'
        loadf x
        remi 10
        addi '0'
        loadf dst.addr
        loadf i
        add
        store

        loadf x
        divi 10
        storef x

        loadf i
        addi 1
        storef i

        loadf x
        push 0
        bne _loop
    loadf i
    storef retval
    .end_frame
; BEGIN {{
    addsp -1
    storei fp
; }} END
    ret

read_row:
    .param src.addr 1
    .param src.len 1
    .param row.addr 1 ; row addr must have size >= (2 * ncols)
    .local col.addr 1
    .local col.len 1
    .local i 1
    .start_frame
; BEGIN {{
    loadi fp
    loadi sp
    storei fp
    addsp 3
; }} END

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
; BEGIN {{
    addsp -3
    storei fp
; }} END
    ret

index_of:
    .param chr 1
    .param str.addr 1
    .param str.len 1
    .local i 1
    .start_frame
; BEGIN {{
    loadi fp
    loadi sp
    storei fp
    addsp 1
; }} END

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
; BEGIN {{
    addsp -1
    storei fp
; }} END
    ret

num_columns:
    .param str.addr 1
    .param str.len 1
    .local n 1
    .local i 1
    .start_frame
; BEGIN {{
    loadi fp
    loadi sp
    storei fp
    addsp 2
; }} END

    push 0
    loadf str.len
    beq _empty

    push 1
    storef n

    .for i 0 to str.len
; BEGIN {{
    push 0
    storef i
    _loop.0:
; }} END
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
    .end_for
; BEGIN {{
    loadf i
    addi 1
    storef i
    loadf i
    loadf str.len
    blt _loop.0
; }} END

    loadf n
    storef retval
    jump _end
    _empty:
    push 0
    storef retval
    _end:
    .end_frame
; BEGIN {{
    addsp -2
    storei fp
; }} END
    ret

memcpy:
    .param dst.addr 1
    .param src.addr 1
    .param len 1
    .local i 1
    .start_frame
; BEGIN {{
    loadi fp
    loadi sp
    storei fp
    addsp 1
; }} END

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
; BEGIN {{
    addsp -1
    storei fp
; }} END
    ret

read_path_from_stdin:
    .param path.addr 1
    .param path.len 1
    .start_frame
; BEGIN {{
    loadi fp
    loadi sp
    storei fp
    addsp 0
; }} END

    push .L.PROMPT.PATH.len
    loadi pc
    addi 2, PROMPT.PATH
    push .fd.stdout
    ecall .cc.write
    storef retval
    loadf retval
    push 0
    blt _end

    .call env.read p:.fd.stdin lf:path.addr lf:path.len ret:retval
; BEGIN {{
    loadf path.len
    loadf path.addr
    push .fd.stdin
    ecall .cc.read
    storef retval
; }} END

    _end:
    .end_frame
; BEGIN {{
    addsp -0
    storei fp
; }} END
    ret

PROMPT.PATH:
    .string "Enter" 0x20 "path:" 0x20

PRESET_PATH:
    .string "example/d.csv"
