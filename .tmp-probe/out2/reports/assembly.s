# Complete decoded machine-code evidence. Generated source intentionally omits this noise.

# E15Vec.compareTo at 0x1578ec (220 bytes)
0x1578ec  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1578f0  fd030faa           mov      x29, x15
0x1578f4  ef4100d1           sub      x15, x15, #0x10
0x1578f8  e40301aa           mov      x4, x1
0x1578fc  e30302aa           mov      x3, x2
0x157900  a1831ff8           stur     x1, [x29, #-8]
0x157904  a2031ff8           stur     x2, [x29, #-0x10]
0x157908  502740f9           ldr      x16, [x26, #0x48]
0x15790c  ff0110eb           cmp      x15, x16
0x157910  89050054           b.ls     #0x1579c0
0x157914  e00303aa           mov      x0, x3
0x157918  e20316aa           mov      x2, x22
0x15791c  e10316aa           mov      x1, x22
0x157920  840780d2           mov      x4, #0x3c
0x157924  60000036           tbz      w0, #0, #0x157930
0x157928  04f05ff8           ldur     x4, [x0, #-1]
0x15792c  847c4cd3           ubfx     x4, x4, #0xc, #0x14
0x157930  9fd00bf1           cmp      x4, #0x2f4
0x157934  c0000054           b.eq     #0x15794c
0x157938  68234091           add      x8, x27, #8, lsl #12
0x15793c  08d945f9           ldr      x8, [x8, #0xbb0]  # pool[4468] = snapshotRef(15668)
0x157940  63234091           add      x3, x27, #8, lsl #12
0x157944  63dc45f9           ldr      x3, [x3, #0xbb8]  # pool[4469] = null
0x157948  0a9e0594           bl       #0x2bf170
0x15794c  a0835ff8           ldur     x0, [x29, #-8]
0x157950  017040f8           ldur     x1, [x0, #7]
0x157954  227c019b           mul      x2, x1, x1
0x157958  01f040f8           ldur     x1, [x0, #0xf]
0x15795c  207c019b           mul      x0, x1, x1
0x157960  4300008b           add      x3, x2, x0
0x157964  a0035ff8           ldur     x0, [x29, #-0x10]
0x157968  017040f8           ldur     x1, [x0, #7]
0x15796c  227c019b           mul      x2, x1, x1
0x157970  01f040f8           ldur     x1, [x0, #0xf]
0x157974  207c019b           mul      x0, x1, x1
0x157978  4400008b           add      x4, x2, x0
0x15797c  60787f93           sbfiz    x0, x3, #1, #0x1f
0x157980  7f0480eb           cmp      x3, x0, asr #1
0x157984  60000054           b.eq     #0x157990
0x157988  fea60594           bl       #0x2c1580
0x15798c  037000f8           stur     x3, [x0, #7]
0x157990  e20300aa           mov      x2, x0
0x157994  80787f93           sbfiz    x0, x4, #1, #0x1f
0x157998  9f0480eb           cmp      x4, x0, asr #1
0x15799c  60000054           b.eq     #0x1579a8
0x1579a0  f8a60594           bl       #0x2c1580
0x1579a4  047000f8           stur     x4, [x0, #7]
0x1579a8  e10302aa           mov      x1, x2
0x1579ac  e20300aa           mov      x2, x0
0x1579b0  d7280094           bl       #0x161d0c
0x1579b4  ef031daa           mov      x15, x29
0x1579b8  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1579bc  c0035fd6           ret      
0x1579c0  90a60594           bl       #0x2c1400
0x1579c4  d4ffff17           b        #0x157914
# CFG: 0x1578ec->0x157914/ConditionalFalse 0x1578ec->0x1579c0/ConditionalTrue 0x157914->0x157928/ConditionalFalse 0x157914->0x157930/ConditionalTrue 0x157928->0x157930/Fallthrough 0x157930->0x157938/ConditionalFalse 0x157930->0x15794c/ConditionalTrue 0x157938->0x15794c/Fallthrough 0x15794c->0x157988/ConditionalFalse 0x15794c->0x157990/ConditionalTrue 0x157988->0x157990/Fallthrough 0x157990->0x1579a0/ConditionalFalse 0x157990->0x1579a8/ConditionalTrue 0x1579a0->0x1579a8/Fallthrough 0x1579c0->0x157914/Branch

# top_level.e19Ackermann at 0x1579c8 (288 bytes)
0x1579c8  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1579cc  fd030faa           mov      x29, x15
0x1579d0  ef6100d1           sub      x15, x15, #0x18
0x1579d4  803041b8           ldur     w0, [x4, #0x13]
0x1579d8  010800d1           sub      x1, x0, #2
0x1579dc  a2cb218b           add      x2, x29, w1, sxtw #2
0x1579e0  420840f9           ldr      x2, [x2, #0x10]
0x1579e4  3f080071           cmp      w1, #2
0x1579e8  0b010054           b.lt     #0x157a08
0x1579ec  a0cb218b           add      x0, x29, w1, sxtw #2
0x1579f0  000440f9           ldr      x0, [x0, #8]
0x1579f4  017c4193           sbfx     x1, x0, #1, #0x1f
0x1579f8  40000036           tbz      w0, #0, #0x157a00
0x1579fc  017040f8           ldur     x1, [x0, #7]
0x157a00  e00301aa           mov      x0, x1
0x157a04  02000014           b        #0x157a0c
0x157a08  400080d2           mov      x0, #2
0x157a0c  502740f9           ldr      x16, [x26, #0x48]
0x157a10  ff0110eb           cmp      x15, x16
0x157a14  69060054           b.ls     #0x157ae0
0x157a18  417c4193           sbfx     x1, x2, #1, #0x1f
0x157a1c  42000036           tbz      w2, #0, #0x157a24
0x157a20  417040f8           ldur     x1, [x2, #7]
0x157a24  410100b5           cbnz     x1, #0x157a4c
0x157a28  02040091           add      x2, x0, #1
0x157a2c  40787f93           sbfiz    x0, x2, #1, #0x1f
0x157a30  5f0480eb           cmp      x2, x0, asr #1
0x157a34  60000054           b.eq     #0x157a40
0x157a38  d2a60594           bl       #0x2c1580
0x157a3c  027000f8           stur     x2, [x0, #7]
0x157a40  ef031daa           mov      x15, x29
0x157a44  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x157a48  c0035fd6           ret      
0x157a4c  a00100b5           cbnz     x0, #0x157a80
0x157a50  220400d1           sub      x2, x1, #1
0x157a54  40787f93           sbfiz    x0, x2, #1, #0x1f
0x157a58  5f0480eb           cmp      x2, x0, asr #1
0x157a5c  60000054           b.eq     #0x157a68
0x157a60  c8a60594           bl       #0x2c1580
0x157a64  027000f8           stur     x2, [x0, #7]
0x157a68  e00100f9           str      x0, [x15]
0x157a6c  644741f9           ldr      x4, [x27, #0x288]  # pool[79] = snapshotRef(22)
0x157a70  d6ffff97           bl       #0x1579c8
0x157a74  ef031daa           mov      x15, x29
0x157a78  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x157a7c  c0035fd6           ret      
0x157a80  230400d1           sub      x3, x1, #1
0x157a84  a3831ff8           stur     x3, [x29, #-8]
0x157a88  040400d1           sub      x4, x0, #1
0x157a8c  80787f93           sbfiz    x0, x4, #1, #0x1f
0x157a90  9f0480eb           cmp      x4, x0, asr #1
0x157a94  60000054           b.eq     #0x157aa0
0x157a98  baa60594           bl       #0x2c1580
0x157a9c  047000f8           stur     x4, [x0, #7]
0x157aa0  e00900a9           stp      x0, x2, [x15]
0x157aa4  643b41f9           ldr      x4, [x27, #0x270]  # pool[76] = snapshotRef(23)
0x157aa8  c8ffff97           bl       #0x1579c8
0x157aac  e30300aa           mov      x3, x0
0x157ab0  a2835ff8           ldur     x2, [x29, #-8]
0x157ab4  40787f93           sbfiz    x0, x2, #1, #0x1f
0x157ab8  5f0480eb           cmp      x2, x0, asr #1
0x157abc  60000054           b.eq     #0x157ac8
0x157ac0  b0a60594           bl       #0x2c1580
0x157ac4  027000f8           stur     x2, [x0, #7]
0x157ac8  e30100a9           stp      x3, x0, [x15]
0x157acc  643b41f9           ldr      x4, [x27, #0x270]  # pool[76] = snapshotRef(23)
0x157ad0  beffff97           bl       #0x1579c8
0x157ad4  ef031daa           mov      x15, x29
0x157ad8  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x157adc  c0035fd6           ret      
0x157ae0  48a60594           bl       #0x2c1400
0x157ae4  cdffff17           b        #0x157a18
# CFG: 0x1579c8->0x1579ec/ConditionalFalse 0x1579c8->0x157a08/ConditionalTrue 0x1579ec->0x1579fc/ConditionalFalse 0x1579ec->0x157a00/ConditionalTrue 0x1579fc->0x157a00/Fallthrough 0x157a00->0x157a0c/Branch 0x157a08->0x157a0c/Fallthrough 0x157a0c->0x157a18/ConditionalFalse 0x157a0c->0x157ae0/ConditionalTrue 0x157a18->0x157a20/ConditionalFalse 0x157a18->0x157a24/ConditionalTrue 0x157a20->0x157a24/Fallthrough 0x157a24->0x157a28/ConditionalFalse 0x157a24->0x157a4c/ConditionalTrue 0x157a28->0x157a38/ConditionalFalse 0x157a28->0x157a40/ConditionalTrue 0x157a38->0x157a40/Fallthrough 0x157a4c->0x157a50/ConditionalFalse 0x157a4c->0x157a80/ConditionalTrue 0x157a50->0x157a60/ConditionalFalse 0x157a50->0x157a68/ConditionalTrue 0x157a60->0x157a68/Fallthrough 0x157a80->0x157a98/ConditionalFalse 0x157a80->0x157aa0/ConditionalTrue 0x157a98->0x157aa0/Fallthrough 0x157aa0->0x157ac0/ConditionalFalse 0x157aa0->0x157ac8/ConditionalTrue 0x157ac0->0x157ac8/Fallthrough 0x157ae0->0x157a18/Branch

# top_level.e19Ackermann at 0x157ae8 (132 bytes)
0x157ae8  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x157aec  fd030faa           mov      x29, x15
0x157af0  ef4100d1           sub      x15, x15, #0x10
0x157af4  803041b8           ldur     w0, [x4, #0x13]
0x157af8  011000d1           sub      x1, x0, #4
0x157afc  a2cb218b           add      x2, x29, w1, sxtw #2
0x157b00  420840f9           ldr      x2, [x2, #0x10]
0x157b04  3f080071           cmp      w1, #2
0x157b08  0b010054           b.lt     #0x157b28
0x157b0c  a0cb218b           add      x0, x29, w1, sxtw #2
0x157b10  000440f9           ldr      x0, [x0, #8]
0x157b14  017c4193           sbfx     x1, x0, #1, #0x1f
0x157b18  40000036           tbz      w0, #0, #0x157b20
0x157b1c  017040f8           ldur     x1, [x0, #7]
0x157b20  e30301aa           mov      x3, x1
0x157b24  02000014           b        #0x157b2c
0x157b28  430080d2           mov      x3, #2
0x157b2c  502740f9           ldr      x16, [x26, #0x48]
0x157b30  ff0110eb           cmp      x15, x16
0x157b34  89010054           b.ls     #0x157b64
0x157b38  60787f93           sbfiz    x0, x3, #1, #0x1f
0x157b3c  7f0480eb           cmp      x3, x0, asr #1
0x157b40  60000054           b.eq     #0x157b4c
0x157b44  8fa60594           bl       #0x2c1580
0x157b48  037000f8           stur     x3, [x0, #7]
0x157b4c  e00900a9           stp      x0, x2, [x15]
0x157b50  643b41f9           ldr      x4, [x27, #0x270]  # pool[76] = snapshotRef(23)
0x157b54  9dffff97           bl       #0x1579c8
0x157b58  ef031daa           mov      x15, x29
0x157b5c  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x157b60  c0035fd6           ret      
0x157b64  27a60594           bl       #0x2c1400
0x157b68  f4ffff17           b        #0x157b38
# CFG: 0x157ae8->0x157b0c/ConditionalFalse 0x157ae8->0x157b28/ConditionalTrue 0x157b0c->0x157b1c/ConditionalFalse 0x157b0c->0x157b20/ConditionalTrue 0x157b1c->0x157b20/Fallthrough 0x157b20->0x157b2c/Branch 0x157b28->0x157b2c/Fallthrough 0x157b2c->0x157b38/ConditionalFalse 0x157b2c->0x157b64/ConditionalTrue 0x157b38->0x157b44/ConditionalFalse 0x157b38->0x157b4c/ConditionalTrue 0x157b44->0x157b4c/Fallthrough 0x157b64->0x157b38/Branch

# ProbeApp.build at 0x1ea76c (220 bytes)
0x1ea76c  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1ea770  fd030faa           mov      x29, x15
0x1ea774  ef4100d1           sub      x15, x15, #0x10
0x1ea778  e30301aa           mov      x3, x1
0x1ea77c  e00302aa           mov      x0, x2
0x1ea780  61234091           add      x1, x27, #8, lsl #12
0x1ea784  210c44f9           ldr      x1, [x1, #0x818]  # pool[4353] = ProbeApp.<anonymous closure>
0x1ea788  e20316aa           mov      x2, x22
0x1ea78c  93570394           bl       #0x2c05d8
0x1ea790  a0831ff8           stur     x0, [x29, #-8]
0x1ea794  6685ff97           bl       #0x1cbd2c
0x1ea798  e10300aa           mov      x1, x0
0x1ea79c  a0835ff8           ldur     x0, [x29, #-8]
0x1ea7a0  a1031ff8           stur     x1, [x29, #-0x10]
0x1ea7a4  20b000b8           stur     w0, [x1, #0xb]
0x1ea7a8  2b000094           bl       #0x1ea854
0x1ea7ac  e10300aa           mov      x1, x0
0x1ea7b0  a0035ff8           ldur     x0, [x29, #-0x10]
0x1ea7b4  a1831ff8           stur     x1, [x29, #-8]
0x1ea7b8  207001b8           stur     w0, [x1, #0x17]
0x1ea7bc  c0820091           add      x0, x22, #0x20
0x1ea7c0  203004b8           stur     w0, [x1, #0x43]
0x1ea7c4  c2c20091           add      x2, x22, #0x30
0x1ea7c8  22b000b8           stur     w2, [x1, #0xb]
0x1ea7cc  22f000b8           stur     w2, [x1, #0xf]
0x1ea7d0  1e000094           bl       #0x1ea848
0x1ea7d4  a1835ff8           ldur     x1, [x29, #-8]
0x1ea7d8  013001b8           stur     w1, [x0, #0x13]
0x1ea7dc  61234091           add      x1, x27, #8, lsl #12
0x1ea7e0  211044f9           ldr      x1, [x1, #0x820]  # pool[4354] = snapshotRef(34490)
0x1ea7e4  017001b8           stur     w1, [x0, #0x17]
0x1ea7e8  61234091           add      x1, x27, #8, lsl #12
0x1ea7ec  211444f9           ldr      x1, [x1, #0x828]  # pool[4355] = snapshotRef(34645)
0x1ea7f0  01f002b8           stur     w1, [x0, #0x2f]
0x1ea7f4  61234091           add      x1, x27, #8, lsl #12
0x1ea7f8  211844f9           ldr      x1, [x1, #0x830]  # pool[4356] = "clutter edge-case probe"
0x1ea7fc  01f003b8           stur     w1, [x0, #0x3f]
0x1ea800  61234091           add      x1, x27, #8, lsl #12
0x1ea804  211c44f9           ldr      x1, [x1, #0x838]  # pool[4357] = snapshotInstance(ThemeMode)
0x1ea808  017005b8           stur     w1, [x0, #0x57]
0x1ea80c  61c355f9           ldr      x1, [x27, #0x2b80]  # pool[1390] = snapshotInstance(Duration)
0x1ea810  01b005b8           stur     w1, [x0, #0x5b]
0x1ea814  614354f9           ldr      x1, [x27, #0x2880]  # pool[1294] = snapshotInstance(_Linear)
0x1ea818  01f005b8           stur     w1, [x0, #0x5f]
0x1ea81c  61234091           add      x1, x27, #8, lsl #12
0x1ea820  212044f9           ldr      x1, [x1, #0x840]  # pool[4358] = snapshotRef(34582) nestedStrings["US", "en"]
0x1ea824  017007b8           stur     w1, [x0, #0x77]
0x1ea828  c1c20091           add      x1, x22, #0x30
0x1ea82c  01b007b8           stur     w1, [x0, #0x7b]
0x1ea830  01f007b8           stur     w1, [x0, #0x7f]
0x1ea834  c1820091           add      x1, x22, #0x20
0x1ea838  013008b8           stur     w1, [x0, #0x83]
0x1ea83c  ef031daa           mov      x15, x29
0x1ea840  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1ea844  c0035fd6           ret      

# ProbeApp.<anonymous closure> at 0x1ea860 (4160 bytes)
0x1ea860  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1ea864  fd030faa           mov      x29, x15
0x1ea868  efe101d1           sub      x15, x15, #0x78
0x1ea86c  a00f40f9           ldr      x0, [x29, #0x18]
0x1ea870  017041b8           ldur     w1, [x0, #0x17]
0x1ea874  21801c8b           add      x1, x1, x28, lsl #32
0x1ea878  a1831ff8           stur     x1, [x29, #-8]
0x1ea87c  502740f9           ldr      x16, [x26, #0x48]
0x1ea880  ff0110eb           cmp      x15, x16
0x1ea884  e97d0054           b.ls     #0x1eb840
0x1ea888  410080d2           mov      x1, #2
0x1ea88c  5e560394           bl       #0x2c0204
0x1ea890  e10300aa           mov      x1, x0
0x1ea894  a0835ff8           ldur     x0, [x29, #-8]
0x1ea898  a1031ff8           stur     x1, [x29, #-0x10]
0x1ea89c  20b000b8           stur     w0, [x1, #0xb]
0x1ea8a0  6a170094           bl       #0x1f0648
0x1ea8a4  e20300aa           mov      x2, x0
0x1ea8a8  a2831ef8           stur     x2, [x29, #-0x18]
0x1ea8ac  40787f93           sbfiz    x0, x2, #1, #0x1f
0x1ea8b0  5f0480eb           cmp      x2, x0, asr #1
0x1ea8b4  60000054           b.eq     #0x1ea8c0
0x1ea8b8  325b0394           bl       #0x2c1580
0x1ea8bc  027000f8           stur     x2, [x0, #7]
0x1ea8c0  e30300aa           mov      x3, x0
0x1ea8c4  a1035ff8           ldur     x1, [x29, #-0x10]
0x1ea8c8  a3831ff8           stur     x3, [x29, #-8]
0x1ea8cc  20f000b8           stur     w0, [x1, #0xf]
0x1ea8d0  e0000036           tbz      w0, #0, #0x1ea8ec
0x1ea8d4  30f05f38           ldurb    w16, [x1, #-1]
0x1ea8d8  11f05f38           ldurb    w17, [x0, #-1]
0x1ea8dc  300a508a           and      x16, x17, x16, lsr #2
0x1ea8e0  1f825cea           tst      x16, x28, lsr #32
0x1ea8e4  40000054           b.eq     #0x1ea8ec
0x1ea8e8  1e540394           bl       #0x2bf960
0x1ea8ec  52b9fd97           bl       #0x158e34
0x1ea8f0  e30300aa           mov      x3, x0
0x1ea8f4  00106e1e           fmov     d0, #1.00000000
0x1ea8f8  a3831df8           stur     x3, [x29, #-0x28]
0x1ea8fc  607000fc           stur     d0, [x3, #7]
0x1ea900  e00303aa           mov      x0, x3
0x1ea904  a4035ff8           ldur     x4, [x29, #-0x10]
0x1ea908  803001b8           stur     w0, [x4, #0x13]
0x1ea90c  90f05f38           ldurb    w16, [x4, #-1]
0x1ea910  11f05f38           ldurb    w17, [x0, #-1]
0x1ea914  300a508a           and      x16, x17, x16, lsr #2
0x1ea918  1f825cea           tst      x16, x28, lsr #32
0x1ea91c  40000054           b.eq     #0x1ea924
0x1ea920  28540394           bl       #0x2bf9c0
0x1ea924  a0835ef8           ldur     x0, [x29, #-0x18]
0x1ea928  8000f8b6           tbz      x0, #0x3f, #0x1ea938
0x1ea92c  e10300cb           neg      x1, x0
0x1ea930  e50301aa           mov      x5, x1
0x1ea934  02000014           b        #0x1ea93c
0x1ea938  e50300aa           mov      x5, x0
0x1ea93c  600080d2           mov      x0, #3
0x1ea940  a5031ef8           stur     x5, [x29, #-0x20]
0x1ea944  a10cc09a           sdiv     x1, x5, x0
0x1ea948  2694009b           msub     x6, x1, x0, x5
0x1ea94c  df001feb           cmp      x6, xzr
0x1ea950  cb770054           b.lt     #0x1eb848
0x1ea954  a6831ef8           stur     x6, [x29, #-0x18]
0x1ea958  c2840191           add      x2, x6, #0x61
0x1ea95c  e10316aa           mov      x1, x22
0x1ea960  3449fd97           bl       #0x13ce30
0x1ea964  e10316aa           mov      x1, x22
0x1ea968  820080d2           mov      x2, #4
0x1ea96c  a0031df8           stur     x0, [x29, #-0x30]
0x1ea970  615a0394           bl       #0x2c12f4
0x1ea974  70234091           add      x16, x27, #8, lsl #12
0x1ea978  102644f9           ldr      x16, [x16, #0x848]  # pool[4359] = snapshotRef(870)
0x1ea97c  10f000b8           stur     w16, [x0, #0xf]
0x1ea980  a1035ef8           ldur     x1, [x29, #-0x20]
0x1ea984  81000037           tbnz     w1, #0, #0x1ea994
0x1ea988  63234091           add      x3, x27, #8, lsl #12
0x1ea98c  632844f9           ldr      x3, [x3, #0x850]  # pool[4360] = "v v"
0x1ea990  02000014           b        #0x1ea998
0x1ea994  e30316aa           mov      x3, x22
0x1ea998  a2835df8           ldur     x2, [x29, #-0x28]
0x1ea99c  033001b8           stur     w3, [x0, #0x13]
0x1ea9a0  70234091           add      x16, x27, #8, lsl #12
0x1ea9a4  102e44f9           ldr      x16, [x16, #0x858]  # pool[4361] = snapshotRef(17932)
0x1ea9a8  e04100a9           stp      x0, x16, [x15]
0x1ea9ac  c705fd97           bl       #0x12c0c8
0x1ea9b0  e10316aa           mov      x1, x22
0x1ea9b4  820080d2           mov      x2, #4
0x1ea9b8  a0831cf8           stur     x0, [x29, #-0x38]
0x1ea9bc  4e5a0394           bl       #0x2c12f4
0x1ea9c0  e20300aa           mov      x2, x0
0x1ea9c4  70234091           add      x16, x27, #8, lsl #12
0x1ea9c8  103244f9           ldr      x16, [x16, #0x860]  # pool[4362] = snapshotRef(260)
0x1ea9cc  50f000b8           stur     w16, [x2, #0xf]
0x1ea9d0  a3035ef8           ldur     x3, [x29, #-0x20]
0x1ea9d4  60787f93           sbfiz    x0, x3, #1, #0x1f
0x1ea9d8  7f0480eb           cmp      x3, x0, asr #1
0x1ea9dc  60000054           b.eq     #0x1ea9e8
0x1ea9e0  e85a0394           bl       #0x2c1580
0x1ea9e4  037000f8           stur     x3, [x0, #7]
0x1ea9e8  a0031cf8           stur     x0, [x29, #-0x40]
0x1ea9ec  403001b8           stur     w0, [x2, #0x13]
0x1ea9f0  e20100f9           str      x2, [x15]
0x1ea9f4  e217fd97           bl       #0x13097c
0x1ea9f8  e20300aa           mov      x2, x0
0x1ea9fc  a0035ef8           ldur     x0, [x29, #-0x20]
0x1eaa00  210c80d2           mov      x1, #0x61
0x1eaa04  040cc19a           sdiv     x4, x0, x1
0x1eaa08  8380019b           msub     x3, x4, x1, x0
0x1eaa0c  7f001feb           cmp      x3, xzr
0x1eaa10  0b720054           b.lt     #0x1eb850
0x1eaa14  810c80d2           mov      x1, #0x64
0x1eaa18  050cc19a           sdiv     x5, x0, x1
0x1eaa1c  a480019b           msub     x4, x5, x1, x0
0x1eaa20  9f001feb           cmp      x4, xzr
0x1eaa24  ab710054           b.lt     #0x1eb858
0x1eaa28  8000629e           scvtf    d0, x4
0x1eaa2c  615f5afd           ldr      d1, [x27, #0x34b8]  # pool[1685] = 4636737291354636288
0x1eaa30  0218611e           fdiv     d2, d0, d1
0x1eaa34  e10302aa           mov      x1, x2
0x1eaa38  e20303aa           mov      x2, x3
0x1eaa3c  401ca24e           mov      v0.16b, v2.16b
0x1eaa40  4a160094           bl       #0x1f0368
0x1eaa44  a0831bf8           stur     x0, [x29, #-0x48]
0x1eaa48  45160094           bl       #0x1f035c
0x1eaa4c  e30300aa           mov      x3, x0
0x1eaa50  a0835bf8           ldur     x0, [x29, #-0x48]
0x1eaa54  a3031bf8           stur     x3, [x29, #-0x50]
0x1eaa58  60b000b8           stur     w0, [x3, #0xb]
0x1eaa5c  a0835df8           ldur     x0, [x29, #-0x28]
0x1eaa60  603003b8           stur     w0, [x3, #0x33]
0x1eaa64  61234091           add      x1, x27, #8, lsl #12
0x1eaa68  213444f9           ldr      x1, [x1, #0x868]  # pool[4363] = snapshotRef(18423)
0x1eaa6c  420680d2           mov      x2, #0x32
0x1eaa70  215a0394           bl       #0x2c12f4
0x1eaa74  e10300aa           mov      x1, x0
0x1eaa78  a0035bf8           ldur     x0, [x29, #-0x50]
0x1eaa7c  a1831bf8           stur     x1, [x29, #-0x48]
0x1eaa80  20f000b8           stur     w0, [x1, #0xf]
0x1eaa84  b0035cf8           ldur     x16, [x29, #-0x40]
0x1eaa88  f00100f9           str      x16, [x15]
0x1eaa8c  8f18fd97           bl       #0x130cc8
0x1eaa90  e10316aa           mov      x1, x22
0x1eaa94  820080d2           mov      x2, #4
0x1eaa98  a0031bf8           stur     x0, [x29, #-0x50]
0x1eaa9c  165a0394           bl       #0x2c12f4
0x1eaaa0  e20300aa           mov      x2, x0
0x1eaaa4  a0035bf8           ldur     x0, [x29, #-0x50]
0x1eaaa8  a2831af8           stur     x2, [x29, #-0x58]
0x1eaaac  40f000b8           stur     w0, [x2, #0xf]
0x1eaab0  70234091           add      x16, x27, #8, lsl #12
0x1eaab4  103a44f9           ldr      x16, [x16, #0x870]  # pool[4364] = snapshotRef(458)
0x1eaab8  503001b8           stur     w16, [x2, #0x13]
0x1eaabc  61b743f9           ldr      x1, [x27, #0x768]  # pool[235] = snapshotRef(18312)
0x1eaac0  c1550394           bl       #0x2c01c4
0x1eaac4  e10300aa           mov      x1, x0
0x1eaac8  a0835af8           ldur     x0, [x29, #-0x58]
0x1eaacc  20f000b8           stur     w0, [x1, #0xf]
0x1eaad0  820080d2           mov      x2, #4
0x1eaad4  22b000b8           stur     w2, [x1, #0xb]
0x1eaad8  f3150094           bl       #0x1f02a4
0x1eaadc  e00100f9           str      x0, [x15]
0x1eaae0  7a18fd97           bl       #0x130cc8
0x1eaae4  a0031bf8           stur     x0, [x29, #-0x50]
0x1eaae8  1d160094           bl       #0x1f035c
0x1eaaec  e10300aa           mov      x1, x0
0x1eaaf0  a0035bf8           ldur     x0, [x29, #-0x50]
0x1eaaf4  20b000b8           stur     w0, [x1, #0xb]
0x1eaaf8  a2835df8           ldur     x2, [x29, #-0x28]
0x1eaafc  223003b8           stur     w2, [x1, #0x33]
0x1eab00  e00301aa           mov      x0, x1
0x1eab04  a1835bf8           ldur     x1, [x29, #-0x48]
0x1eab08  394c0091           add      x25, x1, #0x13
0x1eab0c  200300b9           str      w0, [x25]
0x1eab10  e0000036           tbz      w0, #0, #0x1eab2c
0x1eab14  30f05f38           ldurb    w16, [x1, #-1]
0x1eab18  11f05f38           ldurb    w17, [x0, #-1]
0x1eab1c  300a508a           and      x16, x17, x16, lsr #2
0x1eab20  1f825cea           tst      x16, x28, lsr #32
0x1eab24  40000054           b.eq     #0x1eab2c
0x1eab28  7d520394           bl       #0x2bf51c
0x1eab2c  70234091           add      x16, x27, #8, lsl #12
0x1eab30  103a44f9           ldr      x16, [x16, #0x870]  # pool[4364] = snapshotRef(458)
0x1eab34  be035df8           ldur     x30, [x29, #-0x30]
0x1eab38  fe4100a9           stp      x30, x16, [x15]
0x1eab3c  68240294           bl       #0x273cdc
0x1eab40  80002037           tbnz     w0, #4, #0x1eab50
0x1eab44  63234091           add      x3, x27, #8, lsl #12
0x1eab48  633c44f9           ldr      x3, [x3, #0x878]  # pool[4365] = "alpha"
0x1eab4c  11000014           b        #0x1eab90
0x1eab50  70234091           add      x16, x27, #8, lsl #12
0x1eab54  104244f9           ldr      x16, [x16, #0x880]  # pool[4366] = snapshotRef(734)
0x1eab58  be035df8           ldur     x30, [x29, #-0x30]
0x1eab5c  fe4100a9           stp      x30, x16, [x15]
0x1eab60  5f240294           bl       #0x273cdc
0x1eab64  c0002036           tbz      w0, #4, #0x1eab7c
0x1eab68  703b70f9           ldr      x16, [x27, #0x6070]  # pool[3084] = snapshotRef(343)
0x1eab6c  be035df8           ldur     x30, [x29, #-0x30]
0x1eab70  fe4100a9           stp      x30, x16, [x15]
0x1eab74  5a240294           bl       #0x273cdc
0x1eab78  80002037           tbnz     w0, #4, #0x1eab88
0x1eab7c  63234091           add      x3, x27, #8, lsl #12
0x1eab80  634444f9           ldr      x3, [x3, #0x888]  # pool[4367] = "beta-or-gamma"
0x1eab84  03000014           b        #0x1eab90
0x1eab88  63234091           add      x3, x27, #8, lsl #12
0x1eab8c  634844f9           ldr      x3, [x3, #0x890]  # pool[4368] = "other"
0x1eab90  a0835df8           ldur     x0, [x29, #-0x28]
0x1eab94  a1035ef8           ldur     x1, [x29, #-0x20]
0x1eab98  a2835ff8           ldur     x2, [x29, #-8]
0x1eab9c  a3031bf8           stur     x3, [x29, #-0x50]
0x1eaba0  ef150094           bl       #0x1f035c
0x1eaba4  e10300aa           mov      x1, x0
0x1eaba8  a0035bf8           ldur     x0, [x29, #-0x50]
0x1eabac  20b000b8           stur     w0, [x1, #0xb]
0x1eabb0  a2835df8           ldur     x2, [x29, #-0x28]
0x1eabb4  223003b8           stur     w2, [x1, #0x33]
0x1eabb8  e00301aa           mov      x0, x1
0x1eabbc  a1835bf8           ldur     x1, [x29, #-0x48]
0x1eabc0  395c0091           add      x25, x1, #0x17
0x1eabc4  200300b9           str      w0, [x25]
0x1eabc8  e0000036           tbz      w0, #0, #0x1eabe4
0x1eabcc  30f05f38           ldurb    w16, [x1, #-1]
0x1eabd0  11f05f38           ldurb    w17, [x0, #-1]
0x1eabd4  300a508a           and      x16, x17, x16, lsr #2
0x1eabd8  1f825cea           tst      x16, x28, lsr #32
0x1eabdc  40000054           b.eq     #0x1eabe4
0x1eabe0  4f520394           bl       #0x2bf51c
0x1eabe4  a1035ef8           ldur     x1, [x29, #-0x20]
0x1eabe8  95150094           bl       #0x1f023c
0x1eabec  e20300aa           mov      x2, x0
0x1eabf0  40787f93           sbfiz    x0, x2, #1, #0x1f
0x1eabf4  5f0480eb           cmp      x2, x0, asr #1
0x1eabf8  60000054           b.eq     #0x1eac04
0x1eabfc  615a0394           bl       #0x2c1580
0x1eac00  027000f8           stur     x2, [x0, #7]
0x1eac04  e00100f9           str      x0, [x15]
0x1eac08  3018fd97           bl       #0x130cc8
0x1eac0c  a0031bf8           stur     x0, [x29, #-0x50]
0x1eac10  d3150094           bl       #0x1f035c
0x1eac14  e10300aa           mov      x1, x0
0x1eac18  a0035bf8           ldur     x0, [x29, #-0x50]
0x1eac1c  20b000b8           stur     w0, [x1, #0xb]
0x1eac20  a2835df8           ldur     x2, [x29, #-0x28]
0x1eac24  223003b8           stur     w2, [x1, #0x33]
0x1eac28  e00301aa           mov      x0, x1
0x1eac2c  a1835bf8           ldur     x1, [x29, #-0x48]
0x1eac30  396c0091           add      x25, x1, #0x1b
0x1eac34  200300b9           str      w0, [x25]
0x1eac38  e0000036           tbz      w0, #0, #0x1eac54
0x1eac3c  30f05f38           ldurb    w16, [x1, #-1]
0x1eac40  11f05f38           ldurb    w17, [x0, #-1]
0x1eac44  300a508a           and      x16, x17, x16, lsr #2
0x1eac48  1f825cea           tst      x16, x28, lsr #32
0x1eac4c  40000054           b.eq     #0x1eac54
0x1eac50  33520394           bl       #0x2bf51c
0x1eac54  a1835cf8           ldur     x1, [x29, #-0x38]
0x1eac58  1f150094           bl       #0x1f00d4
0x1eac5c  e00100f9           str      x0, [x15]
0x1eac60  1a18fd97           bl       #0x130cc8
0x1eac64  a0831cf8           stur     x0, [x29, #-0x38]
0x1eac68  bd150094           bl       #0x1f035c
0x1eac6c  e10300aa           mov      x1, x0
0x1eac70  a0835cf8           ldur     x0, [x29, #-0x38]
0x1eac74  20b000b8           stur     w0, [x1, #0xb]
0x1eac78  a3835df8           ldur     x3, [x29, #-0x28]
0x1eac7c  233003b8           stur     w3, [x1, #0x33]
0x1eac80  e00301aa           mov      x0, x1
0x1eac84  a1835bf8           ldur     x1, [x29, #-0x48]
0x1eac88  397c0091           add      x25, x1, #0x1f
0x1eac8c  200300b9           str      w0, [x25]
0x1eac90  e0000036           tbz      w0, #0, #0x1eacac
0x1eac94  30f05f38           ldurb    w16, [x1, #-1]
0x1eac98  11f05f38           ldurb    w17, [x0, #-1]
0x1eac9c  300a508a           and      x16, x17, x16, lsr #2
0x1eaca0  1f825cea           tst      x16, x28, lsr #32
0x1eaca4  40000054           b.eq     #0x1eacac
0x1eaca8  1d520394           bl       #0x2bf51c
0x1eacac  a0035ef8           ldur     x0, [x29, #-0x20]
0x1eacb0  210180d2           mov      x1, #9
0x1eacb4  020cc19a           sdiv     x2, x0, x1
0x1eacb8  4480019b           msub     x4, x2, x1, x0
0x1eacbc  9f001feb           cmp      x4, xzr
0x1eacc0  0b5d0054           b.lt     #0x1eb860
0x1eacc4  a48319f8           stur     x4, [x29, #-0x68]
0x1eacc8  e10080d2           mov      x1, #7
0x1eaccc  020cc19a           sdiv     x2, x0, x1
0x1eacd0  4580019b           msub     x5, x2, x1, x0
0x1eacd4  bf001feb           cmp      x5, xzr
0x1eacd8  8b5c0054           b.lt     #0x1eb868
0x1eacdc  a5031af8           stur     x5, [x29, #-0x60]
0x1eace0  e10316aa           mov      x1, x22
0x1eace4  820080d2           mov      x2, #4
0x1eace8  83590394           bl       #0x2c12f4
0x1eacec  70234091           add      x16, x27, #8, lsl #12
0x1eacf0  104e44f9           ldr      x16, [x16, #0x898]  # pool[4369] = snapshotRef(610)
0x1eacf4  10f000b8           stur     w16, [x0, #0xf]
0x1eacf8  a1835ff8           ldur     x1, [x29, #-8]
0x1eacfc  013001b8           stur     w1, [x0, #0x13]
0x1ead00  e00100f9           str      x0, [x15]
0x1ead04  1e17fd97           bl       #0x13097c
0x1ead08  e10300aa           mov      x1, x0
0x1ead0c  a08359f8           ldur     x0, [x29, #-0x68]
0x1ead10  02f87fd3           lsl      x2, x0, #1
0x1ead14  a0035af8           ldur     x0, [x29, #-0x60]
0x1ead18  03f87fd3           lsl      x3, x0, #1
0x1ead1c  e40301aa           mov      x4, x1
0x1ead20  c10080d2           mov      x1, #6
0x1ead24  4100a0f2           movk     x1, #2, lsl #16
0x1ead28  f6530394           bl       #0x2bfd00
0x1ead2c  e10300aa           mov      x1, x0
0x1ead30  6f140094           bl       #0x1efeec
0x1ead34  e20300aa           mov      x2, x0
0x1ead38  40787f93           sbfiz    x0, x2, #1, #0x1f
0x1ead3c  5f0480eb           cmp      x2, x0, asr #1
0x1ead40  60000054           b.eq     #0x1ead4c
0x1ead44  0f5a0394           bl       #0x2c1580
0x1ead48  027000f8           stur     x2, [x0, #7]
0x1ead4c  e00100f9           str      x0, [x15]
0x1ead50  de17fd97           bl       #0x130cc8
0x1ead54  a0831cf8           stur     x0, [x29, #-0x38]
0x1ead58  81150094           bl       #0x1f035c
0x1ead5c  e10300aa           mov      x1, x0
0x1ead60  a0835cf8           ldur     x0, [x29, #-0x38]
0x1ead64  20b000b8           stur     w0, [x1, #0xb]
0x1ead68  a3835df8           ldur     x3, [x29, #-0x28]
0x1ead6c  233003b8           stur     w3, [x1, #0x33]
0x1ead70  e00301aa           mov      x0, x1
0x1ead74  a1835bf8           ldur     x1, [x29, #-0x48]
0x1ead78  398c0091           add      x25, x1, #0x23
0x1ead7c  200300b9           str      w0, [x25]
0x1ead80  e0000036           tbz      w0, #0, #0x1ead9c
0x1ead84  30f05f38           ldurb    w16, [x1, #-1]
0x1ead88  11f05f38           ldurb    w17, [x0, #-1]
0x1ead8c  300a508a           and      x16, x17, x16, lsr #2
0x1ead90  1f825cea           tst      x16, x28, lsr #32
0x1ead94  40000054           b.eq     #0x1ead9c
0x1ead98  e1510394           bl       #0x2bf51c
0x1ead9c  e10316aa           mov      x1, x22
0x1eada0  820080d2           mov      x2, #4
0x1eada4  54590394           bl       #0x2c12f4
0x1eada8  70234091           add      x16, x27, #8, lsl #12
0x1eadac  105244f9           ldr      x16, [x16, #0x8a0]  # pool[4370] = snapshotRef(295)
0x1eadb0  10f000b8           stur     w16, [x0, #0xf]
0x1eadb4  a1035ef8           ldur     x1, [x29, #-0x20]
0x1eadb8  620180d2           mov      x2, #0xb
0x1eadbc  240cc29a           sdiv     x4, x1, x2
0x1eadc0  8384029b           msub     x3, x4, x2, x1
0x1eadc4  7f001feb           cmp      x3, xzr
0x1eadc8  4b550054           b.lt     #0x1eb870
0x1eadcc  62f87fd3           lsl      x2, x3, #1
0x1eadd0  023001b8           stur     w2, [x0, #0x13]
0x1eadd4  70db51f9           ldr      x16, [x27, #0x23b0]  # pool[1140] = snapshotRef(17985)
0x1eadd8  e04100a9           stp      x0, x16, [x15]
0x1eaddc  bb04fd97           bl       #0x12c0c8
0x1eade0  70db5bf9           ldr      x16, [x27, #0x37b0]  # pool[1780] = snapshotRef(17903)
0x1eade4  e04100a9           stp      x0, x16, [x15]
0x1eade8  646f4ef9           ldr      x4, [x27, #0x1cd8]  # pool[921] = snapshotRef(34545)
0x1eadec  f8130094           bl       #0x1efdcc
0x1eadf0  e00100f9           str      x0, [x15]
0x1eadf4  b517fd97           bl       #0x130cc8
0x1eadf8  a0831cf8           stur     x0, [x29, #-0x38]
0x1eadfc  58150094           bl       #0x1f035c
0x1eae00  e10300aa           mov      x1, x0
0x1eae04  a0835cf8           ldur     x0, [x29, #-0x38]
0x1eae08  20b000b8           stur     w0, [x1, #0xb]
0x1eae0c  a2835df8           ldur     x2, [x29, #-0x28]
0x1eae10  223003b8           stur     w2, [x1, #0x33]
0x1eae14  e00301aa           mov      x0, x1
0x1eae18  a1835bf8           ldur     x1, [x29, #-0x48]
0x1eae1c  399c0091           add      x25, x1, #0x27
0x1eae20  200300b9           str      w0, [x25]
0x1eae24  e0000036           tbz      w0, #0, #0x1eae40
0x1eae28  30f05f38           ldurb    w16, [x1, #-1]
0x1eae2c  11f05f38           ldurb    w17, [x0, #-1]
0x1eae30  300a508a           and      x16, x17, x16, lsr #2
0x1eae34  1f825cea           tst      x16, x28, lsr #32
0x1eae38  40000054           b.eq     #0x1eae40
0x1eae3c  b8510394           bl       #0x2bf51c
0x1eae40  a3035ef8           ldur     x3, [x29, #-0x20]
0x1eae44  a00080d2           mov      x0, #5
0x1eae48  610cc09a           sdiv     x1, x3, x0
0x1eae4c  248c009b           msub     x4, x1, x0, x3
0x1eae50  9f001feb           cmp      x4, xzr
0x1eae54  2b510054           b.lt     #0x1eb878
0x1eae58  a4031af8           stur     x4, [x29, #-0x60]
0x1eae5c  010080d2           mov      x1, #0
0x1eae60  000080d2           mov      x0, #0
0x1eae64  502740f9           ldr      x16, [x26, #0x48]
0x1eae68  ff0110eb           cmp      x15, x16
0x1eae6c  a9500054           b.ls     #0x1eb880
0x1eae70  1f0004eb           cmp      x0, x4
0x1eae74  2a030054           b.ge     #0x1eaed8
0x1eae78  e50301aa           mov      x5, x1
0x1eae7c  010080d2           mov      x1, #0
0x1eae80  502740f9           ldr      x16, [x26, #0x48]
0x1eae84  ff0110eb           cmp      x15, x16
0x1eae88  09500054           b.ls     #0x1eb888
0x1eae8c  3f0004eb           cmp      x1, x4
0x1eae90  ca010054           b.ge     #0x1eaec8
0x1eae94  067c019b           mul      x6, x0, x1
0x1eae98  df1800f1           cmp      x6, #6
0x1eae9c  2c010054           b.gt     #0x1eaec0
0x1eaea0  0600018b           add      x6, x0, x1
0x1eaea4  df0004eb           cmp      x6, x4
0x1eaea8  a0010054           b.eq     #0x1eaedc
0x1eaeac  a6040091           add      x6, x5, #1
0x1eaeb0  27040091           add      x7, x1, #1
0x1eaeb4  e50306aa           mov      x5, x6
0x1eaeb8  e10307aa           mov      x1, x7
0x1eaebc  f1ffff17           b        #0x1eae80
0x1eaec0  e10305aa           mov      x1, x5
0x1eaec4  02000014           b        #0x1eaecc
0x1eaec8  a1900191           add      x1, x5, #0x64
0x1eaecc  05040091           add      x5, x0, #1
0x1eaed0  e00305aa           mov      x0, x5
0x1eaed4  e4ffff17           b        #0x1eae64
0x1eaed8  e50301aa           mov      x5, x1
0x1eaedc  a0787f93           sbfiz    x0, x5, #1, #0x1f
0x1eaee0  bf0480eb           cmp      x5, x0, asr #1
0x1eaee4  60000054           b.eq     #0x1eaef0
0x1eaee8  a6590394           bl       #0x2c1580
0x1eaeec  057000f8           stur     x5, [x0, #7]
0x1eaef0  e00100f9           str      x0, [x15]
0x1eaef4  7517fd97           bl       #0x130cc8
0x1eaef8  a0831cf8           stur     x0, [x29, #-0x38]
0x1eaefc  18150094           bl       #0x1f035c
0x1eaf00  e10300aa           mov      x1, x0
0x1eaf04  a0835cf8           ldur     x0, [x29, #-0x38]
0x1eaf08  20b000b8           stur     w0, [x1, #0xb]
0x1eaf0c  a2835df8           ldur     x2, [x29, #-0x28]
0x1eaf10  223003b8           stur     w2, [x1, #0x33]
0x1eaf14  e00301aa           mov      x0, x1
0x1eaf18  a1835bf8           ldur     x1, [x29, #-0x48]
0x1eaf1c  39ac0091           add      x25, x1, #0x2b
0x1eaf20  200300b9           str      w0, [x25]
0x1eaf24  e0000036           tbz      w0, #0, #0x1eaf40
0x1eaf28  30f05f38           ldurb    w16, [x1, #-1]
0x1eaf2c  11f05f38           ldurb    w17, [x0, #-1]
0x1eaf30  300a508a           and      x16, x17, x16, lsr #2
0x1eaf34  1f825cea           tst      x16, x28, lsr #32
0x1eaf38  40000054           b.eq     #0x1eaf40
0x1eaf3c  78510394           bl       #0x2bf51c
0x1eaf40  a0035ef8           ldur     x0, [x29, #-0x20]
0x1eaf44  c0000037           tbnz     w0, #0, #0x1eaf5c
0x1eaf48  a1035cf8           ldur     x1, [x29, #-0x40]
0x1eaf4c  f00300aa           mov      x16, x0
0x1eaf50  e00302aa           mov      x0, x2
0x1eaf54  e20310aa           mov      x2, x16
0x1eaf58  07000014           b        #0x1eaf74
0x1eaf5c  b0035cf8           ldur     x16, [x29, #-0x40]
0x1eaf60  f00100f9           str      x16, [x15]
0x1eaf64  5917fd97           bl       #0x130cc8
0x1eaf68  e10300aa           mov      x1, x0
0x1eaf6c  a0835df8           ldur     x0, [x29, #-0x28]
0x1eaf70  a2035ef8           ldur     x2, [x29, #-0x20]
0x1eaf74  a5835ef8           ldur     x5, [x29, #-0x18]
0x1eaf78  a3035af8           ldur     x3, [x29, #-0x60]
0x1eaf7c  a4035cf8           ldur     x4, [x29, #-0x40]
0x1eaf80  20130094           bl       #0x1efc00
0x1eaf84  a0831cf8           stur     x0, [x29, #-0x38]
0x1eaf88  f5140094           bl       #0x1f035c
0x1eaf8c  e10300aa           mov      x1, x0
0x1eaf90  a0835cf8           ldur     x0, [x29, #-0x38]
0x1eaf94  20b000b8           stur     w0, [x1, #0xb]
0x1eaf98  a2835df8           ldur     x2, [x29, #-0x28]
0x1eaf9c  223003b8           stur     w2, [x1, #0x33]
0x1eafa0  e00301aa           mov      x0, x1
0x1eafa4  a1835bf8           ldur     x1, [x29, #-0x48]
0x1eafa8  39bc0091           add      x25, x1, #0x2f
0x1eafac  200300b9           str      w0, [x25]
0x1eafb0  e0000036           tbz      w0, #0, #0x1eafcc
0x1eafb4  30f05f38           ldurb    w16, [x1, #-1]
0x1eafb8  11f05f38           ldurb    w17, [x0, #-1]
0x1eafbc  300a508a           and      x16, x17, x16, lsr #2
0x1eafc0  1f825cea           tst      x16, x28, lsr #32
0x1eafc4  40000054           b.eq     #0x1eafcc
0x1eafc8  55510394           bl       #0x2bf51c
0x1eafcc  a1035af8           ldur     x1, [x29, #-0x60]
0x1eafd0  bd120094           bl       #0x1efac4
0x1eafd4  610b44f9           ldr      x1, [x27, #0x810]  # pool[256] = snapshotRef(18530)
0x1eafd8  a0831cf8           stur     x0, [x29, #-0x38]
0x1eafdc  b7120094           bl       #0x1efab8
0x1eafe0  e30300aa           mov      x3, x0
0x1eafe4  a0835cf8           ldur     x0, [x29, #-0x38]
0x1eafe8  a3031bf8           stur     x3, [x29, #-0x50]
0x1eafec  60f000b8           stur     w0, [x3, #0xf]
0x1eaff0  a2035ff8           ldur     x2, [x29, #-0x10]
0x1eaff4  61234091           add      x1, x27, #8, lsl #12
0x1eaff8  215444f9           ldr      x1, [x1, #0x8a8]  # pool[4371] = ProbeApp.<anonymous closure>
0x1eaffc  77550394           bl       #0x2c05d8
0x1eb000  e10300aa           mov      x1, x0
0x1eb004  a0035bf8           ldur     x0, [x29, #-0x50]
0x1eb008  013001b8           stur     w1, [x0, #0x13]
0x1eb00c  a1835bf8           ldur     x1, [x29, #-0x48]
0x1eb010  39cc0091           add      x25, x1, #0x33
0x1eb014  200300b9           str      w0, [x25]
0x1eb018  e0000036           tbz      w0, #0, #0x1eb034
0x1eb01c  30f05f38           ldurb    w16, [x1, #-1]
0x1eb020  11f05f38           ldurb    w17, [x0, #-1]
0x1eb024  300a508a           and      x16, x17, x16, lsr #2
0x1eb028  1f825cea           tst      x16, x28, lsr #32
0x1eb02c  40000054           b.eq     #0x1eb034
0x1eb030  3b510394           bl       #0x2bf51c
0x1eb034  a0035ef8           ldur     x0, [x29, #-0x20]
0x1eb038  007c40d3           ubfx     x0, x0, #0, #0x20
0x1eb03c  02040012           and      w2, w0, #3
0x1eb040  a28319f8           stur     x2, [x29, #-0x68]
0x1eb044  e10302aa           mov      x1, x2
0x1eb048  217c40d3           ubfx     x1, x1, #0, #0x20
0x1eb04c  f2110094           bl       #0x1ef814
0x1eb050  017040b8           ldur     w1, [x0, #7]
0x1eb054  21801c8b           add      x1, x1, x28, lsl #32
0x1eb058  e20300aa           mov      x2, x0
0x1eb05c  01f5fc97           bl       #0x128460
0x1eb060  e00100f9           str      x0, [x15]
0x1eb064  1917fd97           bl       #0x130cc8
0x1eb068  a0831cf8           stur     x0, [x29, #-0x38]
0x1eb06c  bc140094           bl       #0x1f035c
0x1eb070  e10300aa           mov      x1, x0
0x1eb074  a0835cf8           ldur     x0, [x29, #-0x38]
0x1eb078  20b000b8           stur     w0, [x1, #0xb]
0x1eb07c  a2835df8           ldur     x2, [x29, #-0x28]
0x1eb080  223003b8           stur     w2, [x1, #0x33]
0x1eb084  e00301aa           mov      x0, x1
0x1eb088  a1835bf8           ldur     x1, [x29, #-0x48]
0x1eb08c  39dc0091           add      x25, x1, #0x37
0x1eb090  200300b9           str      w0, [x25]
0x1eb094  e0000036           tbz      w0, #0, #0x1eb0b0
0x1eb098  30f05f38           ldurb    w16, [x1, #-1]
0x1eb09c  11f05f38           ldurb    w17, [x0, #-1]
0x1eb0a0  300a508a           and      x16, x17, x16, lsr #2
0x1eb0a4  1f825cea           tst      x16, x28, lsr #32
0x1eb0a8  40000054           b.eq     #0x1eb0b0
0x1eb0ac  1c510394           bl       #0x2bf51c
0x1eb0b0  38100094           bl       #0x1ef190
0x1eb0b4  e00100f9           str      x0, [x15]
0x1eb0b8  0417fd97           bl       #0x130cc8
0x1eb0bc  a0831cf8           stur     x0, [x29, #-0x38]
0x1eb0c0  a7140094           bl       #0x1f035c
0x1eb0c4  e10300aa           mov      x1, x0
0x1eb0c8  a0835cf8           ldur     x0, [x29, #-0x38]
0x1eb0cc  20b000b8           stur     w0, [x1, #0xb]
0x1eb0d0  a2835df8           ldur     x2, [x29, #-0x28]
0x1eb0d4  223003b8           stur     w2, [x1, #0x33]
0x1eb0d8  e00301aa           mov      x0, x1
0x1eb0dc  a1835bf8           ldur     x1, [x29, #-0x48]
0x1eb0e0  39ec0091           add      x25, x1, #0x3b
0x1eb0e4  200300b9           str      w0, [x25]
0x1eb0e8  e0000036           tbz      w0, #0, #0x1eb104
0x1eb0ec  30f05f38           ldurb    w16, [x1, #-1]
0x1eb0f0  11f05f38           ldurb    w17, [x0, #-1]
0x1eb0f4  300a508a           and      x16, x17, x16, lsr #2
0x1eb0f8  1f825cea           tst      x16, x28, lsr #32
0x1eb0fc  40000054           b.eq     #0x1eb104
0x1eb100  07510394           bl       #0x2bf51c
0x1eb104  20100094           bl       #0x1ef184
0x1eb108  a0831cf8           stur     x0, [x29, #-0x38]
0x1eb10c  1e100094           bl       #0x1ef184
0x1eb110  a1835cf8           ldur     x1, [x29, #-0x38]
0x1eb114  e20300aa           mov      x2, x0
0x1eb118  09100094           bl       #0x1ef13c
0x1eb11c  a0831cf8           stur     x0, [x29, #-0x38]
0x1eb120  8f140094           bl       #0x1f035c
0x1eb124  e10300aa           mov      x1, x0
0x1eb128  a0835cf8           ldur     x0, [x29, #-0x38]
0x1eb12c  20b000b8           stur     w0, [x1, #0xb]
0x1eb130  a2835df8           ldur     x2, [x29, #-0x28]
0x1eb134  223003b8           stur     w2, [x1, #0x33]
0x1eb138  e00301aa           mov      x0, x1
0x1eb13c  a1835bf8           ldur     x1, [x29, #-0x48]
0x1eb140  39fc0091           add      x25, x1, #0x3f
0x1eb144  200300b9           str      w0, [x25]
0x1eb148  e0000036           tbz      w0, #0, #0x1eb164
0x1eb14c  30f05f38           ldurb    w16, [x1, #-1]
0x1eb150  11f05f38           ldurb    w17, [x0, #-1]
0x1eb154  300a508a           and      x16, x17, x16, lsr #2
0x1eb158  1f825cea           tst      x16, x28, lsr #32
0x1eb15c  40000054           b.eq     #0x1eb164
0x1eb160  ef500394           bl       #0x2bf51c
0x1eb164  890f0094           bl       #0x1eef88
0x1eb168  e20300aa           mov      x2, x0
0x1eb16c  40787f93           sbfiz    x0, x2, #1, #0x1f
0x1eb170  5f0480eb           cmp      x2, x0, asr #1
0x1eb174  60000054           b.eq     #0x1eb180
0x1eb178  02590394           bl       #0x2c1580
0x1eb17c  027000f8           stur     x2, [x0, #7]
0x1eb180  e00100f9           str      x0, [x15]
0x1eb184  d116fd97           bl       #0x130cc8
0x1eb188  a0831cf8           stur     x0, [x29, #-0x38]
0x1eb18c  74140094           bl       #0x1f035c
0x1eb190  e10300aa           mov      x1, x0
0x1eb194  a0835cf8           ldur     x0, [x29, #-0x38]
0x1eb198  20b000b8           stur     w0, [x1, #0xb]
0x1eb19c  a2835df8           ldur     x2, [x29, #-0x28]
0x1eb1a0  223003b8           stur     w2, [x1, #0x33]
0x1eb1a4  e00301aa           mov      x0, x1
0x1eb1a8  a1835bf8           ldur     x1, [x29, #-0x48]
0x1eb1ac  390c0191           add      x25, x1, #0x43
0x1eb1b0  200300b9           str      w0, [x25]
0x1eb1b4  e0000036           tbz      w0, #0, #0x1eb1d0
0x1eb1b8  30f05f38           ldurb    w16, [x1, #-1]
0x1eb1bc  11f05f38           ldurb    w17, [x0, #-1]
0x1eb1c0  300a508a           and      x16, x17, x16, lsr #2
0x1eb1c4  1f825cea           tst      x16, x28, lsr #32
0x1eb1c8  40000054           b.eq     #0x1eb1d0
0x1eb1cc  d4500394           bl       #0x2bf51c
0x1eb1d0  6b0f0094           bl       #0x1eef7c
0x1eb1d4  e10300aa           mov      x1, x0
0x1eb1d8  a0035af8           ldur     x0, [x29, #-0x60]
0x1eb1dc  207000f8           stur     x0, [x1, #7]
0x1eb1e0  800080d2           mov      x0, #4
0x1eb1e4  20f000f8           stur     x0, [x1, #0xf]
0x1eb1e8  e20301aa           mov      x2, x1
0x1eb1ec  61234091           add      x1, x27, #8, lsl #12
0x1eb1f0  215844f9           ldr      x1, [x1, #0x8b0]  # pool[4372] = snapshotInstance(E15Vec)
0x1eb1f4  beb1fd97           bl       #0x1578ec
0x1eb1f8  e20300aa           mov      x2, x0
0x1eb1fc  40787f93           sbfiz    x0, x2, #1, #0x1f
0x1eb200  5f0480eb           cmp      x2, x0, asr #1
0x1eb204  60000054           b.eq     #0x1eb210
0x1eb208  de580394           bl       #0x2c1580
0x1eb20c  027000f8           stur     x2, [x0, #7]
0x1eb210  e00100f9           str      x0, [x15]
0x1eb214  ad16fd97           bl       #0x130cc8
0x1eb218  a0831cf8           stur     x0, [x29, #-0x38]
0x1eb21c  50140094           bl       #0x1f035c
0x1eb220  e10300aa           mov      x1, x0
0x1eb224  a0835cf8           ldur     x0, [x29, #-0x38]
0x1eb228  20b000b8           stur     w0, [x1, #0xb]
0x1eb22c  a2835df8           ldur     x2, [x29, #-0x28]
0x1eb230  223003b8           stur     w2, [x1, #0x33]
0x1eb234  e00301aa           mov      x0, x1
0x1eb238  a1835bf8           ldur     x1, [x29, #-0x48]
0x1eb23c  391c0191           add      x25, x1, #0x47
0x1eb240  200300b9           str      w0, [x25]
0x1eb244  e0000036           tbz      w0, #0, #0x1eb260
0x1eb248  30f05f38           ldurb    w16, [x1, #-1]
0x1eb24c  11f05f38           ldurb    w17, [x0, #-1]
0x1eb250  300a508a           and      x16, x17, x16, lsr #2
0x1eb254  1f825cea           tst      x16, x28, lsr #32
0x1eb258  40000054           b.eq     #0x1eb260
0x1eb25c  b0500394           bl       #0x2bf51c
0x1eb260  61b743f9           ldr      x1, [x27, #0x768]  # pool[235] = snapshotRef(18312)
0x1eb264  1010fd97           bl       #0x12f2a4
0x1eb268  e10300aa           mov      x1, x0
0x1eb26c  608345f9           ldr      x0, [x27, #0xb00]  # pool[350] = snapshotRef(51101)
0x1eb270  a1831cf8           stur     x1, [x29, #-0x38]
0x1eb274  20b001b8           stur     w0, [x1, #0x1b]
0x1eb278  3fb000b8           stur     wzr, [x1, #0xb]
0x1eb27c  608745f9           ldr      x0, [x27, #0xb08]  # pool[351] = snapshotRef(47572)
0x1eb280  20f000b8           stur     w0, [x1, #0xf]
0x1eb284  3f3001b8           stur     wzr, [x1, #0x13]
0x1eb288  3f7001b8           stur     wzr, [x1, #0x17]
0x1eb28c  b0035cf8           ldur     x16, [x29, #-0x40]
0x1eb290  f00100f9           str      x16, [x15]
0x1eb294  8d16fd97           bl       #0x130cc8
0x1eb298  a1835cf8           ldur     x1, [x29, #-0x38]
0x1eb29c  e20300aa           mov      x2, x0
0x1eb2a0  f03f0294           bl       #0x27b260
0x1eb2a4  a1835cf8           ldur     x1, [x29, #-0x38]
0x1eb2a8  62234091           add      x2, x27, #8, lsl #12
0x1eb2ac  423844f9           ldr      x2, [x2, #0x870]  # pool[4364] = snapshotRef(458)
0x1eb2b0  ec3f0294           bl       #0x27b260
0x1eb2b4  a1835cf8           ldur     x1, [x29, #-0x38]
0x1eb2b8  390b0094           bl       #0x1edf9c
0x1eb2bc  e00100f9           str      x0, [x15]
0x1eb2c0  8216fd97           bl       #0x130cc8
0x1eb2c4  a0831cf8           stur     x0, [x29, #-0x38]
0x1eb2c8  25140094           bl       #0x1f035c
0x1eb2cc  e10300aa           mov      x1, x0
0x1eb2d0  a0835cf8           ldur     x0, [x29, #-0x38]
0x1eb2d4  20b000b8           stur     w0, [x1, #0xb]
0x1eb2d8  a3835df8           ldur     x3, [x29, #-0x28]
0x1eb2dc  233003b8           stur     w3, [x1, #0x33]
0x1eb2e0  e00301aa           mov      x0, x1
0x1eb2e4  a1835bf8           ldur     x1, [x29, #-0x48]
0x1eb2e8  392c0191           add      x25, x1, #0x4b
0x1eb2ec  200300b9           str      w0, [x25]
0x1eb2f0  e0000036           tbz      w0, #0, #0x1eb30c
0x1eb2f4  30f05f38           ldurb    w16, [x1, #-1]
0x1eb2f8  11f05f38           ldurb    w17, [x0, #-1]
0x1eb2fc  300a508a           and      x16, x17, x16, lsr #2
0x1eb300  1f825cea           tst      x16, x28, lsr #32
0x1eb304  40000054           b.eq     #0x1eb30c
0x1eb308  85500394           bl       #0x2bf51c
0x1eb30c  e10316aa           mov      x1, x22
0x1eb310  c20080d2           mov      x2, #6
0x1eb314  f8570394           bl       #0x2c12f4
0x1eb318  70234091           add      x16, x27, #8, lsl #12
0x1eb31c  105e44f9           ldr      x16, [x16, #0x8b8]  # pool[4373] = "[{\"a\":"
0x1eb320  10f000b8           stur     w16, [x0, #0xf]
0x1eb324  a1035cf8           ldur     x1, [x29, #-0x40]
0x1eb328  013001b8           stur     w1, [x0, #0x13]
0x1eb32c  70234091           add      x16, x27, #8, lsl #12
0x1eb330  106244f9           ldr      x16, [x16, #0x8c0]  # pool[4374] = "},{\"b\":null}]"
0x1eb334  107001b8           stur     w16, [x0, #0x17]
0x1eb338  e00100f9           str      x0, [x15]
0x1eb33c  9015fd97           bl       #0x13097c
0x1eb340  e10300aa           mov      x1, x0
0x1eb344  b30a0094           bl       #0x1ede10
0x1eb348  01b040b8           ldur     w1, [x0, #0xb]
0x1eb34c  e10100f9           str      x1, [x15]
0x1eb350  aa240194           bl       #0x2345f8
0x1eb354  a0831cf8           stur     x0, [x29, #-0x38]
0x1eb358  01140094           bl       #0x1f035c
0x1eb35c  e10300aa           mov      x1, x0
0x1eb360  a0835cf8           ldur     x0, [x29, #-0x38]
0x1eb364  20b000b8           stur     w0, [x1, #0xb]
0x1eb368  a2835df8           ldur     x2, [x29, #-0x28]
0x1eb36c  223003b8           stur     w2, [x1, #0x33]
0x1eb370  e00301aa           mov      x0, x1
0x1eb374  a1835bf8           ldur     x1, [x29, #-0x48]
0x1eb378  393c0191           add      x25, x1, #0x4f
0x1eb37c  200300b9           str      w0, [x25]
0x1eb380  e0000036           tbz      w0, #0, #0x1eb39c
0x1eb384  30f05f38           ldurb    w16, [x1, #-1]
0x1eb388  11f05f38           ldurb    w17, [x0, #-1]
0x1eb38c  300a508a           and      x16, x17, x16, lsr #2
0x1eb390  1f825cea           tst      x16, x28, lsr #32
0x1eb394  40000054           b.eq     #0x1eb39c
0x1eb398  61500394           bl       #0x2bf51c
0x1eb39c  a0035ef8           ldur     x0, [x29, #-0x20]
0x1eb3a0  017d80d2           mov      x1, #0x3e8
0x1eb3a4  040cc19a           sdiv     x4, x0, x1
0x1eb3a8  8380019b           msub     x3, x4, x1, x0
0x1eb3ac  7f001feb           cmp      x3, xzr
0x1eb3b0  0b270054           b.lt     #0x1eb890
0x1eb3b4  6000629e           scvtf    d0, x3
0x1eb3b8  0110641e           fmov     d1, #8.00000000
0x1eb3bc  0218611e           fdiv     d2, d0, d1
0x1eb3c0  401ca24e           mov      v0.16b, v2.16b
0x1eb3c4  e3020094           bl       #0x1ebf50
0x1eb3c8  a0831cf8           stur     x0, [x29, #-0x38]
0x1eb3cc  e4130094           bl       #0x1f035c
0x1eb3d0  e10300aa           mov      x1, x0
0x1eb3d4  a0835cf8           ldur     x0, [x29, #-0x38]
0x1eb3d8  20b000b8           stur     w0, [x1, #0xb]
0x1eb3dc  a2835df8           ldur     x2, [x29, #-0x28]
0x1eb3e0  223003b8           stur     w2, [x1, #0x33]
0x1eb3e4  e00301aa           mov      x0, x1
0x1eb3e8  a1835bf8           ldur     x1, [x29, #-0x48]
0x1eb3ec  394c0191           add      x25, x1, #0x53
0x1eb3f0  200300b9           str      w0, [x25]
0x1eb3f4  e0000036           tbz      w0, #0, #0x1eb410
0x1eb3f8  30f05f38           ldurb    w16, [x1, #-1]
0x1eb3fc  11f05f38           ldurb    w17, [x0, #-1]
0x1eb400  300a508a           and      x16, x17, x16, lsr #2
0x1eb404  1f825cea           tst      x16, x28, lsr #32
0x1eb408  40000054           b.eq     #0x1eb410
0x1eb40c  44500394           bl       #0x2bf51c
0x1eb410  a0835ef8           ldur     x0, [x29, #-0x18]
0x1eb414  01f87fd3           lsl      x1, x0, #1
0x1eb418  a38359f8           ldur     x3, [x29, #-0x68]
0x1eb41c  64781f53           lsl      w4, w3, #1
0x1eb420  e40500a9           stp      x4, x1, [x15]
0x1eb424  643b41f9           ldr      x4, [x27, #0x270]  # pool[76] = snapshotRef(23)
0x1eb428  68b1fd97           bl       #0x1579c8
0x1eb42c  e00100f9           str      x0, [x15]
0x1eb430  2616fd97           bl       #0x130cc8
0x1eb434  a0831cf8           stur     x0, [x29, #-0x38]
0x1eb438  c9130094           bl       #0x1f035c
0x1eb43c  e10300aa           mov      x1, x0
0x1eb440  a0835cf8           ldur     x0, [x29, #-0x38]
0x1eb444  20b000b8           stur     w0, [x1, #0xb]
0x1eb448  a2835df8           ldur     x2, [x29, #-0x28]
0x1eb44c  223003b8           stur     w2, [x1, #0x33]
0x1eb450  e00301aa           mov      x0, x1
0x1eb454  a1835bf8           ldur     x1, [x29, #-0x48]
0x1eb458  395c0191           add      x25, x1, #0x57
0x1eb45c  200300b9           str      w0, [x25]
0x1eb460  e0000036           tbz      w0, #0, #0x1eb47c
0x1eb464  30f05f38           ldurb    w16, [x1, #-1]
0x1eb468  11f05f38           ldurb    w17, [x0, #-1]
0x1eb46c  300a508a           and      x16, x17, x16, lsr #2
0x1eb470  1f825cea           tst      x16, x28, lsr #32
0x1eb474  40000054           b.eq     #0x1eb47c
0x1eb478  29500394           bl       #0x2bf51c
0x1eb47c  b2020094           bl       #0x1ebf44
0x1eb480  e10300aa           mov      x1, x0
0x1eb484  ad020094           bl       #0x1ebf38
0x1eb488  a0831cf8           stur     x0, [x29, #-0x38]
0x1eb48c  b4130094           bl       #0x1f035c
0x1eb490  e10300aa           mov      x1, x0
0x1eb494  a0835cf8           ldur     x0, [x29, #-0x38]
0x1eb498  20b000b8           stur     w0, [x1, #0xb]
0x1eb49c  a2835df8           ldur     x2, [x29, #-0x28]
0x1eb4a0  223003b8           stur     w2, [x1, #0x33]
0x1eb4a4  e00301aa           mov      x0, x1
0x1eb4a8  a1835bf8           ldur     x1, [x29, #-0x48]
0x1eb4ac  396c0191           add      x25, x1, #0x5b
0x1eb4b0  200300b9           str      w0, [x25]
0x1eb4b4  e0000036           tbz      w0, #0, #0x1eb4d0
0x1eb4b8  30f05f38           ldurb    w16, [x1, #-1]
0x1eb4bc  11f05f38           ldurb    w17, [x0, #-1]
0x1eb4c0  300a508a           and      x16, x17, x16, lsr #2
0x1eb4c4  1f825cea           tst      x16, x28, lsr #32
0x1eb4c8  40000054           b.eq     #0x1eb4d0
0x1eb4cc  14500394           bl       #0x2bf51c
0x1eb4d0  a1035df8           ldur     x1, [x29, #-0x30]
0x1eb4d4  55020094           bl       #0x1ebe28
0x1eb4d8  70234091           add      x16, x27, #8, lsl #12
0x1eb4dc  106644f9           ldr      x16, [x16, #0x8c8]  # pool[4375] = snapshotInstance(E21Mode)
0x1eb4e0  1f00106b           cmp      w0, w16
0x1eb4e4  60000054           b.eq     #0x1eb4f0
0x1eb4e8  013041f8           ldur     x1, [x0, #0x13]
0x1eb4ec  6100f8b6           tbz      x1, #0x3f, #0x1eb4f8
0x1eb4f0  65b345f9           ldr      x5, [x27, #0xb60]  # pool[362] = snapshotRef(471)
0x1eb4f4  02000014           b        #0x1eb4fc
0x1eb4f8  65bb45f9           ldr      x5, [x27, #0xb70]  # pool[364] = snapshotRef(167)
0x1eb4fc  a0835df8           ldur     x0, [x29, #-0x28]
0x1eb500  a1835ef8           ldur     x1, [x29, #-0x18]
0x1eb504  a2035ef8           ldur     x2, [x29, #-0x20]
0x1eb508  a4835ff8           ldur     x4, [x29, #-8]
0x1eb50c  a3035cf8           ldur     x3, [x29, #-0x40]
0x1eb510  a5031df8           stur     x5, [x29, #-0x30]
0x1eb514  92130094           bl       #0x1f035c
0x1eb518  e10300aa           mov      x1, x0
0x1eb51c  a0035df8           ldur     x0, [x29, #-0x30]
0x1eb520  20b000b8           stur     w0, [x1, #0xb]
0x1eb524  a2835df8           ldur     x2, [x29, #-0x28]
0x1eb528  223003b8           stur     w2, [x1, #0x33]
0x1eb52c  e00301aa           mov      x0, x1
0x1eb530  a1835bf8           ldur     x1, [x29, #-0x48]
0x1eb534  397c0191           add      x25, x1, #0x5f
0x1eb538  200300b9           str      w0, [x25]
0x1eb53c  e0000036           tbz      w0, #0, #0x1eb558
0x1eb540  30f05f38           ldurb    w16, [x1, #-1]
0x1eb544  11f05f38           ldurb    w17, [x0, #-1]
0x1eb548  300a508a           and      x16, x17, x16, lsr #2
0x1eb54c  1f825cea           tst      x16, x28, lsr #32
0x1eb550  40000054           b.eq     #0x1eb558
0x1eb554  f24f0394           bl       #0x2bf51c
0x1eb558  500080d2           mov      x16, #2
0x1eb55c  f00100f9           str      x16, [x15]
0x1eb560  26240194           bl       #0x2345f8
0x1eb564  a0031df8           stur     x0, [x29, #-0x30]
0x1eb568  7d130094           bl       #0x1f035c
0x1eb56c  e10300aa           mov      x1, x0
0x1eb570  a0035df8           ldur     x0, [x29, #-0x30]
0x1eb574  20b000b8           stur     w0, [x1, #0xb]
0x1eb578  a3835df8           ldur     x3, [x29, #-0x28]
0x1eb57c  233003b8           stur     w3, [x1, #0x33]
0x1eb580  e00301aa           mov      x0, x1
0x1eb584  a1835bf8           ldur     x1, [x29, #-0x48]
0x1eb588  398c0191           add      x25, x1, #0x63
0x1eb58c  200300b9           str      w0, [x25]
0x1eb590  e0000036           tbz      w0, #0, #0x1eb5ac
0x1eb594  30f05f38           ldurb    w16, [x1, #-1]
0x1eb598  11f05f38           ldurb    w17, [x0, #-1]
0x1eb59c  300a508a           and      x16, x17, x16, lsr #2
0x1eb5a0  1f825cea           tst      x16, x28, lsr #32
0x1eb5a4  40000054           b.eq     #0x1eb5ac
0x1eb5a8  dd4f0394           bl       #0x2bf51c
0x1eb5ac  e10316aa           mov      x1, x22
0x1eb5b0  420080d2           mov      x2, #2
0x1eb5b4  50570394           bl       #0x2c12f4
0x1eb5b8  e20300aa           mov      x2, x0
0x1eb5bc  a0035cf8           ldur     x0, [x29, #-0x40]
0x1eb5c0  a2031df8           stur     x2, [x29, #-0x30]
0x1eb5c4  40f000b8           stur     w0, [x2, #0xf]
0x1eb5c8  e10316aa           mov      x1, x22
0x1eb5cc  fe520394           bl       #0x2c01c4
0x1eb5d0  e30300aa           mov      x3, x0
0x1eb5d4  a0035df8           ldur     x0, [x29, #-0x30]
0x1eb5d8  a3831cf8           stur     x3, [x29, #-0x38]
0x1eb5dc  60f000b8           stur     w0, [x3, #0xf]
0x1eb5e0  400080d2           mov      x0, #2
0x1eb5e4  60b000b8           stur     w0, [x3, #0xb]
0x1eb5e8  a2035ff8           ldur     x2, [x29, #-0x10]
0x1eb5ec  61234091           add      x1, x27, #8, lsl #12
0x1eb5f0  216844f9           ldr      x1, [x1, #0x8d0]  # pool[4376] = ProbeApp.<anonymous closure>
0x1eb5f4  f9530394           bl       #0x2c05d8
0x1eb5f8  e10300aa           mov      x1, x0
0x1eb5fc  a2835cf8           ldur     x2, [x29, #-0x38]
0x1eb600  16010094           bl       #0x1eba58
0x1eb604  a0031ff8           stur     x0, [x29, #-0x10]
0x1eb608  55130094           bl       #0x1f035c
0x1eb60c  e10300aa           mov      x1, x0
0x1eb610  a0035ff8           ldur     x0, [x29, #-0x10]
0x1eb614  20b000b8           stur     w0, [x1, #0xb]
0x1eb618  a4835df8           ldur     x4, [x29, #-0x28]
0x1eb61c  243003b8           stur     w4, [x1, #0x33]
0x1eb620  e00301aa           mov      x0, x1
0x1eb624  a1835bf8           ldur     x1, [x29, #-0x48]
0x1eb628  399c0191           add      x25, x1, #0x67
0x1eb62c  200300b9           str      w0, [x25]
0x1eb630  e0000036           tbz      w0, #0, #0x1eb64c
0x1eb634  30f05f38           ldurb    w16, [x1, #-1]
0x1eb638  11f05f38           ldurb    w17, [x0, #-1]
0x1eb63c  300a508a           and      x16, x17, x16, lsr #2
0x1eb640  1f825cea           tst      x16, x28, lsr #32
0x1eb644  40000054           b.eq     #0x1eb64c
0x1eb648  b54f0394           bl       #0x2bf51c
0x1eb64c  a0035ef8           ldur     x0, [x29, #-0x20]
0x1eb650  007c40d3           ubfx     x0, x0, #0, #0x20
0x1eb654  01000012           and      w1, w0, #1
0x1eb658  61000034           cbz      w1, #0x1eb664
0x1eb65c  c0c20091           add      x0, x22, #0x30
0x1eb660  02000014           b        #0x1eb668
0x1eb664  c0820091           add      x0, x22, #0x20
0x1eb668  a1835ef8           ldur     x1, [x29, #-0x18]
0x1eb66c  610000b4           cbz      x1, #0x1eb678
0x1eb670  c2c20091           add      x2, x22, #0x30
0x1eb674  02000014           b        #0x1eb67c
0x1eb678  c2820091           add      x2, x22, #0x20
0x1eb67c  a1035ef8           ldur     x1, [x29, #-0x20]
0x1eb680  230280d2           mov      x3, #0x11
0x1eb684  260cc39a           sdiv     x6, x1, x3
0x1eb688  c584039b           msub     x5, x6, x3, x1
0x1eb68c  bf001feb           cmp      x5, xzr
0x1eb690  4b100054           b.lt     #0x1eb898
0x1eb694  e10300aa           mov      x1, x0
0x1eb698  e30305aa           mov      x3, x5
0x1eb69c  c6000094           bl       #0x1eb9b4
0x1eb6a0  e20300aa           mov      x2, x0
0x1eb6a4  40787f93           sbfiz    x0, x2, #1, #0x1f
0x1eb6a8  5f0480eb           cmp      x2, x0, asr #1
0x1eb6ac  60000054           b.eq     #0x1eb6b8
0x1eb6b0  b4570394           bl       #0x2c1580
0x1eb6b4  027000f8           stur     x2, [x0, #7]
0x1eb6b8  e00100f9           str      x0, [x15]
0x1eb6bc  8315fd97           bl       #0x130cc8
0x1eb6c0  a0031ff8           stur     x0, [x29, #-0x10]
0x1eb6c4  26130094           bl       #0x1f035c
0x1eb6c8  e10300aa           mov      x1, x0
0x1eb6cc  a0035ff8           ldur     x0, [x29, #-0x10]
0x1eb6d0  20b000b8           stur     w0, [x1, #0xb]
0x1eb6d4  a3835df8           ldur     x3, [x29, #-0x28]
0x1eb6d8  233003b8           stur     w3, [x1, #0x33]
0x1eb6dc  e00301aa           mov      x0, x1
0x1eb6e0  a1835bf8           ldur     x1, [x29, #-0x48]
0x1eb6e4  39ac0191           add      x25, x1, #0x6b
0x1eb6e8  200300b9           str      w0, [x25]
0x1eb6ec  e0000036           tbz      w0, #0, #0x1eb708
0x1eb6f0  30f05f38           ldurb    w16, [x1, #-1]
0x1eb6f4  11f05f38           ldurb    w17, [x0, #-1]
0x1eb6f8  300a508a           and      x16, x17, x16, lsr #2
0x1eb6fc  1f825cea           tst      x16, x28, lsr #32
0x1eb700  40000054           b.eq     #0x1eb708
0x1eb704  864f0394           bl       #0x2bf51c
0x1eb708  e10316aa           mov      x1, x22
0x1eb70c  820080d2           mov      x2, #4
0x1eb710  f9560394           bl       #0x2c12f4
0x1eb714  70234091           add      x16, x27, #8, lsl #12
0x1eb718  102644f9           ldr      x16, [x16, #0x848]  # pool[4359] = snapshotRef(870)
0x1eb71c  10f000b8           stur     w16, [x0, #0xf]
0x1eb720  a1835ff8           ldur     x1, [x29, #-8]
0x1eb724  013001b8           stur     w1, [x0, #0x13]
0x1eb728  e00100f9           str      x0, [x15]
0x1eb72c  9414fd97           bl       #0x13097c
0x1eb730  e10300aa           mov      x1, x0
0x1eb734  61000094           bl       #0x1eb8b8
0x1eb738  60002037           tbnz     w0, #4, #0x1eb744
0x1eb73c  62b345f9           ldr      x2, [x27, #0xb60]  # pool[362] = snapshotRef(471)
0x1eb740  02000014           b        #0x1eb748
0x1eb744  62bb45f9           ldr      x2, [x27, #0xb70]  # pool[364] = snapshotRef(167)
0x1eb748  a0835df8           ldur     x0, [x29, #-0x28]
0x1eb74c  a1835bf8           ldur     x1, [x29, #-0x48]
0x1eb750  a2831ff8           stur     x2, [x29, #-8]
0x1eb754  02130094           bl       #0x1f035c
0x1eb758  e10300aa           mov      x1, x0
0x1eb75c  a0835ff8           ldur     x0, [x29, #-8]
0x1eb760  20b000b8           stur     w0, [x1, #0xb]
0x1eb764  a0835df8           ldur     x0, [x29, #-0x28]
0x1eb768  203003b8           stur     w0, [x1, #0x33]
0x1eb76c  e00301aa           mov      x0, x1
0x1eb770  a1835bf8           ldur     x1, [x29, #-0x48]
0x1eb774  39bc0191           add      x25, x1, #0x6f
0x1eb778  200300b9           str      w0, [x25]
0x1eb77c  e0000036           tbz      w0, #0, #0x1eb798
0x1eb780  30f05f38           ldurb    w16, [x1, #-1]
0x1eb784  11f05f38           ldurb    w17, [x0, #-1]
0x1eb788  300a508a           and      x16, x17, x16, lsr #2
0x1eb78c  1f825cea           tst      x16, x28, lsr #32
0x1eb790  40000054           b.eq     #0x1eb798
0x1eb794  624f0394           bl       #0x2bf51c
0x1eb798  61234091           add      x1, x27, #8, lsl #12
0x1eb79c  213444f9           ldr      x1, [x1, #0x868]  # pool[4363] = snapshotRef(18423)
0x1eb7a0  89520394           bl       #0x2c01c4
0x1eb7a4  e10300aa           mov      x1, x0
0x1eb7a8  a0835bf8           ldur     x0, [x29, #-0x48]
0x1eb7ac  a1831ff8           stur     x1, [x29, #-8]
0x1eb7b0  20f000b8           stur     w0, [x1, #0xf]
0x1eb7b4  400680d2           mov      x0, #0x32
0x1eb7b8  20b000b8           stur     w0, [x1, #0xb]
0x1eb7bc  3c000094           bl       #0x1eb8ac
0x1eb7c0  e10300aa           mov      x1, x0
0x1eb7c4  60234091           add      x0, x27, #8, lsl #12
0x1eb7c8  006c44f9           ldr      x0, [x0, #0x8d8]  # pool[4377] = snapshotInstance(Axis)
0x1eb7cc  a1031ff8           stur     x1, [x29, #-0x10]
0x1eb7d0  20f000b8           stur     w0, [x1, #0xf]
0x1eb7d4  60234091           add      x0, x27, #8, lsl #12
0x1eb7d8  007044f9           ldr      x0, [x0, #0x8e0]  # pool[4378] = snapshotInstance(MainAxisAlignment)
0x1eb7dc  203001b8           stur     w0, [x1, #0x13]
0x1eb7e0  60234091           add      x0, x27, #8, lsl #12
0x1eb7e4  007444f9           ldr      x0, [x0, #0x8e8]  # pool[4379] = snapshotInstance(MainAxisSize)
0x1eb7e8  207001b8           stur     w0, [x1, #0x17]
0x1eb7ec  60234091           add      x0, x27, #8, lsl #12
0x1eb7f0  007844f9           ldr      x0, [x0, #0x8f0]  # pool[4380] = snapshotInstance(CrossAxisAlignment)
0x1eb7f4  20b001b8           stur     w0, [x1, #0x1b]
0x1eb7f8  60234091           add      x0, x27, #8, lsl #12
0x1eb7fc  007c44f9           ldr      x0, [x0, #0x8f8]  # pool[4381] = snapshotInstance(VerticalDirection)
0x1eb800  203002b8           stur     w0, [x1, #0x23]
0x1eb804  60234091           add      x0, x27, #8, lsl #12
0x1eb808  008044f9           ldr      x0, [x0, #0x900]  # pool[4382] = snapshotInstance(Clip)
0x1eb80c  20b002b8           stur     w0, [x1, #0x2b]
0x1eb810  3ff002f8           stur     xzr, [x1, #0x2f]
0x1eb814  a0835ff8           ldur     x0, [x29, #-8]
0x1eb818  20b000b8           stur     w0, [x1, #0xb]
0x1eb81c  21000094           bl       #0x1eb8a0
0x1eb820  61234091           add      x1, x27, #8, lsl #12
0x1eb824  218444f9           ldr      x1, [x1, #0x908]  # pool[4383] = snapshotInstance(Alignment)
0x1eb828  01f000b8           stur     w1, [x0, #0xf]
0x1eb82c  a1035ff8           ldur     x1, [x29, #-0x10]
0x1eb830  01b000b8           stur     w1, [x0, #0xb]
0x1eb834  ef031daa           mov      x15, x29
0x1eb838  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1eb83c  c0035fd6           ret      
0x1eb840  f0560394           bl       #0x2c1400
0x1eb844  11fcff17           b        #0x1ea888
0x1eb848  c600008b           add      x6, x6, x0
0x1eb84c  42fcff17           b        #0x1ea954
0x1eb850  6300018b           add      x3, x3, x1
0x1eb854  70fcff17           b        #0x1eaa14
0x1eb858  8400018b           add      x4, x4, x1
0x1eb85c  73fcff17           b        #0x1eaa28
0x1eb860  8400018b           add      x4, x4, x1
0x1eb864  18fdff17           b        #0x1eacc4
0x1eb868  a500018b           add      x5, x5, x1
0x1eb86c  1cfdff17           b        #0x1eacdc
0x1eb870  6300028b           add      x3, x3, x2
0x1eb874  56fdff17           b        #0x1eadcc
0x1eb878  8400008b           add      x4, x4, x0
0x1eb87c  77fdff17           b        #0x1eae58
0x1eb880  e0560394           bl       #0x2c1400
0x1eb884  7bfdff17           b        #0x1eae70
0x1eb888  de560394           bl       #0x2c1400
0x1eb88c  80fdff17           b        #0x1eae8c
0x1eb890  6300018b           add      x3, x3, x1
0x1eb894  c8feff17           b        #0x1eb3b4
0x1eb898  a500038b           add      x5, x5, x3
0x1eb89c  7effff17           b        #0x1eb694
# CFG: 0x1ea860->0x1ea888/ConditionalFalse 0x1ea860->0x1eb840/ConditionalTrue 0x1ea888->0x1ea8b8/ConditionalFalse 0x1ea888->0x1ea8c0/ConditionalTrue 0x1ea8b8->0x1ea8c0/Fallthrough 0x1ea8c0->0x1ea8d4/ConditionalFalse 0x1ea8c0->0x1ea8ec/ConditionalTrue 0x1ea8d4->0x1ea8e8/ConditionalFalse 0x1ea8d4->0x1ea8ec/ConditionalTrue 0x1ea8e8->0x1ea8ec/Fallthrough 0x1ea8ec->0x1ea920/ConditionalFalse 0x1ea8ec->0x1ea924/ConditionalTrue 0x1ea920->0x1ea924/Fallthrough 0x1ea924->0x1ea92c/ConditionalFalse 0x1ea924->0x1ea938/ConditionalTrue 0x1ea92c->0x1ea93c/Branch 0x1ea938->0x1ea93c/Fallthrough 0x1ea93c->0x1ea954/ConditionalFalse 0x1ea93c->0x1eb848/ConditionalTrue 0x1ea954->0x1ea988/ConditionalFalse 0x1ea954->0x1ea994/ConditionalTrue 0x1ea988->0x1ea998/Branch 0x1ea994->0x1ea998/Fallthrough 0x1ea998->0x1ea9e0/ConditionalFalse 0x1ea998->0x1ea9e8/ConditionalTrue 0x1ea9e0->0x1ea9e8/Fallthrough 0x1ea9e8->0x1eaa14/ConditionalFalse 0x1ea9e8->0x1eb850/ConditionalTrue 0x1eaa14->0x1eaa28/ConditionalFalse 0x1eaa14->0x1eb858/ConditionalTrue 0x1eaa28->0x1eab14/ConditionalFalse 0x1eaa28->0x1eab2c/ConditionalTrue 0x1eab14->0x1eab28/ConditionalFalse 0x1eab14->0x1eab2c/ConditionalTrue 0x1eab28->0x1eab2c/Fallthrough 0x1eab2c->0x1eab44/ConditionalFalse 0x1eab2c->0x1eab50/ConditionalTrue 0x1eab44->0x1eab90/Branch 0x1eab50->0x1eab68/ConditionalFalse 0x1eab50->0x1eab7c/ConditionalTrue 0x1eab68->0x1eab7c/ConditionalFalse 0x1eab68->0x1eab88/ConditionalTrue 0x1eab7c->0x1eab90/Branch 0x1eab88->0x1eab90/Fallthrough 0x1eab90->0x1eabcc/ConditionalFalse 0x1eab90->0x1eabe4/ConditionalTrue 0x1eabcc->0x1eabe0/ConditionalFalse 0x1eabcc->0x1eabe4/ConditionalTrue 0x1eabe0->0x1eabe4/Fallthrough 0x1eabe4->0x1eabfc/ConditionalFalse 0x1eabe4->0x1eac04/ConditionalTrue 0x1eabfc->0x1eac04/Fallthrough 0x1eac04->0x1eac3c/ConditionalFalse 0x1eac04->0x1eac54/ConditionalTrue 0x1eac3c->0x1eac50/ConditionalFalse 0x1eac3c->0x1eac54/ConditionalTrue 0x1eac50->0x1eac54/Fallthrough 0x1eac54->0x1eac94/ConditionalFalse 0x1eac54->0x1eacac/ConditionalTrue 0x1eac94->0x1eaca8/ConditionalFalse 0x1eac94->0x1eacac/ConditionalTrue 0x1eaca8->0x1eacac/Fallthrough 0x1eacac->0x1eacc4/ConditionalFalse 0x1eacac->0x1eb860/ConditionalTrue 0x1eacc4->0x1eacdc/ConditionalFalse 0x1eacc4->0x1eb868/ConditionalTrue 0x1eacdc->0x1ead44/ConditionalFalse 0x1eacdc->0x1ead4c/ConditionalTrue 0x1ead44->0x1ead4c/Fallthrough 0x1ead4c->0x1ead84/ConditionalFalse 0x1ead4c->0x1ead9c/ConditionalTrue 0x1ead84->0x1ead98/ConditionalFalse 0x1ead84->0x1ead9c/ConditionalTrue 0x1ead98->0x1ead9c/Fallthrough 0x1ead9c->0x1eadcc/ConditionalFalse 0x1ead9c->0x1eb870/ConditionalTrue 0x1eadcc->0x1eae28/ConditionalFalse 0x1eadcc->0x1eae40/ConditionalTrue 0x1eae28->0x1eae3c/ConditionalFalse 0x1eae28->0x1eae40/ConditionalTrue 0x1eae3c->0x1eae40/Fallthrough 0x1eae40->0x1eae58/ConditionalFalse 0x1eae40->0x1eb878/ConditionalTrue 0x1eae58->0x1eae64/Fallthrough 0x1eae64->0x1eae70/ConditionalFalse 0x1eae64->0x1eb880/ConditionalTrue 0x1eae70->0x1eae78/ConditionalFalse 0x1eae70->0x1eaed8/ConditionalTrue 0x1eae78->0x1eae80/Fallthrough 0x1eae80->0x1eae8c/ConditionalFalse 0x1eae80->0x1eb888/ConditionalTrue 0x1eae8c->0x1eae94/ConditionalFalse 0x1eae8c->0x1eaec8/ConditionalTrue 0x1eae94->0x1eaea0/ConditionalFalse 0x1eae94->0x1eaec0/ConditionalTrue 0x1eaea0->0x1eaeac/ConditionalFalse 0x1eaea0->0x1eaedc/ConditionalTrue 0x1eaeac->0x1eae80/Branch 0x1eaec0->0x1eaecc/Branch 0x1eaec8->0x1eaecc/Fallthrough 0x1eaecc->0x1eae64/Branch 0x1eaed8->0x1eaedc/Fallthrough 0x1eaedc->0x1eaee8/ConditionalFalse 0x1eaedc->0x1eaef0/ConditionalTrue 0x1eaee8->0x1eaef0/Fallthrough 0x1eaef0->0x1eaf28/ConditionalFalse 0x1eaef0->0x1eaf40/ConditionalTrue 0x1eaf28->0x1eaf3c/ConditionalFalse 0x1eaf28->0x1eaf40/ConditionalTrue 0x1eaf3c->0x1eaf40/Fallthrough 0x1eaf40->0x1eaf48/ConditionalFalse 0x1eaf40->0x1eaf5c/ConditionalTrue 0x1eaf48->0x1eaf74/Branch 0x1eaf5c->0x1eaf74/Fallthrough 0x1eaf74->0x1eafb4/ConditionalFalse 0x1eaf74->0x1eafcc/ConditionalTrue 0x1eafb4->0x1eafc8/ConditionalFalse 0x1eafb4->0x1eafcc/ConditionalTrue 0x1eafc8->0x1eafcc/Fallthrough 0x1eafcc->0x1eb01c/ConditionalFalse 0x1eafcc->0x1eb034/ConditionalTrue 0x1eb01c->0x1eb030/ConditionalFalse 0x1eb01c->0x1eb034/ConditionalTrue 0x1eb030->0x1eb034/Fallthrough 0x1eb034->0x1eb098/ConditionalFalse 0x1eb034->0x1eb0b0/ConditionalTrue 0x1eb098->0x1eb0ac/ConditionalFalse 0x1eb098->0x1eb0b0/ConditionalTrue 0x1eb0ac->0x1eb0b0/Fallthrough 0x1eb0b0->0x1eb0ec/ConditionalFalse 0x1eb0b0->0x1eb104/ConditionalTrue 0x1eb0ec->0x1eb100/ConditionalFalse 0x1eb0ec->0x1eb104/ConditionalTrue 0x1eb100->0x1eb104/Fallthrough 0x1eb104->0x1eb14c/ConditionalFalse 0x1eb104->0x1eb164/ConditionalTrue 0x1eb14c->0x1eb160/ConditionalFalse 0x1eb14c->0x1eb164/ConditionalTrue 0x1eb160->0x1eb164/Fallthrough 0x1eb164->0x1eb178/ConditionalFalse 0x1eb164->0x1eb180/ConditionalTrue 0x1eb178->0x1eb180/Fallthrough 0x1eb180->0x1eb1b8/ConditionalFalse 0x1eb180->0x1eb1d0/ConditionalTrue 0x1eb1b8->0x1eb1cc/ConditionalFalse 0x1eb1b8->0x1eb1d0/ConditionalTrue 0x1eb1cc->0x1eb1d0/Fallthrough 0x1eb1d0->0x1eb208/ConditionalFalse 0x1eb1d0->0x1eb210/ConditionalTrue 0x1eb208->0x1eb210/Fallthrough 0x1eb210->0x1eb248/ConditionalFalse 0x1eb210->0x1eb260/ConditionalTrue 0x1eb248->0x1eb25c/ConditionalFalse 0x1eb248->0x1eb260/ConditionalTrue 0x1eb25c->0x1eb260/Fallthrough 0x1eb260->0x1eb2f4/ConditionalFalse 0x1eb260->0x1eb30c/ConditionalTrue 0x1eb2f4->0x1eb308/ConditionalFalse 0x1eb2f4->0x1eb30c/ConditionalTrue 0x1eb308->0x1eb30c/Fallthrough 0x1eb30c->0x1eb384/ConditionalFalse 0x1eb30c->0x1eb39c/ConditionalTrue 0x1eb384->0x1eb398/ConditionalFalse 0x1eb384->0x1eb39c/ConditionalTrue 0x1eb398->0x1eb39c/Fallthrough 0x1eb39c->0x1eb3b4/ConditionalFalse 0x1eb39c->0x1eb890/ConditionalTrue 0x1eb3b4->0x1eb3f8/ConditionalFalse 0x1eb3b4->0x1eb410/ConditionalTrue 0x1eb3f8->0x1eb40c/ConditionalFalse 0x1eb3f8->0x1eb410/ConditionalTrue 0x1eb40c->0x1eb410/Fallthrough 0x1eb410->0x1eb464/ConditionalFalse 0x1eb410->0x1eb47c/ConditionalTrue 0x1eb464->0x1eb478/ConditionalFalse 0x1eb464->0x1eb47c/ConditionalTrue 0x1eb478->0x1eb47c/Fallthrough 0x1eb47c->0x1eb4b8/ConditionalFalse 0x1eb47c->0x1eb4d0/ConditionalTrue 0x1eb4b8->0x1eb4cc/ConditionalFalse 0x1eb4b8->0x1eb4d0/ConditionalTrue 0x1eb4cc->0x1eb4d0/Fallthrough 0x1eb4d0->0x1eb4e8/ConditionalFalse 0x1eb4d0->0x1eb4f0/ConditionalTrue 0x1eb4e8->0x1eb4f0/ConditionalFalse 0x1eb4e8->0x1eb4f8/ConditionalTrue 0x1eb4f0->0x1eb4fc/Branch 0x1eb4f8->0x1eb4fc/Fallthrough 0x1eb4fc->0x1eb540/ConditionalFalse 0x1eb4fc->0x1eb558/ConditionalTrue 0x1eb540->0x1eb554/ConditionalFalse 0x1eb540->0x1eb558/ConditionalTrue 0x1eb554->0x1eb558/Fallthrough 0x1eb558->0x1eb594/ConditionalFalse 0x1eb558->0x1eb5ac/ConditionalTrue 0x1eb594->0x1eb5a8/ConditionalFalse 0x1eb594->0x1eb5ac/ConditionalTrue 0x1eb5a8->0x1eb5ac/Fallthrough 0x1eb5ac->0x1eb634/ConditionalFalse 0x1eb5ac->0x1eb64c/ConditionalTrue 0x1eb634->0x1eb648/ConditionalFalse 0x1eb634->0x1eb64c/ConditionalTrue 0x1eb648->0x1eb64c/Fallthrough 0x1eb64c->0x1eb65c/ConditionalFalse 0x1eb64c->0x1eb664/ConditionalTrue 0x1eb65c->0x1eb668/Branch 0x1eb664->0x1eb668/Fallthrough 0x1eb668->0x1eb670/ConditionalFalse 0x1eb668->0x1eb678/ConditionalTrue 0x1eb670->0x1eb67c/Branch 0x1eb678->0x1eb67c/Fallthrough 0x1eb67c->0x1eb694/ConditionalFalse 0x1eb67c->0x1eb898/ConditionalTrue 0x1eb694->0x1eb6b0/ConditionalFalse 0x1eb694->0x1eb6b8/ConditionalTrue 0x1eb6b0->0x1eb6b8/Fallthrough 0x1eb6b8->0x1eb6f0/ConditionalFalse 0x1eb6b8->0x1eb708/ConditionalTrue 0x1eb6f0->0x1eb704/ConditionalFalse 0x1eb6f0->0x1eb708/ConditionalTrue 0x1eb704->0x1eb708/Fallthrough 0x1eb708->0x1eb73c/ConditionalFalse 0x1eb708->0x1eb744/ConditionalTrue 0x1eb73c->0x1eb748/Branch 0x1eb744->0x1eb748/Fallthrough 0x1eb748->0x1eb780/ConditionalFalse 0x1eb748->0x1eb798/ConditionalTrue 0x1eb780->0x1eb794/ConditionalFalse 0x1eb780->0x1eb798/ConditionalTrue 0x1eb794->0x1eb798/Fallthrough 0x1eb840->0x1ea888/Branch 0x1eb848->0x1ea954/Branch 0x1eb850->0x1eaa14/Branch 0x1eb858->0x1eaa28/Branch 0x1eb860->0x1eacc4/Branch 0x1eb868->0x1eacdc/Branch 0x1eb870->0x1eadcc/Branch 0x1eb878->0x1eae58/Branch 0x1eb880->0x1eae70/Branch 0x1eb888->0x1eae8c/Branch 0x1eb890->0x1eb3b4/Branch 0x1eb898->0x1eb694/Branch

# top_level.e25Intrinsics at 0x1eb8b8 (252 bytes)
0x1eb8b8  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1eb8bc  fd030faa           mov      x29, x15
0x1eb8c0  efa100d1           sub      x15, x15, #0x28
0x1eb8c4  a1831ff8           stur     x1, [x29, #-8]
0x1eb8c8  502740f9           ldr      x16, [x26, #0x48]
0x1eb8cc  ff0110eb           cmp      x15, x16
0x1eb8d0  c9060054           b.ls     #0x1eb9a8
0x1eb8d4  70234091           add      x16, x27, #8, lsl #12
0x1eb8d8  108e44f9           ldr      x16, [x16, #0x918]  # pool[4385] = "ey"
0x1eb8dc  f00500a9           stp      x16, x1, [x15]
0x1eb8e0  da14fd97           bl       #0x130c48
0x1eb8e4  a0831ef8           stur     x0, [x29, #-0x18]
0x1eb8e8  037040b8           ldur     w3, [x0, #7]
0x1eb8ec  a3031ff8           stur     x3, [x29, #-0x10]
0x1eb8f0  a3000035           cbnz     w3, #0x1eb904
0x1eb8f4  c0c20091           add      x0, x22, #0x30
0x1eb8f8  ef031daa           mov      x15, x29
0x1eb8fc  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1eb900  c0035fd6           ret      
0x1eb904  e10300aa           mov      x1, x0
0x1eb908  62234091           add      x2, x27, #8, lsl #12
0x1eb90c  422444f9           ldr      x2, [x2, #0x848]  # pool[4359] = snapshotRef(870)
0x1eb910  64a340f9           ldr      x4, [x27, #0x140]  # pool[38] = snapshotRef(34541)
0x1eb914  0725fd97           bl       #0x134d30
0x1eb918  a0002036           tbz      w0, #4, #0x1eb92c
0x1eb91c  c0c20091           add      x0, x22, #0x30
0x1eb920  ef031daa           mov      x15, x29
0x1eb924  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1eb928  c0035fd6           ret      
0x1eb92c  a2835ef8           ldur     x2, [x29, #-0x18]
0x1eb930  a3035ff8           ldur     x3, [x29, #-0x10]
0x1eb934  607c4193           sbfx     x0, x3, #1, #0x1f
0x1eb938  010080d2           mov      x1, #0
0x1eb93c  3f0000eb           cmp      x1, x0
0x1eb940  82030054           b.hs     #0x1eb9b0
0x1eb944  41f05ff8           ldur     x1, [x2, #-1]
0x1eb948  217c4cd3           ubfx     x1, x1, #0xc, #0x14
0x1eb94c  21f87fd3           lsl      x1, x1, #1
0x1eb950  3ff00271           cmp      w1, #0xbc
0x1eb954  a1000054           b.ne     #0x1eb968
0x1eb958  413c4039           ldrb     w1, [x2, #0xf]
0x1eb95c  3fac01f1           cmp      x1, #0x6b
0x1eb960  c1010054           b.ne     #0x1eb998
0x1eb964  04000014           b        #0x1eb974
0x1eb968  41f04078           ldurh    w1, [x2, #0xf]
0x1eb96c  3fac01f1           cmp      x1, #0x6b
0x1eb970  41010054           b.ne     #0x1eb998
0x1eb974  a1835ff8           ldur     x1, [x29, #-8]
0x1eb978  70234091           add      x16, x27, #8, lsl #12
0x1eb97c  108e44f9           ldr      x16, [x16, #0x918]  # pool[4385] = "ey"
0x1eb980  3f00106b           cmp      w1, w16
0x1eb984  d0820091           add      x16, x22, #0x20
0x1eb988  d1c20091           add      x17, x22, #0x30
0x1eb98c  0202919a           csel     x2, x16, x17, eq
0x1eb990  e00302aa           mov      x0, x2
0x1eb994  02000014           b        #0x1eb99c
0x1eb998  c0c20091           add      x0, x22, #0x30
0x1eb99c  ef031daa           mov      x15, x29
0x1eb9a0  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1eb9a4  c0035fd6           ret      
0x1eb9a8  96560394           bl       #0x2c1400
0x1eb9ac  caffff17           b        #0x1eb8d4
0x1eb9b0  e0570394           bl       #0x2c1930
# CFG: 0x1eb8b8->0x1eb8d4/ConditionalFalse 0x1eb8b8->0x1eb9a8/ConditionalTrue 0x1eb8d4->0x1eb8f4/ConditionalFalse 0x1eb8d4->0x1eb904/ConditionalTrue 0x1eb904->0x1eb91c/ConditionalFalse 0x1eb904->0x1eb92c/ConditionalTrue 0x1eb92c->0x1eb944/ConditionalFalse 0x1eb92c->0x1eb9b0/ConditionalTrue 0x1eb944->0x1eb958/ConditionalFalse 0x1eb944->0x1eb968/ConditionalTrue 0x1eb958->0x1eb964/ConditionalFalse 0x1eb958->0x1eb998/ConditionalTrue 0x1eb964->0x1eb974/Branch 0x1eb968->0x1eb974/ConditionalFalse 0x1eb968->0x1eb998/ConditionalTrue 0x1eb974->0x1eb99c/Branch 0x1eb998->0x1eb99c/Fallthrough 0x1eb9a8->0x1eb8d4/Branch

# top_level.e24Knot at 0x1eb9b4 (164 bytes)
0x1eb9b4  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1eb9b8  fd030faa           mov      x29, x15
0x1eb9bc  41012037           tbnz     w1, #4, #0x1eb9e4
0x1eb9c0  a2002037           tbnz     w2, #4, #0x1eb9d4
0x1eb9c4  200080d2           mov      x0, #1
0x1eb9c8  ef031daa           mov      x15, x29
0x1eb9cc  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1eb9d0  c0035fd6           ret      
0x1eb9d4  400080d2           mov      x0, #2
0x1eb9d8  ef031daa           mov      x15, x29
0x1eb9dc  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1eb9e0  c0035fd6           ret      
0x1eb9e4  a301f8b6           tbz      x3, #0x3f, #0x1eba18
0x1eb9e8  e10303aa           mov      x1, x3
0x1eb9ec  502740f9           ldr      x16, [x26, #0x48]
0x1eb9f0  ff0110eb           cmp      x15, x16
0x1eb9f4  a9020054           b.ls     #0x1eba48
0x1eb9f8  81000036           tbz      w1, #0, #0x1eba08
0x1eb9fc  e00301cb           neg      x0, x1
0x1eba00  e10300aa           mov      x1, x0
0x1eba04  faffff17           b        #0x1eb9ec
0x1eba08  600080d2           mov      x0, #3
0x1eba0c  ef031daa           mov      x15, x29
0x1eba10  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1eba14  c0035fd6           ret      
0x1eba18  e10303aa           mov      x1, x3
0x1eba1c  502740f9           ldr      x16, [x26, #0x48]
0x1eba20  ff0110eb           cmp      x15, x16
0x1eba24  69010054           b.ls     #0x1eba50
0x1eba28  20fc4193           asr      x0, x1, #1
0x1eba2c  1f1000f1           cmp      x0, #4
0x1eba30  6d000054           b.le     #0x1eba3c
0x1eba34  e10300aa           mov      x1, x0
0x1eba38  f9ffff17           b        #0x1eba1c
0x1eba3c  ef031daa           mov      x15, x29
0x1eba40  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1eba44  c0035fd6           ret      
0x1eba48  6e560394           bl       #0x2c1400
0x1eba4c  ebffff17           b        #0x1eb9f8
0x1eba50  6c560394           bl       #0x2c1400
0x1eba54  f5ffff17           b        #0x1eba28
# CFG: 0x1eb9b4->0x1eb9c0/ConditionalFalse 0x1eb9b4->0x1eb9e4/ConditionalTrue 0x1eb9c0->0x1eb9c4/ConditionalFalse 0x1eb9c0->0x1eb9d4/ConditionalTrue 0x1eb9e4->0x1eb9e8/ConditionalFalse 0x1eb9e4->0x1eba18/ConditionalTrue 0x1eb9e8->0x1eb9ec/Fallthrough 0x1eb9ec->0x1eb9f8/ConditionalFalse 0x1eb9ec->0x1eba48/ConditionalTrue 0x1eb9f8->0x1eb9fc/ConditionalFalse 0x1eb9f8->0x1eba08/ConditionalTrue 0x1eb9fc->0x1eb9ec/Branch 0x1eba18->0x1eba1c/Fallthrough 0x1eba1c->0x1eba28/ConditionalFalse 0x1eba1c->0x1eba50/ConditionalTrue 0x1eba28->0x1eba34/ConditionalFalse 0x1eba28->0x1eba3c/ConditionalTrue 0x1eba34->0x1eba1c/Branch 0x1eba48->0x1eb9f8/Branch 0x1eba50->0x1eba28/Branch

# top_level.e23DynamicApply at 0x1eba58 (92 bytes)
0x1eba58  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1eba5c  fd030faa           mov      x29, x15
0x1eba60  ef2100d1           sub      x15, x15, #8
0x1eba64  502740f9           ldr      x16, [x26, #0x48]
0x1eba68  ff0110eb           cmp      x15, x16
0x1eba6c  09020054           b.ls     #0x1ebaac
0x1eba70  11000094           bl       #0x1ebab4
0x1eba74  810780d2           mov      x1, #0x3c
0x1eba78  60000036           tbz      w0, #0, #0x1eba84
0x1eba7c  01f05ff8           ldur     x1, [x0, #-1]
0x1eba80  217c4cd3           ubfx     x1, x1, #0xc, #0x14
0x1eba84  e00100f9           str      x0, [x15]
0x1eba88  e00301aa           mov      x0, x1
0x1eba8c  644741f9           ldr      x4, [x27, #0x288]  # pool[79] = snapshotRef(22)
0x1eba90  91e682d2           mov      x17, #0x1734
0x1eba94  1e00118b           add      x30, x0, x17
0x1eba98  be7a7ef8           ldr      x30, [x21, x30, lsl #3]
0x1eba9c  c0033fd6           blr      x30
0x1ebaa0  ef031daa           mov      x15, x29
0x1ebaa4  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1ebaa8  c0035fd6           ret      
0x1ebaac  55560394           bl       #0x2c1400
0x1ebab0  f0ffff17           b        #0x1eba70
# CFG: 0x1eba58->0x1eba70/ConditionalFalse 0x1eba58->0x1ebaac/ConditionalTrue 0x1eba70->0x1eba7c/ConditionalFalse 0x1eba70->0x1eba84/ConditionalTrue 0x1eba7c->0x1eba84/Fallthrough 0x1ebaac->0x1eba70/Branch

# E21Mode.parse at 0x1ebe28 (64 bytes)
0x1ebe28  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1ebe2c  fd030faa           mov      x29, x15
0x1ebe30  ef4100d1           sub      x15, x15, #0x10
0x1ebe34  502740f9           ldr      x16, [x26, #0x48]
0x1ebe38  ff0110eb           cmp      x15, x16
0x1ebe3c  29010054           b.ls     #0x1ebe60
0x1ebe40  70234091           add      x16, x27, #8, lsl #12
0x1ebe44  10aa44f9           ldr      x16, [x16, #0x950]  # pool[4392] = snapshotRef(18106)
0x1ebe48  e14100a9           stp      x1, x16, [x15]
0x1ebe4c  640f44f9           ldr      x4, [x27, #0x818]  # pool[257] = snapshotRef(54)
0x1ebe50  06000094           bl       #0x1ebe68
0x1ebe54  ef031daa           mov      x15, x29
0x1ebe58  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1ebe5c  c0035fd6           ret      
0x1ebe60  68550394           bl       #0x2c1400
0x1ebe64  f7ffff17           b        #0x1ebe40
# CFG: 0x1ebe28->0x1ebe40/ConditionalFalse 0x1ebe28->0x1ebe60/ConditionalTrue 0x1ebe60->0x1ebe40/Branch

# E20Combo.greet at 0x1ebf38 (12 bytes)
0x1ebf38  60234091           add      x0, x27, #8, lsl #12
0x1ebf3c  00b844f9           ldr      x0, [x0, #0x970]  # pool[4396] = "base+combo"
0x1ebf40  c0035fd6           ret      

# package:edge_probe/probe_code.dart.E20Combo at 0x1ebf44 (12 bytes)
0x1ebf44  822380d2           mov      x2, #0x11c
0x1ebf48  2209a0f2           movk     x2, #0x49, lsl #16
0x1ebf4c  8a500314           b        #0x2c0174

# top_level.e18NumericEdges at 0x1ebf50 (524 bytes)
0x1ebf50  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1ebf54  fd030faa           mov      x29, x15
0x1ebf58  ef8100d1           sub      x15, x15, #0x20
0x1ebf5c  011ca04e           mov      v1.16b, v0.16b
0x1ebf60  a0831efc           stur     d0, [x29, #-0x18]
0x1ebf64  502740f9           ldr      x16, [x26, #0x48]
0x1ebf68  ff0110eb           cmp      x15, x16
0x1ebf6c  090d0054           b.ls     #0x1ec10c
0x1ebf70  2020611e           fcmp     d1, d1
0x1ebf74  c7000054           b.vc     #0x1ebf8c
0x1ebf78  60234091           add      x0, x27, #8, lsl #12
0x1ebf7c  00bc44f9           ldr      x0, [x0, #0x978]  # pool[4397] = "nan"
0x1ebf80  ef031daa           mov      x15, x29
0x1ebf84  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1ebf88  c0035fd6           ret      
0x1ebf8c  201ca14e           mov      v0.16b, v1.16b
0x1ebf90  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1ebf94  fd030faa           mov      x29, x15
0x1ebf98  efed7c92           and      x15, x15, #0xfffffffffffffff0
0x1ebf9c  ff010091           mov      sp, x15
0x1ebfa0  50b743f9           ldr      x16, [x26, #0x768]
0x1ebfa4  506703f9           str      x16, [x26, #0x6c8]
0x1ebfa8  00023fd6           blr      x16
0x1ebfac  100180d2           mov      x16, #8
0x1ebfb0  506703f9           str      x16, [x26, #0x6c8]
0x1ebfb4  504b43f9           ldr      x16, [x26, #0x690]
0x1ebfb8  1f0640d1           sub      sp, x16, #1, lsl #12
0x1ebfbc  ef031daa           mov      x15, x29
0x1ebfc0  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1ebfc4  011ca04e           mov      v1.16b, v0.16b
0x1ebfc8  a0835efc           ldur     d0, [x29, #-0x18]
0x1ebfcc  0020611e           fcmp     d0, d1
0x1ebfd0  a1020054           b.ne     #0x1ec024
0x1ebfd4  e10316aa           mov      x1, x22
0x1ebfd8  820080d2           mov      x2, #4
0x1ebfdc  c6540394           bl       #0x2c12f4
0x1ebfe0  70234091           add      x16, x27, #8, lsl #12
0x1ebfe4  10c244f9           ldr      x16, [x16, #0x980]  # pool[4398] = "integral:"
0x1ebfe8  10f000b8           stur     w16, [x0, #0xf]
0x1ebfec  a1835efc           ldur     d1, [x29, #-0x18]
0x1ebff0  2020611e           fcmp     d1, d1
0x1ebff4  06090054           b.vs     #0x1ec114
0x1ebff8  2100789e           fcvtzs   x1, d1
0x1ebffc  30fc5e93           asr      x16, x1, #0x1e
0x1ec000  1ffe81eb           cmp      x16, x1, asr #63
0x1ec004  81080054           b.ne     #0x1ec114
0x1ec008  21f87fd3           lsl      x1, x1, #1
0x1ec00c  013001b8           stur     w1, [x0, #0x13]
0x1ec010  e00100f9           str      x0, [x15]
0x1ec014  5a12fd97           bl       #0x13097c
0x1ec018  ef031daa           mov      x15, x29
0x1ec01c  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1ec020  c0035fd6           ret      
0x1ec024  011ca04e           mov      v1.16b, v0.16b
0x1ec028  201ca14e           mov      v0.16b, v1.16b
0x1ec02c  e10316aa           mov      x1, x22
0x1ec030  a5010094           bl       #0x1ec6c4
0x1ec034  e10316aa           mov      x1, x22
0x1ec038  020180d2           mov      x2, #8
0x1ec03c  a0831ff8           stur     x0, [x29, #-8]
0x1ec040  ad540394           bl       #0x2c12f4
0x1ec044  a0031ff8           stur     x0, [x29, #-0x10]
0x1ec048  70234091           add      x16, x27, #8, lsl #12
0x1ec04c  10c644f9           ldr      x16, [x16, #0x988]  # pool[4399] = "frac:"
0x1ec050  10f000b8           stur     w16, [x0, #0xf]
0x1ec054  a1835ff8           ldur     x1, [x29, #-8]
0x1ec058  020280d2           mov      x2, #0x10
0x1ec05c  89000094           bl       #0x1ec280
0x1ec060  a1035ff8           ldur     x1, [x29, #-0x10]
0x1ec064  394c0091           add      x25, x1, #0x13
0x1ec068  200300b9           str      w0, [x25]
0x1ec06c  e0000036           tbz      w0, #0, #0x1ec088
0x1ec070  30f05f38           ldurb    w16, [x1, #-1]
0x1ec074  11f05f38           ldurb    w17, [x0, #-1]
0x1ec078  300a508a           and      x16, x17, x16, lsr #2
0x1ec07c  1f825cea           tst      x16, x28, lsr #32
0x1ec080  40000054           b.eq     #0x1ec088
0x1ec084  264d0394           bl       #0x2bf51c
0x1ec088  a0035ff8           ldur     x0, [x29, #-0x10]
0x1ec08c  70e351f9           ldr      x16, [x27, #0x23c0]  # pool[1142] = snapshotRef(758)
0x1ec090  107001b8           stur     w16, [x0, #0x17]
0x1ec094  a0835efc           ldur     d0, [x29, #-0x18]
0x1ec098  410b46a9           ldp      x1, x2, [x26, #0x60]
0x1ec09c  21400091           add      x1, x1, #0x10
0x1ec0a0  5f0001eb           cmp      x2, x1
0x1ec0a4  e9040054           b.ls     #0x1ec140
0x1ec0a8  413300f9           str      x1, [x26, #0x60]
0x1ec0ac  213c00d1           sub      x1, x1, #0xf
0x1ec0b0  82339cd2           mov      x2, #0xe19c
0x1ec0b4  6200a0f2           movk     x2, #3, lsl #16
0x1ec0b8  22f01ff8           stur     x2, [x1, #-1]
0x1ec0bc  bf3a03d5           dmb      ishst
0x1ec0c0  207000fc           stur     d0, [x1, #7]
0x1ec0c4  420080d2           mov      x2, #2
0x1ec0c8  25000094           bl       #0x1ec15c
0x1ec0cc  a1035ff8           ldur     x1, [x29, #-0x10]
0x1ec0d0  396c0091           add      x25, x1, #0x1b
0x1ec0d4  200300b9           str      w0, [x25]
0x1ec0d8  e0000036           tbz      w0, #0, #0x1ec0f4
0x1ec0dc  30f05f38           ldurb    w16, [x1, #-1]
0x1ec0e0  11f05f38           ldurb    w17, [x0, #-1]
0x1ec0e4  300a508a           and      x16, x17, x16, lsr #2
0x1ec0e8  1f825cea           tst      x16, x28, lsr #32
0x1ec0ec  40000054           b.eq     #0x1ec0f4
0x1ec0f0  0b4d0394           bl       #0x2bf51c
0x1ec0f4  b0035ff8           ldur     x16, [x29, #-0x10]
0x1ec0f8  f00100f9           str      x16, [x15]
0x1ec0fc  2012fd97           bl       #0x13097c
0x1ec100  ef031daa           mov      x15, x29
0x1ec104  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1ec108  c0035fd6           ret      
0x1ec10c  dd540394           bl       #0x2c1480
0x1ec110  98ffff17           b        #0x1ebf70
0x1ec114  e10d9f3c           str      q1, [x15, #-0x10]!
0x1ec118  e08d1ff8           str      x0, [x15, #-8]!
0x1ec11c  2040601e           fmov     d0, d1
0x1ec120  800980d2           mov      x0, #0x4c
0x1ec124  7e2b53f9           ldr      x30, [x27, #0x2650]  # pool[1224] = snapshotRef(951)
0x1ec128  de7340f8           ldur     x30, [x30, #7]
0x1ec12c  c0033fd6           blr      x30
0x1ec130  e10300aa           mov      x1, x0
0x1ec134  e08540f8           ldr      x0, [x15], #8
0x1ec138  e105c13c           ldr      q1, [x15], #0x10
0x1ec13c  b4ffff17           b        #0x1ec00c
0x1ec140  e00d9f3c           str      q0, [x15, #-0x10]!
0x1ec144  e08d1ff8           str      x0, [x15, #-8]!
0x1ec148  3f540394           bl       #0x2c1244
0x1ec14c  e10300aa           mov      x1, x0
0x1ec150  e08540f8           ldr      x0, [x15], #8
0x1ec154  e005c13c           ldr      q0, [x15], #0x10
0x1ec158  daffff17           b        #0x1ec0c0
# CFG: 0x1ebf50->0x1ebf70/ConditionalFalse 0x1ebf50->0x1ec10c/ConditionalTrue 0x1ebf70->0x1ebf78/ConditionalFalse 0x1ebf70->0x1ebf8c/ConditionalTrue 0x1ebf8c->0x1ebfd4/ConditionalFalse 0x1ebf8c->0x1ec024/ConditionalTrue 0x1ebfd4->0x1ebff8/ConditionalFalse 0x1ebfd4->0x1ec114/ConditionalTrue 0x1ebff8->0x1ec008/ConditionalFalse 0x1ebff8->0x1ec114/ConditionalTrue 0x1ec008->0x1ec00c/Fallthrough 0x1ec024->0x1ec070/ConditionalFalse 0x1ec024->0x1ec088/ConditionalTrue 0x1ec070->0x1ec084/ConditionalFalse 0x1ec070->0x1ec088/ConditionalTrue 0x1ec084->0x1ec088/Fallthrough 0x1ec088->0x1ec0a8/ConditionalFalse 0x1ec088->0x1ec140/ConditionalTrue 0x1ec0a8->0x1ec0c0/Fallthrough 0x1ec0c0->0x1ec0dc/ConditionalFalse 0x1ec0c0->0x1ec0f4/ConditionalTrue 0x1ec0dc->0x1ec0f0/ConditionalFalse 0x1ec0dc->0x1ec0f4/ConditionalTrue 0x1ec0f0->0x1ec0f4/Fallthrough 0x1ec10c->0x1ebf70/Branch 0x1ec114->0x1ec00c/Branch 0x1ec140->0x1ec0c0/Branch

# top_level.e17JsonRoundTrip at 0x1ede10 (244 bytes)
0x1ede10  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1ede14  fd030faa           mov      x29, x15
0x1ede18  ef8100d1           sub      x15, x15, #0x20
0x1ede1c  502740f9           ldr      x16, [x26, #0x48]
0x1ede20  ff0110eb           cmp      x15, x16
0x1ede24  c9060054           b.ls     #0x1edefc
0x1ede28  37000094           bl       #0x1edf04
0x1ede2c  e30300aa           mov      x3, x0
0x1ede30  e20316aa           mov      x2, x22
0x1ede34  e10316aa           mov      x1, x22
0x1ede38  a3831ff8           stur     x3, [x29, #-8]
0x1ede3c  840780d2           mov      x4, #0x3c
0x1ede40  60000036           tbz      w0, #0, #0x1ede4c
0x1ede44  04f05ff8           ldur     x4, [x0, #-1]
0x1ede48  847c4cd3           ubfx     x4, x4, #0xc, #0x14
0x1ede4c  846801d1           sub      x4, x4, #0x5a
0x1ede50  9f0800f1           cmp      x4, #2
0x1ede54  89010054           b.ls     #0x1ede84
0x1ede58  845800d1           sub      x4, x4, #0x16
0x1ede5c  9fdc00f1           cmp      x4, #0x37
0x1ede60  29010054           b.ls     #0x1ede84
0x1ede64  9ff81ff1           cmp      x4, #0x7fe
0x1ede68  e0000054           b.eq     #0x1ede84
0x1ede6c  9fe420f1           cmp      x4, #0x839
0x1ede70  a0000054           b.eq     #0x1ede84
0x1ede74  68ef68f9           ldr      x8, [x27, #0x51d8]  # pool[2617] = snapshotRef(17254)
0x1ede78  63234091           add      x3, x27, #8, lsl #12
0x1ede7c  63f444f9           ldr      x3, [x3, #0x9e8]  # pool[4411] = null
0x1ede80  bc440394           bl       #0x2bf170
0x1ede84  a0835ff8           ldur     x0, [x29, #-8]
0x1ede88  01f05ff8           ldur     x1, [x0, #-1]
0x1ede8c  217c4cd3           ubfx     x1, x1, #0xc, #0x14
0x1ede90  709b6ff9           ldr      x16, [x27, #0x5f30]  # pool[3044] = snapshotRef(18167)
0x1ede94  e04100a9           stp      x0, x16, [x15]
0x1ede98  e00301aa           mov      x0, x1
0x1ede9c  640f44f9           ldr      x4, [x27, #0x818]  # pool[257] = snapshotRef(54)
0x1edea0  b10588d2           mov      x17, #0x402d
0x1edea4  1e00118b           add      x30, x0, x17
0x1edea8  be7a7ef8           ldr      x30, [x21, x30, lsl #3]
0x1edeac  c0033fd6           blr      x30
0x1edeb0  61234091           add      x1, x27, #8, lsl #12
0x1edeb4  21fc44f9           ldr      x1, [x1, #0x9f8]  # pool[4413] = <anonymous closure>
0x1edeb8  e20316aa           mov      x2, x22
0x1edebc  a0831ff8           stur     x0, [x29, #-8]
0x1edec0  c6490394           bl       #0x2c05d8
0x1edec4  70234091           add      x16, x27, #8, lsl #12
0x1edec8  100245f9           ldr      x16, [x16, #0xa00]  # pool[4414] = snapshotRef(17976)
0x1edecc  be835ff8           ldur     x30, [x29, #-8]
0x1eded0  fec100a9           stp      x30, x16, [x15, #8]
0x1eded4  e00100f9           str      x0, [x15]
0x1eded8  642340f9           ldr      x4, [x27, #0x40]  # pool[6] = snapshotRef(55)
0x1ededc  f55eff97           bl       #0x1c5ab0
0x1edee0  017040b8           ldur     w1, [x0, #7]
0x1edee4  21801c8b           add      x1, x1, x28, lsl #32
0x1edee8  e20300aa           mov      x2, x0
0x1edeec  5de9fc97           bl       #0x128460
0x1edef0  ef031daa           mov      x15, x29
0x1edef4  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1edef8  c0035fd6           ret      
0x1edefc  414d0394           bl       #0x2c1400
0x1edf00  caffff17           b        #0x1ede28
# CFG: 0x1ede10->0x1ede28/ConditionalFalse 0x1ede10->0x1edefc/ConditionalTrue 0x1ede28->0x1ede44/ConditionalFalse 0x1ede28->0x1ede4c/ConditionalTrue 0x1ede44->0x1ede4c/Fallthrough 0x1ede4c->0x1ede58/ConditionalFalse 0x1ede4c->0x1ede84/ConditionalTrue 0x1ede58->0x1ede64/ConditionalFalse 0x1ede58->0x1ede84/ConditionalTrue 0x1ede64->0x1ede6c/ConditionalFalse 0x1ede64->0x1ede84/ConditionalTrue 0x1ede6c->0x1ede74/ConditionalFalse 0x1ede6c->0x1ede84/ConditionalTrue 0x1ede74->0x1ede84/Fallthrough 0x1edefc->0x1ede28/Branch

# top_level.<anonymous closure> at 0x1edf48 (84 bytes)
0x1edf48  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1edf4c  fd030faa           mov      x29, x15
0x1edf50  ef4100d1           sub      x15, x15, #0x10
0x1edf54  502740f9           ldr      x16, [x26, #0x48]
0x1edf58  ff0110eb           cmp      x15, x16
0x1edf5c  c9010054           b.ls     #0x1edf94
0x1edf60  a00b40f9           ldr      x0, [x29, #0x10]
0x1edf64  01f05ff8           ldur     x1, [x0, #-1]
0x1edf68  217c4cd3           ubfx     x1, x1, #0xc, #0x14
0x1edf6c  705774f9           ldr      x16, [x27, #0x68a8]  # pool[3347] = snapshotRef(17921)
0x1edf70  e04100a9           stp      x0, x16, [x15]
0x1edf74  e00301aa           mov      x0, x1
0x1edf78  646f4ef9           ldr      x4, [x27, #0x1cd8]  # pool[921] = snapshotRef(34545)
0x1edf7c  1efc3cd1           sub      x30, x0, #0xf3f
0x1edf80  be7a7ef8           ldr      x30, [x21, x30, lsl #3]
0x1edf84  c0033fd6           blr      x30
0x1edf88  ef031daa           mov      x15, x29
0x1edf8c  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1edf90  c0035fd6           ret      
0x1edf94  1b4d0394           bl       #0x2c1400
0x1edf98  f2ffff17           b        #0x1edf60
# CFG: 0x1edf48->0x1edf60/ConditionalFalse 0x1edf48->0x1edf94/ConditionalTrue 0x1edf94->0x1edf60/Branch

# top_level.e16SortedCopy at 0x1edf9c (108 bytes)
0x1edf9c  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1edfa0  fd030faa           mov      x29, x15
0x1edfa4  ef4100d1           sub      x15, x15, #0x10
0x1edfa8  e20301aa           mov      x2, x1
0x1edfac  a1831ff8           stur     x1, [x29, #-8]
0x1edfb0  502740f9           ldr      x16, [x26, #0x48]
0x1edfb4  ff0110eb           cmp      x15, x16
0x1edfb8  49020054           b.ls     #0x1ee000
0x1edfbc  61234091           add      x1, x27, #8, lsl #12
0x1edfc0  210845f9           ldr      x1, [x1, #0xa10]  # pool[4416] = snapshotRef(18544)
0x1edfc4  eb030094           bl       #0x1eef70
0x1edfc8  e10300aa           mov      x1, x0
0x1edfcc  e20316aa           mov      x2, x22
0x1edfd0  a0031ff8           stur     x0, [x29, #-0x10]
0x1edfd4  76030094           bl       #0x1eedac
0x1edfd8  a1035ff8           ldur     x1, [x29, #-0x10]
0x1edfdc  a2835ff8           ldur     x2, [x29, #-8]
0x1edfe0  0a000094           bl       #0x1ee008
0x1edfe4  a2035ff8           ldur     x2, [x29, #-0x10]
0x1edfe8  61234091           add      x1, x27, #8, lsl #12
0x1edfec  210845f9           ldr      x1, [x1, #0xa10]  # pool[4416] = snapshotRef(18544)
0x1edff0  cb93fd97           bl       #0x152f1c
0x1edff4  ef031daa           mov      x15, x29
0x1edff8  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1edffc  c0035fd6           ret      
0x1ee000  004d0394           bl       #0x2c1400
0x1ee004  eeffff17           b        #0x1edfbc
# CFG: 0x1edf9c->0x1edfbc/ConditionalFalse 0x1edf9c->0x1ee000/ConditionalTrue 0x1ee000->0x1edfbc/Branch

# package:edge_probe/probe_code.dart.E15Vec at 0x1eef7c (12 bytes)
0x1eef7c  824388d2           mov      x2, #0x421c
0x1eef80  e205a0f2           movk     x2, #0x2f, lsl #16
0x1eef84  7c440314           b        #0x2c0174

# E14Statics.bump at 0x1eef88 (168 bytes)
0x1eef88  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1eef8c  fd030faa           mov      x29, x15
0x1eef90  ef2100d1           sub      x15, x15, #8
0x1eef94  502740f9           ldr      x16, [x26, #0x48]
0x1eef98  ff0110eb           cmp      x15, x16
0x1eef9c  69040054           b.ls     #0x1ef028
0x1eefa0  403f40f9           ldr      x0, [x26, #0x78]
0x1eefa4  00d443f9           ldr      x0, [x0, #0x7a8]
0x1eefa8  017c4193           sbfx     x1, x0, #1, #0x1f
0x1eefac  40000036           tbz      w0, #0, #0x1eefb4
0x1eefb0  017040f8           ldur     x1, [x0, #7]
0x1eefb4  23040091           add      x3, x1, #1
0x1eefb8  a3831ff8           stur     x3, [x29, #-8]
0x1eefbc  60787f93           sbfiz    x0, x3, #1, #0x1f
0x1eefc0  7f0480eb           cmp      x3, x0, asr #1
0x1eefc4  60000054           b.eq     #0x1eefd0
0x1eefc8  6e490394           bl       #0x2c1580
0x1eefcc  037000f8           stur     x3, [x0, #7]
0x1eefd0  e20300aa           mov      x2, x0
0x1eefd4  403f40f9           ldr      x0, [x26, #0x78]
0x1eefd8  02d403f9           str      x2, [x0, #0x7a8]
0x1eefdc  403f40f9           ldr      x0, [x26, #0x78]
0x1eefe0  00d043f9           ldr      x0, [x0, #0x7a0]
0x1eefe4  504b40f9           ldr      x16, [x26, #0x90]
0x1eefe8  1f00106b           cmp      w0, w16
0x1eefec  81000054           b.ne     #0x1eeffc
0x1eeff0  62234091           add      x2, x27, #8, lsl #12
0x1eeff4  42e445f9           ldr      x2, [x2, #0xbc8]  # pool[4471] = E14Statics.stamp
0x1eeff8  f0400394           bl       #0x2bf3b8
0x1eeffc  017c4193           sbfx     x1, x0, #1, #0x1f
0x1ef000  40000036           tbz      w0, #0, #0x1ef008
0x1ef004  017040f8           ldur     x1, [x0, #7]
0x1ef008  22000012           and      w2, w1, #1
0x1ef00c  427c40d3           ubfx     x2, x2, #0, #0x20
0x1ef010  a1835ff8           ldur     x1, [x29, #-8]
0x1ef014  2300028b           add      x3, x1, x2
0x1ef018  600c0091           add      x0, x3, #3
0x1ef01c  ef031daa           mov      x15, x29
0x1ef020  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1ef024  c0035fd6           ret      
0x1ef028  f6480394           bl       #0x2c1400
0x1ef02c  ddffff17           b        #0x1eefa0
# CFG: 0x1eef88->0x1eefa0/ConditionalFalse 0x1eef88->0x1ef028/ConditionalTrue 0x1eefa0->0x1eefb0/ConditionalFalse 0x1eefa0->0x1eefb4/ConditionalTrue 0x1eefb0->0x1eefb4/Fallthrough 0x1eefb4->0x1eefc8/ConditionalFalse 0x1eefb4->0x1eefd0/ConditionalTrue 0x1eefc8->0x1eefd0/Fallthrough 0x1eefd0->0x1eeff0/ConditionalFalse 0x1eefd0->0x1eeffc/ConditionalTrue 0x1eeff0->0x1eeffc/Fallthrough 0x1eeffc->0x1ef004/ConditionalFalse 0x1eeffc->0x1ef008/ConditionalTrue 0x1ef004->0x1ef008/Fallthrough 0x1ef028->0x1eefa0/Branch

# E14Statics.init:stamp at 0x1ef030 (44 bytes)
0x1ef030  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1ef034  fd030faa           mov      x29, x15
0x1ef038  502740f9           ldr      x16, [x26, #0x48]
0x1ef03c  ff0110eb           cmp      x15, x16
0x1ef040  a9000054           b.ls     #0x1ef054
0x1ef044  1f000094           bl       #0x1ef0c0
0x1ef048  ef031daa           mov      x15, x29
0x1ef04c  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1ef050  c0035fd6           ret      
0x1ef054  eb480394           bl       #0x2c1400
0x1ef058  fbffff17           b        #0x1ef044
# CFG: 0x1ef030->0x1ef044/ConditionalFalse 0x1ef030->0x1ef054/ConditionalTrue 0x1ef054->0x1ef044/Branch

# E13Dynamic.probe at 0x1ef13c (72 bytes)
0x1ef13c  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1ef140  fd030faa           mov      x29, x15
0x1ef144  ef2100d1           sub      x15, x15, #8
0x1ef148  502740f9           ldr      x16, [x26, #0x48]
0x1ef14c  ff0110eb           cmp      x15, x16
0x1ef150  69010054           b.ls     #0x1ef17c
0x1ef154  e20100f9           str      x2, [x15]
0x1ef158  040080d2           mov      x4, #0
0x1ef15c  e00140f9           ldr      x0, [x15]
0x1ef160  70234091           add      x16, x27, #8, lsl #12
0x1ef164  10c22f91           add      x16, x16, #0xbf0
0x1ef168  1e1640a9           ldp      x30, x5, [x16]
0x1ef16c  c0033fd6           blr      x30
0x1ef170  ef031daa           mov      x15, x29
0x1ef174  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1ef178  c0035fd6           ret      
0x1ef17c  a1480394           bl       #0x2c1400
0x1ef180  f5ffff17           b        #0x1ef154
# CFG: 0x1ef13c->0x1ef154/ConditionalFalse 0x1ef13c->0x1ef17c/ConditionalTrue 0x1ef17c->0x1ef154/Branch

# package:edge_probe/probe_code.dart.E13Dynamic at 0x1ef184 (12 bytes)
0x1ef184  82238cd2           mov      x2, #0x611c
0x1ef188  e205a0f2           movk     x2, #0x2f, lsl #16
0x1ef18c  fa430314           b        #0x2c0174

# top_level.e12TearOffs at 0x1ef190 (280 bytes)
0x1ef190  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1ef194  fd030faa           mov      x29, x15
0x1ef198  ef6100d1           sub      x15, x15, #0x18
0x1ef19c  800080d2           mov      x0, #4
0x1ef1a0  502740f9           ldr      x16, [x26, #0x48]
0x1ef1a4  ff0110eb           cmp      x15, x16
0x1ef1a8  c9070054           b.ls     #0x1ef2a0
0x1ef1ac  e20300aa           mov      x2, x0
0x1ef1b0  e10316aa           mov      x1, x22
0x1ef1b4  50480394           bl       #0x2c12f4
0x1ef1b8  a0831ff8           stur     x0, [x29, #-8]
0x1ef1bc  d00080d2           mov      x16, #6
0x1ef1c0  10f000b8           stur     w16, [x0, #0xf]
0x1ef1c4  500080d2           mov      x16, #2
0x1ef1c8  103001b8           stur     w16, [x0, #0x13]
0x1ef1cc  610b44f9           ldr      x1, [x27, #0x810]  # pool[256] = snapshotRef(18530)
0x1ef1d0  fd430394           bl       #0x2c01c4
0x1ef1d4  e20300aa           mov      x2, x0
0x1ef1d8  a0835ff8           ldur     x0, [x29, #-8]
0x1ef1dc  a2031ff8           stur     x2, [x29, #-0x10]
0x1ef1e0  40f000b8           stur     w0, [x2, #0xf]
0x1ef1e4  800080d2           mov      x0, #4
0x1ef1e8  40b000b8           stur     w0, [x2, #0xb]
0x1ef1ec  e10302aa           mov      x1, x2
0x1ef1f0  8ce9ff97           bl       #0x1e9820
0x1ef1f4  e10316aa           mov      x1, x22
0x1ef1f8  c20180d2           mov      x2, #0xe
0x1ef1fc  a0831ff8           stur     x0, [x29, #-8]
0x1ef200  3d480394           bl       #0x2c12f4
0x1ef204  a0831ef8           stur     x0, [x29, #-0x18]
0x1ef208  900080d2           mov      x16, #4
0x1ef20c  10f000b8           stur     w16, [x0, #0xf]
0x1ef210  a1835ff8           ldur     x1, [x29, #-8]
0x1ef214  013001b8           stur     w1, [x0, #0x13]
0x1ef218  a2035ff8           ldur     x2, [x29, #-0x10]
0x1ef21c  61234091           add      x1, x27, #8, lsl #12
0x1ef220  210046f9           ldr      x1, [x1, #0xc00]  # pool[4478] = ListBase.sort
0x1ef224  630b44f9           ldr      x3, [x27, #0x810]  # pool[256] = snapshotRef(18530)
0x1ef228  7b440394           bl       #0x2c0414
0x1ef22c  e10300aa           mov      x1, x0
0x1ef230  a0835ef8           ldur     x0, [x29, #-0x18]
0x1ef234  017001b8           stur     w1, [x0, #0x17]
0x1ef238  a2035ff8           ldur     x2, [x29, #-0x10]
0x1ef23c  61234091           add      x1, x27, #8, lsl #12
0x1ef240  210446f9           ldr      x1, [x1, #0xc08]  # pool[4479] = _GrowableList.removeLast
0x1ef244  630b44f9           ldr      x3, [x27, #0x810]  # pool[256] = snapshotRef(18530)
0x1ef248  73440394           bl       #0x2c0414
0x1ef24c  e10300aa           mov      x1, x0
0x1ef250  a0835ef8           ldur     x0, [x29, #-0x18]
0x1ef254  01b001b8           stur     w1, [x0, #0x1b]
0x1ef258  70234091           add      x16, x27, #8, lsl #12
0x1ef25c  100a46f9           ldr      x16, [x16, #0xc10]  # pool[4480] = snapshotRef(33556)
0x1ef260  10f001b8           stur     w16, [x0, #0x1f]
0x1ef264  70234091           add      x16, x27, #8, lsl #12
0x1ef268  100e46f9           ldr      x16, [x16, #0xc18]  # pool[4481] = snapshotRef(33558)
0x1ef26c  103002b8           stur     w16, [x0, #0x23]
0x1ef270  70234091           add      x16, x27, #8, lsl #12
0x1ef274  101246f9           ldr      x16, [x16, #0xc20]  # pool[4482] = snapshotRef(33557)
0x1ef278  107002b8           stur     w16, [x0, #0x27]
0x1ef27c  61fb52f9           ldr      x1, [x27, #0x25f0]  # pool[1212] = snapshotRef(18124)
0x1ef280  d1430394           bl       #0x2c01c4
0x1ef284  a1835ef8           ldur     x1, [x29, #-0x18]
0x1ef288  01f000b8           stur     w1, [x0, #0xf]
0x1ef28c  c10180d2           mov      x1, #0xe
0x1ef290  01b000b8           stur     w1, [x0, #0xb]
0x1ef294  ef031daa           mov      x15, x29
0x1ef298  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1ef29c  c0035fd6           ret      
0x1ef2a0  58480394           bl       #0x2c1400
0x1ef2a4  c2ffff17           b        #0x1ef1ac
# CFG: 0x1ef190->0x1ef1ac/ConditionalFalse 0x1ef190->0x1ef2a0/ConditionalTrue 0x1ef2a0->0x1ef1ac/Branch

# top_level.e11SyncGen at 0x1ef814 (284 bytes)
0x1ef814  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1ef818  fd030faa           mov      x29, x15
0x1ef81c  ef8100d1           sub      x15, x15, #0x20
0x1ef820  b6831ff8           stur     x22, [x29, #-8]
0x1ef824  a1031ff8           stur     x1, [x29, #-0x10]
0x1ef828  502740f9           ldr      x16, [x26, #0x48]
0x1ef82c  ff0110eb           cmp      x15, x16
0x1ef830  89070054           b.ls     #0x1ef920
0x1ef834  600b44f9           ldr      x0, [x27, #0x810]  # pool[256] = snapshotRef(18530)
0x1ef838  1ba2fe97           bl       #0x1980a4
0x1ef83c  e00316aa           mov      x0, x22
0x1ef840  b6a1fe97           bl       #0x197f18
0x1ef844  040080d2           mov      x4, #0
0x1ef848  a2035ff8           ldur     x2, [x29, #-0x10]
0x1ef84c  030080d2           mov      x3, #0
0x1ef850  a4831ef8           stur     x4, [x29, #-0x18]
0x1ef854  502740f9           ldr      x16, [x26, #0x48]
0x1ef858  ff0110eb           cmp      x15, x16
0x1ef85c  69060054           b.ls     #0x1ef928
0x1ef860  9f0002eb           cmp      x4, x2
0x1ef864  6a050054           b.ge     #0x1ef910
0x1ef868  a4020037           tbnz     w4, #0, #0x1ef8bc
0x1ef86c  a0cb238b           add      x0, x29, w3, sxtw #2
0x1ef870  00805ff8           ldur     x0, [x0, #-8]
0x1ef874  057041b8           ldur     w5, [x0, #0x17]
0x1ef878  a5801c8b           add      x5, x5, x28, lsl #32
0x1ef87c  80787f93           sbfiz    x0, x4, #1, #0x1f
0x1ef880  9f0480eb           cmp      x4, x0, asr #1
0x1ef884  60000054           b.eq     #0x1ef890
0x1ef888  3e470394           bl       #0x2c1580
0x1ef88c  047000f8           stur     x4, [x0, #7]
0x1ef890  a07001b8           stur     w0, [x5, #0x17]
0x1ef894  e0000036           tbz      w0, #0, #0x1ef8b0
0x1ef898  b0f05f38           ldurb    w16, [x5, #-1]
0x1ef89c  11f05f38           ldurb    w17, [x0, #-1]
0x1ef8a0  300a508a           and      x16, x17, x16, lsr #2
0x1ef8a4  1f825cea           tst      x16, x28, lsr #32
0x1ef8a8  40000054           b.eq     #0x1ef8b0
0x1ef8ac  4d400394           bl       #0x2bf9e0
0x1ef8b0  c0820091           add      x0, x22, #0x20
0x1ef8b4  1f000094           bl       #0x1ef930
0x1ef8b8  13000014           b        #0x1ef904
0x1ef8bc  e00303aa           mov      x0, x3
0x1ef8c0  a1cb208b           add      x1, x29, w0, sxtw #2
0x1ef8c4  21805ff8           ldur     x1, [x1, #-8]
0x1ef8c8  227041b8           ldur     w2, [x1, #0x17]
0x1ef8cc  42801c8b           add      x2, x2, x28, lsl #32
0x1ef8d0  a1835ef8           ldur     x1, [x29, #-0x18]
0x1ef8d4  a2031ef8           stur     x2, [x29, #-0x20]
0x1ef8d8  cfffff97           bl       #0x1ef814
0x1ef8dc  a1035ef8           ldur     x1, [x29, #-0x20]
0x1ef8e0  20b001b8           stur     w0, [x1, #0x1b]
0x1ef8e4  30f05f38           ldurb    w16, [x1, #-1]
0x1ef8e8  11f05f38           ldurb    w17, [x0, #-1]
0x1ef8ec  300a508a           and      x16, x17, x16, lsr #2
0x1ef8f0  1f825cea           tst      x16, x28, lsr #32
0x1ef8f4  40000054           b.eq     #0x1ef8fc
0x1ef8f8  1a400394           bl       #0x2bf960
0x1ef8fc  c0820091           add      x0, x22, #0x20
0x1ef900  0c000094           bl       #0x1ef930
0x1ef904  a1835ef8           ldur     x1, [x29, #-0x18]
0x1ef908  24040091           add      x4, x1, #1
0x1ef90c  cfffff17           b        #0x1ef848
0x1ef910  c0c20091           add      x0, x22, #0x30
0x1ef914  ef031daa           mov      x15, x29
0x1ef918  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1ef91c  c0035fd6           ret      
0x1ef920  b8460394           bl       #0x2c1400
0x1ef924  c4ffff17           b        #0x1ef834
0x1ef928  b6460394           bl       #0x2c1400
0x1ef92c  cdffff17           b        #0x1ef860
# CFG: 0x1ef814->0x1ef834/ConditionalFalse 0x1ef814->0x1ef920/ConditionalTrue 0x1ef834->0x1ef848/Fallthrough 0x1ef848->0x1ef860/ConditionalFalse 0x1ef848->0x1ef928/ConditionalTrue 0x1ef860->0x1ef868/ConditionalFalse 0x1ef860->0x1ef910/ConditionalTrue 0x1ef868->0x1ef86c/ConditionalFalse 0x1ef868->0x1ef8bc/ConditionalTrue 0x1ef86c->0x1ef888/ConditionalFalse 0x1ef86c->0x1ef890/ConditionalTrue 0x1ef888->0x1ef890/Fallthrough 0x1ef890->0x1ef898/ConditionalFalse 0x1ef890->0x1ef8b0/ConditionalTrue 0x1ef898->0x1ef8ac/ConditionalFalse 0x1ef898->0x1ef8b0/ConditionalTrue 0x1ef8ac->0x1ef8b0/Fallthrough 0x1ef8b0->0x1ef904/Branch 0x1ef8bc->0x1ef8f8/ConditionalFalse 0x1ef8bc->0x1ef8fc/ConditionalTrue 0x1ef8f8->0x1ef8fc/Fallthrough 0x1ef8fc->0x1ef904/Fallthrough 0x1ef904->0x1ef848/Branch 0x1ef920->0x1ef834/Branch 0x1ef928->0x1ef860/Branch

# top_level.e10AsyncLoop at 0x1efac4 (316 bytes)
0x1efac4  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1efac8  fd030faa           mov      x29, x15
0x1efacc  efe100d1           sub      x15, x15, #0x38
0x1efad0  b6831ff8           stur     x22, [x29, #-8]
0x1efad4  a1031ff8           stur     x1, [x29, #-0x10]
0x1efad8  502740f9           ldr      x16, [x26, #0x48]
0x1efadc  ff0110eb           cmp      x15, x16
0x1efae0  89080054           b.ls     #0x1efbf0
0x1efae4  600b44f9           ldr      x0, [x27, #0x810]  # pool[256] = snapshotRef(18530)
0x1efae8  0e6bfd97           bl       #0x14a720
0x1efaec  030080d2           mov      x3, #0
0x1efaf0  020080d2           mov      x2, #0
0x1efaf4  a0035ff8           ldur     x0, [x29, #-0x10]
0x1efaf8  a3831ef8           stur     x3, [x29, #-0x18]
0x1efafc  a2031ef8           stur     x2, [x29, #-0x20]
0x1efb00  502740f9           ldr      x16, [x26, #0x48]
0x1efb04  ff0110eb           cmp      x15, x16
0x1efb08  89070054           b.ls     #0x1efbf8
0x1efb0c  5f0000eb           cmp      x2, x0
0x1efb10  6a050054           b.ge     #0x1efbbc
0x1efb14  610b44f9           ldr      x1, [x27, #0x810]  # pool[256] = snapshotRef(18530)
0x1efb18  e66afd97           bl       #0x14a6b0
0x1efb1c  a0831df8           stur     x0, [x29, #-0x28]
0x1efb20  1fb000f8           stur     xzr, [x0, #0xb]
0x1efb24  403f40f9           ldr      x0, [x26, #0x78]
0x1efb28  005443f9           ldr      x0, [x0, #0x6a8]
0x1efb2c  504b40f9           ldr      x16, [x26, #0x90]
0x1efb30  1f00106b           cmp      w0, w16
0x1efb34  61000054           b.ne     #0x1efb40
0x1efb38  627b40f9           ldr      x2, [x27, #0xf0]  # pool[28] = Zone._current
0x1efb3c  393e0394           bl       #0x2bf420
0x1efb40  a3835df8           ldur     x3, [x29, #-0x28]
0x1efb44  603001b8           stur     w0, [x3, #0x13]
0x1efb48  a4035ef8           ldur     x4, [x29, #-0x20]
0x1efb4c  80787f93           sbfiz    x0, x4, #1, #0x1f
0x1efb50  9f0480eb           cmp      x4, x0, asr #1
0x1efb54  60000054           b.eq     #0x1efb60
0x1efb58  8a460394           bl       #0x2c1580
0x1efb5c  047000f8           stur     x4, [x0, #7]
0x1efb60  e10303aa           mov      x1, x3
0x1efb64  e20300aa           mov      x2, x0
0x1efb68  dd5afd97           bl       #0x1466dc
0x1efb6c  a0835df8           ldur     x0, [x29, #-0x28]
0x1efb70  5b6afd97           bl       #0x14a4dc
0x1efb74  e20300aa           mov      x2, x0
0x1efb78  a3835ef8           ldur     x3, [x29, #-0x18]
0x1efb7c  60787f93           sbfiz    x0, x3, #1, #0x1f
0x1efb80  7f0480eb           cmp      x3, x0, asr #1
0x1efb84  60000054           b.eq     #0x1efb90
0x1efb88  7e460394           bl       #0x2c1580
0x1efb8c  037000f8           stur     x3, [x0, #7]
0x1efb90  e20100a9           stp      x2, x0, [x15]
0x1efb94  b73c0394           bl       #0x2bee70
0x1efb98  037c4193           sbfx     x3, x0, #1, #0x1f
0x1efb9c  40000036           tbz      w0, #0, #0x1efba4
0x1efba0  037040f8           ldur     x3, [x0, #7]
0x1efba4  7f9001f1           cmp      x3, #0x64
0x1efba8  8c000054           b.gt     #0x1efbb8
0x1efbac  a0035ef8           ldur     x0, [x29, #-0x20]
0x1efbb0  02040091           add      x2, x0, #1
0x1efbb4  d0ffff17           b        #0x1efaf4
0x1efbb8  9c69fd17           b        #0x14a228
0x1efbbc  616b41f9           ldr      x1, [x27, #0x2d0]  # pool[88] = snapshotRef(18499)
0x1efbc0  622b4ff9           ldr      x2, [x27, #0x1e50]  # pool[968] = snapshotInstance(Duration)
0x1efbc4  92fffe97           bl       #0x1afa0c
0x1efbc8  e10300aa           mov      x1, x0
0x1efbcc  a1831df8           stur     x1, [x29, #-0x28]
0x1efbd0  436afd97           bl       #0x14a4dc
0x1efbd4  a2835ef8           ldur     x2, [x29, #-0x18]
0x1efbd8  40787f93           sbfiz    x0, x2, #1, #0x1f
0x1efbdc  5f0480eb           cmp      x2, x0, asr #1
0x1efbe0  60000054           b.eq     #0x1efbec
0x1efbe4  67460394           bl       #0x2c1580
0x1efbe8  027000f8           stur     x2, [x0, #7]
0x1efbec  8f69fd17           b        #0x14a228
0x1efbf0  04460394           bl       #0x2c1400
0x1efbf4  bcffff17           b        #0x1efae4
0x1efbf8  02460394           bl       #0x2c1400
0x1efbfc  c4ffff17           b        #0x1efb0c
# CFG: 0x1efac4->0x1efae4/ConditionalFalse 0x1efac4->0x1efbf0/ConditionalTrue 0x1efae4->0x1efaf4/Fallthrough 0x1efaf4->0x1efb0c/ConditionalFalse 0x1efaf4->0x1efbf8/ConditionalTrue 0x1efb0c->0x1efb14/ConditionalFalse 0x1efb0c->0x1efbbc/ConditionalTrue 0x1efb14->0x1efb38/ConditionalFalse 0x1efb14->0x1efb40/ConditionalTrue 0x1efb38->0x1efb40/Fallthrough 0x1efb40->0x1efb58/ConditionalFalse 0x1efb40->0x1efb60/ConditionalTrue 0x1efb58->0x1efb60/Fallthrough 0x1efb60->0x1efb88/ConditionalFalse 0x1efb60->0x1efb90/ConditionalTrue 0x1efb88->0x1efb90/Fallthrough 0x1efb90->0x1efba0/ConditionalFalse 0x1efb90->0x1efba4/ConditionalTrue 0x1efba0->0x1efba4/Fallthrough 0x1efba4->0x1efbac/ConditionalFalse 0x1efba4->0x1efbb8/ConditionalTrue 0x1efbac->0x1efaf4/Branch 0x1efbbc->0x1efbe4/ConditionalFalse 0x1efbbc->0x1efbec/ConditionalTrue 0x1efbe4->0x1efbec/Fallthrough 0x1efbf0->0x1efae4/Branch 0x1efbf8->0x1efb0c/Branch

# top_level.e09TryRethrow at 0x1efc00 (460 bytes)
0x1efc00  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1efc04  fd030faa           mov      x29, x15
0x1efc08  ef6102d1           sub      x15, x15, #0x98
0x1efc0c  a10319f8           stur     x1, [x29, #-0x70]
0x1efc10  502740f9           ldr      x16, [x26, #0x48]
0x1efc14  ff0110eb           cmp      x15, x16
0x1efc18  690d0054           b.ls     #0x1efdc4
0x1efc1c  70db40f9           ldr      x16, [x27, #0x1b0]  # pool[52] = snapshotRef(903)
0x1efc20  7e234091           add      x30, x27, #8, lsl #12
0x1efc24  de4f44f9           ldr      x30, [x30, #0x898]  # pool[4369] = snapshotRef(610)
0x1efc28  fe4100a9           stp      x30, x16, [x15]
0x1efc2c  0704fd97           bl       #0x130c48
0x1efc30  e30300aa           mov      x3, x0
0x1efc34  a00359f8           ldur     x0, [x29, #-0x70]
0x1efc38  a38318f8           stur     x3, [x29, #-0x78]
0x1efc3c  810780d2           mov      x1, #0x3c
0x1efc40  60000036           tbz      w0, #0, #0x1efc4c
0x1efc44  01f05ff8           ldur     x1, [x0, #-1]
0x1efc48  217c4cd3           ubfx     x1, x1, #0xc, #0x14
0x1efc4c  30f000d1           sub      x16, x1, #0x3c
0x1efc50  1f0600f1           cmp      x16, #1
0x1efc54  68070054           b.hi     #0x1efd40
0x1efc58  e10316aa           mov      x1, x22
0x1efc5c  820080d2           mov      x2, #4
0x1efc60  a5450394           bl       #0x2c12f4
0x1efc64  70234091           add      x16, x27, #8, lsl #12
0x1efc68  106246f9           ldr      x16, [x16, #0xcc0]  # pool[4502] = "ok:"
0x1efc6c  10f000b8           stur     w16, [x0, #0xf]
0x1efc70  a10359f8           ldur     x1, [x29, #-0x70]
0x1efc74  013001b8           stur     w1, [x0, #0x13]
0x1efc78  e00100f9           str      x0, [x15]
0x1efc7c  4003fd97           bl       #0x13097c
0x1efc80  a00319f8           stur     x0, [x29, #-0x70]
0x1efc84  b08358f8           ldur     x16, [x29, #-0x78]
0x1efc88  7e234091           add      x30, x27, #8, lsl #12
0x1efc8c  de6746f9           ldr      x30, [x30, #0xcc8]  # pool[4503] = snapshotRef(109)
0x1efc90  fe4100a9           stp      x30, x16, [x15]
0x1efc94  ed03fd97           bl       #0x130c48
0x1efc98  a00359f8           ldur     x0, [x29, #-0x70]
0x1efc9c  ef031daa           mov      x15, x29
0x1efca0  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1efca4  c0035fd6           ret      
0x1efca8  af6302d1           sub      x15, x29, #0x98
0x1efcac  e30300aa           mov      x3, x0
0x1efcb0  a00319f8           stur     x0, [x29, #-0x70]
0x1efcb4  e00301aa           mov      x0, x1
0x1efcb8  a10318f8           stur     x1, [x29, #-0x80]
0x1efcbc  810780d2           mov      x1, #0x3c
0x1efcc0  63000036           tbz      w3, #0, #0x1efccc
0x1efcc4  61f05ff8           ldur     x1, [x3, #-1]
0x1efcc8  217c4cd3           ubfx     x1, x1, #0xc, #0x14
0x1efccc  3f0823f1           cmp      x1, #0x8c2
0x1efcd0  a0040054           b.eq     #0x1efd64
0x1efcd4  e10316aa           mov      x1, x22
0x1efcd8  820080d2           mov      x2, #4
0x1efcdc  86450394           bl       #0x2c12f4
0x1efce0  a08317f8           stur     x0, [x29, #-0x88]
0x1efce4  70234091           add      x16, x27, #8, lsl #12
0x1efce8  106a46f9           ldr      x16, [x16, #0xcd0]  # pool[4504] = "fallback"
0x1efcec  10f000b8           stur     w16, [x0, #0xf]
0x1efcf0  b00358f8           ldur     x16, [x29, #-0x80]
0x1efcf4  f00100f9           str      x16, [x15]
0x1efcf8  f0dbfd97           bl       #0x166cb8
0x1efcfc  e10300aa           mov      x1, x0
0x1efd00  a08357f8           ldur     x0, [x29, #-0x88]
0x1efd04  013001b8           stur     w1, [x0, #0x13]
0x1efd08  e00100f9           str      x0, [x15]
0x1efd0c  1c03fd97           bl       #0x13097c
0x1efd10  b08359f8           ldur     x16, [x29, #-0x68]
0x1efd14  e04100a9           stp      x0, x16, [x15]
0x1efd18  cc03fd97           bl       #0x130c48
0x1efd1c  a08317f8           stur     x0, [x29, #-0x88]
0x1efd20  70234091           add      x16, x27, #8, lsl #12
0x1efd24  106646f9           ldr      x16, [x16, #0xcc8]  # pool[4503] = snapshotRef(109)
0x1efd28  f00100a9           stp      x16, x0, [x15]
0x1efd2c  c703fd97           bl       #0x130c48
0x1efd30  a08357f8           ldur     x0, [x29, #-0x88]
0x1efd34  ef031daa           mov      x15, x29
0x1efd38  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1efd3c  c0035fd6           ret      
0x1efd40  490afd97           bl       #0x132664
0x1efd44  e10300aa           mov      x1, x0
0x1efd48  60234091           add      x0, x27, #8, lsl #12
0x1efd4c  006c46f9           ldr      x0, [x0, #0xcd8]  # pool[4505] = "bad"
0x1efd50  a18317f8           stur     x1, [x29, #-0x88]
0x1efd54  207000b8           stur     w0, [x1, #7]
0x1efd58  e00301aa           mov      x0, x1
0x1efd5c  e73d0394           bl       #0x2bf4f8
0x1efd60  000020d4           brk      #0
0x1efd64  e00303aa           mov      x0, x3
0x1efd68  017040b8           ldur     w1, [x0, #7]
0x1efd6c  21801c8b           add      x1, x1, x28, lsl #32
0x1efd70  b08359f8           ldur     x16, [x29, #-0x68]
0x1efd74  e14100a9           stp      x1, x16, [x15]
0x1efd78  b403fd97           bl       #0x130c48
0x1efd7c  e20300aa           mov      x2, x0
0x1efd80  a00359f8           ldur     x0, [x29, #-0x70]
0x1efd84  a10358f8           ldur     x1, [x29, #-0x80]
0x1efd88  a28318f8           stur     x2, [x29, #-0x78]
0x1efd8c  d03d0394           bl       #0x2bf4cc
0x1efd90  000020d4           brk      #0
0x1efd94  af6302d1           sub      x15, x29, #0x98
0x1efd98  a00319f8           stur     x0, [x29, #-0x70]
0x1efd9c  a18318f8           stur     x1, [x29, #-0x78]
0x1efda0  b08359f8           ldur     x16, [x29, #-0x68]
0x1efda4  7e234091           add      x30, x27, #8, lsl #12
0x1efda8  de6746f9           ldr      x30, [x30, #0xcc8]  # pool[4503] = snapshotRef(109)
0x1efdac  fe4100a9           stp      x30, x16, [x15]
0x1efdb0  a603fd97           bl       #0x130c48
0x1efdb4  a00359f8           ldur     x0, [x29, #-0x70]
0x1efdb8  a18358f8           ldur     x1, [x29, #-0x78]
0x1efdbc  c43d0394           bl       #0x2bf4cc
0x1efdc0  000020d4           brk      #0
0x1efdc4  8f450394           bl       #0x2c1400
0x1efdc8  95ffff17           b        #0x1efc1c
# CFG: 0x1efc00->0x1efc1c/ConditionalFalse 0x1efc00->0x1efdc4/ConditionalTrue 0x1efc1c->0x1efc44/ConditionalFalse 0x1efc1c->0x1efc4c/ConditionalTrue 0x1efc44->0x1efc4c/Fallthrough 0x1efc4c->0x1efc58/ConditionalFalse 0x1efc4c->0x1efd40/ConditionalTrue 0x1efca8->0x1efcc4/ConditionalFalse 0x1efca8->0x1efccc/ConditionalTrue 0x1efcc4->0x1efccc/Fallthrough 0x1efccc->0x1efcd4/ConditionalFalse 0x1efccc->0x1efd64/ConditionalTrue 0x1efd40->0x1efd64/Fallthrough 0x1efd64->0x1efdc4/Fallthrough 0x1efdc4->0x1efc1c/Branch

# top_level.e07GenericBound at 0x1efdcc (156 bytes)
0x1efdcc  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1efdd0  fd030faa           mov      x29, x15
0x1efdd4  ef8100d1           sub      x15, x15, #0x20
0x1efdd8  80f040b8           ldur     w0, [x4, #0xf]
0x1efddc  60000035           cbnz     w0, #0x1efde8
0x1efde0  e10316aa           mov      x1, x22
0x1efde4  05000014           b        #0x1efdf8
0x1efde8  817041b8           ldur     w1, [x4, #0x17]
0x1efdec  a2cb218b           add      x2, x29, w1, sxtw #2
0x1efdf0  420840f9           ldr      x2, [x2, #0x10]
0x1efdf4  e10302aa           mov      x1, x2
0x1efdf8  502740f9           ldr      x16, [x26, #0x48]
0x1efdfc  ff0110eb           cmp      x15, x16
0x1efe00  09030054           b.ls     #0x1efe60
0x1efe04  80000035           cbnz     w0, #0x1efe14
0x1efe08  60234091           add      x0, x27, #8, lsl #12
0x1efe0c  007046f9           ldr      x0, [x0, #0xce0]  # pool[4506] = snapshotRef(18221)
0x1efe10  02000014           b        #0x1efe18
0x1efe14  e00301aa           mov      x0, x1
0x1efe18  a0831ff8           stur     x0, [x29, #-8]
0x1efe1c  61234091           add      x1, x27, #8, lsl #12
0x1efe20  217446f9           ldr      x1, [x1, #0xce8]  # pool[4507] = <anonymous closure>
0x1efe24  e20316aa           mov      x2, x22
0x1efe28  ec410394           bl       #0x2c05d8
0x1efe2c  e10300aa           mov      x1, x0
0x1efe30  a0835ff8           ldur     x0, [x29, #-8]
0x1efe34  20b000b8           stur     w0, [x1, #0xb]
0x1efe38  70234091           add      x16, x27, #8, lsl #12
0x1efe3c  107a46f9           ldr      x16, [x16, #0xcf0]  # pool[4508] = snapshotRef(18388)
0x1efe40  be0b40f9           ldr      x30, [x29, #0x10]
0x1efe44  fec100a9           stp      x30, x16, [x15, #8]
0x1efe48  e10100f9           str      x1, [x15]
0x1efe4c  64f374f9           ldr      x4, [x27, #0x69e0]  # pool[3386] = snapshotRef(34655)
0x1efe50  2bf40294           bl       #0x2acefc
0x1efe54  ef031daa           mov      x15, x29
0x1efe58  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1efe5c  c0035fd6           ret      
0x1efe60  68450394           bl       #0x2c1400
0x1efe64  e8ffff17           b        #0x1efe04
# CFG: 0x1efdcc->0x1efde0/ConditionalFalse 0x1efdcc->0x1efde8/ConditionalTrue 0x1efde0->0x1efdf8/Branch 0x1efde8->0x1efdf8/Fallthrough 0x1efdf8->0x1efe04/ConditionalFalse 0x1efdf8->0x1efe60/ConditionalTrue 0x1efe04->0x1efe08/ConditionalFalse 0x1efe04->0x1efe14/ConditionalTrue 0x1efe08->0x1efe18/Branch 0x1efe14->0x1efe18/Fallthrough 0x1efe60->0x1efe04/Branch

# top_level.<anonymous closure> at 0x1efe68 (120 bytes)
0x1efe68  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1efe6c  fd030faa           mov      x29, x15
0x1efe70  ef6100d1           sub      x15, x15, #0x18
0x1efe74  502740f9           ldr      x16, [x26, #0x48]
0x1efe78  ff0110eb           cmp      x15, x16
0x1efe7c  e9020054           b.ls     #0x1efed8
0x1efe80  a00b40f9           ldr      x0, [x29, #0x10]
0x1efe84  810780d2           mov      x1, #0x3c
0x1efe88  60000036           tbz      w0, #0, #0x1efe94
0x1efe8c  01f05ff8           ldur     x1, [x0, #-1]
0x1efe90  217c4cd3           ubfx     x1, x1, #0xc, #0x14
0x1efe94  500080d2           mov      x16, #2
0x1efe98  f00100a9           stp      x16, x0, [x15]
0x1efe9c  e00301aa           mov      x0, x1
0x1efea0  1ef43fd1           sub      x30, x0, #0xffd
0x1efea4  be7a7ef8           ldr      x30, [x21, x30, lsl #3]
0x1efea8  c0033fd6           blr      x30
0x1efeac  61234091           add      x1, x27, #8, lsl #12
0x1efeb0  217846f9           ldr      x1, [x1, #0xcf0]  # pool[4508] = snapshotRef(18388)
0x1efeb4  a0831ff8           stur     x0, [x29, #-8]
0x1efeb8  0a000094           bl       #0x1efee0
0x1efebc  a10f40f9           ldr      x1, [x29, #0x18]
0x1efec0  01b000b8           stur     w1, [x0, #0xb]
0x1efec4  a1835ff8           ldur     x1, [x29, #-8]
0x1efec8  01f000b8           stur     w1, [x0, #0xf]
0x1efecc  ef031daa           mov      x15, x29
0x1efed0  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1efed4  c0035fd6           ret      
0x1efed8  4a450394           bl       #0x2c1400
0x1efedc  e9ffff17           b        #0x1efe80
# CFG: 0x1efe68->0x1efe80/ConditionalFalse 0x1efe68->0x1efed8/ConditionalTrue 0x1efe80->0x1efe8c/ConditionalFalse 0x1efe80->0x1efe94/ConditionalTrue 0x1efe8c->0x1efe94/Fallthrough 0x1efed8->0x1efe80/Branch

# top_level.e06RecordDestructure at 0x1efeec (380 bytes)
0x1efeec  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1efef0  fd030faa           mov      x29, x15
0x1efef4  efe100d1           sub      x15, x15, #0x38
0x1efef8  a1831ff8           stur     x1, [x29, #-8]
0x1efefc  502740f9           ldr      x16, [x26, #0x48]
0x1eff00  ff0110eb           cmp      x15, x16
0x1eff04  a90a0054           b.ls     #0x1f0058
0x1eff08  420080d2           mov      x2, #2
0x1eff0c  830080d2           mov      x3, #4
0x1eff10  0d400394           bl       #0x2bff44
0x1eff14  e10316aa           mov      x1, x22
0x1eff18  820080d2           mov      x2, #4
0x1eff1c  a0031ff8           stur     x0, [x29, #-0x10]
0x1eff20  f5440394           bl       #0x2c12f4
0x1eff24  e10300aa           mov      x1, x0
0x1eff28  a0035ff8           ldur     x0, [x29, #-0x10]
0x1eff2c  a1831ef8           stur     x1, [x29, #-0x18]
0x1eff30  20f000b8           stur     w0, [x1, #0xf]
0x1eff34  c20080d2           mov      x2, #6
0x1eff38  030180d2           mov      x3, #8
0x1eff3c  02400394           bl       #0x2bff44
0x1eff40  a3835ef8           ldur     x3, [x29, #-0x18]
0x1eff44  603001b8           stur     w0, [x3, #0x13]
0x1eff48  040080d2           mov      x4, #0
0x1eff4c  000080d2           mov      x0, #0
0x1eff50  a4831df8           stur     x4, [x29, #-0x28]
0x1eff54  502740f9           ldr      x16, [x26, #0x48]
0x1eff58  ff0110eb           cmp      x15, x16
0x1eff5c  29080054           b.ls     #0x1f0060
0x1eff60  1f0800f1           cmp      x0, #2
0x1eff64  ea040054           b.ge     #0x1f0000
0x1eff68  7008008b           add      x16, x3, x0, lsl #2
0x1eff6c  05f240b8           ldur     w5, [x16, #0xf]
0x1eff70  a5801c8b           add      x5, x5, x28, lsl #32
0x1eff74  a5031ff8           stur     x5, [x29, #-0x10]
0x1eff78  06040091           add      x6, x0, #1
0x1eff7c  a6031ef8           stur     x6, [x29, #-0x20]
0x1eff80  bf00166b           cmp      w5, w22
0x1eff84  21010054           b.ne     #0x1effa8
0x1eff88  e00305aa           mov      x0, x5
0x1eff8c  e20316aa           mov      x2, x22
0x1eff90  e10316aa           mov      x1, x22
0x1eff94  68234091           add      x8, x27, #8, lsl #12
0x1eff98  089d46f9           ldr      x8, [x8, #0xd38]  # pool[4517] = snapshotRef(34479)
0x1eff9c  63234091           add      x3, x27, #8, lsl #12
0x1effa0  63a046f9           ldr      x3, [x3, #0xd40]  # pool[4518] = null
0x1effa4  31000094           bl       #0x1f0068
0x1effa8  a1835df8           ldur     x1, [x29, #-0x28]
0x1effac  a0035ff8           ldur     x0, [x29, #-0x10]
0x1effb0  02f040b8           ldur     w2, [x0, #0xf]
0x1effb4  42801c8b           add      x2, x2, x28, lsl #32
0x1effb8  033041b8           ldur     w3, [x0, #0x13]
0x1effbc  63801c8b           add      x3, x3, x28, lsl #32
0x1effc0  800780d2           mov      x0, #0x3c
0x1effc4  62000036           tbz      w2, #0, #0x1effd0
0x1effc8  40f05ff8           ldur     x0, [x2, #-1]
0x1effcc  007c4cd3           ubfx     x0, x0, #0xc, #0x14
0x1effd0  e30900a9           stp      x3, x2, [x15]
0x1effd4  1ec03fd1           sub      x30, x0, #0xff0
0x1effd8  be7a7ef8           ldr      x30, [x21, x30, lsl #3]
0x1effdc  c0033fd6           blr      x30
0x1effe0  017c4193           sbfx     x1, x0, #1, #0x1f
0x1effe4  40000036           tbz      w0, #0, #0x1effec
0x1effe8  017040f8           ldur     x1, [x0, #7]
0x1effec  a2835df8           ldur     x2, [x29, #-0x28]
0x1efff0  4400018b           add      x4, x2, x1
0x1efff4  a0035ef8           ldur     x0, [x29, #-0x20]
0x1efff8  a3835ef8           ldur     x3, [x29, #-0x18]
0x1efffc  d5ffff17           b        #0x1eff50
0x1f0000  a1835ff8           ldur     x1, [x29, #-8]
0x1f0004  e20304aa           mov      x2, x4
0x1f0008  23f040b8           ldur     w3, [x1, #0xf]
0x1f000c  63801c8b           add      x3, x3, x28, lsl #32
0x1f0010  243041b8           ldur     w4, [x1, #0x13]
0x1f0014  84801c8b           add      x4, x4, x28, lsl #32
0x1f0018  257041b8           ldur     w5, [x1, #0x17]
0x1f001c  a5801c8b           add      x5, x5, x28, lsl #32
0x1f0020  617c4193           sbfx     x1, x3, #1, #0x1f
0x1f0024  43000036           tbz      w3, #0, #0x1f002c
0x1f0028  617040f8           ldur     x1, [x3, #7]
0x1f002c  4300018b           add      x3, x2, x1
0x1f0030  817c4193           sbfx     x1, x4, #1, #0x1f
0x1f0034  44000036           tbz      w4, #0, #0x1f003c
0x1f0038  817040f8           ldur     x1, [x4, #7]
0x1f003c  620001cb           sub      x2, x3, x1
0x1f0040  a17040b8           ldur     w1, [x5, #7]
0x1f0044  237c4193           sbfx     x3, x1, #1, #0x1f
0x1f0048  4000038b           add      x0, x2, x3
0x1f004c  ef031daa           mov      x15, x29
0x1f0050  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1f0054  c0035fd6           ret      
0x1f0058  ea440394           bl       #0x2c1400
0x1f005c  abffff17           b        #0x1eff08
0x1f0060  e8440394           bl       #0x2c1400
0x1f0064  bfffff17           b        #0x1eff60
# CFG: 0x1efeec->0x1eff08/ConditionalFalse 0x1efeec->0x1f0058/ConditionalTrue 0x1eff08->0x1eff50/Fallthrough 0x1eff50->0x1eff60/ConditionalFalse 0x1eff50->0x1f0060/ConditionalTrue 0x1eff60->0x1eff68/ConditionalFalse 0x1eff60->0x1f0000/ConditionalTrue 0x1eff68->0x1eff88/ConditionalFalse 0x1eff68->0x1effa8/ConditionalTrue 0x1eff88->0x1effa8/Fallthrough 0x1effa8->0x1effc8/ConditionalFalse 0x1effa8->0x1effd0/ConditionalTrue 0x1effc8->0x1effd0/Fallthrough 0x1effd0->0x1effe8/ConditionalFalse 0x1effd0->0x1effec/ConditionalTrue 0x1effe8->0x1effec/Fallthrough 0x1effec->0x1eff50/Branch 0x1f0000->0x1f0028/ConditionalFalse 0x1f0000->0x1f002c/ConditionalTrue 0x1f0028->0x1f002c/Fallthrough 0x1f002c->0x1f0038/ConditionalFalse 0x1f002c->0x1f003c/ConditionalTrue 0x1f0038->0x1f003c/Fallthrough 0x1f0058->0x1eff08/Branch 0x1f0060->0x1eff60/Branch

# top_level.e05NullFlow at 0x1f00d4 (360 bytes)
0x1f00d4  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1f00d8  fd030faa           mov      x29, x15
0x1f00dc  efa100d1           sub      x15, x15, #0x28
0x1f00e0  e00301aa           mov      x0, x1
0x1f00e4  a1831ff8           stur     x1, [x29, #-8]
0x1f00e8  502740f9           ldr      x16, [x26, #0x48]
0x1f00ec  ff0110eb           cmp      x15, x16
0x1f00f0  290a0054           b.ls     #0x1f0234
0x1f00f4  e10300aa           mov      x1, x0
0x1f00f8  62234091           add      x2, x27, #8, lsl #12
0x1f00fc  422444f9           ldr      x2, [x2, #0x848]  # pool[4359] = snapshotRef(870)
0x1f0100  ebe4fc97           bl       #0x1294ac
0x1f0104  e10300aa           mov      x1, x0
0x1f0108  a0835ff8           ldur     x0, [x29, #-8]
0x1f010c  02f040b8           ldur     w2, [x0, #0xf]
0x1f0110  42801c8b           add      x2, x2, x28, lsl #32
0x1f0114  5f00016b           cmp      w2, w1
0x1f0118  61000054           b.ne     #0x1f0124
0x1f011c  e40316aa           mov      x4, x22
0x1f0120  02000014           b        #0x1f0128
0x1f0124  e40301aa           mov      x4, x1
0x1f0128  430080d2           mov      x3, #2
0x1f012c  e20303aa           mov      x2, x3
0x1f0130  a4031ff8           stur     x4, [x29, #-0x10]
0x1f0134  e10316aa           mov      x1, x22
0x1f0138  6f440394           bl       #0x2c12f4
0x1f013c  a0831ef8           stur     x0, [x29, #-0x18]
0x1f0140  70234091           add      x16, x27, #8, lsl #12
0x1f0144  105244f9           ldr      x16, [x16, #0x8a0]  # pool[4370] = snapshotRef(295)
0x1f0148  10f000b8           stur     w16, [x0, #0xf]
0x1f014c  61b743f9           ldr      x1, [x27, #0x768]  # pool[235] = snapshotRef(18312)
0x1f0150  1d400394           bl       #0x2c01c4
0x1f0154  e30300aa           mov      x3, x0
0x1f0158  a0835ef8           ldur     x0, [x29, #-0x18]
0x1f015c  a3031ef8           stur     x3, [x29, #-0x20]
0x1f0160  60f000b8           stur     w0, [x3, #0xf]
0x1f0164  400080d2           mov      x0, #2
0x1f0168  60b000b8           stur     w0, [x3, #0xb]
0x1f016c  a4035ff8           ldur     x4, [x29, #-0x10]
0x1f0170  9f00166b           cmp      w4, w22
0x1f0174  61000054           b.ne     #0x1f0180
0x1f0178  e20316aa           mov      x2, x22
0x1f017c  09000014           b        #0x1f01a0
0x1f0180  80f05ff8           ldur     x0, [x4, #-1]
0x1f0184  007c4cd3           ubfx     x0, x0, #0xc, #0x14
0x1f0188  e10304aa           mov      x1, x4
0x1f018c  62db40f9           ldr      x2, [x27, #0x1b0]  # pool[52] = snapshotRef(903)
0x1f0190  1e0440d1           sub      x30, x0, #1, lsl #12
0x1f0194  be7a7ef8           ldr      x30, [x21, x30, lsl #3]
0x1f0198  c0033fd6           blr      x30
0x1f019c  e20300aa           mov      x2, x0
0x1f01a0  5f00166b           cmp      w2, w22
0x1f01a4  60000054           b.eq     #0x1f01b0
0x1f01a8  a1035ef8           ldur     x1, [x29, #-0x20]
0x1f01ac  a9defc97           bl       #0x127c50
0x1f01b0  a0035ff8           ldur     x0, [x29, #-0x10]
0x1f01b4  1f00166b           cmp      w0, w22
0x1f01b8  01010054           b.ne     #0x1f01d8
0x1f01bc  70e36af9           ldr      x16, [x27, #0x55c0]  # pool[2742] = snapshotRef(272)
0x1f01c0  f00100f9           str      x16, [x15]
0x1f01c4  a1035ef8           ldur     x1, [x29, #-0x20]
0x1f01c8  641342f9           ldr      x4, [x27, #0x420]  # pool[130] = snapshotRef(34522)
0x1f01cc  a8ca0294           bl       #0x2a2c6c
0x1f01d0  e30300aa           mov      x3, x0
0x1f01d4  02000014           b        #0x1f01dc
0x1f01d8  e30300aa           mov      x3, x0
0x1f01dc  a0835ff8           ldur     x0, [x29, #-8]
0x1f01e0  e10300aa           mov      x1, x0
0x1f01e4  a3031ff8           stur     x3, [x29, #-0x10]
0x1f01e8  62234091           add      x2, x27, #8, lsl #12
0x1f01ec  42ac46f9           ldr      x2, [x2, #0xd58]  # pool[4521] = "missing"
0x1f01f0  afe4fc97           bl       #0x1294ac
0x1f01f4  a1835ff8           ldur     x1, [x29, #-8]
0x1f01f8  22f040b8           ldur     w2, [x1, #0xf]
0x1f01fc  42801c8b           add      x2, x2, x28, lsl #32
0x1f0200  5f00006b           cmp      w2, w0
0x1f0204  41000054           b.ne     #0x1f020c
0x1f0208  e00316aa           mov      x0, x22
0x1f020c  1f00166b           cmp      w0, w22
0x1f0210  c1000054           b.ne     #0x1f0228
0x1f0214  a3035ff8           ldur     x3, [x29, #-0x10]
0x1f0218  62234091           add      x2, x27, #8, lsl #12
0x1f021c  42ac46f9           ldr      x2, [x2, #0xd58]  # pool[4521] = "missing"
0x1f0220  b1280394           bl       #0x2ba4e4
0x1f0224  a0035ff8           ldur     x0, [x29, #-0x10]
0x1f0228  ef031daa           mov      x15, x29
0x1f022c  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1f0230  c0035fd6           ret      
0x1f0234  73440394           bl       #0x2c1400
0x1f0238  afffff17           b        #0x1f00f4
# CFG: 0x1f00d4->0x1f00f4/ConditionalFalse 0x1f00d4->0x1f0234/ConditionalTrue 0x1f00f4->0x1f011c/ConditionalFalse 0x1f00f4->0x1f0124/ConditionalTrue 0x1f011c->0x1f0128/Branch 0x1f0124->0x1f0128/Fallthrough 0x1f0128->0x1f0178/ConditionalFalse 0x1f0128->0x1f0180/ConditionalTrue 0x1f0178->0x1f01a0/Branch 0x1f0180->0x1f01a0/Fallthrough 0x1f01a0->0x1f01a8/ConditionalFalse 0x1f01a0->0x1f01b0/ConditionalTrue 0x1f01a8->0x1f01b0/Fallthrough 0x1f01b0->0x1f01bc/ConditionalFalse 0x1f01b0->0x1f01d8/ConditionalTrue 0x1f01bc->0x1f01dc/Branch 0x1f01d8->0x1f01dc/Fallthrough 0x1f01dc->0x1f0208/ConditionalFalse 0x1f01dc->0x1f020c/ConditionalTrue 0x1f0208->0x1f020c/Fallthrough 0x1f020c->0x1f0214/ConditionalFalse 0x1f020c->0x1f0228/ConditionalTrue 0x1f0214->0x1f0228/Fallthrough 0x1f0234->0x1f00f4/Branch

# top_level.e04BitTwiddle at 0x1f023c (104 bytes)
0x1f023c  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1f0240  fd030faa           mov      x29, x15
0x1f0244  e3fb7eb2           orr      x3, xzr, #0xfffffffffffffffd
0x1f0248  e20080d2           mov      x2, #7
0x1f024c  e40301aa           mov      x4, x1
0x1f0250  847c40d3           ubfx     x4, x4, #0, #0x20
0x1f0254  85701d53           lsl      w5, w4, #3
0x1f0258  a41c0012           and      w4, w5, #0xff
0x1f025c  25fc4293           asr      x5, x1, #2
0x1f0260  e60301aa           mov      x6, x1
0x1f0264  c67c40d3           ubfx     x6, x6, #0, #0x20
0x1f0268  c70c0012           and      w7, w6, #0xf
0x1f026c  e77c40d3           ubfx     x7, x7, #0, #0x20
0x1f0270  a60007aa           orr      x6, x5, x7
0x1f0274  250cc29a           sdiv     x5, x1, x2
0x1f0278  220cc39a           sdiv     x2, x1, x3
0x1f027c  f0fb7eb2           orr      x16, xzr, #0xfffffffffffffffd
0x1f0280  437c109b           mul      x3, x2, x16
0x1f0284  220003cb           sub      x2, x1, x3
0x1f0288  847c40d3           ubfx     x4, x4, #0, #0x20
0x1f028c  810006ca           eor      x1, x4, x6
0x1f0290  230005ca           eor      x3, x1, x5
0x1f0294  600002ca           eor      x0, x3, x2
0x1f0298  ef031daa           mov      x15, x29
0x1f029c  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1f02a0  c0035fd6           ret      

# top_level.e02Cascade at 0x1f02a4 (184 bytes)
0x1f02a4  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1f02a8  fd030faa           mov      x29, x15
0x1f02ac  ef4100d1           sub      x15, x15, #0x10
0x1f02b0  e00301aa           mov      x0, x1
0x1f02b4  a1831ff8           stur     x1, [x29, #-8]
0x1f02b8  502740f9           ldr      x16, [x26, #0x48]
0x1f02bc  ff0110eb           cmp      x15, x16
0x1f02c0  a9040054           b.ls     #0x1f0354
0x1f02c4  61b743f9           ldr      x1, [x27, #0x768]  # pool[235] = snapshotRef(18312)
0x1f02c8  020080d2           mov      x2, #0
0x1f02cc  19e1fc97           bl       #0x128730
0x1f02d0  e10300aa           mov      x1, x0
0x1f02d4  a2835ff8           ldur     x2, [x29, #-8]
0x1f02d8  a0831ff8           stur     x0, [x29, #-8]
0x1f02dc  5ddefc97           bl       #0x127c50
0x1f02e0  a1835ff8           ldur     x1, [x29, #-8]
0x1f02e4  640b42f9           ldr      x4, [x27, #0x410]  # pool[128] = snapshotRef(34574)
0x1f02e8  10b8fd97           bl       #0x15e328
0x1f02ec  a0835ff8           ldur     x0, [x29, #-8]
0x1f02f0  01b040b8           ldur     w1, [x0, #0xb]
0x1f02f4  02f040b8           ldur     w2, [x0, #0xf]
0x1f02f8  42801c8b           add      x2, x2, x28, lsl #32
0x1f02fc  43b040b8           ldur     w3, [x2, #0xb]
0x1f0300  227c4193           sbfx     x2, x1, #1, #0x1f
0x1f0304  a2031ff8           stur     x2, [x29, #-0x10]
0x1f0308  617c4193           sbfx     x1, x3, #1, #0x1f
0x1f030c  5f0001eb           cmp      x2, x1
0x1f0310  61000054           b.ne     #0x1f031c
0x1f0314  e10300aa           mov      x1, x0
0x1f0318  b2e9fc97           bl       #0x12a9e0
0x1f031c  a0835ff8           ldur     x0, [x29, #-8]
0x1f0320  a1035ff8           ldur     x1, [x29, #-0x10]
0x1f0324  22040091           add      x2, x1, #1
0x1f0328  43f87fd3           lsl      x3, x2, #1
0x1f032c  03b000b8           stur     w3, [x0, #0xb]
0x1f0330  02f040b8           ldur     w2, [x0, #0xf]
0x1f0334  42801c8b           add      x2, x2, x28, lsl #32
0x1f0338  4308018b           add      x3, x2, x1, lsl #2
0x1f033c  70234091           add      x16, x27, #8, lsl #12
0x1f0340  10b246f9           ldr      x16, [x16, #0xd60]  # pool[4522] = "done"
0x1f0344  70f000b8           stur     w16, [x3, #0xf]
0x1f0348  ef031daa           mov      x15, x29
0x1f034c  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1f0350  c0035fd6           ret      
0x1f0354  2b440394           bl       #0x2c1400
0x1f0358  dbffff17           b        #0x1f02c4
# CFG: 0x1f02a4->0x1f02c4/ConditionalFalse 0x1f02a4->0x1f0354/ConditionalTrue 0x1f02c4->0x1f0314/ConditionalFalse 0x1f02c4->0x1f031c/ConditionalTrue 0x1f0314->0x1f031c/Fallthrough 0x1f0354->0x1f02c4/Branch

# top_level.e01InterpChain at 0x1f0368 (456 bytes)
0x1f0368  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1f036c  fd030faa           mov      x29, x15
0x1f0370  efa100d1           sub      x15, x15, #0x28
0x1f0374  e30301aa           mov      x3, x1
0x1f0378  e00302aa           mov      x0, x2
0x1f037c  a1831ff8           stur     x1, [x29, #-8]
0x1f0380  a2031ff8           stur     x2, [x29, #-0x10]
0x1f0384  a0031efc           stur     d0, [x29, #-0x20]
0x1f0388  502740f9           ldr      x16, [x26, #0x48]
0x1f038c  ff0110eb           cmp      x15, x16
0x1f0390  a90b0054           b.ls     #0x1f0504
0x1f0394  e10316aa           mov      x1, x22
0x1f0398  020380d2           mov      x2, #0x18
0x1f039c  d6430394           bl       #0x2c12f4
0x1f03a0  e30300aa           mov      x3, x0
0x1f03a4  a3831ef8           stur     x3, [x29, #-0x18]
0x1f03a8  70234091           add      x16, x27, #8, lsl #12
0x1f03ac  10b646f9           ldr      x16, [x16, #0xd68]  # pool[4523] = "user="
0x1f03b0  70f000b8           stur     w16, [x3, #0xf]
0x1f03b4  a4835ff8           ldur     x4, [x29, #-8]
0x1f03b8  643001b8           stur     w4, [x3, #0x13]
0x1f03bc  70234091           add      x16, x27, #8, lsl #12
0x1f03c0  10ba46f9           ldr      x16, [x16, #0xd70]  # pool[4524] = " id="
0x1f03c4  707001b8           stur     w16, [x3, #0x17]
0x1f03c8  a5035ff8           ldur     x5, [x29, #-0x10]
0x1f03cc  a2040091           add      x2, x5, #1
0x1f03d0  40787f93           sbfiz    x0, x2, #1, #0x1f
0x1f03d4  5f0480eb           cmp      x2, x0, asr #1
0x1f03d8  60000054           b.eq     #0x1f03e4
0x1f03dc  69440394           bl       #0x2c1580
0x1f03e0  027000f8           stur     x2, [x0, #7]
0x1f03e4  e10303aa           mov      x1, x3
0x1f03e8  396c0091           add      x25, x1, #0x1b
0x1f03ec  200300b9           str      w0, [x25]
0x1f03f0  e0000036           tbz      w0, #0, #0x1f040c
0x1f03f4  30f05f38           ldurb    w16, [x1, #-1]
0x1f03f8  11f05f38           ldurb    w17, [x0, #-1]
0x1f03fc  300a508a           and      x16, x17, x16, lsr #2
0x1f0400  1f825cea           tst      x16, x28, lsr #32
0x1f0404  40000054           b.eq     #0x1f040c
0x1f0408  453c0394           bl       #0x2bf51c
0x1f040c  70234091           add      x16, x27, #8, lsl #12
0x1f0410  10be46f9           ldr      x16, [x16, #0xd78]  # pool[4525] = " pct="
0x1f0414  70f001b8           stur     w16, [x3, #0x1f]
0x1f0418  a1035efc           ldur     d1, [x29, #-0x20]
0x1f041c  605f5afd           ldr      d0, [x27, #0x34b8]  # pool[1685] = 4636737291354636288
0x1f0420  2208601e           fmul     d2, d1, d0
0x1f0424  410346a9           ldp      x1, x0, [x26, #0x60]
0x1f0428  21400091           add      x1, x1, #0x10
0x1f042c  1f0001eb           cmp      x0, x1
0x1f0430  e9060054           b.ls     #0x1f050c
0x1f0434  413300f9           str      x1, [x26, #0x60]
0x1f0438  213c00d1           sub      x1, x1, #0xf
0x1f043c  80339cd2           mov      x0, #0xe19c
0x1f0440  6000a0f2           movk     x0, #3, lsl #16
0x1f0444  20f01ff8           stur     x0, [x1, #-1]
0x1f0448  bf3a03d5           dmb      ishst
0x1f044c  227000fc           stur     d2, [x1, #7]
0x1f0450  220080d2           mov      x2, #1
0x1f0454  37000094           bl       #0x1f0530
0x1f0458  a1835ef8           ldur     x1, [x29, #-0x18]
0x1f045c  398c0091           add      x25, x1, #0x23
0x1f0460  200300b9           str      w0, [x25]
0x1f0464  e0000036           tbz      w0, #0, #0x1f0480
0x1f0468  30f05f38           ldurb    w16, [x1, #-1]
0x1f046c  11f05f38           ldurb    w17, [x0, #-1]
0x1f0470  300a508a           and      x16, x17, x16, lsr #2
0x1f0474  1f825cea           tst      x16, x28, lsr #32
0x1f0478  40000054           b.eq     #0x1f0480
0x1f047c  283c0394           bl       #0x2bf51c
0x1f0480  a2835ef8           ldur     x2, [x29, #-0x18]
0x1f0484  70234091           add      x16, x27, #8, lsl #12
0x1f0488  10c246f9           ldr      x16, [x16, #0xd80]  # pool[4526] = "% nested=inner-"
0x1f048c  507002b8           stur     w16, [x2, #0x27]
0x1f0490  e10302aa           mov      x1, x2
0x1f0494  a0835ff8           ldur     x0, [x29, #-8]
0x1f0498  39ac0091           add      x25, x1, #0x2b
0x1f049c  200300b9           str      w0, [x25]
0x1f04a0  e0000036           tbz      w0, #0, #0x1f04bc
0x1f04a4  30f05f38           ldurb    w16, [x1, #-1]
0x1f04a8  11f05f38           ldurb    w17, [x0, #-1]
0x1f04ac  300a508a           and      x16, x17, x16, lsr #2
0x1f04b0  1f825cea           tst      x16, x28, lsr #32
0x1f04b4  40000054           b.eq     #0x1f04bc
0x1f04b8  193c0394           bl       #0x2bf51c
0x1f04bc  70234091           add      x16, x27, #8, lsl #12
0x1f04c0  10c646f9           ldr      x16, [x16, #0xd88]  # pool[4527] = " bool="
0x1f04c4  50f002b8           stur     w16, [x2, #0x2f]
0x1f04c8  a0035ff8           ldur     x0, [x29, #-0x10]
0x1f04cc  1f2800f1           cmp      x0, #0xa
0x1f04d0  d0820091           add      x16, x22, #0x20
0x1f04d4  d1c20091           add      x17, x22, #0x30
0x1f04d8  01c2919a           csel     x1, x16, x17, gt
0x1f04dc  413003b8           stur     w1, [x2, #0x33]
0x1f04e0  70234091           add      x16, x27, #8, lsl #12
0x1f04e4  10ca46f9           ldr      x16, [x16, #0xd90]  # pool[4528] = " nullish="
0x1f04e8  507003b8           stur     w16, [x2, #0x37]
0x1f04ec  56b003b8           stur     w22, [x2, #0x3b]
0x1f04f0  e20100f9           str      x2, [x15]
0x1f04f4  2201fd97           bl       #0x13097c
0x1f04f8  ef031daa           mov      x15, x29
0x1f04fc  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1f0500  c0035fd6           ret      
0x1f0504  df430394           bl       #0x2c1480
0x1f0508  a3ffff17           b        #0x1f0394
0x1f050c  e20d9f3c           str      q2, [x15, #-0x10]!
0x1f0510  e415bfa9           stp      x4, x5, [x15, #-0x10]!
0x1f0514  e38d1ff8           str      x3, [x15, #-8]!
0x1f0518  4b430394           bl       #0x2c1244
0x1f051c  e10300aa           mov      x1, x0
0x1f0520  e38540f8           ldr      x3, [x15], #8
0x1f0524  e415c1a8           ldp      x4, x5, [x15], #0x10
0x1f0528  e205c13c           ldr      q2, [x15], #0x10
0x1f052c  c8ffff17           b        #0x1f044c
# CFG: 0x1f0368->0x1f0394/ConditionalFalse 0x1f0368->0x1f0504/ConditionalTrue 0x1f0394->0x1f03dc/ConditionalFalse 0x1f0394->0x1f03e4/ConditionalTrue 0x1f03dc->0x1f03e4/Fallthrough 0x1f03e4->0x1f03f4/ConditionalFalse 0x1f03e4->0x1f040c/ConditionalTrue 0x1f03f4->0x1f0408/ConditionalFalse 0x1f03f4->0x1f040c/ConditionalTrue 0x1f0408->0x1f040c/Fallthrough 0x1f040c->0x1f0434/ConditionalFalse 0x1f040c->0x1f050c/ConditionalTrue 0x1f0434->0x1f044c/Fallthrough 0x1f044c->0x1f0468/ConditionalFalse 0x1f044c->0x1f0480/ConditionalTrue 0x1f0468->0x1f047c/ConditionalFalse 0x1f0468->0x1f0480/ConditionalTrue 0x1f047c->0x1f0480/Fallthrough 0x1f0480->0x1f04a4/ConditionalFalse 0x1f0480->0x1f04bc/ConditionalTrue 0x1f04a4->0x1f04b8/ConditionalFalse 0x1f04a4->0x1f04bc/ConditionalTrue 0x1f04b8->0x1f04bc/Fallthrough 0x1f0504->0x1f0394/Branch 0x1f050c->0x1f044c/Branch

# top_level.seedNow at 0x1f0648 (60 bytes)
0x1f0648  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1f064c  fd030faa           mov      x29, x15
0x1f0650  502740f9           ldr      x16, [x26, #0x48]
0x1f0654  ff0110eb           cmp      x15, x16
0x1f0658  29010054           b.ls     #0x1f067c
0x1f065c  99faff97           bl       #0x1ef0c0
0x1f0660  017c4193           sbfx     x1, x0, #1, #0x1f
0x1f0664  40000036           tbz      w0, #0, #0x1f066c
0x1f0668  017040f8           ldur     x1, [x0, #7]
0x1f066c  e00301aa           mov      x0, x1
0x1f0670  ef031daa           mov      x15, x29
0x1f0674  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1f0678  c0035fd6           ret      
0x1f067c  61430394           bl       #0x2c1400
0x1f0680  f7ffff17           b        #0x1f065c
# CFG: 0x1f0648->0x1f065c/ConditionalFalse 0x1f0648->0x1f067c/ConditionalTrue 0x1f065c->0x1f0668/ConditionalFalse 0x1f065c->0x1f066c/ConditionalTrue 0x1f0668->0x1f066c/Fallthrough 0x1f067c->0x1f065c/Branch

# top_level.main at 0x1f0684 (48 bytes)
0x1f0684  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1f0688  fd030faa           mov      x29, x15
0x1f068c  502740f9           ldr      x16, [x26, #0x48]
0x1f0690  ff0110eb           cmp      x15, x16
0x1f0694  c9000054           b.ls     #0x1f06ac
0x1f0698  07000094           bl       #0x1f06b4
0x1f069c  e00316aa           mov      x0, x22
0x1f06a0  ef031daa           mov      x15, x29
0x1f06a4  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1f06a8  c0035fd6           ret      
0x1f06ac  55430394           bl       #0x2c1400
0x1f06b0  faffff17           b        #0x1f0698
# CFG: 0x1f0684->0x1f0698/ConditionalFalse 0x1f0684->0x1f06ac/ConditionalTrue 0x1f06ac->0x1f0698/Branch

# ProbeApp.<anonymous closure> at 0x214324 (108 bytes)
0x214324  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x214328  fd030faa           mov      x29, x15
0x21432c  ef4100d1           sub      x15, x15, #0x10
0x214330  a00b40f9           ldr      x0, [x29, #0x10]
0x214334  037041b8           ldur     w3, [x0, #0x17]
0x214338  63801c8b           add      x3, x3, x28, lsl #32
0x21433c  a3831ff8           stur     x3, [x29, #-8]
0x214340  502740f9           ldr      x16, [x26, #0x48]
0x214344  ff0110eb           cmp      x15, x16
0x214348  09020054           b.ls     #0x214388
0x21434c  e10316aa           mov      x1, x22
0x214350  820080d2           mov      x2, #4
0x214354  e8b30294           bl       #0x2c12f4
0x214358  70234091           add      x16, x27, #8, lsl #12
0x21435c  108a44f9           ldr      x16, [x16, #0x910]  # pool[4384] = snapshotRef(427)
0x214360  10f000b8           stur     w16, [x0, #0xf]
0x214364  a1835ff8           ldur     x1, [x29, #-8]
0x214368  22f040b8           ldur     w2, [x1, #0xf]
0x21436c  42801c8b           add      x2, x2, x28, lsl #32
0x214370  023001b8           stur     w2, [x0, #0x13]
0x214374  e00100f9           str      x0, [x15]
0x214378  8171fc97           bl       #0x13097c
0x21437c  ef031daa           mov      x15, x29
0x214380  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x214384  c0035fd6           ret      
0x214388  1eb40294           bl       #0x2c1400
0x21438c  f0ffff17           b        #0x21434c
# CFG: 0x214324->0x21434c/ConditionalFalse 0x214324->0x214388/ConditionalTrue 0x214388->0x21434c/Branch

# ProbeApp.<anonymous closure> at 0x214390 (144 bytes)
0x214390  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x214394  fd030faa           mov      x29, x15
0x214398  ef8100d1           sub      x15, x15, #0x20
0x21439c  a01340f9           ldr      x0, [x29, #0x20]
0x2143a0  017041b8           ldur     w1, [x0, #0x17]
0x2143a4  21801c8b           add      x1, x1, x28, lsl #32
0x2143a8  a1831ff8           stur     x1, [x29, #-8]
0x2143ac  502740f9           ldr      x16, [x26, #0x48]
0x2143b0  ff0110eb           cmp      x15, x16
0x2143b4  29030054           b.ls     #0x214418
0x2143b8  a00b40f9           ldr      x0, [x29, #0x10]
0x2143bc  02f040b8           ldur     w2, [x0, #0xf]
0x2143c0  42801c8b           add      x2, x2, x28, lsl #32
0x2143c4  5f00166b           cmp      w2, w22
0x2143c8  61000054           b.ne     #0x2143d4
0x2143cc  000080d2           mov      x0, #0
0x2143d0  02000014           b        #0x2143d8
0x2143d4  e00302aa           mov      x0, x2
0x2143d8  e00100f9           str      x0, [x15]
0x2143dc  3b72fc97           bl       #0x130cc8
0x2143e0  e10300aa           mov      x1, x0
0x2143e4  a0835ff8           ldur     x0, [x29, #-8]
0x2143e8  a1831ef8           stur     x1, [x29, #-0x18]
0x2143ec  023041b8           ldur     w2, [x0, #0x13]
0x2143f0  42801c8b           add      x2, x2, x28, lsl #32
0x2143f4  a2031ff8           stur     x2, [x29, #-0x10]
0x2143f8  d96fff97           bl       #0x1f035c
0x2143fc  a1835ef8           ldur     x1, [x29, #-0x18]
0x214400  01b000b8           stur     w1, [x0, #0xb]
0x214404  a1035ff8           ldur     x1, [x29, #-0x10]
0x214408  013003b8           stur     w1, [x0, #0x33]
0x21440c  ef031daa           mov      x15, x29
0x214410  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x214414  c0035fd6           ret      
0x214418  fab30294           bl       #0x2c1400
0x21441c  e7ffff17           b        #0x2143b8
# CFG: 0x214390->0x2143b8/ConditionalFalse 0x214390->0x214418/ConditionalTrue 0x2143b8->0x2143cc/ConditionalFalse 0x2143b8->0x2143d4/ConditionalTrue 0x2143cc->0x2143d8/Branch 0x2143d4->0x2143d8/Fallthrough 0x214418->0x2143b8/Branch

# E13Dynamic.noSuchMethod at 0x2190bc (220 bytes)
0x2190bc  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x2190c0  fd030faa           mov      x29, x15
0x2190c4  ef4100d1           sub      x15, x15, #0x10
0x2190c8  502740f9           ldr      x16, [x26, #0x48]
0x2190cc  ff0110eb           cmp      x15, x16
0x2190d0  09060054           b.ls     #0x219190
0x2190d4  e10316aa           mov      x1, x22
0x2190d8  020180d2           mov      x2, #8
0x2190dc  86a00294           bl       #0x2c12f4
0x2190e0  a0831ff8           stur     x0, [x29, #-8]
0x2190e4  70274091           add      x16, x27, #9, lsl #12
0x2190e8  105e43f9           ldr      x16, [x16, #0x6b8]  # pool[4821] = "unhandled:"
0x2190ec  10f000b8           stur     w16, [x0, #0xf]
0x2190f0  a10b40f9           ldr      x1, [x29, #0x10]
0x2190f4  08010094           bl       #0x219514
0x2190f8  a1835ff8           ldur     x1, [x29, #-8]
0x2190fc  394c0091           add      x25, x1, #0x13
0x219100  200300b9           str      w0, [x25]
0x219104  e0000036           tbz      w0, #0, #0x219120
0x219108  30f05f38           ldurb    w16, [x1, #-1]
0x21910c  11f05f38           ldurb    w17, [x0, #-1]
0x219110  300a508a           and      x16, x17, x16, lsr #2
0x219114  1f825cea           tst      x16, x28, lsr #32
0x219118  40000054           b.eq     #0x219120
0x21911c  00990294           bl       #0x2bf51c
0x219120  a0835ff8           ldur     x0, [x29, #-8]
0x219124  70e351f9           ldr      x16, [x27, #0x23c0]  # pool[1142] = snapshotRef(758)
0x219128  107001b8           stur     w16, [x0, #0x17]
0x21912c  a10b40f9           ldr      x1, [x29, #0x10]
0x219130  1a000094           bl       #0x219198
0x219134  01f05ff8           ldur     x1, [x0, #-1]
0x219138  217c4cd3           ubfx     x1, x1, #0xc, #0x14
0x21913c  e00100f9           str      x0, [x15]
0x219140  e00301aa           mov      x0, x1
0x219144  1e103cd1           sub      x30, x0, #0xf04
0x219148  be7a7ef8           ldr      x30, [x21, x30, lsl #3]
0x21914c  c0033fd6           blr      x30
0x219150  a1835ff8           ldur     x1, [x29, #-8]
0x219154  396c0091           add      x25, x1, #0x1b
0x219158  200300b9           str      w0, [x25]
0x21915c  e0000036           tbz      w0, #0, #0x219178
0x219160  30f05f38           ldurb    w16, [x1, #-1]
0x219164  11f05f38           ldurb    w17, [x0, #-1]
0x219168  300a508a           and      x16, x17, x16, lsr #2
0x21916c  1f825cea           tst      x16, x28, lsr #32
0x219170  40000054           b.eq     #0x219178
0x219174  ea980294           bl       #0x2bf51c
0x219178  b0835ff8           ldur     x16, [x29, #-8]
0x21917c  f00100f9           str      x16, [x15]
0x219180  ff5dfc97           bl       #0x13097c
0x219184  ef031daa           mov      x15, x29
0x219188  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x21918c  c0035fd6           ret      
0x219190  9ca00294           bl       #0x2c1400
0x219194  d0ffff17           b        #0x2190d4
# CFG: 0x2190bc->0x2190d4/ConditionalFalse 0x2190bc->0x219190/ConditionalTrue 0x2190d4->0x219108/ConditionalFalse 0x2190d4->0x219120/ConditionalTrue 0x219108->0x21911c/ConditionalFalse 0x219108->0x219120/ConditionalTrue 0x21911c->0x219120/Fallthrough 0x219120->0x219160/ConditionalFalse 0x219120->0x219178/ConditionalTrue 0x219160->0x219174/ConditionalFalse 0x219160->0x219178/ConditionalTrue 0x219174->0x219178/Fallthrough 0x219190->0x2190d4/Branch

# E21Mode._enumToString at 0x235a8c (100 bytes)
0x235a8c  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x235a90  fd030faa           mov      x29, x15
0x235a94  ef4100d1           sub      x15, x15, #0x10
0x235a98  e00301aa           mov      x0, x1
0x235a9c  a1831ff8           stur     x1, [x29, #-8]
0x235aa0  502740f9           ldr      x16, [x26, #0x48]
0x235aa4  ff0110eb           cmp      x15, x16
0x235aa8  09020054           b.ls     #0x235ae8
0x235aac  e10316aa           mov      x1, x22
0x235ab0  820080d2           mov      x2, #4
0x235ab4  102e0294           bl       #0x2c12f4
0x235ab8  70274091           add      x16, x27, #9, lsl #12
0x235abc  105a43f9           ldr      x16, [x16, #0x6b0]  # pool[4820] = "E21Mode."
0x235ac0  10f000b8           stur     w16, [x0, #0xf]
0x235ac4  a1835ff8           ldur     x1, [x29, #-8]
0x235ac8  22f040b8           ldur     w2, [x1, #0xf]
0x235acc  42801c8b           add      x2, x2, x28, lsl #32
0x235ad0  023001b8           stur     w2, [x0, #0x13]
0x235ad4  e00100f9           str      x0, [x15]
0x235ad8  a9ebfb97           bl       #0x13097c
0x235adc  ef031daa           mov      x15, x29
0x235ae0  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x235ae4  c0035fd6           ret      
0x235ae8  462e0294           bl       #0x2c1400
0x235aec  f0ffff17           b        #0x235aac
# CFG: 0x235a8c->0x235aac/ConditionalFalse 0x235a8c->0x235ae8/ConditionalTrue 0x235ae8->0x235aac/Branch

# E15Vec.get:hashCode at 0x25a500 (56 bytes)
0x25a500  e20140f9           ldr      x2, [x15]
0x25a504  437040f8           ldur     x3, [x2, #7]
0x25a508  44f040f8           ldur     x4, [x2, #0xf]
0x25a50c  620004ca           eor      x2, x3, x4
0x25a510  40787f93           sbfiz    x0, x2, #1, #0x1f
0x25a514  5f0480eb           cmp      x2, x0, asr #1
0x25a518  e0000054           b.eq     #0x25a534
0x25a51c  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x25a520  fd030faa           mov      x29, x15
0x25a524  179c0194           bl       #0x2c1580
0x25a528  ef031daa           mov      x15, x29
0x25a52c  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x25a530  027000f8           stur     x2, [x0, #7]
0x25a534  c0035fd6           ret      
# CFG: 0x25a500->0x25a51c/ConditionalFalse 0x25a500->0x25a534/ConditionalTrue 0x25a51c->0x25a534/Fallthrough

# E15Vec.== at 0x270270 (88 bytes)
0x270270  e10140f9           ldr      x1, [x15]
0x270274  3f00166b           cmp      w1, w22
0x270278  61000054           b.ne     #0x270284
0x27027c  c0c20091           add      x0, x22, #0x30
0x270280  c0035fd6           ret      
0x270284  820780d2           mov      x2, #0x3c
0x270288  61000036           tbz      w1, #0, #0x270294
0x27028c  22f05ff8           ldur     x2, [x1, #-1]
0x270290  427c4cd3           ubfx     x2, x2, #0xc, #0x14
0x270294  5fd00bf1           cmp      x2, #0x2f4
0x270298  41010054           b.ne     #0x2702c0
0x27029c  e20540f9           ldr      x2, [x15, #8]
0x2702a0  437040f8           ldur     x3, [x2, #7]
0x2702a4  227040f8           ldur     x2, [x1, #7]
0x2702a8  7f0002eb           cmp      x3, x2
0x2702ac  d0820091           add      x16, x22, #0x20
0x2702b0  d1c20091           add      x17, x22, #0x30
0x2702b4  0102919a           csel     x1, x16, x17, eq
0x2702b8  e00301aa           mov      x0, x1
0x2702bc  02000014           b        #0x2702c4
0x2702c0  c0c20091           add      x0, x22, #0x30
0x2702c4  c0035fd6           ret      
# CFG: 0x270270->0x27027c/ConditionalFalse 0x270270->0x270284/ConditionalTrue 0x270284->0x27028c/ConditionalFalse 0x270284->0x270294/ConditionalTrue 0x27028c->0x270294/Fallthrough 0x270294->0x27029c/ConditionalFalse 0x270294->0x2702c0/ConditionalTrue 0x27029c->0x2702c4/Branch 0x2702c0->0x2702c4/Fallthrough
