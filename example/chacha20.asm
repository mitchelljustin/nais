.define stdout 0x1

main:
    .local str 32

    .start_frame

    .end_frame
    push 0 ; OK
    ecall ecall.exit
