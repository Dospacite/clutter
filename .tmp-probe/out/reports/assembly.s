# Complete decoded machine-code evidence. Generated source intentionally omits this noise.

# E15Vec.compareTo at 0x1577b0 (220 bytes)
0x1577b0  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1577b4  fd030faa           mov      x29, x15
0x1577b8  ef4100d1           sub      x15, x15, #0x10
0x1577bc  e40301aa           mov      x4, x1
0x1577c0  e30302aa           mov      x3, x2
0x1577c4  a1831ff8           stur     x1, [x29, #-8]
0x1577c8  a2031ff8           stur     x2, [x29, #-0x10]
0x1577cc  502740f9           ldr      x16, [x26, #0x48]
0x1577d0  ff0110eb           cmp      x15, x16
0x1577d4  89050054           b.ls     #0x157884
0x1577d8  e00303aa           mov      x0, x3
0x1577dc  e20316aa           mov      x2, x22
0x1577e0  e10316aa           mov      x1, x22
0x1577e4  840780d2           mov      x4, #0x3c
0x1577e8  60000036           tbz      w0, #0, #0x1577f4
0x1577ec  04f05ff8           ldur     x4, [x0, #-1]
0x1577f0  847c4cd3           ubfx     x4, x4, #0xc, #0x14
0x1577f4  9fd00bf1           cmp      x4, #0x2f4
0x1577f8  c0000054           b.eq     #0x157810
0x1577fc  68234091           add      x8, x27, #8, lsl #12
0x157800  08d545f9           ldr      x8, [x8, #0xba8]  # pool[4467] = snapshotRef(15618)
0x157804  63234091           add      x3, x27, #8, lsl #12
0x157808  63d845f9           ldr      x3, [x3, #0xbb0]  # pool[4468] = null
0x15780c  db230594           bl       #0x2a0778
0x157810  a0835ff8           ldur     x0, [x29, #-8]
0x157814  017040f8           ldur     x1, [x0, #7]
0x157818  227c019b           mul      x2, x1, x1
0x15781c  01f040f8           ldur     x1, [x0, #0xf]
0x157820  207c019b           mul      x0, x1, x1
0x157824  4300008b           add      x3, x2, x0
0x157828  a0035ff8           ldur     x0, [x29, #-0x10]
0x15782c  017040f8           ldur     x1, [x0, #7]
0x157830  227c019b           mul      x2, x1, x1
0x157834  01f040f8           ldur     x1, [x0, #0xf]
0x157838  207c019b           mul      x0, x1, x1
0x15783c  4400008b           add      x4, x2, x0
0x157840  60787f93           sbfiz    x0, x3, #1, #0x1f
0x157844  7f0480eb           cmp      x3, x0, asr #1
0x157848  60000054           b.eq     #0x157854
0x15784c  cf2c0594           bl       #0x2a2b88
0x157850  037000f8           stur     x3, [x0, #7]
0x157854  e20300aa           mov      x2, x0
0x157858  80787f93           sbfiz    x0, x4, #1, #0x1f
0x15785c  9f0480eb           cmp      x4, x0, asr #1
0x157860  60000054           b.eq     #0x15786c
0x157864  c92c0594           bl       #0x2a2b88
0x157868  047000f8           stur     x4, [x0, #7]
0x15786c  e10302aa           mov      x1, x2
0x157870  e20300aa           mov      x2, x0
0x157874  82cc0094           bl       #0x18aa7c
0x157878  ef031daa           mov      x15, x29
0x15787c  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x157880  c0035fd6           ret      
0x157884  612c0594           bl       #0x2a2a08
0x157888  d4ffff17           b        #0x1577d8
# CFG: 0x1577b0->0x1577d8/ConditionalFalse 0x1577b0->0x157884/ConditionalTrue 0x1577d8->0x1577ec/ConditionalFalse 0x1577d8->0x1577f4/ConditionalTrue 0x1577ec->0x1577f4/Fallthrough 0x1577f4->0x1577fc/ConditionalFalse 0x1577f4->0x157810/ConditionalTrue 0x1577fc->0x157810/Fallthrough 0x157810->0x15784c/ConditionalFalse 0x157810->0x157854/ConditionalTrue 0x15784c->0x157854/Fallthrough 0x157854->0x157864/ConditionalFalse 0x157854->0x15786c/ConditionalTrue 0x157864->0x15786c/Fallthrough 0x157884->0x1577d8/Branch

# top_level.e19Ackermann at 0x15788c (288 bytes)
0x15788c  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x157890  fd030faa           mov      x29, x15
0x157894  ef6100d1           sub      x15, x15, #0x18
0x157898  803041b8           ldur     w0, [x4, #0x13]
0x15789c  010800d1           sub      x1, x0, #2
0x1578a0  a2cb218b           add      x2, x29, w1, sxtw #2
0x1578a4  420840f9           ldr      x2, [x2, #0x10]
0x1578a8  3f080071           cmp      w1, #2
0x1578ac  0b010054           b.lt     #0x1578cc
0x1578b0  a0cb218b           add      x0, x29, w1, sxtw #2
0x1578b4  000440f9           ldr      x0, [x0, #8]
0x1578b8  017c4193           sbfx     x1, x0, #1, #0x1f
0x1578bc  40000036           tbz      w0, #0, #0x1578c4
0x1578c0  017040f8           ldur     x1, [x0, #7]
0x1578c4  e00301aa           mov      x0, x1
0x1578c8  02000014           b        #0x1578d0
0x1578cc  400080d2           mov      x0, #2
0x1578d0  502740f9           ldr      x16, [x26, #0x48]
0x1578d4  ff0110eb           cmp      x15, x16
0x1578d8  69060054           b.ls     #0x1579a4
0x1578dc  417c4193           sbfx     x1, x2, #1, #0x1f
0x1578e0  42000036           tbz      w2, #0, #0x1578e8
0x1578e4  417040f8           ldur     x1, [x2, #7]
0x1578e8  410100b5           cbnz     x1, #0x157910
0x1578ec  02040091           add      x2, x0, #1
0x1578f0  40787f93           sbfiz    x0, x2, #1, #0x1f
0x1578f4  5f0480eb           cmp      x2, x0, asr #1
0x1578f8  60000054           b.eq     #0x157904
0x1578fc  a32c0594           bl       #0x2a2b88
0x157900  027000f8           stur     x2, [x0, #7]
0x157904  ef031daa           mov      x15, x29
0x157908  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x15790c  c0035fd6           ret      
0x157910  a00100b5           cbnz     x0, #0x157944
0x157914  220400d1           sub      x2, x1, #1
0x157918  40787f93           sbfiz    x0, x2, #1, #0x1f
0x15791c  5f0480eb           cmp      x2, x0, asr #1
0x157920  60000054           b.eq     #0x15792c
0x157924  992c0594           bl       #0x2a2b88
0x157928  027000f8           stur     x2, [x0, #7]
0x15792c  e00100f9           str      x0, [x15]
0x157930  644741f9           ldr      x4, [x27, #0x288]  # pool[79] = snapshotRef(22)
0x157934  d6ffff97           bl       #0x15788c
0x157938  ef031daa           mov      x15, x29
0x15793c  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x157940  c0035fd6           ret      
0x157944  230400d1           sub      x3, x1, #1
0x157948  a3831ff8           stur     x3, [x29, #-8]
0x15794c  040400d1           sub      x4, x0, #1
0x157950  80787f93           sbfiz    x0, x4, #1, #0x1f
0x157954  9f0480eb           cmp      x4, x0, asr #1
0x157958  60000054           b.eq     #0x157964
0x15795c  8b2c0594           bl       #0x2a2b88
0x157960  047000f8           stur     x4, [x0, #7]
0x157964  e00900a9           stp      x0, x2, [x15]
0x157968  643b41f9           ldr      x4, [x27, #0x270]  # pool[76] = snapshotRef(23)
0x15796c  c8ffff97           bl       #0x15788c
0x157970  e30300aa           mov      x3, x0
0x157974  a2835ff8           ldur     x2, [x29, #-8]
0x157978  40787f93           sbfiz    x0, x2, #1, #0x1f
0x15797c  5f0480eb           cmp      x2, x0, asr #1
0x157980  60000054           b.eq     #0x15798c
0x157984  812c0594           bl       #0x2a2b88
0x157988  027000f8           stur     x2, [x0, #7]
0x15798c  e30100a9           stp      x3, x0, [x15]
0x157990  643b41f9           ldr      x4, [x27, #0x270]  # pool[76] = snapshotRef(23)
0x157994  beffff97           bl       #0x15788c
0x157998  ef031daa           mov      x15, x29
0x15799c  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1579a0  c0035fd6           ret      
0x1579a4  192c0594           bl       #0x2a2a08
0x1579a8  cdffff17           b        #0x1578dc
# CFG: 0x15788c->0x1578b0/ConditionalFalse 0x15788c->0x1578cc/ConditionalTrue 0x1578b0->0x1578c0/ConditionalFalse 0x1578b0->0x1578c4/ConditionalTrue 0x1578c0->0x1578c4/Fallthrough 0x1578c4->0x1578d0/Branch 0x1578cc->0x1578d0/Fallthrough 0x1578d0->0x1578dc/ConditionalFalse 0x1578d0->0x1579a4/ConditionalTrue 0x1578dc->0x1578e4/ConditionalFalse 0x1578dc->0x1578e8/ConditionalTrue 0x1578e4->0x1578e8/Fallthrough 0x1578e8->0x1578ec/ConditionalFalse 0x1578e8->0x157910/ConditionalTrue 0x1578ec->0x1578fc/ConditionalFalse 0x1578ec->0x157904/ConditionalTrue 0x1578fc->0x157904/Fallthrough 0x157910->0x157914/ConditionalFalse 0x157910->0x157944/ConditionalTrue 0x157914->0x157924/ConditionalFalse 0x157914->0x15792c/ConditionalTrue 0x157924->0x15792c/Fallthrough 0x157944->0x15795c/ConditionalFalse 0x157944->0x157964/ConditionalTrue 0x15795c->0x157964/Fallthrough 0x157964->0x157984/ConditionalFalse 0x157964->0x15798c/ConditionalTrue 0x157984->0x15798c/Fallthrough 0x1579a4->0x1578dc/Branch

# top_level.e19Ackermann at 0x1579ac (132 bytes)
0x1579ac  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1579b0  fd030faa           mov      x29, x15
0x1579b4  ef4100d1           sub      x15, x15, #0x10
0x1579b8  803041b8           ldur     w0, [x4, #0x13]
0x1579bc  011000d1           sub      x1, x0, #4
0x1579c0  a2cb218b           add      x2, x29, w1, sxtw #2
0x1579c4  420840f9           ldr      x2, [x2, #0x10]
0x1579c8  3f080071           cmp      w1, #2
0x1579cc  0b010054           b.lt     #0x1579ec
0x1579d0  a0cb218b           add      x0, x29, w1, sxtw #2
0x1579d4  000440f9           ldr      x0, [x0, #8]
0x1579d8  017c4193           sbfx     x1, x0, #1, #0x1f
0x1579dc  40000036           tbz      w0, #0, #0x1579e4
0x1579e0  017040f8           ldur     x1, [x0, #7]
0x1579e4  e30301aa           mov      x3, x1
0x1579e8  02000014           b        #0x1579f0
0x1579ec  430080d2           mov      x3, #2
0x1579f0  502740f9           ldr      x16, [x26, #0x48]
0x1579f4  ff0110eb           cmp      x15, x16
0x1579f8  89010054           b.ls     #0x157a28
0x1579fc  60787f93           sbfiz    x0, x3, #1, #0x1f
0x157a00  7f0480eb           cmp      x3, x0, asr #1
0x157a04  60000054           b.eq     #0x157a10
0x157a08  602c0594           bl       #0x2a2b88
0x157a0c  037000f8           stur     x3, [x0, #7]
0x157a10  e00900a9           stp      x0, x2, [x15]
0x157a14  643b41f9           ldr      x4, [x27, #0x270]  # pool[76] = snapshotRef(23)
0x157a18  9dffff97           bl       #0x15788c
0x157a1c  ef031daa           mov      x15, x29
0x157a20  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x157a24  c0035fd6           ret      
0x157a28  f82b0594           bl       #0x2a2a08
0x157a2c  f4ffff17           b        #0x1579fc
# CFG: 0x1579ac->0x1579d0/ConditionalFalse 0x1579ac->0x1579ec/ConditionalTrue 0x1579d0->0x1579e0/ConditionalFalse 0x1579d0->0x1579e4/ConditionalTrue 0x1579e0->0x1579e4/Fallthrough 0x1579e4->0x1579f0/Branch 0x1579ec->0x1579f0/Fallthrough 0x1579f0->0x1579fc/ConditionalFalse 0x1579f0->0x157a28/ConditionalTrue 0x1579fc->0x157a08/ConditionalFalse 0x1579fc->0x157a10/ConditionalTrue 0x157a08->0x157a10/Fallthrough 0x157a28->0x1579fc/Branch

# ProbeApp.build at 0x1eac4c (220 bytes)
0x1eac4c  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1eac50  fd030faa           mov      x29, x15
0x1eac54  ef4100d1           sub      x15, x15, #0x10
0x1eac58  e30301aa           mov      x3, x1
0x1eac5c  e00302aa           mov      x0, x2
0x1eac60  61234091           add      x1, x27, #8, lsl #12
0x1eac64  210c44f9           ldr      x1, [x1, #0x818]  # pool[4353] = ProbeApp.<anonymous closure>
0x1eac68  e20316aa           mov      x2, x22
0x1eac6c  dddb0294           bl       #0x2a1be0
0x1eac70  a0831ff8           stur     x0, [x29, #-8]
0x1eac74  6685ff97           bl       #0x1cc20c
0x1eac78  e10300aa           mov      x1, x0
0x1eac7c  a0835ff8           ldur     x0, [x29, #-8]
0x1eac80  a1031ff8           stur     x1, [x29, #-0x10]
0x1eac84  20b000b8           stur     w0, [x1, #0xb]
0x1eac88  2b000094           bl       #0x1ead34
0x1eac8c  e10300aa           mov      x1, x0
0x1eac90  a0035ff8           ldur     x0, [x29, #-0x10]
0x1eac94  a1831ff8           stur     x1, [x29, #-8]
0x1eac98  207001b8           stur     w0, [x1, #0x17]
0x1eac9c  c0820091           add      x0, x22, #0x20
0x1eaca0  203004b8           stur     w0, [x1, #0x43]
0x1eaca4  c2c20091           add      x2, x22, #0x30
0x1eaca8  22b000b8           stur     w2, [x1, #0xb]
0x1eacac  22f000b8           stur     w2, [x1, #0xf]
0x1eacb0  1e000094           bl       #0x1ead28
0x1eacb4  a1835ff8           ldur     x1, [x29, #-8]
0x1eacb8  013001b8           stur     w1, [x0, #0x13]
0x1eacbc  61234091           add      x1, x27, #8, lsl #12
0x1eacc0  211044f9           ldr      x1, [x1, #0x820]  # pool[4354] = snapshotRef(34355)
0x1eacc4  017001b8           stur     w1, [x0, #0x17]
0x1eacc8  61234091           add      x1, x27, #8, lsl #12
0x1eaccc  211444f9           ldr      x1, [x1, #0x828]  # pool[4355] = snapshotRef(34510)
0x1eacd0  01f002b8           stur     w1, [x0, #0x2f]
0x1eacd4  61234091           add      x1, x27, #8, lsl #12
0x1eacd8  211844f9           ldr      x1, [x1, #0x830]  # pool[4356] = "clutter edge-case probe"
0x1eacdc  01f003b8           stur     w1, [x0, #0x3f]
0x1eace0  61234091           add      x1, x27, #8, lsl #12
0x1eace4  211c44f9           ldr      x1, [x1, #0x838]  # pool[4357] = snapshotInstance(ThemeMode)
0x1eace8  017005b8           stur     w1, [x0, #0x57]
0x1eacec  61c355f9           ldr      x1, [x27, #0x2b80]  # pool[1390] = snapshotInstance(Duration)
0x1eacf0  01b005b8           stur     w1, [x0, #0x5b]
0x1eacf4  614354f9           ldr      x1, [x27, #0x2880]  # pool[1294] = snapshotInstance(_Linear)
0x1eacf8  01f005b8           stur     w1, [x0, #0x5f]
0x1eacfc  61234091           add      x1, x27, #8, lsl #12
0x1ead00  212044f9           ldr      x1, [x1, #0x840]  # pool[4358] = snapshotRef(34479) nestedStrings["US", "en"]
0x1ead04  017007b8           stur     w1, [x0, #0x77]
0x1ead08  c1c20091           add      x1, x22, #0x30
0x1ead0c  01b007b8           stur     w1, [x0, #0x7b]
0x1ead10  01f007b8           stur     w1, [x0, #0x7f]
0x1ead14  c1820091           add      x1, x22, #0x20
0x1ead18  013008b8           stur     w1, [x0, #0x83]
0x1ead1c  ef031daa           mov      x15, x29
0x1ead20  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1ead24  c0035fd6           ret      

# ProbeApp.<anonymous closure> at 0x1ead40 (2956 bytes)
0x1ead40  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1ead44  fd030faa           mov      x29, x15
0x1ead48  efe100d1           sub      x15, x15, #0x38
0x1ead4c  a00f40f9           ldr      x0, [x29, #0x18]
0x1ead50  017041b8           ldur     w1, [x0, #0x17]
0x1ead54  21801c8b           add      x1, x1, x28, lsl #32
0x1ead58  a1831ff8           stur     x1, [x29, #-8]
0x1ead5c  502740f9           ldr      x16, [x26, #0x48]
0x1ead60  ff0110eb           cmp      x15, x16
0x1ead64  895a0054           b.ls     #0x1eb8b4
0x1ead68  210080d2           mov      x1, #1
0x1ead6c  a8da0294           bl       #0x2a180c
0x1ead70  e10300aa           mov      x1, x0
0x1ead74  a0835ff8           ldur     x0, [x29, #-8]
0x1ead78  a1031ff8           stur     x1, [x29, #-0x10]
0x1ead7c  20b000b8           stur     w0, [x1, #0xb]
0x1ead80  deb7fd97           bl       #0x158cf8
0x1ead84  00106e1e           fmov     d0, #1.00000000
0x1ead88  a0831ff8           stur     x0, [x29, #-8]
0x1ead8c  007000fc           stur     d0, [x0, #7]
0x1ead90  a2035ff8           ldur     x2, [x29, #-0x10]
0x1ead94  40f000b8           stur     w0, [x2, #0xf]
0x1ead98  1f130094           bl       #0x1efa14
0x1ead9c  a0831ef8           stur     x0, [x29, #-0x18]
0x1eada0  1a130094           bl       #0x1efa08
0x1eada4  e30300aa           mov      x3, x0
0x1eada8  a0835ef8           ldur     x0, [x29, #-0x18]
0x1eadac  a3031ef8           stur     x3, [x29, #-0x20]
0x1eadb0  60b000b8           stur     w0, [x3, #0xb]
0x1eadb4  a0835ff8           ldur     x0, [x29, #-8]
0x1eadb8  603003b8           stur     w0, [x3, #0x33]
0x1eadbc  61234091           add      x1, x27, #8, lsl #12
0x1eadc0  212444f9           ldr      x1, [x1, #0x848]  # pool[4359] = snapshotRef(18372)
0x1eadc4  020680d2           mov      x2, #0x30
0x1eadc8  cdde0294           bl       #0x2a28fc
0x1eadcc  e30300aa           mov      x3, x0
0x1eadd0  a0035ef8           ldur     x0, [x29, #-0x20]
0x1eadd4  a3831ef8           stur     x3, [x29, #-0x18]
0x1eadd8  60f000b8           stur     w0, [x3, #0xf]
0x1eaddc  e10316aa           mov      x1, x22
0x1eade0  820080d2           mov      x2, #4
0x1eade4  c6de0294           bl       #0x2a28fc
0x1eade8  a0031ef8           stur     x0, [x29, #-0x20]
0x1eadec  70234091           add      x16, x27, #8, lsl #12
0x1eadf0  102a44f9           ldr      x16, [x16, #0x850]  # pool[4360] = snapshotRef(734)
0x1eadf4  10f000b8           stur     w16, [x0, #0xf]
0x1eadf8  70234091           add      x16, x27, #8, lsl #12
0x1eadfc  102e44f9           ldr      x16, [x16, #0x858]  # pool[4361] = snapshotRef(458)
0x1eae00  103001b8           stur     w16, [x0, #0x13]
0x1eae04  61b743f9           ldr      x1, [x27, #0x768]  # pool[235] = snapshotRef(18261)
0x1eae08  71da0294           bl       #0x2a17cc
0x1eae0c  e10300aa           mov      x1, x0
0x1eae10  a0035ef8           ldur     x0, [x29, #-0x20]
0x1eae14  20f000b8           stur     w0, [x1, #0xf]
0x1eae18  820080d2           mov      x2, #4
0x1eae1c  22b000b8           stur     w2, [x1, #0xb]
0x1eae20  cc120094           bl       #0x1ef950
0x1eae24  e00100f9           str      x0, [x15]
0x1eae28  7917fd97           bl       #0x130c0c
0x1eae2c  a0031ef8           stur     x0, [x29, #-0x20]
0x1eae30  f6120094           bl       #0x1efa08
0x1eae34  e10300aa           mov      x1, x0
0x1eae38  a0035ef8           ldur     x0, [x29, #-0x20]
0x1eae3c  20b000b8           stur     w0, [x1, #0xb]
0x1eae40  a2835ff8           ldur     x2, [x29, #-8]
0x1eae44  223003b8           stur     w2, [x1, #0x33]
0x1eae48  e00301aa           mov      x0, x1
0x1eae4c  a1835ef8           ldur     x1, [x29, #-0x18]
0x1eae50  394c0091           add      x25, x1, #0x13
0x1eae54  200300b9           str      w0, [x25]
0x1eae58  e0000036           tbz      w0, #0, #0x1eae74
0x1eae5c  30f05f38           ldurb    w16, [x1, #-1]
0x1eae60  11f05f38           ldurb    w17, [x0, #-1]
0x1eae64  300a508a           and      x16, x17, x16, lsr #2
0x1eae68  1f825cea           tst      x16, x28, lsr #32
0x1eae6c  40000054           b.eq     #0x1eae74
0x1eae70  2dd70294           bl       #0x2a0b24
0x1eae74  e5120094           bl       #0x1efa08
0x1eae78  e10300aa           mov      x1, x0
0x1eae7c  60234091           add      x0, x27, #8, lsl #12
0x1eae80  003044f9           ldr      x0, [x0, #0x860]  # pool[4362] = "beta-or-gamma"
0x1eae84  20b000b8           stur     w0, [x1, #0xb]
0x1eae88  a2835ff8           ldur     x2, [x29, #-8]
0x1eae8c  223003b8           stur     w2, [x1, #0x33]
0x1eae90  e00301aa           mov      x0, x1
0x1eae94  a1835ef8           ldur     x1, [x29, #-0x18]
0x1eae98  395c0091           add      x25, x1, #0x17
0x1eae9c  200300b9           str      w0, [x25]
0x1eaea0  e0000036           tbz      w0, #0, #0x1eaebc
0x1eaea4  30f05f38           ldurb    w16, [x1, #-1]
0x1eaea8  11f05f38           ldurb    w17, [x0, #-1]
0x1eaeac  300a508a           and      x16, x17, x16, lsr #2
0x1eaeb0  1f825cea           tst      x16, x28, lsr #32
0x1eaeb4  40000054           b.eq     #0x1eaebc
0x1eaeb8  1bd70294           bl       #0x2a0b24
0x1eaebc  a3120094           bl       #0x1ef948
0x1eaec0  e20300aa           mov      x2, x0
0x1eaec4  40787f93           sbfiz    x0, x2, #1, #0x1f
0x1eaec8  5f0480eb           cmp      x2, x0, asr #1
0x1eaecc  60000054           b.eq     #0x1eaed8
0x1eaed0  2edf0294           bl       #0x2a2b88
0x1eaed4  027000f8           stur     x2, [x0, #7]
0x1eaed8  e00100f9           str      x0, [x15]
0x1eaedc  4c17fd97           bl       #0x130c0c
0x1eaee0  a0031ef8           stur     x0, [x29, #-0x20]
0x1eaee4  c9120094           bl       #0x1efa08
0x1eaee8  e10300aa           mov      x1, x0
0x1eaeec  a0035ef8           ldur     x0, [x29, #-0x20]
0x1eaef0  20b000b8           stur     w0, [x1, #0xb]
0x1eaef4  a3835ff8           ldur     x3, [x29, #-8]
0x1eaef8  233003b8           stur     w3, [x1, #0x33]
0x1eaefc  e00301aa           mov      x0, x1
0x1eaf00  a1835ef8           ldur     x1, [x29, #-0x18]
0x1eaf04  396c0091           add      x25, x1, #0x1b
0x1eaf08  200300b9           str      w0, [x25]
0x1eaf0c  e0000036           tbz      w0, #0, #0x1eaf28
0x1eaf10  30f05f38           ldurb    w16, [x1, #-1]
0x1eaf14  11f05f38           ldurb    w17, [x0, #-1]
0x1eaf18  300a508a           and      x16, x17, x16, lsr #2
0x1eaf1c  1f825cea           tst      x16, x28, lsr #32
0x1eaf20  40000054           b.eq     #0x1eaf28
0x1eaf24  00d70294           bl       #0x2a0b24
0x1eaf28  e10316aa           mov      x1, x22
0x1eaf2c  820080d2           mov      x2, #4
0x1eaf30  73de0294           bl       #0x2a28fc
0x1eaf34  70234091           add      x16, x27, #8, lsl #12
0x1eaf38  103644f9           ldr      x16, [x16, #0x868]  # pool[4363] = snapshotRef(870)
0x1eaf3c  10f000b8           stur     w16, [x0, #0xf]
0x1eaf40  70234091           add      x16, x27, #8, lsl #12
0x1eaf44  103a44f9           ldr      x16, [x16, #0x870]  # pool[4364] = "v v"
0x1eaf48  103001b8           stur     w16, [x0, #0x13]
0x1eaf4c  70234091           add      x16, x27, #8, lsl #12
0x1eaf50  103e44f9           ldr      x16, [x16, #0x878]  # pool[4365] = snapshotRef(17882)
0x1eaf54  e04100a9           stp      x0, x16, [x15]
0x1eaf58  2d04fd97           bl       #0x12c00c
0x1eaf5c  e10300aa           mov      x1, x0
0x1eaf60  20120094           bl       #0x1ef7e0
0x1eaf64  e00100f9           str      x0, [x15]
0x1eaf68  2917fd97           bl       #0x130c0c
0x1eaf6c  a0031ef8           stur     x0, [x29, #-0x20]
0x1eaf70  a6120094           bl       #0x1efa08
0x1eaf74  e10300aa           mov      x1, x0
0x1eaf78  a0035ef8           ldur     x0, [x29, #-0x20]
0x1eaf7c  20b000b8           stur     w0, [x1, #0xb]
0x1eaf80  a5835ff8           ldur     x5, [x29, #-8]
0x1eaf84  253003b8           stur     w5, [x1, #0x33]
0x1eaf88  e00301aa           mov      x0, x1
0x1eaf8c  a1835ef8           ldur     x1, [x29, #-0x18]
0x1eaf90  397c0091           add      x25, x1, #0x1f
0x1eaf94  200300b9           str      w0, [x25]
0x1eaf98  e0000036           tbz      w0, #0, #0x1eafb4
0x1eaf9c  30f05f38           ldurb    w16, [x1, #-1]
0x1eafa0  11f05f38           ldurb    w17, [x0, #-1]
0x1eafa4  300a508a           and      x16, x17, x16, lsr #2
0x1eafa8  1f825cea           tst      x16, x28, lsr #32
0x1eafac  40000054           b.eq     #0x1eafb4
0x1eafb0  ddd60294           bl       #0x2a0b24
0x1eafb4  c20180d2           mov      x2, #0xe
0x1eafb8  030280d2           mov      x3, #0x10
0x1eafbc  64234091           add      x4, x27, #8, lsl #12
0x1eafc0  844044f9           ldr      x4, [x4, #0x880]  # pool[4366] = snapshotRef(610)
0x1eafc4  c10080d2           mov      x1, #6
0x1eafc8  4100a0f2           movk     x1, #2, lsl #16
0x1eafcc  cfd80294           bl       #0x2a1308
0x1eafd0  e10300aa           mov      x1, x0
0x1eafd4  89110094           bl       #0x1ef5f8
0x1eafd8  e20300aa           mov      x2, x0
0x1eafdc  40787f93           sbfiz    x0, x2, #1, #0x1f
0x1eafe0  5f0480eb           cmp      x2, x0, asr #1
0x1eafe4  60000054           b.eq     #0x1eaff0
0x1eafe8  e8de0294           bl       #0x2a2b88
0x1eafec  027000f8           stur     x2, [x0, #7]
0x1eaff0  e00100f9           str      x0, [x15]
0x1eaff4  0617fd97           bl       #0x130c0c
0x1eaff8  a0031ef8           stur     x0, [x29, #-0x20]
0x1eaffc  83120094           bl       #0x1efa08
0x1eb000  e10300aa           mov      x1, x0
0x1eb004  a0035ef8           ldur     x0, [x29, #-0x20]
0x1eb008  20b000b8           stur     w0, [x1, #0xb]
0x1eb00c  a3835ff8           ldur     x3, [x29, #-8]
0x1eb010  233003b8           stur     w3, [x1, #0x33]
0x1eb014  e00301aa           mov      x0, x1
0x1eb018  a1835ef8           ldur     x1, [x29, #-0x18]
0x1eb01c  398c0091           add      x25, x1, #0x23
0x1eb020  200300b9           str      w0, [x25]
0x1eb024  e0000036           tbz      w0, #0, #0x1eb040
0x1eb028  30f05f38           ldurb    w16, [x1, #-1]
0x1eb02c  11f05f38           ldurb    w17, [x0, #-1]
0x1eb030  300a508a           and      x16, x17, x16, lsr #2
0x1eb034  1f825cea           tst      x16, x28, lsr #32
0x1eb038  40000054           b.eq     #0x1eb040
0x1eb03c  bad60294           bl       #0x2a0b24
0x1eb040  e10316aa           mov      x1, x22
0x1eb044  820080d2           mov      x2, #4
0x1eb048  2dde0294           bl       #0x2a28fc
0x1eb04c  70234091           add      x16, x27, #8, lsl #12
0x1eb050  104644f9           ldr      x16, [x16, #0x888]  # pool[4367] = snapshotRef(295)
0x1eb054  10f000b8           stur     w16, [x0, #0xf]
0x1eb058  500080d2           mov      x16, #2
0x1eb05c  103001b8           stur     w16, [x0, #0x13]
0x1eb060  70db51f9           ldr      x16, [x27, #0x23b0]  # pool[1140] = snapshotRef(17935)
0x1eb064  e04100a9           stp      x0, x16, [x15]
0x1eb068  e903fd97           bl       #0x12c00c
0x1eb06c  70db5bf9           ldr      x16, [x27, #0x37b0]  # pool[1780] = snapshotRef(17853)
0x1eb070  e04100a9           stp      x0, x16, [x15]
0x1eb074  646f4ef9           ldr      x4, [x27, #0x1cd8]  # pool[921] = snapshotRef(34410)
0x1eb078  18110094           bl       #0x1ef4d8
0x1eb07c  e00100f9           str      x0, [x15]
0x1eb080  e316fd97           bl       #0x130c0c
0x1eb084  a0031ef8           stur     x0, [x29, #-0x20]
0x1eb088  60120094           bl       #0x1efa08
0x1eb08c  e10300aa           mov      x1, x0
0x1eb090  a0035ef8           ldur     x0, [x29, #-0x20]
0x1eb094  20b000b8           stur     w0, [x1, #0xb]
0x1eb098  a2835ff8           ldur     x2, [x29, #-8]
0x1eb09c  223003b8           stur     w2, [x1, #0x33]
0x1eb0a0  e00301aa           mov      x0, x1
0x1eb0a4  a1835ef8           ldur     x1, [x29, #-0x18]
0x1eb0a8  399c0091           add      x25, x1, #0x27
0x1eb0ac  200300b9           str      w0, [x25]
0x1eb0b0  e0000036           tbz      w0, #0, #0x1eb0cc
0x1eb0b4  30f05f38           ldurb    w16, [x1, #-1]
0x1eb0b8  11f05f38           ldurb    w17, [x0, #-1]
0x1eb0bc  300a508a           and      x16, x17, x16, lsr #2
0x1eb0c0  1f825cea           tst      x16, x28, lsr #32
0x1eb0c4  40000054           b.eq     #0x1eb0cc
0x1eb0c8  97d60294           bl       #0x2a0b24
0x1eb0cc  010080d2           mov      x1, #0
0x1eb0d0  000080d2           mov      x0, #0
0x1eb0d4  502740f9           ldr      x16, [x26, #0x48]
0x1eb0d8  ff0110eb           cmp      x15, x16
0x1eb0dc  093f0054           b.ls     #0x1eb8bc
0x1eb0e0  1f1000f1           cmp      x0, #4
0x1eb0e4  2a030054           b.ge     #0x1eb148
0x1eb0e8  e30301aa           mov      x3, x1
0x1eb0ec  010080d2           mov      x1, #0
0x1eb0f0  502740f9           ldr      x16, [x26, #0x48]
0x1eb0f4  ff0110eb           cmp      x15, x16
0x1eb0f8  693e0054           b.ls     #0x1eb8c4
0x1eb0fc  3f1000f1           cmp      x1, #4
0x1eb100  ca010054           b.ge     #0x1eb138
0x1eb104  047c019b           mul      x4, x0, x1
0x1eb108  9f1800f1           cmp      x4, #6
0x1eb10c  2c010054           b.gt     #0x1eb130
0x1eb110  0400018b           add      x4, x0, x1
0x1eb114  9f1000f1           cmp      x4, #4
0x1eb118  a0010054           b.eq     #0x1eb14c
0x1eb11c  64040091           add      x4, x3, #1
0x1eb120  25040091           add      x5, x1, #1
0x1eb124  e30304aa           mov      x3, x4
0x1eb128  e10305aa           mov      x1, x5
0x1eb12c  f1ffff17           b        #0x1eb0f0
0x1eb130  e10303aa           mov      x1, x3
0x1eb134  02000014           b        #0x1eb13c
0x1eb138  61900191           add      x1, x3, #0x64
0x1eb13c  03040091           add      x3, x0, #1
0x1eb140  e00303aa           mov      x0, x3
0x1eb144  e4ffff17           b        #0x1eb0d4
0x1eb148  e30301aa           mov      x3, x1
0x1eb14c  60787f93           sbfiz    x0, x3, #1, #0x1f
0x1eb150  7f0480eb           cmp      x3, x0, asr #1
0x1eb154  60000054           b.eq     #0x1eb160
0x1eb158  8cde0294           bl       #0x2a2b88
0x1eb15c  037000f8           stur     x3, [x0, #7]
0x1eb160  e00100f9           str      x0, [x15]
0x1eb164  aa16fd97           bl       #0x130c0c
0x1eb168  a0031ef8           stur     x0, [x29, #-0x20]
0x1eb16c  27120094           bl       #0x1efa08
0x1eb170  e10300aa           mov      x1, x0
0x1eb174  a0035ef8           ldur     x0, [x29, #-0x20]
0x1eb178  20b000b8           stur     w0, [x1, #0xb]
0x1eb17c  a2835ff8           ldur     x2, [x29, #-8]
0x1eb180  223003b8           stur     w2, [x1, #0x33]
0x1eb184  e00301aa           mov      x0, x1
0x1eb188  a1835ef8           ldur     x1, [x29, #-0x18]
0x1eb18c  39ac0091           add      x25, x1, #0x2b
0x1eb190  200300b9           str      w0, [x25]
0x1eb194  e0000036           tbz      w0, #0, #0x1eb1b0
0x1eb198  30f05f38           ldurb    w16, [x1, #-1]
0x1eb19c  11f05f38           ldurb    w17, [x0, #-1]
0x1eb1a0  300a508a           and      x16, x17, x16, lsr #2
0x1eb1a4  1f825cea           tst      x16, x28, lsr #32
0x1eb1a8  40000054           b.eq     #0x1eb1b0
0x1eb1ac  5ed60294           bl       #0x2a0b24
0x1eb1b0  6c100094           bl       #0x1ef360
0x1eb1b4  a0031ef8           stur     x0, [x29, #-0x20]
0x1eb1b8  14120094           bl       #0x1efa08
0x1eb1bc  e10300aa           mov      x1, x0
0x1eb1c0  a0035ef8           ldur     x0, [x29, #-0x20]
0x1eb1c4  20b000b8           stur     w0, [x1, #0xb]
0x1eb1c8  a2835ff8           ldur     x2, [x29, #-8]
0x1eb1cc  223003b8           stur     w2, [x1, #0x33]
0x1eb1d0  e00301aa           mov      x0, x1
0x1eb1d4  a1835ef8           ldur     x1, [x29, #-0x18]
0x1eb1d8  39bc0091           add      x25, x1, #0x2f
0x1eb1dc  200300b9           str      w0, [x25]
0x1eb1e0  e0000036           tbz      w0, #0, #0x1eb1fc
0x1eb1e4  30f05f38           ldurb    w16, [x1, #-1]
0x1eb1e8  11f05f38           ldurb    w17, [x0, #-1]
0x1eb1ec  300a508a           and      x16, x17, x16, lsr #2
0x1eb1f0  1f825cea           tst      x16, x28, lsr #32
0x1eb1f4  40000054           b.eq     #0x1eb1fc
0x1eb1f8  4bd60294           bl       #0x2a0b24
0x1eb1fc  0a100094           bl       #0x1ef224
0x1eb200  610b44f9           ldr      x1, [x27, #0x810]  # pool[256] = snapshotRef(18479)
0x1eb204  a0031ef8           stur     x0, [x29, #-0x20]
0x1eb208  04100094           bl       #0x1ef218
0x1eb20c  e30300aa           mov      x3, x0
0x1eb210  a0035ef8           ldur     x0, [x29, #-0x20]
0x1eb214  a3831df8           stur     x3, [x29, #-0x28]
0x1eb218  60f000b8           stur     w0, [x3, #0xf]
0x1eb21c  a2035ff8           ldur     x2, [x29, #-0x10]
0x1eb220  61234091           add      x1, x27, #8, lsl #12
0x1eb224  214844f9           ldr      x1, [x1, #0x890]  # pool[4368] = ProbeApp.<anonymous closure>
0x1eb228  6eda0294           bl       #0x2a1be0
0x1eb22c  e10300aa           mov      x1, x0
0x1eb230  a0835df8           ldur     x0, [x29, #-0x28]
0x1eb234  013001b8           stur     w1, [x0, #0x13]
0x1eb238  a1835ef8           ldur     x1, [x29, #-0x18]
0x1eb23c  39cc0091           add      x25, x1, #0x33
0x1eb240  200300b9           str      w0, [x25]
0x1eb244  e0000036           tbz      w0, #0, #0x1eb260
0x1eb248  30f05f38           ldurb    w16, [x1, #-1]
0x1eb24c  11f05f38           ldurb    w17, [x0, #-1]
0x1eb250  300a508a           and      x16, x17, x16, lsr #2
0x1eb254  1f825cea           tst      x16, x28, lsr #32
0x1eb258  40000054           b.eq     #0x1eb260
0x1eb25c  32d60294           bl       #0x2a0b24
0x1eb260  610080d2           mov      x1, #3
0x1eb264  440f0094           bl       #0x1eef74
0x1eb268  017040b8           ldur     w1, [x0, #7]
0x1eb26c  21801c8b           add      x1, x1, x28, lsl #32
0x1eb270  e20300aa           mov      x2, x0
0x1eb274  4cf4fc97           bl       #0x1283a4
0x1eb278  e00100f9           str      x0, [x15]
0x1eb27c  6416fd97           bl       #0x130c0c
0x1eb280  a0031ff8           stur     x0, [x29, #-0x10]
0x1eb284  e1110094           bl       #0x1efa08
0x1eb288  e10300aa           mov      x1, x0
0x1eb28c  a0035ff8           ldur     x0, [x29, #-0x10]
0x1eb290  20b000b8           stur     w0, [x1, #0xb]
0x1eb294  a2835ff8           ldur     x2, [x29, #-8]
0x1eb298  223003b8           stur     w2, [x1, #0x33]
0x1eb29c  e00301aa           mov      x0, x1
0x1eb2a0  a1835ef8           ldur     x1, [x29, #-0x18]
0x1eb2a4  39dc0091           add      x25, x1, #0x37
0x1eb2a8  200300b9           str      w0, [x25]
0x1eb2ac  e0000036           tbz      w0, #0, #0x1eb2c8
0x1eb2b0  30f05f38           ldurb    w16, [x1, #-1]
0x1eb2b4  11f05f38           ldurb    w17, [x0, #-1]
0x1eb2b8  300a508a           and      x16, x17, x16, lsr #2
0x1eb2bc  1f825cea           tst      x16, x28, lsr #32
0x1eb2c0  40000054           b.eq     #0x1eb2c8
0x1eb2c4  18d60294           bl       #0x2a0b24
0x1eb2c8  d70e0094           bl       #0x1eee24
0x1eb2cc  e20300aa           mov      x2, x0
0x1eb2d0  40787f93           sbfiz    x0, x2, #1, #0x1f
0x1eb2d4  5f0480eb           cmp      x2, x0, asr #1
0x1eb2d8  60000054           b.eq     #0x1eb2e4
0x1eb2dc  2bde0294           bl       #0x2a2b88
0x1eb2e0  027000f8           stur     x2, [x0, #7]
0x1eb2e4  e00100f9           str      x0, [x15]
0x1eb2e8  4916fd97           bl       #0x130c0c
0x1eb2ec  a0031ff8           stur     x0, [x29, #-0x10]
0x1eb2f0  c6110094           bl       #0x1efa08
0x1eb2f4  e10300aa           mov      x1, x0
0x1eb2f8  a0035ff8           ldur     x0, [x29, #-0x10]
0x1eb2fc  20b000b8           stur     w0, [x1, #0xb]
0x1eb300  a2835ff8           ldur     x2, [x29, #-8]
0x1eb304  223003b8           stur     w2, [x1, #0x33]
0x1eb308  e00301aa           mov      x0, x1
0x1eb30c  a1835ef8           ldur     x1, [x29, #-0x18]
0x1eb310  39ec0091           add      x25, x1, #0x3b
0x1eb314  200300b9           str      w0, [x25]
0x1eb318  e0000036           tbz      w0, #0, #0x1eb334
0x1eb31c  30f05f38           ldurb    w16, [x1, #-1]
0x1eb320  11f05f38           ldurb    w17, [x0, #-1]
0x1eb324  300a508a           and      x16, x17, x16, lsr #2
0x1eb328  1f825cea           tst      x16, x28, lsr #32
0x1eb32c  40000054           b.eq     #0x1eb334
0x1eb330  fdd50294           bl       #0x2a0b24
0x1eb334  b90e0094           bl       #0x1eee18
0x1eb338  e10300aa           mov      x1, x0
0x1eb33c  e0fb7eb2           orr      x0, xzr, #0xfffffffffffffffd
0x1eb340  207000f8           stur     x0, [x1, #7]
0x1eb344  800080d2           mov      x0, #4
0x1eb348  20f000f8           stur     x0, [x1, #0xf]
0x1eb34c  e20301aa           mov      x2, x1
0x1eb350  61234091           add      x1, x27, #8, lsl #12
0x1eb354  214c44f9           ldr      x1, [x1, #0x898]  # pool[4369] = snapshotInstance(E15Vec)
0x1eb358  16b1fd97           bl       #0x1577b0
0x1eb35c  e20300aa           mov      x2, x0
0x1eb360  40787f93           sbfiz    x0, x2, #1, #0x1f
0x1eb364  5f0480eb           cmp      x2, x0, asr #1
0x1eb368  60000054           b.eq     #0x1eb374
0x1eb36c  07de0294           bl       #0x2a2b88
0x1eb370  027000f8           stur     x2, [x0, #7]
0x1eb374  e00100f9           str      x0, [x15]
0x1eb378  2516fd97           bl       #0x130c0c
0x1eb37c  a0031ff8           stur     x0, [x29, #-0x10]
0x1eb380  a2110094           bl       #0x1efa08
0x1eb384  e10300aa           mov      x1, x0
0x1eb388  a0035ff8           ldur     x0, [x29, #-0x10]
0x1eb38c  20b000b8           stur     w0, [x1, #0xb]
0x1eb390  a2835ff8           ldur     x2, [x29, #-8]
0x1eb394  223003b8           stur     w2, [x1, #0x33]
0x1eb398  e00301aa           mov      x0, x1
0x1eb39c  a1835ef8           ldur     x1, [x29, #-0x18]
0x1eb3a0  39fc0091           add      x25, x1, #0x3f
0x1eb3a4  200300b9           str      w0, [x25]
0x1eb3a8  e0000036           tbz      w0, #0, #0x1eb3c4
0x1eb3ac  30f05f38           ldurb    w16, [x1, #-1]
0x1eb3b0  11f05f38           ldurb    w17, [x0, #-1]
0x1eb3b4  300a508a           and      x16, x17, x16, lsr #2
0x1eb3b8  1f825cea           tst      x16, x28, lsr #32
0x1eb3bc  40000054           b.eq     #0x1eb3c4
0x1eb3c0  d9d50294           bl       #0x2a0b24
0x1eb3c4  61b743f9           ldr      x1, [x27, #0x768]  # pool[235] = snapshotRef(18261)
0x1eb3c8  880ffd97           bl       #0x12f1e8
0x1eb3cc  e30300aa           mov      x3, x0
0x1eb3d0  608345f9           ldr      x0, [x27, #0xb00]  # pool[350] = snapshotRef(50909)
0x1eb3d4  a3031ff8           stur     x3, [x29, #-0x10]
0x1eb3d8  60b001b8           stur     w0, [x3, #0x1b]
0x1eb3dc  7fb000b8           stur     wzr, [x3, #0xb]
0x1eb3e0  608745f9           ldr      x0, [x27, #0xb08]  # pool[351] = snapshotRef(47394)
0x1eb3e4  60f000b8           stur     w0, [x3, #0xf]
0x1eb3e8  7f3001b8           stur     wzr, [x3, #0x13]
0x1eb3ec  7f7001b8           stur     wzr, [x3, #0x17]
0x1eb3f0  e10303aa           mov      x1, x3
0x1eb3f4  62234091           add      x2, x27, #8, lsl #12
0x1eb3f8  422844f9           ldr      x2, [x2, #0x850]  # pool[4360] = snapshotRef(734)
0x1eb3fc  a4860194           bl       #0x24ce8c
0x1eb400  a1035ff8           ldur     x1, [x29, #-0x10]
0x1eb404  62234091           add      x2, x27, #8, lsl #12
0x1eb408  422c44f9           ldr      x2, [x2, #0x858]  # pool[4361] = snapshotRef(458)
0x1eb40c  a0860194           bl       #0x24ce8c
0x1eb410  a1035ff8           ldur     x1, [x29, #-0x10]
0x1eb414  890a0094           bl       #0x1ede38
0x1eb418  e00100f9           str      x0, [x15]
0x1eb41c  fc15fd97           bl       #0x130c0c
0x1eb420  a0031ff8           stur     x0, [x29, #-0x10]
0x1eb424  79110094           bl       #0x1efa08
0x1eb428  e10300aa           mov      x1, x0
0x1eb42c  a0035ff8           ldur     x0, [x29, #-0x10]
0x1eb430  20b000b8           stur     w0, [x1, #0xb]
0x1eb434  a2835ff8           ldur     x2, [x29, #-8]
0x1eb438  223003b8           stur     w2, [x1, #0x33]
0x1eb43c  e00301aa           mov      x0, x1
0x1eb440  a1835ef8           ldur     x1, [x29, #-0x18]
0x1eb444  390c0191           add      x25, x1, #0x43
0x1eb448  200300b9           str      w0, [x25]
0x1eb44c  e0000036           tbz      w0, #0, #0x1eb468
0x1eb450  30f05f38           ldurb    w16, [x1, #-1]
0x1eb454  11f05f38           ldurb    w17, [x0, #-1]
0x1eb458  300a508a           and      x16, x17, x16, lsr #2
0x1eb45c  1f825cea           tst      x16, x28, lsr #32
0x1eb460  40000054           b.eq     #0x1eb468
0x1eb464  b0d50294           bl       #0x2a0b24
0x1eb468  100a0094           bl       #0x1edca8
0x1eb46c  01b040b8           ldur     w1, [x0, #0xb]
0x1eb470  e10100f9           str      x1, [x15]
0x1eb474  94950094           bl       #0x210ac4
0x1eb478  a0031ff8           stur     x0, [x29, #-0x10]
0x1eb47c  63110094           bl       #0x1efa08
0x1eb480  e10300aa           mov      x1, x0
0x1eb484  a0035ff8           ldur     x0, [x29, #-0x10]
0x1eb488  20b000b8           stur     w0, [x1, #0xb]
0x1eb48c  a2835ff8           ldur     x2, [x29, #-8]
0x1eb490  223003b8           stur     w2, [x1, #0x33]
0x1eb494  e00301aa           mov      x0, x1
0x1eb498  a1835ef8           ldur     x1, [x29, #-0x18]
0x1eb49c  391c0191           add      x25, x1, #0x47
0x1eb4a0  200300b9           str      w0, [x25]
0x1eb4a4  e0000036           tbz      w0, #0, #0x1eb4c0
0x1eb4a8  30f05f38           ldurb    w16, [x1, #-1]
0x1eb4ac  11f05f38           ldurb    w17, [x0, #-1]
0x1eb4b0  300a508a           and      x16, x17, x16, lsr #2
0x1eb4b4  1f825cea           tst      x16, x28, lsr #32
0x1eb4b8  40000054           b.eq     #0x1eb4c0
0x1eb4bc  9ad50294           bl       #0x2a0b24
0x1eb4c0  94020094           bl       #0x1ebf10
0x1eb4c4  a0031ff8           stur     x0, [x29, #-0x10]
0x1eb4c8  50110094           bl       #0x1efa08
0x1eb4cc  e10300aa           mov      x1, x0
0x1eb4d0  a0035ff8           ldur     x0, [x29, #-0x10]
0x1eb4d4  20b000b8           stur     w0, [x1, #0xb]
0x1eb4d8  a2835ff8           ldur     x2, [x29, #-8]
0x1eb4dc  223003b8           stur     w2, [x1, #0x33]
0x1eb4e0  e00301aa           mov      x0, x1
0x1eb4e4  a1835ef8           ldur     x1, [x29, #-0x18]
0x1eb4e8  392c0191           add      x25, x1, #0x4b
0x1eb4ec  200300b9           str      w0, [x25]
0x1eb4f0  e0000036           tbz      w0, #0, #0x1eb50c
0x1eb4f4  30f05f38           ldurb    w16, [x1, #-1]
0x1eb4f8  11f05f38           ldurb    w17, [x0, #-1]
0x1eb4fc  300a508a           and      x16, x17, x16, lsr #2
0x1eb500  1f825cea           tst      x16, x28, lsr #32
0x1eb504  40000054           b.eq     #0x1eb50c
0x1eb508  87d50294           bl       #0x2a0b24
0x1eb50c  900080d2           mov      x16, #4
0x1eb510  9e0080d2           mov      x30, #4
0x1eb514  fe4100a9           stp      x30, x16, [x15]
0x1eb518  643b41f9           ldr      x4, [x27, #0x270]  # pool[76] = snapshotRef(23)
0x1eb51c  dcb0fd97           bl       #0x15788c
0x1eb520  e00100f9           str      x0, [x15]
0x1eb524  ba15fd97           bl       #0x130c0c
0x1eb528  a0031ff8           stur     x0, [x29, #-0x10]
0x1eb52c  37110094           bl       #0x1efa08
0x1eb530  e10300aa           mov      x1, x0
0x1eb534  a0035ff8           ldur     x0, [x29, #-0x10]
0x1eb538  20b000b8           stur     w0, [x1, #0xb]
0x1eb53c  a2835ff8           ldur     x2, [x29, #-8]
0x1eb540  223003b8           stur     w2, [x1, #0x33]
0x1eb544  e00301aa           mov      x0, x1
0x1eb548  a1835ef8           ldur     x1, [x29, #-0x18]
0x1eb54c  393c0191           add      x25, x1, #0x4f
0x1eb550  200300b9           str      w0, [x25]
0x1eb554  e0000036           tbz      w0, #0, #0x1eb570
0x1eb558  30f05f38           ldurb    w16, [x1, #-1]
0x1eb55c  11f05f38           ldurb    w17, [x0, #-1]
0x1eb560  300a508a           and      x16, x17, x16, lsr #2
0x1eb564  1f825cea           tst      x16, x28, lsr #32
0x1eb568  40000054           b.eq     #0x1eb570
0x1eb56c  6ed50294           bl       #0x2a0b24
0x1eb570  65020094           bl       #0x1ebf04
0x1eb574  e10300aa           mov      x1, x0
0x1eb578  60020094           bl       #0x1ebef8
0x1eb57c  a0031ff8           stur     x0, [x29, #-0x10]
0x1eb580  22110094           bl       #0x1efa08
0x1eb584  e10300aa           mov      x1, x0
0x1eb588  a0035ff8           ldur     x0, [x29, #-0x10]
0x1eb58c  20b000b8           stur     w0, [x1, #0xb]
0x1eb590  a2835ff8           ldur     x2, [x29, #-8]
0x1eb594  223003b8           stur     w2, [x1, #0x33]
0x1eb598  e00301aa           mov      x0, x1
0x1eb59c  a1835ef8           ldur     x1, [x29, #-0x18]
0x1eb5a0  394c0191           add      x25, x1, #0x53
0x1eb5a4  200300b9           str      w0, [x25]
0x1eb5a8  e0000036           tbz      w0, #0, #0x1eb5c4
0x1eb5ac  30f05f38           ldurb    w16, [x1, #-1]
0x1eb5b0  11f05f38           ldurb    w17, [x0, #-1]
0x1eb5b4  300a508a           and      x16, x17, x16, lsr #2
0x1eb5b8  1f825cea           tst      x16, x28, lsr #32
0x1eb5bc  40000054           b.eq     #0x1eb5c4
0x1eb5c0  59d50294           bl       #0x2a0b24
0x1eb5c4  08020094           bl       #0x1ebde4
0x1eb5c8  70234091           add      x16, x27, #8, lsl #12
0x1eb5cc  105244f9           ldr      x16, [x16, #0x8a0]  # pool[4370] = snapshotInstance(E21Mode)
0x1eb5d0  1f00106b           cmp      w0, w16
0x1eb5d4  60000054           b.eq     #0x1eb5e0
0x1eb5d8  013041f8           ldur     x1, [x0, #0x13]
0x1eb5dc  6100f8b6           tbz      x1, #0x3f, #0x1eb5e8
0x1eb5e0  61b345f9           ldr      x1, [x27, #0xb60]  # pool[362] = snapshotRef(471)
0x1eb5e4  02000014           b        #0x1eb5ec
0x1eb5e8  61bb45f9           ldr      x1, [x27, #0xb70]  # pool[364] = snapshotRef(167)
0x1eb5ec  a0835ff8           ldur     x0, [x29, #-8]
0x1eb5f0  a1031ff8           stur     x1, [x29, #-0x10]
0x1eb5f4  05110094           bl       #0x1efa08
0x1eb5f8  e10300aa           mov      x1, x0
0x1eb5fc  a0035ff8           ldur     x0, [x29, #-0x10]
0x1eb600  20b000b8           stur     w0, [x1, #0xb]
0x1eb604  a2835ff8           ldur     x2, [x29, #-8]
0x1eb608  223003b8           stur     w2, [x1, #0x33]
0x1eb60c  e00301aa           mov      x0, x1
0x1eb610  a1835ef8           ldur     x1, [x29, #-0x18]
0x1eb614  395c0191           add      x25, x1, #0x57
0x1eb618  200300b9           str      w0, [x25]
0x1eb61c  e0000036           tbz      w0, #0, #0x1eb638
0x1eb620  30f05f38           ldurb    w16, [x1, #-1]
0x1eb624  11f05f38           ldurb    w17, [x0, #-1]
0x1eb628  300a508a           and      x16, x17, x16, lsr #2
0x1eb62c  1f825cea           tst      x16, x28, lsr #32
0x1eb630  40000054           b.eq     #0x1eb638
0x1eb634  3cd50294           bl       #0x2a0b24
0x1eb638  500080d2           mov      x16, #2
0x1eb63c  f00100f9           str      x16, [x15]
0x1eb640  21950094           bl       #0x210ac4
0x1eb644  a0031ff8           stur     x0, [x29, #-0x10]
0x1eb648  f0100094           bl       #0x1efa08
0x1eb64c  e10300aa           mov      x1, x0
0x1eb650  a0035ff8           ldur     x0, [x29, #-0x10]
0x1eb654  20b000b8           stur     w0, [x1, #0xb]
0x1eb658  a3835ff8           ldur     x3, [x29, #-8]
0x1eb65c  233003b8           stur     w3, [x1, #0x33]
0x1eb660  e00301aa           mov      x0, x1
0x1eb664  a1835ef8           ldur     x1, [x29, #-0x18]
0x1eb668  396c0191           add      x25, x1, #0x5b
0x1eb66c  200300b9           str      w0, [x25]
0x1eb670  e0000036           tbz      w0, #0, #0x1eb68c
0x1eb674  30f05f38           ldurb    w16, [x1, #-1]
0x1eb678  11f05f38           ldurb    w17, [x0, #-1]
0x1eb67c  300a508a           and      x16, x17, x16, lsr #2
0x1eb680  1f825cea           tst      x16, x28, lsr #32
0x1eb684  40000054           b.eq     #0x1eb68c
0x1eb688  27d50294           bl       #0x2a0b24
0x1eb68c  e10316aa           mov      x1, x22
0x1eb690  020080d2           mov      x2, #0
0x1eb694  f8f3fc97           bl       #0x128674
0x1eb698  61234091           add      x1, x27, #8, lsl #12
0x1eb69c  215444f9           ldr      x1, [x1, #0x8a8]  # pool[4371] = ProbeApp.<anonymous closure>
0x1eb6a0  e20316aa           mov      x2, x22
0x1eb6a4  a0031ff8           stur     x0, [x29, #-0x10]
0x1eb6a8  4ed90294           bl       #0x2a1be0
0x1eb6ac  e10300aa           mov      x1, x0
0x1eb6b0  a2035ff8           ldur     x2, [x29, #-0x10]
0x1eb6b4  d8000094           bl       #0x1eba14
0x1eb6b8  a0031ff8           stur     x0, [x29, #-0x10]
0x1eb6bc  d3100094           bl       #0x1efa08
0x1eb6c0  e10300aa           mov      x1, x0
0x1eb6c4  a0035ff8           ldur     x0, [x29, #-0x10]
0x1eb6c8  20b000b8           stur     w0, [x1, #0xb]
0x1eb6cc  a2835ff8           ldur     x2, [x29, #-8]
0x1eb6d0  223003b8           stur     w2, [x1, #0x33]
0x1eb6d4  e00301aa           mov      x0, x1
0x1eb6d8  a1835ef8           ldur     x1, [x29, #-0x18]
0x1eb6dc  397c0191           add      x25, x1, #0x5f
0x1eb6e0  200300b9           str      w0, [x25]
0x1eb6e4  e0000036           tbz      w0, #0, #0x1eb700
0x1eb6e8  30f05f38           ldurb    w16, [x1, #-1]
0x1eb6ec  11f05f38           ldurb    w17, [x0, #-1]
0x1eb6f0  300a508a           and      x16, x17, x16, lsr #2
0x1eb6f4  1f825cea           tst      x16, x28, lsr #32
0x1eb6f8  40000054           b.eq     #0x1eb700
0x1eb6fc  0ad50294           bl       #0x2a0b24
0x1eb700  c2000094           bl       #0x1eba08
0x1eb704  a0031ff8           stur     x0, [x29, #-0x10]
0x1eb708  c0000094           bl       #0x1eba08
0x1eb70c  a1035ff8           ldur     x1, [x29, #-0x10]
0x1eb710  e20300aa           mov      x2, x0
0x1eb714  ab000094           bl       #0x1eb9c0
0x1eb718  a0031ff8           stur     x0, [x29, #-0x10]
0x1eb71c  bb100094           bl       #0x1efa08
0x1eb720  e10300aa           mov      x1, x0
0x1eb724  a0035ff8           ldur     x0, [x29, #-0x10]
0x1eb728  20b000b8           stur     w0, [x1, #0xb]
0x1eb72c  a2835ff8           ldur     x2, [x29, #-8]
0x1eb730  223003b8           stur     w2, [x1, #0x33]
0x1eb734  e00301aa           mov      x0, x1
0x1eb738  a1835ef8           ldur     x1, [x29, #-0x18]
0x1eb73c  398c0191           add      x25, x1, #0x63
0x1eb740  200300b9           str      w0, [x25]
0x1eb744  e0000036           tbz      w0, #0, #0x1eb760
0x1eb748  30f05f38           ldurb    w16, [x1, #-1]
0x1eb74c  11f05f38           ldurb    w17, [x0, #-1]
0x1eb750  300a508a           and      x16, x17, x16, lsr #2
0x1eb754  1f825cea           tst      x16, x28, lsr #32
0x1eb758  40000054           b.eq     #0x1eb760
0x1eb75c  f2d40294           bl       #0x2a0b24
0x1eb760  aa100094           bl       #0x1efa08
0x1eb764  e10300aa           mov      x1, x0
0x1eb768  60234091           add      x0, x27, #8, lsl #12
0x1eb76c  005844f9           ldr      x0, [x0, #0x8b0]  # pool[4372] = snapshotRef(558)
0x1eb770  20b000b8           stur     w0, [x1, #0xb]
0x1eb774  a2835ff8           ldur     x2, [x29, #-8]
0x1eb778  223003b8           stur     w2, [x1, #0x33]
0x1eb77c  e00301aa           mov      x0, x1
0x1eb780  a1835ef8           ldur     x1, [x29, #-0x18]
0x1eb784  399c0191           add      x25, x1, #0x67
0x1eb788  200300b9           str      w0, [x25]
0x1eb78c  e0000036           tbz      w0, #0, #0x1eb7a8
0x1eb790  30f05f38           ldurb    w16, [x1, #-1]
0x1eb794  11f05f38           ldurb    w17, [x0, #-1]
0x1eb798  300a508a           and      x16, x17, x16, lsr #2
0x1eb79c  1f825cea           tst      x16, x28, lsr #32
0x1eb7a0  40000054           b.eq     #0x1eb7a8
0x1eb7a4  e0d40294           bl       #0x2a0b24
0x1eb7a8  4f000094           bl       #0x1eb8e4
0x1eb7ac  60002037           tbnz     w0, #4, #0x1eb7b8
0x1eb7b0  62b345f9           ldr      x2, [x27, #0xb60]  # pool[362] = snapshotRef(471)
0x1eb7b4  02000014           b        #0x1eb7bc
0x1eb7b8  62bb45f9           ldr      x2, [x27, #0xb70]  # pool[364] = snapshotRef(167)
0x1eb7bc  a0835ff8           ldur     x0, [x29, #-8]
0x1eb7c0  a1835ef8           ldur     x1, [x29, #-0x18]
0x1eb7c4  a2031ff8           stur     x2, [x29, #-0x10]
0x1eb7c8  90100094           bl       #0x1efa08
0x1eb7cc  e10300aa           mov      x1, x0
0x1eb7d0  a0035ff8           ldur     x0, [x29, #-0x10]
0x1eb7d4  20b000b8           stur     w0, [x1, #0xb]
0x1eb7d8  a0835ff8           ldur     x0, [x29, #-8]
0x1eb7dc  203003b8           stur     w0, [x1, #0x33]
0x1eb7e0  e00301aa           mov      x0, x1
0x1eb7e4  a1835ef8           ldur     x1, [x29, #-0x18]
0x1eb7e8  39ac0191           add      x25, x1, #0x6b
0x1eb7ec  200300b9           str      w0, [x25]
0x1eb7f0  e0000036           tbz      w0, #0, #0x1eb80c
0x1eb7f4  30f05f38           ldurb    w16, [x1, #-1]
0x1eb7f8  11f05f38           ldurb    w17, [x0, #-1]
0x1eb7fc  300a508a           and      x16, x17, x16, lsr #2
0x1eb800  1f825cea           tst      x16, x28, lsr #32
0x1eb804  40000054           b.eq     #0x1eb80c
0x1eb808  c7d40294           bl       #0x2a0b24
0x1eb80c  61234091           add      x1, x27, #8, lsl #12
0x1eb810  212444f9           ldr      x1, [x1, #0x848]  # pool[4359] = snapshotRef(18372)
0x1eb814  eed70294           bl       #0x2a17cc
0x1eb818  e10300aa           mov      x1, x0
0x1eb81c  a0835ef8           ldur     x0, [x29, #-0x18]
0x1eb820  a1831ff8           stur     x1, [x29, #-8]
0x1eb824  20f000b8           stur     w0, [x1, #0xf]
0x1eb828  000680d2           mov      x0, #0x30
0x1eb82c  20b000b8           stur     w0, [x1, #0xb]
0x1eb830  2a000094           bl       #0x1eb8d8
0x1eb834  e10300aa           mov      x1, x0
0x1eb838  60234091           add      x0, x27, #8, lsl #12
0x1eb83c  005c44f9           ldr      x0, [x0, #0x8b8]  # pool[4373] = snapshotInstance(Axis)
0x1eb840  a1031ff8           stur     x1, [x29, #-0x10]
0x1eb844  20f000b8           stur     w0, [x1, #0xf]
0x1eb848  60234091           add      x0, x27, #8, lsl #12
0x1eb84c  006044f9           ldr      x0, [x0, #0x8c0]  # pool[4374] = snapshotInstance(MainAxisAlignment)
0x1eb850  203001b8           stur     w0, [x1, #0x13]
0x1eb854  60234091           add      x0, x27, #8, lsl #12
0x1eb858  006444f9           ldr      x0, [x0, #0x8c8]  # pool[4375] = snapshotInstance(MainAxisSize)
0x1eb85c  207001b8           stur     w0, [x1, #0x17]
0x1eb860  60234091           add      x0, x27, #8, lsl #12
0x1eb864  006844f9           ldr      x0, [x0, #0x8d0]  # pool[4376] = snapshotInstance(CrossAxisAlignment)
0x1eb868  20b001b8           stur     w0, [x1, #0x1b]
0x1eb86c  60234091           add      x0, x27, #8, lsl #12
0x1eb870  006c44f9           ldr      x0, [x0, #0x8d8]  # pool[4377] = snapshotInstance(VerticalDirection)
0x1eb874  203002b8           stur     w0, [x1, #0x23]
0x1eb878  60234091           add      x0, x27, #8, lsl #12
0x1eb87c  007044f9           ldr      x0, [x0, #0x8e0]  # pool[4378] = snapshotInstance(Clip)
0x1eb880  20b002b8           stur     w0, [x1, #0x2b]
0x1eb884  3ff002f8           stur     xzr, [x1, #0x2f]
0x1eb888  a0835ff8           ldur     x0, [x29, #-8]
0x1eb88c  20b000b8           stur     w0, [x1, #0xb]
0x1eb890  0f000094           bl       #0x1eb8cc
0x1eb894  61234091           add      x1, x27, #8, lsl #12
0x1eb898  217444f9           ldr      x1, [x1, #0x8e8]  # pool[4379] = snapshotInstance(Alignment)
0x1eb89c  01f000b8           stur     w1, [x0, #0xf]
0x1eb8a0  a1035ff8           ldur     x1, [x29, #-0x10]
0x1eb8a4  01b000b8           stur     w1, [x0, #0xb]
0x1eb8a8  ef031daa           mov      x15, x29
0x1eb8ac  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1eb8b0  c0035fd6           ret      
0x1eb8b4  55dc0294           bl       #0x2a2a08
0x1eb8b8  2cfdff17           b        #0x1ead68
0x1eb8bc  53dc0294           bl       #0x2a2a08
0x1eb8c0  08feff17           b        #0x1eb0e0
0x1eb8c4  51dc0294           bl       #0x2a2a08
0x1eb8c8  0dfeff17           b        #0x1eb0fc
# CFG: 0x1ead40->0x1ead68/ConditionalFalse 0x1ead40->0x1eb8b4/ConditionalTrue 0x1ead68->0x1eae5c/ConditionalFalse 0x1ead68->0x1eae74/ConditionalTrue 0x1eae5c->0x1eae70/ConditionalFalse 0x1eae5c->0x1eae74/ConditionalTrue 0x1eae70->0x1eae74/Fallthrough 0x1eae74->0x1eaea4/ConditionalFalse 0x1eae74->0x1eaebc/ConditionalTrue 0x1eaea4->0x1eaeb8/ConditionalFalse 0x1eaea4->0x1eaebc/ConditionalTrue 0x1eaeb8->0x1eaebc/Fallthrough 0x1eaebc->0x1eaed0/ConditionalFalse 0x1eaebc->0x1eaed8/ConditionalTrue 0x1eaed0->0x1eaed8/Fallthrough 0x1eaed8->0x1eaf10/ConditionalFalse 0x1eaed8->0x1eaf28/ConditionalTrue 0x1eaf10->0x1eaf24/ConditionalFalse 0x1eaf10->0x1eaf28/ConditionalTrue 0x1eaf24->0x1eaf28/Fallthrough 0x1eaf28->0x1eaf9c/ConditionalFalse 0x1eaf28->0x1eafb4/ConditionalTrue 0x1eaf9c->0x1eafb0/ConditionalFalse 0x1eaf9c->0x1eafb4/ConditionalTrue 0x1eafb0->0x1eafb4/Fallthrough 0x1eafb4->0x1eafe8/ConditionalFalse 0x1eafb4->0x1eaff0/ConditionalTrue 0x1eafe8->0x1eaff0/Fallthrough 0x1eaff0->0x1eb028/ConditionalFalse 0x1eaff0->0x1eb040/ConditionalTrue 0x1eb028->0x1eb03c/ConditionalFalse 0x1eb028->0x1eb040/ConditionalTrue 0x1eb03c->0x1eb040/Fallthrough 0x1eb040->0x1eb0b4/ConditionalFalse 0x1eb040->0x1eb0cc/ConditionalTrue 0x1eb0b4->0x1eb0c8/ConditionalFalse 0x1eb0b4->0x1eb0cc/ConditionalTrue 0x1eb0c8->0x1eb0cc/Fallthrough 0x1eb0cc->0x1eb0d4/Fallthrough 0x1eb0d4->0x1eb0e0/ConditionalFalse 0x1eb0d4->0x1eb8bc/ConditionalTrue 0x1eb0e0->0x1eb0e8/ConditionalFalse 0x1eb0e0->0x1eb148/ConditionalTrue 0x1eb0e8->0x1eb0f0/Fallthrough 0x1eb0f0->0x1eb0fc/ConditionalFalse 0x1eb0f0->0x1eb8c4/ConditionalTrue 0x1eb0fc->0x1eb104/ConditionalFalse 0x1eb0fc->0x1eb138/ConditionalTrue 0x1eb104->0x1eb110/ConditionalFalse 0x1eb104->0x1eb130/ConditionalTrue 0x1eb110->0x1eb11c/ConditionalFalse 0x1eb110->0x1eb14c/ConditionalTrue 0x1eb11c->0x1eb0f0/Branch 0x1eb130->0x1eb13c/Branch 0x1eb138->0x1eb13c/Fallthrough 0x1eb13c->0x1eb0d4/Branch 0x1eb148->0x1eb14c/Fallthrough 0x1eb14c->0x1eb158/ConditionalFalse 0x1eb14c->0x1eb160/ConditionalTrue 0x1eb158->0x1eb160/Fallthrough 0x1eb160->0x1eb198/ConditionalFalse 0x1eb160->0x1eb1b0/ConditionalTrue 0x1eb198->0x1eb1ac/ConditionalFalse 0x1eb198->0x1eb1b0/ConditionalTrue 0x1eb1ac->0x1eb1b0/Fallthrough 0x1eb1b0->0x1eb1e4/ConditionalFalse 0x1eb1b0->0x1eb1fc/ConditionalTrue 0x1eb1e4->0x1eb1f8/ConditionalFalse 0x1eb1e4->0x1eb1fc/ConditionalTrue 0x1eb1f8->0x1eb1fc/Fallthrough 0x1eb1fc->0x1eb248/ConditionalFalse 0x1eb1fc->0x1eb260/ConditionalTrue 0x1eb248->0x1eb25c/ConditionalFalse 0x1eb248->0x1eb260/ConditionalTrue 0x1eb25c->0x1eb260/Fallthrough 0x1eb260->0x1eb2b0/ConditionalFalse 0x1eb260->0x1eb2c8/ConditionalTrue 0x1eb2b0->0x1eb2c4/ConditionalFalse 0x1eb2b0->0x1eb2c8/ConditionalTrue 0x1eb2c4->0x1eb2c8/Fallthrough 0x1eb2c8->0x1eb2dc/ConditionalFalse 0x1eb2c8->0x1eb2e4/ConditionalTrue 0x1eb2dc->0x1eb2e4/Fallthrough 0x1eb2e4->0x1eb31c/ConditionalFalse 0x1eb2e4->0x1eb334/ConditionalTrue 0x1eb31c->0x1eb330/ConditionalFalse 0x1eb31c->0x1eb334/ConditionalTrue 0x1eb330->0x1eb334/Fallthrough 0x1eb334->0x1eb36c/ConditionalFalse 0x1eb334->0x1eb374/ConditionalTrue 0x1eb36c->0x1eb374/Fallthrough 0x1eb374->0x1eb3ac/ConditionalFalse 0x1eb374->0x1eb3c4/ConditionalTrue 0x1eb3ac->0x1eb3c0/ConditionalFalse 0x1eb3ac->0x1eb3c4/ConditionalTrue 0x1eb3c0->0x1eb3c4/Fallthrough 0x1eb3c4->0x1eb450/ConditionalFalse 0x1eb3c4->0x1eb468/ConditionalTrue 0x1eb450->0x1eb464/ConditionalFalse 0x1eb450->0x1eb468/ConditionalTrue 0x1eb464->0x1eb468/Fallthrough 0x1eb468->0x1eb4a8/ConditionalFalse 0x1eb468->0x1eb4c0/ConditionalTrue 0x1eb4a8->0x1eb4bc/ConditionalFalse 0x1eb4a8->0x1eb4c0/ConditionalTrue 0x1eb4bc->0x1eb4c0/Fallthrough 0x1eb4c0->0x1eb4f4/ConditionalFalse 0x1eb4c0->0x1eb50c/ConditionalTrue 0x1eb4f4->0x1eb508/ConditionalFalse 0x1eb4f4->0x1eb50c/ConditionalTrue 0x1eb508->0x1eb50c/Fallthrough 0x1eb50c->0x1eb558/ConditionalFalse 0x1eb50c->0x1eb570/ConditionalTrue 0x1eb558->0x1eb56c/ConditionalFalse 0x1eb558->0x1eb570/ConditionalTrue 0x1eb56c->0x1eb570/Fallthrough 0x1eb570->0x1eb5ac/ConditionalFalse 0x1eb570->0x1eb5c4/ConditionalTrue 0x1eb5ac->0x1eb5c0/ConditionalFalse 0x1eb5ac->0x1eb5c4/ConditionalTrue 0x1eb5c0->0x1eb5c4/Fallthrough 0x1eb5c4->0x1eb5d8/ConditionalFalse 0x1eb5c4->0x1eb5e0/ConditionalTrue 0x1eb5d8->0x1eb5e0/ConditionalFalse 0x1eb5d8->0x1eb5e8/ConditionalTrue 0x1eb5e0->0x1eb5ec/Branch 0x1eb5e8->0x1eb5ec/Fallthrough 0x1eb5ec->0x1eb620/ConditionalFalse 0x1eb5ec->0x1eb638/ConditionalTrue 0x1eb620->0x1eb634/ConditionalFalse 0x1eb620->0x1eb638/ConditionalTrue 0x1eb634->0x1eb638/Fallthrough 0x1eb638->0x1eb674/ConditionalFalse 0x1eb638->0x1eb68c/ConditionalTrue 0x1eb674->0x1eb688/ConditionalFalse 0x1eb674->0x1eb68c/ConditionalTrue 0x1eb688->0x1eb68c/Fallthrough 0x1eb68c->0x1eb6e8/ConditionalFalse 0x1eb68c->0x1eb700/ConditionalTrue 0x1eb6e8->0x1eb6fc/ConditionalFalse 0x1eb6e8->0x1eb700/ConditionalTrue 0x1eb6fc->0x1eb700/Fallthrough 0x1eb700->0x1eb748/ConditionalFalse 0x1eb700->0x1eb760/ConditionalTrue 0x1eb748->0x1eb75c/ConditionalFalse 0x1eb748->0x1eb760/ConditionalTrue 0x1eb75c->0x1eb760/Fallthrough 0x1eb760->0x1eb790/ConditionalFalse 0x1eb760->0x1eb7a8/ConditionalTrue 0x1eb790->0x1eb7a4/ConditionalFalse 0x1eb790->0x1eb7a8/ConditionalTrue 0x1eb7a4->0x1eb7a8/Fallthrough 0x1eb7a8->0x1eb7b0/ConditionalFalse 0x1eb7a8->0x1eb7b8/ConditionalTrue 0x1eb7b0->0x1eb7bc/Branch 0x1eb7b8->0x1eb7bc/Fallthrough 0x1eb7bc->0x1eb7f4/ConditionalFalse 0x1eb7bc->0x1eb80c/ConditionalTrue 0x1eb7f4->0x1eb808/ConditionalFalse 0x1eb7f4->0x1eb80c/ConditionalTrue 0x1eb808->0x1eb80c/Fallthrough 0x1eb8b4->0x1ead68/Branch 0x1eb8bc->0x1eb0e0/Branch 0x1eb8c4->0x1eb0fc/Branch

# top_level.e25Intrinsics at 0x1eb8e4 (220 bytes)
0x1eb8e4  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1eb8e8  fd030faa           mov      x29, x15
0x1eb8ec  ef8100d1           sub      x15, x15, #0x20
0x1eb8f0  502740f9           ldr      x16, [x26, #0x48]
0x1eb8f4  ff0110eb           cmp      x15, x16
0x1eb8f8  e9050054           b.ls     #0x1eb9b4
0x1eb8fc  70234091           add      x16, x27, #8, lsl #12
0x1eb900  103644f9           ldr      x16, [x16, #0x868]  # pool[4363] = snapshotRef(870)
0x1eb904  7e234091           add      x30, x27, #8, lsl #12
0x1eb908  de7f44f9           ldr      x30, [x30, #0x8f8]  # pool[4381] = "ey"
0x1eb90c  fe4100a9           stp      x30, x16, [x15]
0x1eb910  9f14fd97           bl       #0x130b8c
0x1eb914  a0031ff8           stur     x0, [x29, #-0x10]
0x1eb918  037040b8           ldur     w3, [x0, #7]
0x1eb91c  a3831ff8           stur     x3, [x29, #-8]
0x1eb920  a3000035           cbnz     w3, #0x1eb934
0x1eb924  c0c20091           add      x0, x22, #0x30
0x1eb928  ef031daa           mov      x15, x29
0x1eb92c  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1eb930  c0035fd6           ret      
0x1eb934  e10300aa           mov      x1, x0
0x1eb938  62234091           add      x2, x27, #8, lsl #12
0x1eb93c  423444f9           ldr      x2, [x2, #0x868]  # pool[4363] = snapshotRef(870)
0x1eb940  64a340f9           ldr      x4, [x27, #0x140]  # pool[38] = snapshotRef(34406)
0x1eb944  cc24fd97           bl       #0x134c74
0x1eb948  a0002036           tbz      w0, #4, #0x1eb95c
0x1eb94c  c0c20091           add      x0, x22, #0x30
0x1eb950  ef031daa           mov      x15, x29
0x1eb954  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1eb958  c0035fd6           ret      
0x1eb95c  a2035ff8           ldur     x2, [x29, #-0x10]
0x1eb960  a3835ff8           ldur     x3, [x29, #-8]
0x1eb964  607c4193           sbfx     x0, x3, #1, #0x1f
0x1eb968  010080d2           mov      x1, #0
0x1eb96c  3f0000eb           cmp      x1, x0
0x1eb970  62020054           b.hs     #0x1eb9bc
0x1eb974  41f05ff8           ldur     x1, [x2, #-1]
0x1eb978  217c4cd3           ubfx     x1, x1, #0xc, #0x14
0x1eb97c  21f87fd3           lsl      x1, x1, #1
0x1eb980  3ff00271           cmp      w1, #0xbc
0x1eb984  a1000054           b.ne     #0x1eb998
0x1eb988  413c4039           ldrb     w1, [x2, #0xf]
0x1eb98c  3fac01f1           cmp      x1, #0x6b
0x1eb990  a1000054           b.ne     #0x1eb9a4
0x1eb994  04000014           b        #0x1eb9a4
0x1eb998  41f04078           ldurh    w1, [x2, #0xf]
0x1eb99c  3fac01f1           cmp      x1, #0x6b
0x1eb9a0  20000054           b.eq     #0x1eb9a4
0x1eb9a4  c0c20091           add      x0, x22, #0x30
0x1eb9a8  ef031daa           mov      x15, x29
0x1eb9ac  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1eb9b0  c0035fd6           ret      
0x1eb9b4  15dc0294           bl       #0x2a2a08
0x1eb9b8  d1ffff17           b        #0x1eb8fc
0x1eb9bc  5fdd0294           bl       #0x2a2f38
# CFG: 0x1eb8e4->0x1eb8fc/ConditionalFalse 0x1eb8e4->0x1eb9b4/ConditionalTrue 0x1eb8fc->0x1eb924/ConditionalFalse 0x1eb8fc->0x1eb934/ConditionalTrue 0x1eb934->0x1eb94c/ConditionalFalse 0x1eb934->0x1eb95c/ConditionalTrue 0x1eb95c->0x1eb974/ConditionalFalse 0x1eb95c->0x1eb9bc/ConditionalTrue 0x1eb974->0x1eb988/ConditionalFalse 0x1eb974->0x1eb998/ConditionalTrue 0x1eb988->0x1eb994/ConditionalFalse 0x1eb988->0x1eb9a4/ConditionalTrue 0x1eb994->0x1eb9a4/Branch 0x1eb998->0x1eb9a4/ConditionalTrue 0x1eb998->0x1eb9a4/ConditionalFalse 0x1eb9b4->0x1eb8fc/Branch

# E13Dynamic.probe at 0x1eb9c0 (72 bytes)
0x1eb9c0  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1eb9c4  fd030faa           mov      x29, x15
0x1eb9c8  ef2100d1           sub      x15, x15, #8
0x1eb9cc  502740f9           ldr      x16, [x26, #0x48]
0x1eb9d0  ff0110eb           cmp      x15, x16
0x1eb9d4  69010054           b.ls     #0x1eba00
0x1eb9d8  e20100f9           str      x2, [x15]
0x1eb9dc  040080d2           mov      x4, #0
0x1eb9e0  e00140f9           ldr      x0, [x15]
0x1eb9e4  70234091           add      x16, x27, #8, lsl #12
0x1eb9e8  10022491           add      x16, x16, #0x900
0x1eb9ec  1e1640a9           ldp      x30, x5, [x16]
0x1eb9f0  c0033fd6           blr      x30
0x1eb9f4  ef031daa           mov      x15, x29
0x1eb9f8  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1eb9fc  c0035fd6           ret      
0x1eba00  02dc0294           bl       #0x2a2a08
0x1eba04  f5ffff17           b        #0x1eb9d8
# CFG: 0x1eb9c0->0x1eb9d8/ConditionalFalse 0x1eb9c0->0x1eba00/ConditionalTrue 0x1eba00->0x1eb9d8/Branch

# package:edge_probe/probe_code.dart.E13Dynamic at 0x1eba08 (12 bytes)
0x1eba08  82238cd2           mov      x2, #0x611c
0x1eba0c  e205a0f2           movk     x2, #0x2f, lsl #16
0x1eba10  5bd70214           b        #0x2a177c

# top_level.e23DynamicApply at 0x1eba14 (92 bytes)
0x1eba14  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1eba18  fd030faa           mov      x29, x15
0x1eba1c  ef2100d1           sub      x15, x15, #8
0x1eba20  502740f9           ldr      x16, [x26, #0x48]
0x1eba24  ff0110eb           cmp      x15, x16
0x1eba28  09020054           b.ls     #0x1eba68
0x1eba2c  11000094           bl       #0x1eba70
0x1eba30  810780d2           mov      x1, #0x3c
0x1eba34  60000036           tbz      w0, #0, #0x1eba40
0x1eba38  01f05ff8           ldur     x1, [x0, #-1]
0x1eba3c  217c4cd3           ubfx     x1, x1, #0xc, #0x14
0x1eba40  e00100f9           str      x0, [x15]
0x1eba44  e00301aa           mov      x0, x1
0x1eba48  644741f9           ldr      x4, [x27, #0x288]  # pool[79] = snapshotRef(22)
0x1eba4c  d1e882d2           mov      x17, #0x1746
0x1eba50  1e00118b           add      x30, x0, x17
0x1eba54  be7a7ef8           ldr      x30, [x21, x30, lsl #3]
0x1eba58  c0033fd6           blr      x30
0x1eba5c  ef031daa           mov      x15, x29
0x1eba60  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1eba64  c0035fd6           ret      
0x1eba68  e8db0294           bl       #0x2a2a08
0x1eba6c  f0ffff17           b        #0x1eba2c
# CFG: 0x1eba14->0x1eba2c/ConditionalFalse 0x1eba14->0x1eba68/ConditionalTrue 0x1eba2c->0x1eba38/ConditionalFalse 0x1eba2c->0x1eba40/ConditionalTrue 0x1eba38->0x1eba40/Fallthrough 0x1eba68->0x1eba2c/Branch

# E21Mode.parse at 0x1ebde4 (64 bytes)
0x1ebde4  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1ebde8  fd030faa           mov      x29, x15
0x1ebdec  ef2100d1           sub      x15, x15, #8
0x1ebdf0  502740f9           ldr      x16, [x26, #0x48]
0x1ebdf4  ff0110eb           cmp      x15, x16
0x1ebdf8  29010054           b.ls     #0x1ebe1c
0x1ebdfc  70234091           add      x16, x27, #8, lsl #12
0x1ebe00  10a244f9           ldr      x16, [x16, #0x940]  # pool[4390] = snapshotRef(18055)
0x1ebe04  f00100f9           str      x16, [x15]
0x1ebe08  642740f9           ldr      x4, [x27, #0x48]  # pool[7] = snapshotRef(53)
0x1ebe0c  06000094           bl       #0x1ebe24
0x1ebe10  ef031daa           mov      x15, x29
0x1ebe14  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1ebe18  c0035fd6           ret      
0x1ebe1c  fbda0294           bl       #0x2a2a08
0x1ebe20  f7ffff17           b        #0x1ebdfc
# CFG: 0x1ebde4->0x1ebdfc/ConditionalFalse 0x1ebde4->0x1ebe1c/ConditionalTrue 0x1ebe1c->0x1ebdfc/Branch

# E20Combo.greet at 0x1ebef8 (12 bytes)
0x1ebef8  60234091           add      x0, x27, #8, lsl #12
0x1ebefc  00b444f9           ldr      x0, [x0, #0x968]  # pool[4395] = "base+combo"
0x1ebf00  c0035fd6           ret      

# package:edge_probe/probe_code.dart.E20Combo at 0x1ebf04 (12 bytes)
0x1ebf04  822380d2           mov      x2, #0x11c
0x1ebf08  2209a0f2           movk     x2, #0x49, lsl #16
0x1ebf0c  1cd60214           b        #0x2a177c

# top_level.e18NumericEdges at 0x1ebf10 (412 bytes)
0x1ebf10  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1ebf14  fd030faa           mov      x29, x15
0x1ebf18  ef6100d1           sub      x15, x15, #0x18
0x1ebf1c  0190601e           fmov     d1, #2.50000000
0x1ebf20  502740f9           ldr      x16, [x26, #0x48]
0x1ebf24  ff0110eb           cmp      x15, x16
0x1ebf28  a90a0054           b.ls     #0x1ec07c
0x1ebf2c  201ca14e           mov      v0.16b, v1.16b
0x1ebf30  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1ebf34  fd030faa           mov      x29, x15
0x1ebf38  efed7c92           and      x15, x15, #0xfffffffffffffff0
0x1ebf3c  ff010091           mov      sp, x15
0x1ebf40  50b743f9           ldr      x16, [x26, #0x768]
0x1ebf44  506703f9           str      x16, [x26, #0x6c8]
0x1ebf48  00023fd6           blr      x16
0x1ebf4c  100180d2           mov      x16, #8
0x1ebf50  506703f9           str      x16, [x26, #0x6c8]
0x1ebf54  504b43f9           ldr      x16, [x26, #0x690]
0x1ebf58  1f0640d1           sub      sp, x16, #1, lsl #12
0x1ebf5c  ef031daa           mov      x15, x29
0x1ebf60  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1ebf64  011ca04e           mov      v1.16b, v0.16b
0x1ebf68  0090601e           fmov     d0, #2.50000000
0x1ebf6c  2020601e           fcmp     d1, d0
0x1ebf70  a1020054           b.ne     #0x1ebfc4
0x1ebf74  e10316aa           mov      x1, x22
0x1ebf78  820080d2           mov      x2, #4
0x1ebf7c  60da0294           bl       #0x2a28fc
0x1ebf80  70234091           add      x16, x27, #8, lsl #12
0x1ebf84  10ba44f9           ldr      x16, [x16, #0x970]  # pool[4396] = "integral:"
0x1ebf88  10f000b8           stur     w16, [x0, #0xf]
0x1ebf8c  0090601e           fmov     d0, #2.50000000
0x1ebf90  0020601e           fcmp     d0, d0
0x1ebf94  86070054           b.vs     #0x1ec084
0x1ebf98  0100789e           fcvtzs   x1, d0
0x1ebf9c  30fc5e93           asr      x16, x1, #0x1e
0x1ebfa0  1ffe81eb           cmp      x16, x1, asr #63
0x1ebfa4  01070054           b.ne     #0x1ec084
0x1ebfa8  21f87fd3           lsl      x1, x1, #1
0x1ebfac  013001b8           stur     w1, [x0, #0x13]
0x1ebfb0  e00100f9           str      x0, [x15]
0x1ebfb4  4312fd97           bl       #0x1308c0
0x1ebfb8  ef031daa           mov      x15, x29
0x1ebfbc  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1ebfc0  c0035fd6           ret      
0x1ebfc4  e10316aa           mov      x1, x22
0x1ebfc8  93010094           bl       #0x1ec614
0x1ebfcc  e10316aa           mov      x1, x22
0x1ebfd0  020180d2           mov      x2, #8
0x1ebfd4  a0831ff8           stur     x0, [x29, #-8]
0x1ebfd8  49da0294           bl       #0x2a28fc
0x1ebfdc  a0031ff8           stur     x0, [x29, #-0x10]
0x1ebfe0  70234091           add      x16, x27, #8, lsl #12
0x1ebfe4  10be44f9           ldr      x16, [x16, #0x978]  # pool[4397] = "frac:"
0x1ebfe8  10f000b8           stur     w16, [x0, #0xf]
0x1ebfec  a1835ff8           ldur     x1, [x29, #-8]
0x1ebff0  020280d2           mov      x2, #0x10
0x1ebff4  77000094           bl       #0x1ec1d0
0x1ebff8  a1035ff8           ldur     x1, [x29, #-0x10]
0x1ebffc  394c0091           add      x25, x1, #0x13
0x1ec000  200300b9           str      w0, [x25]
0x1ec004  e0000036           tbz      w0, #0, #0x1ec020
0x1ec008  30f05f38           ldurb    w16, [x1, #-1]
0x1ec00c  11f05f38           ldurb    w17, [x0, #-1]
0x1ec010  300a508a           and      x16, x17, x16, lsr #2
0x1ec014  1f825cea           tst      x16, x28, lsr #32
0x1ec018  40000054           b.eq     #0x1ec020
0x1ec01c  c2d20294           bl       #0x2a0b24
0x1ec020  a0035ff8           ldur     x0, [x29, #-0x10]
0x1ec024  70e351f9           ldr      x16, [x27, #0x23c0]  # pool[1142] = snapshotRef(758)
0x1ec028  107001b8           stur     w16, [x0, #0x17]
0x1ec02c  61234091           add      x1, x27, #8, lsl #12
0x1ec030  21c044f9           ldr      x1, [x1, #0x980]  # pool[4398] = snapshotRef(15147)
0x1ec034  420080d2           mov      x2, #2
0x1ec038  1d000094           bl       #0x1ec0ac
0x1ec03c  a1035ff8           ldur     x1, [x29, #-0x10]
0x1ec040  396c0091           add      x25, x1, #0x1b
0x1ec044  200300b9           str      w0, [x25]
0x1ec048  e0000036           tbz      w0, #0, #0x1ec064
0x1ec04c  30f05f38           ldurb    w16, [x1, #-1]
0x1ec050  11f05f38           ldurb    w17, [x0, #-1]
0x1ec054  300a508a           and      x16, x17, x16, lsr #2
0x1ec058  1f825cea           tst      x16, x28, lsr #32
0x1ec05c  40000054           b.eq     #0x1ec064
0x1ec060  b1d20294           bl       #0x2a0b24
0x1ec064  b0035ff8           ldur     x16, [x29, #-0x10]
0x1ec068  f00100f9           str      x16, [x15]
0x1ec06c  1512fd97           bl       #0x1308c0
0x1ec070  ef031daa           mov      x15, x29
0x1ec074  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1ec078  c0035fd6           ret      
0x1ec07c  83da0294           bl       #0x2a2a88
0x1ec080  abffff17           b        #0x1ebf2c
0x1ec084  e00d9f3c           str      q0, [x15, #-0x10]!
0x1ec088  e08d1ff8           str      x0, [x15, #-8]!
0x1ec08c  800980d2           mov      x0, #0x4c
0x1ec090  7e2b53f9           ldr      x30, [x27, #0x2650]  # pool[1224] = snapshotRef(951)
0x1ec094  de7340f8           ldur     x30, [x30, #7]
0x1ec098  c0033fd6           blr      x30
0x1ec09c  e10300aa           mov      x1, x0
0x1ec0a0  e08540f8           ldr      x0, [x15], #8
0x1ec0a4  e005c13c           ldr      q0, [x15], #0x10
0x1ec0a8  c1ffff17           b        #0x1ebfac
# CFG: 0x1ebf10->0x1ebf2c/ConditionalFalse 0x1ebf10->0x1ec07c/ConditionalTrue 0x1ebf2c->0x1ebf74/ConditionalFalse 0x1ebf2c->0x1ebfc4/ConditionalTrue 0x1ebf74->0x1ebf98/ConditionalFalse 0x1ebf74->0x1ec084/ConditionalTrue 0x1ebf98->0x1ebfa8/ConditionalFalse 0x1ebf98->0x1ec084/ConditionalTrue 0x1ebfa8->0x1ebfac/Fallthrough 0x1ebfc4->0x1ec008/ConditionalFalse 0x1ebfc4->0x1ec020/ConditionalTrue 0x1ec008->0x1ec01c/ConditionalFalse 0x1ec008->0x1ec020/ConditionalTrue 0x1ec01c->0x1ec020/Fallthrough 0x1ec020->0x1ec04c/ConditionalFalse 0x1ec020->0x1ec064/ConditionalTrue 0x1ec04c->0x1ec060/ConditionalFalse 0x1ec04c->0x1ec064/ConditionalTrue 0x1ec060->0x1ec064/Fallthrough 0x1ec07c->0x1ebf2c/Branch 0x1ec084->0x1ebfac/Branch

# top_level.e17JsonRoundTrip at 0x1edca8 (244 bytes)
0x1edca8  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1edcac  fd030faa           mov      x29, x15
0x1edcb0  ef8100d1           sub      x15, x15, #0x20
0x1edcb4  502740f9           ldr      x16, [x26, #0x48]
0x1edcb8  ff0110eb           cmp      x15, x16
0x1edcbc  c9060054           b.ls     #0x1edd94
0x1edcc0  37000094           bl       #0x1edd9c
0x1edcc4  e30300aa           mov      x3, x0
0x1edcc8  e20316aa           mov      x2, x22
0x1edccc  e10316aa           mov      x1, x22
0x1edcd0  a3831ff8           stur     x3, [x29, #-8]
0x1edcd4  840780d2           mov      x4, #0x3c
0x1edcd8  60000036           tbz      w0, #0, #0x1edce4
0x1edcdc  04f05ff8           ldur     x4, [x0, #-1]
0x1edce0  847c4cd3           ubfx     x4, x4, #0xc, #0x14
0x1edce4  846801d1           sub      x4, x4, #0x5a
0x1edce8  9f0800f1           cmp      x4, #2
0x1edcec  89010054           b.ls     #0x1edd1c
0x1edcf0  845800d1           sub      x4, x4, #0x16
0x1edcf4  9fdc00f1           cmp      x4, #0x37
0x1edcf8  29010054           b.ls     #0x1edd1c
0x1edcfc  9ff81ff1           cmp      x4, #0x7fe
0x1edd00  e0000054           b.eq     #0x1edd1c
0x1edd04  9fe420f1           cmp      x4, #0x839
0x1edd08  a0000054           b.eq     #0x1edd1c
0x1edd0c  68ef68f9           ldr      x8, [x27, #0x51d8]  # pool[2617] = snapshotRef(17205)
0x1edd10  63234091           add      x3, x27, #8, lsl #12
0x1edd14  63ec44f9           ldr      x3, [x3, #0x9d8]  # pool[4409] = null
0x1edd18  98ca0294           bl       #0x2a0778
0x1edd1c  a0835ff8           ldur     x0, [x29, #-8]
0x1edd20  01f05ff8           ldur     x1, [x0, #-1]
0x1edd24  217c4cd3           ubfx     x1, x1, #0xc, #0x14
0x1edd28  709b6ff9           ldr      x16, [x27, #0x5f30]  # pool[3044] = snapshotRef(18116)
0x1edd2c  e04100a9           stp      x0, x16, [x15]
0x1edd30  e00301aa           mov      x0, x1
0x1edd34  640f44f9           ldr      x4, [x27, #0x818]  # pool[257] = snapshotRef(54)
0x1edd38  f10788d2           mov      x17, #0x403f
0x1edd3c  1e00118b           add      x30, x0, x17
0x1edd40  be7a7ef8           ldr      x30, [x21, x30, lsl #3]
0x1edd44  c0033fd6           blr      x30
0x1edd48  61234091           add      x1, x27, #8, lsl #12
0x1edd4c  21f444f9           ldr      x1, [x1, #0x9e8]  # pool[4411] = <anonymous closure>
0x1edd50  e20316aa           mov      x2, x22
0x1edd54  a0831ff8           stur     x0, [x29, #-8]
0x1edd58  a2cf0294           bl       #0x2a1be0
0x1edd5c  70234091           add      x16, x27, #8, lsl #12
0x1edd60  10fa44f9           ldr      x16, [x16, #0x9f0]  # pool[4412] = snapshotRef(17926)
0x1edd64  be835ff8           ldur     x30, [x29, #-8]
0x1edd68  fec100a9           stp      x30, x16, [x15, #8]
0x1edd6c  e00100f9           str      x0, [x15]
0x1edd70  642340f9           ldr      x4, [x27, #0x40]  # pool[6] = snapshotRef(55)
0x1edd74  c85aff97           bl       #0x1c4894
0x1edd78  017040b8           ldur     w1, [x0, #7]
0x1edd7c  21801c8b           add      x1, x1, x28, lsl #32
0x1edd80  e20300aa           mov      x2, x0
0x1edd84  88e9fc97           bl       #0x1283a4
0x1edd88  ef031daa           mov      x15, x29
0x1edd8c  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1edd90  c0035fd6           ret      
0x1edd94  1dd30294           bl       #0x2a2a08
0x1edd98  caffff17           b        #0x1edcc0
# CFG: 0x1edca8->0x1edcc0/ConditionalFalse 0x1edca8->0x1edd94/ConditionalTrue 0x1edcc0->0x1edcdc/ConditionalFalse 0x1edcc0->0x1edce4/ConditionalTrue 0x1edcdc->0x1edce4/Fallthrough 0x1edce4->0x1edcf0/ConditionalFalse 0x1edce4->0x1edd1c/ConditionalTrue 0x1edcf0->0x1edcfc/ConditionalFalse 0x1edcf0->0x1edd1c/ConditionalTrue 0x1edcfc->0x1edd04/ConditionalFalse 0x1edcfc->0x1edd1c/ConditionalTrue 0x1edd04->0x1edd0c/ConditionalFalse 0x1edd04->0x1edd1c/ConditionalTrue 0x1edd0c->0x1edd1c/Fallthrough 0x1edd94->0x1edcc0/Branch

# top_level.<anonymous closure> at 0x1edde4 (84 bytes)
0x1edde4  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1edde8  fd030faa           mov      x29, x15
0x1eddec  ef4100d1           sub      x15, x15, #0x10
0x1eddf0  502740f9           ldr      x16, [x26, #0x48]
0x1eddf4  ff0110eb           cmp      x15, x16
0x1eddf8  c9010054           b.ls     #0x1ede30
0x1eddfc  a00b40f9           ldr      x0, [x29, #0x10]
0x1ede00  01f05ff8           ldur     x1, [x0, #-1]
0x1ede04  217c4cd3           ubfx     x1, x1, #0xc, #0x14
0x1ede08  705774f9           ldr      x16, [x27, #0x68a8]  # pool[3347] = snapshotRef(17871)
0x1ede0c  e04100a9           stp      x0, x16, [x15]
0x1ede10  e00301aa           mov      x0, x1
0x1ede14  646f4ef9           ldr      x4, [x27, #0x1cd8]  # pool[921] = snapshotRef(34410)
0x1ede18  1efc3cd1           sub      x30, x0, #0xf3f
0x1ede1c  be7a7ef8           ldr      x30, [x21, x30, lsl #3]
0x1ede20  c0033fd6           blr      x30
0x1ede24  ef031daa           mov      x15, x29
0x1ede28  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1ede2c  c0035fd6           ret      
0x1ede30  f6d20294           bl       #0x2a2a08
0x1ede34  f2ffff17           b        #0x1eddfc
# CFG: 0x1edde4->0x1eddfc/ConditionalFalse 0x1edde4->0x1ede30/ConditionalTrue 0x1ede30->0x1eddfc/Branch

# top_level.e16SortedCopy at 0x1ede38 (108 bytes)
0x1ede38  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1ede3c  fd030faa           mov      x29, x15
0x1ede40  ef4100d1           sub      x15, x15, #0x10
0x1ede44  e20301aa           mov      x2, x1
0x1ede48  a1831ff8           stur     x1, [x29, #-8]
0x1ede4c  502740f9           ldr      x16, [x26, #0x48]
0x1ede50  ff0110eb           cmp      x15, x16
0x1ede54  49020054           b.ls     #0x1ede9c
0x1ede58  61234091           add      x1, x27, #8, lsl #12
0x1ede5c  210445f9           ldr      x1, [x1, #0xa08]  # pool[4415] = snapshotRef(18493)
0x1ede60  eb030094           bl       #0x1eee0c
0x1ede64  e10300aa           mov      x1, x0
0x1ede68  e20316aa           mov      x2, x22
0x1ede6c  a0031ff8           stur     x0, [x29, #-0x10]
0x1ede70  76030094           bl       #0x1eec48
0x1ede74  a1035ff8           ldur     x1, [x29, #-0x10]
0x1ede78  a2835ff8           ldur     x2, [x29, #-8]
0x1ede7c  0a000094           bl       #0x1edea4
0x1ede80  a2035ff8           ldur     x2, [x29, #-0x10]
0x1ede84  61234091           add      x1, x27, #8, lsl #12
0x1ede88  210445f9           ldr      x1, [x1, #0xa08]  # pool[4415] = snapshotRef(18493)
0x1ede8c  d593fd97           bl       #0x152de0
0x1ede90  ef031daa           mov      x15, x29
0x1ede94  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1ede98  c0035fd6           ret      
0x1ede9c  dbd20294           bl       #0x2a2a08
0x1edea0  eeffff17           b        #0x1ede58
# CFG: 0x1ede38->0x1ede58/ConditionalFalse 0x1ede38->0x1ede9c/ConditionalTrue 0x1ede9c->0x1ede58/Branch

# package:edge_probe/probe_code.dart.E15Vec at 0x1eee18 (12 bytes)
0x1eee18  824388d2           mov      x2, #0x421c
0x1eee1c  e205a0f2           movk     x2, #0x2f, lsl #16
0x1eee20  57ca0214           b        #0x2a177c

# E14Statics.bump at 0x1eee24 (168 bytes)
0x1eee24  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1eee28  fd030faa           mov      x29, x15
0x1eee2c  ef2100d1           sub      x15, x15, #8
0x1eee30  502740f9           ldr      x16, [x26, #0x48]
0x1eee34  ff0110eb           cmp      x15, x16
0x1eee38  69040054           b.ls     #0x1eeec4
0x1eee3c  403f40f9           ldr      x0, [x26, #0x78]
0x1eee40  00d443f9           ldr      x0, [x0, #0x7a8]
0x1eee44  017c4193           sbfx     x1, x0, #1, #0x1f
0x1eee48  40000036           tbz      w0, #0, #0x1eee50
0x1eee4c  017040f8           ldur     x1, [x0, #7]
0x1eee50  23040091           add      x3, x1, #1
0x1eee54  a3831ff8           stur     x3, [x29, #-8]
0x1eee58  60787f93           sbfiz    x0, x3, #1, #0x1f
0x1eee5c  7f0480eb           cmp      x3, x0, asr #1
0x1eee60  60000054           b.eq     #0x1eee6c
0x1eee64  49cf0294           bl       #0x2a2b88
0x1eee68  037000f8           stur     x3, [x0, #7]
0x1eee6c  e20300aa           mov      x2, x0
0x1eee70  403f40f9           ldr      x0, [x26, #0x78]
0x1eee74  02d403f9           str      x2, [x0, #0x7a8]
0x1eee78  403f40f9           ldr      x0, [x26, #0x78]
0x1eee7c  00d043f9           ldr      x0, [x0, #0x7a0]
0x1eee80  504b40f9           ldr      x16, [x26, #0x90]
0x1eee84  1f00106b           cmp      w0, w16
0x1eee88  81000054           b.ne     #0x1eee98
0x1eee8c  62234091           add      x2, x27, #8, lsl #12
0x1eee90  42e045f9           ldr      x2, [x2, #0xbc0]  # pool[4470] = E14Statics.stamp
0x1eee94  cbc60294           bl       #0x2a09c0
0x1eee98  017c4193           sbfx     x1, x0, #1, #0x1f
0x1eee9c  40000036           tbz      w0, #0, #0x1eeea4
0x1eeea0  017040f8           ldur     x1, [x0, #7]
0x1eeea4  22000012           and      w2, w1, #1
0x1eeea8  427c40d3           ubfx     x2, x2, #0, #0x20
0x1eeeac  a1835ff8           ldur     x1, [x29, #-8]
0x1eeeb0  2300028b           add      x3, x1, x2
0x1eeeb4  600c0091           add      x0, x3, #3
0x1eeeb8  ef031daa           mov      x15, x29
0x1eeebc  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1eeec0  c0035fd6           ret      
0x1eeec4  d1ce0294           bl       #0x2a2a08
0x1eeec8  ddffff17           b        #0x1eee3c
# CFG: 0x1eee24->0x1eee3c/ConditionalFalse 0x1eee24->0x1eeec4/ConditionalTrue 0x1eee3c->0x1eee4c/ConditionalFalse 0x1eee3c->0x1eee50/ConditionalTrue 0x1eee4c->0x1eee50/Fallthrough 0x1eee50->0x1eee64/ConditionalFalse 0x1eee50->0x1eee6c/ConditionalTrue 0x1eee64->0x1eee6c/Fallthrough 0x1eee6c->0x1eee8c/ConditionalFalse 0x1eee6c->0x1eee98/ConditionalTrue 0x1eee8c->0x1eee98/Fallthrough 0x1eee98->0x1eeea0/ConditionalFalse 0x1eee98->0x1eeea4/ConditionalTrue 0x1eeea0->0x1eeea4/Fallthrough 0x1eeec4->0x1eee3c/Branch

# E14Statics.init:stamp at 0x1eeecc (44 bytes)
0x1eeecc  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1eeed0  fd030faa           mov      x29, x15
0x1eeed4  502740f9           ldr      x16, [x26, #0x48]
0x1eeed8  ff0110eb           cmp      x15, x16
0x1eeedc  a9000054           b.ls     #0x1eeef0
0x1eeee0  06000094           bl       #0x1eeef8
0x1eeee4  ef031daa           mov      x15, x29
0x1eeee8  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1eeeec  c0035fd6           ret      
0x1eeef0  c6ce0294           bl       #0x2a2a08
0x1eeef4  fbffff17           b        #0x1eeee0
# CFG: 0x1eeecc->0x1eeee0/ConditionalFalse 0x1eeecc->0x1eeef0/ConditionalTrue 0x1eeef0->0x1eeee0/Branch

# top_level.e11SyncGen at 0x1eef74 (284 bytes)
0x1eef74  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1eef78  fd030faa           mov      x29, x15
0x1eef7c  ef8100d1           sub      x15, x15, #0x20
0x1eef80  b6831ff8           stur     x22, [x29, #-8]
0x1eef84  a1031ff8           stur     x1, [x29, #-0x10]
0x1eef88  502740f9           ldr      x16, [x26, #0x48]
0x1eef8c  ff0110eb           cmp      x15, x16
0x1eef90  89070054           b.ls     #0x1ef080
0x1eef94  600b44f9           ldr      x0, [x27, #0x810]  # pool[256] = snapshotRef(18479)
0x1eef98  c4dffe97           bl       #0x1a6ea8
0x1eef9c  e00316aa           mov      x0, x22
0x1eefa0  5fdffe97           bl       #0x1a6d1c
0x1eefa4  040080d2           mov      x4, #0
0x1eefa8  a2035ff8           ldur     x2, [x29, #-0x10]
0x1eefac  030080d2           mov      x3, #0
0x1eefb0  a4831ef8           stur     x4, [x29, #-0x18]
0x1eefb4  502740f9           ldr      x16, [x26, #0x48]
0x1eefb8  ff0110eb           cmp      x15, x16
0x1eefbc  69060054           b.ls     #0x1ef088
0x1eefc0  9f0002eb           cmp      x4, x2
0x1eefc4  6a050054           b.ge     #0x1ef070
0x1eefc8  a4020037           tbnz     w4, #0, #0x1ef01c
0x1eefcc  a0cb238b           add      x0, x29, w3, sxtw #2
0x1eefd0  00805ff8           ldur     x0, [x0, #-8]
0x1eefd4  057041b8           ldur     w5, [x0, #0x17]
0x1eefd8  a5801c8b           add      x5, x5, x28, lsl #32
0x1eefdc  80787f93           sbfiz    x0, x4, #1, #0x1f
0x1eefe0  9f0480eb           cmp      x4, x0, asr #1
0x1eefe4  60000054           b.eq     #0x1eeff0
0x1eefe8  e8ce0294           bl       #0x2a2b88
0x1eefec  047000f8           stur     x4, [x0, #7]
0x1eeff0  a07001b8           stur     w0, [x5, #0x17]
0x1eeff4  e0000036           tbz      w0, #0, #0x1ef010
0x1eeff8  b0f05f38           ldurb    w16, [x5, #-1]
0x1eeffc  11f05f38           ldurb    w17, [x0, #-1]
0x1ef000  300a508a           and      x16, x17, x16, lsr #2
0x1ef004  1f825cea           tst      x16, x28, lsr #32
0x1ef008  40000054           b.eq     #0x1ef010
0x1ef00c  f7c70294           bl       #0x2a0fe8
0x1ef010  c0820091           add      x0, x22, #0x20
0x1ef014  1f000094           bl       #0x1ef090
0x1ef018  13000014           b        #0x1ef064
0x1ef01c  e00303aa           mov      x0, x3
0x1ef020  a1cb208b           add      x1, x29, w0, sxtw #2
0x1ef024  21805ff8           ldur     x1, [x1, #-8]
0x1ef028  227041b8           ldur     w2, [x1, #0x17]
0x1ef02c  42801c8b           add      x2, x2, x28, lsl #32
0x1ef030  a1835ef8           ldur     x1, [x29, #-0x18]
0x1ef034  a2031ef8           stur     x2, [x29, #-0x20]
0x1ef038  cfffff97           bl       #0x1eef74
0x1ef03c  a1035ef8           ldur     x1, [x29, #-0x20]
0x1ef040  20b001b8           stur     w0, [x1, #0x1b]
0x1ef044  30f05f38           ldurb    w16, [x1, #-1]
0x1ef048  11f05f38           ldurb    w17, [x0, #-1]
0x1ef04c  300a508a           and      x16, x17, x16, lsr #2
0x1ef050  1f825cea           tst      x16, x28, lsr #32
0x1ef054  40000054           b.eq     #0x1ef05c
0x1ef058  c4c70294           bl       #0x2a0f68
0x1ef05c  c0820091           add      x0, x22, #0x20
0x1ef060  0c000094           bl       #0x1ef090
0x1ef064  a1835ef8           ldur     x1, [x29, #-0x18]
0x1ef068  24040091           add      x4, x1, #1
0x1ef06c  cfffff17           b        #0x1eefa8
0x1ef070  c0c20091           add      x0, x22, #0x30
0x1ef074  ef031daa           mov      x15, x29
0x1ef078  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1ef07c  c0035fd6           ret      
0x1ef080  62ce0294           bl       #0x2a2a08
0x1ef084  c4ffff17           b        #0x1eef94
0x1ef088  60ce0294           bl       #0x2a2a08
0x1ef08c  cdffff17           b        #0x1eefc0
# CFG: 0x1eef74->0x1eef94/ConditionalFalse 0x1eef74->0x1ef080/ConditionalTrue 0x1eef94->0x1eefa8/Fallthrough 0x1eefa8->0x1eefc0/ConditionalFalse 0x1eefa8->0x1ef088/ConditionalTrue 0x1eefc0->0x1eefc8/ConditionalFalse 0x1eefc0->0x1ef070/ConditionalTrue 0x1eefc8->0x1eefcc/ConditionalFalse 0x1eefc8->0x1ef01c/ConditionalTrue 0x1eefcc->0x1eefe8/ConditionalFalse 0x1eefcc->0x1eeff0/ConditionalTrue 0x1eefe8->0x1eeff0/Fallthrough 0x1eeff0->0x1eeff8/ConditionalFalse 0x1eeff0->0x1ef010/ConditionalTrue 0x1eeff8->0x1ef00c/ConditionalFalse 0x1eeff8->0x1ef010/ConditionalTrue 0x1ef00c->0x1ef010/Fallthrough 0x1ef010->0x1ef064/Branch 0x1ef01c->0x1ef058/ConditionalFalse 0x1ef01c->0x1ef05c/ConditionalTrue 0x1ef058->0x1ef05c/Fallthrough 0x1ef05c->0x1ef064/Fallthrough 0x1ef064->0x1eefa8/Branch 0x1ef080->0x1eef94/Branch 0x1ef088->0x1eefc0/Branch

# top_level.e10AsyncLoop at 0x1ef224 (316 bytes)
0x1ef224  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1ef228  fd030faa           mov      x29, x15
0x1ef22c  efc100d1           sub      x15, x15, #0x30
0x1ef230  b6831ff8           stur     x22, [x29, #-8]
0x1ef234  502740f9           ldr      x16, [x26, #0x48]
0x1ef238  ff0110eb           cmp      x15, x16
0x1ef23c  a9080054           b.ls     #0x1ef350
0x1ef240  600b44f9           ldr      x0, [x27, #0x810]  # pool[256] = snapshotRef(18479)
0x1ef244  e86cfd97           bl       #0x14a5e4
0x1ef248  020080d2           mov      x2, #0
0x1ef24c  000080d2           mov      x0, #0
0x1ef250  a2031ff8           stur     x2, [x29, #-0x10]
0x1ef254  a0831ef8           stur     x0, [x29, #-0x18]
0x1ef258  502740f9           ldr      x16, [x26, #0x48]
0x1ef25c  ff0110eb           cmp      x15, x16
0x1ef260  c9070054           b.ls     #0x1ef358
0x1ef264  1f0c00f1           cmp      x0, #3
0x1ef268  8a050054           b.ge     #0x1ef318
0x1ef26c  610b44f9           ldr      x1, [x27, #0x810]  # pool[256] = snapshotRef(18479)
0x1ef270  c16cfd97           bl       #0x14a574
0x1ef274  a0031ef8           stur     x0, [x29, #-0x20]
0x1ef278  1fb000f8           stur     xzr, [x0, #0xb]
0x1ef27c  403f40f9           ldr      x0, [x26, #0x78]
0x1ef280  005443f9           ldr      x0, [x0, #0x6a8]
0x1ef284  504b40f9           ldr      x16, [x26, #0x90]
0x1ef288  1f00106b           cmp      w0, w16
0x1ef28c  61000054           b.ne     #0x1ef298
0x1ef290  627b40f9           ldr      x2, [x27, #0xf0]  # pool[28] = Zone._current
0x1ef294  e5c50294           bl       #0x2a0a28
0x1ef298  a3035ef8           ldur     x3, [x29, #-0x20]
0x1ef29c  603001b8           stur     w0, [x3, #0x13]
0x1ef2a0  a4835ef8           ldur     x4, [x29, #-0x18]
0x1ef2a4  80787f93           sbfiz    x0, x4, #1, #0x1f
0x1ef2a8  9f0480eb           cmp      x4, x0, asr #1
0x1ef2ac  60000054           b.eq     #0x1ef2b8
0x1ef2b0  36ce0294           bl       #0x2a2b88
0x1ef2b4  047000f8           stur     x4, [x0, #7]
0x1ef2b8  e10303aa           mov      x1, x3
0x1ef2bc  e20300aa           mov      x2, x0
0x1ef2c0  b85cfd97           bl       #0x1465a0
0x1ef2c4  a0035ef8           ldur     x0, [x29, #-0x20]
0x1ef2c8  366cfd97           bl       #0x14a3a0
0x1ef2cc  e20300aa           mov      x2, x0
0x1ef2d0  a3035ff8           ldur     x3, [x29, #-0x10]
0x1ef2d4  60787f93           sbfiz    x0, x3, #1, #0x1f
0x1ef2d8  7f0480eb           cmp      x3, x0, asr #1
0x1ef2dc  60000054           b.eq     #0x1ef2e8
0x1ef2e0  2ace0294           bl       #0x2a2b88
0x1ef2e4  037000f8           stur     x3, [x0, #7]
0x1ef2e8  e20100a9           stp      x2, x0, [x15]
0x1ef2ec  afb30294           bl       #0x29c1a8
0x1ef2f0  027c4193           sbfx     x2, x0, #1, #0x1f
0x1ef2f4  40000036           tbz      w0, #0, #0x1ef2fc
0x1ef2f8  027040f8           ldur     x2, [x0, #7]
0x1ef2fc  5f9001f1           cmp      x2, #0x64
0x1ef300  ac000054           b.gt     #0x1ef314
0x1ef304  a0835ef8           ldur     x0, [x29, #-0x18]
0x1ef308  01040091           add      x1, x0, #1
0x1ef30c  e00301aa           mov      x0, x1
0x1ef310  d0ffff17           b        #0x1ef250
0x1ef314  766bfd17           b        #0x14a0ec
0x1ef318  e30302aa           mov      x3, x2
0x1ef31c  616b41f9           ldr      x1, [x27, #0x2d0]  # pool[88] = snapshotRef(18448)
0x1ef320  622b4ff9           ldr      x2, [x27, #0x1e50]  # pool[968] = snapshotInstance(Duration)
0x1ef324  d45efe97           bl       #0x186e74
0x1ef328  e10300aa           mov      x1, x0
0x1ef32c  a1031ef8           stur     x1, [x29, #-0x20]
0x1ef330  1c6cfd97           bl       #0x14a3a0
0x1ef334  a2035ff8           ldur     x2, [x29, #-0x10]
0x1ef338  40787f93           sbfiz    x0, x2, #1, #0x1f
0x1ef33c  5f0480eb           cmp      x2, x0, asr #1
0x1ef340  60000054           b.eq     #0x1ef34c
0x1ef344  11ce0294           bl       #0x2a2b88
0x1ef348  027000f8           stur     x2, [x0, #7]
0x1ef34c  686bfd17           b        #0x14a0ec
0x1ef350  aecd0294           bl       #0x2a2a08
0x1ef354  bbffff17           b        #0x1ef240
0x1ef358  accd0294           bl       #0x2a2a08
0x1ef35c  c2ffff17           b        #0x1ef264
# CFG: 0x1ef224->0x1ef240/ConditionalFalse 0x1ef224->0x1ef350/ConditionalTrue 0x1ef240->0x1ef250/Fallthrough 0x1ef250->0x1ef264/ConditionalFalse 0x1ef250->0x1ef358/ConditionalTrue 0x1ef264->0x1ef26c/ConditionalFalse 0x1ef264->0x1ef318/ConditionalTrue 0x1ef26c->0x1ef290/ConditionalFalse 0x1ef26c->0x1ef298/ConditionalTrue 0x1ef290->0x1ef298/Fallthrough 0x1ef298->0x1ef2b0/ConditionalFalse 0x1ef298->0x1ef2b8/ConditionalTrue 0x1ef2b0->0x1ef2b8/Fallthrough 0x1ef2b8->0x1ef2e0/ConditionalFalse 0x1ef2b8->0x1ef2e8/ConditionalTrue 0x1ef2e0->0x1ef2e8/Fallthrough 0x1ef2e8->0x1ef2f8/ConditionalFalse 0x1ef2e8->0x1ef2fc/ConditionalTrue 0x1ef2f8->0x1ef2fc/Fallthrough 0x1ef2fc->0x1ef304/ConditionalFalse 0x1ef2fc->0x1ef314/ConditionalTrue 0x1ef304->0x1ef250/Branch 0x1ef318->0x1ef344/ConditionalFalse 0x1ef318->0x1ef34c/ConditionalTrue 0x1ef344->0x1ef34c/Fallthrough 0x1ef350->0x1ef240/Branch 0x1ef358->0x1ef264/Branch

# top_level.e09TryRethrow at 0x1ef360 (376 bytes)
0x1ef360  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1ef364  fd030faa           mov      x29, x15
0x1ef368  ef2102d1           sub      x15, x15, #0x88
0x1ef36c  502740f9           ldr      x16, [x26, #0x48]
0x1ef370  ff0110eb           cmp      x15, x16
0x1ef374  e90a0054           b.ls     #0x1ef4d0
0x1ef378  70db40f9           ldr      x16, [x27, #0x1b0]  # pool[52] = snapshotRef(903)
0x1ef37c  7e234091           add      x30, x27, #8, lsl #12
0x1ef380  de4344f9           ldr      x30, [x30, #0x880]  # pool[4366] = snapshotRef(610)
0x1ef384  fe4100a9           stp      x30, x16, [x15]
0x1ef388  0106fd97           bl       #0x130b8c
0x1ef38c  e10316aa           mov      x1, x22
0x1ef390  820080d2           mov      x2, #4
0x1ef394  a08319f8           stur     x0, [x29, #-0x68]
0x1ef398  59cd0294           bl       #0x2a28fc
0x1ef39c  70234091           add      x16, x27, #8, lsl #12
0x1ef3a0  101646f9           ldr      x16, [x16, #0xc28]  # pool[4483] = "ok:"
0x1ef3a4  10f000b8           stur     w16, [x0, #0xf]
0x1ef3a8  d00080d2           mov      x16, #6
0x1ef3ac  103001b8           stur     w16, [x0, #0x13]
0x1ef3b0  b08359f8           ldur     x16, [x29, #-0x68]
0x1ef3b4  7e234091           add      x30, x27, #8, lsl #12
0x1ef3b8  de1b46f9           ldr      x30, [x30, #0xc30]  # pool[4484] = snapshotRef(109)
0x1ef3bc  fe4100a9           stp      x30, x16, [x15]
0x1ef3c0  f305fd97           bl       #0x130b8c
0x1ef3c4  60234091           add      x0, x27, #8, lsl #12
0x1ef3c8  001c46f9           ldr      x0, [x0, #0xc38]  # pool[4485] = "ok:3"
0x1ef3cc  ef031daa           mov      x15, x29
0x1ef3d0  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1ef3d4  c0035fd6           ret      
0x1ef3d8  af2302d1           sub      x15, x29, #0x88
0x1ef3dc  e30300aa           mov      x3, x0
0x1ef3e0  a08319f8           stur     x0, [x29, #-0x68]
0x1ef3e4  e00301aa           mov      x0, x1
0x1ef3e8  a10319f8           stur     x1, [x29, #-0x70]
0x1ef3ec  810780d2           mov      x1, #0x3c
0x1ef3f0  63000036           tbz      w3, #0, #0x1ef3fc
0x1ef3f4  61f05ff8           ldur     x1, [x3, #-1]
0x1ef3f8  217c4cd3           ubfx     x1, x1, #0xc, #0x14
0x1ef3fc  3f0823f1           cmp      x1, #0x8c2
0x1ef400  80030054           b.eq     #0x1ef470
0x1ef404  e10316aa           mov      x1, x22
0x1ef408  820080d2           mov      x2, #4
0x1ef40c  3ccd0294           bl       #0x2a28fc
0x1ef410  a08318f8           stur     x0, [x29, #-0x78]
0x1ef414  70234091           add      x16, x27, #8, lsl #12
0x1ef418  102246f9           ldr      x16, [x16, #0xc40]  # pool[4486] = "fallback"
0x1ef41c  10f000b8           stur     w16, [x0, #0xf]
0x1ef420  b00359f8           ldur     x16, [x29, #-0x70]
0x1ef424  f00100f9           str      x16, [x15]
0x1ef428  ac4ffe97           bl       #0x1832d8
0x1ef42c  e10300aa           mov      x1, x0
0x1ef430  a08358f8           ldur     x0, [x29, #-0x78]
0x1ef434  013001b8           stur     w1, [x0, #0x13]
0x1ef438  e00100f9           str      x0, [x15]
0x1ef43c  2105fd97           bl       #0x1308c0
0x1ef440  b0035af8           ldur     x16, [x29, #-0x60]
0x1ef444  e04100a9           stp      x0, x16, [x15]
0x1ef448  d105fd97           bl       #0x130b8c
0x1ef44c  a08318f8           stur     x0, [x29, #-0x78]
0x1ef450  70234091           add      x16, x27, #8, lsl #12
0x1ef454  101a46f9           ldr      x16, [x16, #0xc30]  # pool[4484] = snapshotRef(109)
0x1ef458  f00100a9           stp      x16, x0, [x15]
0x1ef45c  cc05fd97           bl       #0x130b8c
0x1ef460  a08358f8           ldur     x0, [x29, #-0x78]
0x1ef464  ef031daa           mov      x15, x29
0x1ef468  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1ef46c  c0035fd6           ret      
0x1ef470  e00303aa           mov      x0, x3
0x1ef474  017040b8           ldur     w1, [x0, #7]
0x1ef478  21801c8b           add      x1, x1, x28, lsl #32
0x1ef47c  b0035af8           ldur     x16, [x29, #-0x60]
0x1ef480  e14100a9           stp      x1, x16, [x15]
0x1ef484  c205fd97           bl       #0x130b8c
0x1ef488  e20300aa           mov      x2, x0
0x1ef48c  a08359f8           ldur     x0, [x29, #-0x68]
0x1ef490  a10359f8           ldur     x1, [x29, #-0x70]
0x1ef494  a28318f8           stur     x2, [x29, #-0x78]
0x1ef498  8fc50294           bl       #0x2a0ad4
0x1ef49c  000020d4           brk      #0
0x1ef4a0  af2302d1           sub      x15, x29, #0x88
0x1ef4a4  a08319f8           stur     x0, [x29, #-0x68]
0x1ef4a8  a10319f8           stur     x1, [x29, #-0x70]
0x1ef4ac  b0035af8           ldur     x16, [x29, #-0x60]
0x1ef4b0  7e234091           add      x30, x27, #8, lsl #12
0x1ef4b4  de1b46f9           ldr      x30, [x30, #0xc30]  # pool[4484] = snapshotRef(109)
0x1ef4b8  fe4100a9           stp      x30, x16, [x15]
0x1ef4bc  b405fd97           bl       #0x130b8c
0x1ef4c0  a08359f8           ldur     x0, [x29, #-0x68]
0x1ef4c4  a10359f8           ldur     x1, [x29, #-0x70]
0x1ef4c8  83c50294           bl       #0x2a0ad4
0x1ef4cc  000020d4           brk      #0
0x1ef4d0  4ecd0294           bl       #0x2a2a08
0x1ef4d4  a9ffff17           b        #0x1ef378
# CFG: 0x1ef360->0x1ef378/ConditionalFalse 0x1ef360->0x1ef4d0/ConditionalTrue 0x1ef3d8->0x1ef3f4/ConditionalFalse 0x1ef3d8->0x1ef3fc/ConditionalTrue 0x1ef3f4->0x1ef3fc/Fallthrough 0x1ef3fc->0x1ef404/ConditionalFalse 0x1ef3fc->0x1ef470/ConditionalTrue 0x1ef470->0x1ef4d0/Fallthrough 0x1ef4d0->0x1ef378/Branch

# top_level.e07GenericBound at 0x1ef4d8 (156 bytes)
0x1ef4d8  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1ef4dc  fd030faa           mov      x29, x15
0x1ef4e0  ef8100d1           sub      x15, x15, #0x20
0x1ef4e4  80f040b8           ldur     w0, [x4, #0xf]
0x1ef4e8  60000035           cbnz     w0, #0x1ef4f4
0x1ef4ec  e10316aa           mov      x1, x22
0x1ef4f0  05000014           b        #0x1ef504
0x1ef4f4  817041b8           ldur     w1, [x4, #0x17]
0x1ef4f8  a2cb218b           add      x2, x29, w1, sxtw #2
0x1ef4fc  420840f9           ldr      x2, [x2, #0x10]
0x1ef500  e10302aa           mov      x1, x2
0x1ef504  502740f9           ldr      x16, [x26, #0x48]
0x1ef508  ff0110eb           cmp      x15, x16
0x1ef50c  09030054           b.ls     #0x1ef56c
0x1ef510  80000035           cbnz     w0, #0x1ef520
0x1ef514  60234091           add      x0, x27, #8, lsl #12
0x1ef518  002446f9           ldr      x0, [x0, #0xc48]  # pool[4487] = snapshotRef(18170)
0x1ef51c  02000014           b        #0x1ef524
0x1ef520  e00301aa           mov      x0, x1
0x1ef524  a0831ff8           stur     x0, [x29, #-8]
0x1ef528  61234091           add      x1, x27, #8, lsl #12
0x1ef52c  212846f9           ldr      x1, [x1, #0xc50]  # pool[4488] = <anonymous closure>
0x1ef530  e20316aa           mov      x2, x22
0x1ef534  abc90294           bl       #0x2a1be0
0x1ef538  e10300aa           mov      x1, x0
0x1ef53c  a0835ff8           ldur     x0, [x29, #-8]
0x1ef540  20b000b8           stur     w0, [x1, #0xb]
0x1ef544  70234091           add      x16, x27, #8, lsl #12
0x1ef548  102e46f9           ldr      x16, [x16, #0xc58]  # pool[4489] = snapshotRef(18337)
0x1ef54c  be0b40f9           ldr      x30, [x29, #0x10]
0x1ef550  fec100a9           stp      x30, x16, [x15, #8]
0x1ef554  e10100f9           str      x1, [x15]
0x1ef558  64f374f9           ldr      x4, [x27, #0x69e0]  # pool[3386] = snapshotRef(34520)
0x1ef55c  5f610294           bl       #0x287ad8
0x1ef560  ef031daa           mov      x15, x29
0x1ef564  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1ef568  c0035fd6           ret      
0x1ef56c  27cd0294           bl       #0x2a2a08
0x1ef570  e8ffff17           b        #0x1ef510
# CFG: 0x1ef4d8->0x1ef4ec/ConditionalFalse 0x1ef4d8->0x1ef4f4/ConditionalTrue 0x1ef4ec->0x1ef504/Branch 0x1ef4f4->0x1ef504/Fallthrough 0x1ef504->0x1ef510/ConditionalFalse 0x1ef504->0x1ef56c/ConditionalTrue 0x1ef510->0x1ef514/ConditionalFalse 0x1ef510->0x1ef520/ConditionalTrue 0x1ef514->0x1ef524/Branch 0x1ef520->0x1ef524/Fallthrough 0x1ef56c->0x1ef510/Branch

# top_level.<anonymous closure> at 0x1ef574 (120 bytes)
0x1ef574  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1ef578  fd030faa           mov      x29, x15
0x1ef57c  ef6100d1           sub      x15, x15, #0x18
0x1ef580  502740f9           ldr      x16, [x26, #0x48]
0x1ef584  ff0110eb           cmp      x15, x16
0x1ef588  e9020054           b.ls     #0x1ef5e4
0x1ef58c  a00b40f9           ldr      x0, [x29, #0x10]
0x1ef590  810780d2           mov      x1, #0x3c
0x1ef594  60000036           tbz      w0, #0, #0x1ef5a0
0x1ef598  01f05ff8           ldur     x1, [x0, #-1]
0x1ef59c  217c4cd3           ubfx     x1, x1, #0xc, #0x14
0x1ef5a0  500080d2           mov      x16, #2
0x1ef5a4  f00100a9           stp      x16, x0, [x15]
0x1ef5a8  e00301aa           mov      x0, x1
0x1ef5ac  1ef43fd1           sub      x30, x0, #0xffd
0x1ef5b0  be7a7ef8           ldr      x30, [x21, x30, lsl #3]
0x1ef5b4  c0033fd6           blr      x30
0x1ef5b8  61234091           add      x1, x27, #8, lsl #12
0x1ef5bc  212c46f9           ldr      x1, [x1, #0xc58]  # pool[4489] = snapshotRef(18337)
0x1ef5c0  a0831ff8           stur     x0, [x29, #-8]
0x1ef5c4  0a000094           bl       #0x1ef5ec
0x1ef5c8  a10f40f9           ldr      x1, [x29, #0x18]
0x1ef5cc  01b000b8           stur     w1, [x0, #0xb]
0x1ef5d0  a1835ff8           ldur     x1, [x29, #-8]
0x1ef5d4  01f000b8           stur     w1, [x0, #0xf]
0x1ef5d8  ef031daa           mov      x15, x29
0x1ef5dc  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1ef5e0  c0035fd6           ret      
0x1ef5e4  09cd0294           bl       #0x2a2a08
0x1ef5e8  e9ffff17           b        #0x1ef58c
# CFG: 0x1ef574->0x1ef58c/ConditionalFalse 0x1ef574->0x1ef5e4/ConditionalTrue 0x1ef58c->0x1ef598/ConditionalFalse 0x1ef58c->0x1ef5a0/ConditionalTrue 0x1ef598->0x1ef5a0/Fallthrough 0x1ef5e4->0x1ef58c/Branch

# top_level.e06RecordDestructure at 0x1ef5f8 (380 bytes)
0x1ef5f8  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1ef5fc  fd030faa           mov      x29, x15
0x1ef600  efe100d1           sub      x15, x15, #0x38
0x1ef604  a1831ff8           stur     x1, [x29, #-8]
0x1ef608  502740f9           ldr      x16, [x26, #0x48]
0x1ef60c  ff0110eb           cmp      x15, x16
0x1ef610  a90a0054           b.ls     #0x1ef764
0x1ef614  420080d2           mov      x2, #2
0x1ef618  830080d2           mov      x3, #4
0x1ef61c  ccc70294           bl       #0x2a154c
0x1ef620  e10316aa           mov      x1, x22
0x1ef624  820080d2           mov      x2, #4
0x1ef628  a0031ff8           stur     x0, [x29, #-0x10]
0x1ef62c  b4cc0294           bl       #0x2a28fc
0x1ef630  e10300aa           mov      x1, x0
0x1ef634  a0035ff8           ldur     x0, [x29, #-0x10]
0x1ef638  a1831ef8           stur     x1, [x29, #-0x18]
0x1ef63c  20f000b8           stur     w0, [x1, #0xf]
0x1ef640  c20080d2           mov      x2, #6
0x1ef644  030180d2           mov      x3, #8
0x1ef648  c1c70294           bl       #0x2a154c
0x1ef64c  a3835ef8           ldur     x3, [x29, #-0x18]
0x1ef650  603001b8           stur     w0, [x3, #0x13]
0x1ef654  040080d2           mov      x4, #0
0x1ef658  000080d2           mov      x0, #0
0x1ef65c  a4831df8           stur     x4, [x29, #-0x28]
0x1ef660  502740f9           ldr      x16, [x26, #0x48]
0x1ef664  ff0110eb           cmp      x15, x16
0x1ef668  29080054           b.ls     #0x1ef76c
0x1ef66c  1f0800f1           cmp      x0, #2
0x1ef670  ea040054           b.ge     #0x1ef70c
0x1ef674  7008008b           add      x16, x3, x0, lsl #2
0x1ef678  05f240b8           ldur     w5, [x16, #0xf]
0x1ef67c  a5801c8b           add      x5, x5, x28, lsl #32
0x1ef680  a5031ff8           stur     x5, [x29, #-0x10]
0x1ef684  06040091           add      x6, x0, #1
0x1ef688  a6031ef8           stur     x6, [x29, #-0x20]
0x1ef68c  bf00166b           cmp      w5, w22
0x1ef690  21010054           b.ne     #0x1ef6b4
0x1ef694  e00305aa           mov      x0, x5
0x1ef698  e20316aa           mov      x2, x22
0x1ef69c  e10316aa           mov      x1, x22
0x1ef6a0  68234091           add      x8, x27, #8, lsl #12
0x1ef6a4  085146f9           ldr      x8, [x8, #0xca0]  # pool[4498] = snapshotRef(34344)
0x1ef6a8  63234091           add      x3, x27, #8, lsl #12
0x1ef6ac  635446f9           ldr      x3, [x3, #0xca8]  # pool[4499] = null
0x1ef6b0  31000094           bl       #0x1ef774
0x1ef6b4  a1835df8           ldur     x1, [x29, #-0x28]
0x1ef6b8  a0035ff8           ldur     x0, [x29, #-0x10]
0x1ef6bc  02f040b8           ldur     w2, [x0, #0xf]
0x1ef6c0  42801c8b           add      x2, x2, x28, lsl #32
0x1ef6c4  033041b8           ldur     w3, [x0, #0x13]
0x1ef6c8  63801c8b           add      x3, x3, x28, lsl #32
0x1ef6cc  800780d2           mov      x0, #0x3c
0x1ef6d0  62000036           tbz      w2, #0, #0x1ef6dc
0x1ef6d4  40f05ff8           ldur     x0, [x2, #-1]
0x1ef6d8  007c4cd3           ubfx     x0, x0, #0xc, #0x14
0x1ef6dc  e30900a9           stp      x3, x2, [x15]
0x1ef6e0  1ec03fd1           sub      x30, x0, #0xff0
0x1ef6e4  be7a7ef8           ldr      x30, [x21, x30, lsl #3]
0x1ef6e8  c0033fd6           blr      x30
0x1ef6ec  017c4193           sbfx     x1, x0, #1, #0x1f
0x1ef6f0  40000036           tbz      w0, #0, #0x1ef6f8
0x1ef6f4  017040f8           ldur     x1, [x0, #7]
0x1ef6f8  a2835df8           ldur     x2, [x29, #-0x28]
0x1ef6fc  4400018b           add      x4, x2, x1
0x1ef700  a0035ef8           ldur     x0, [x29, #-0x20]
0x1ef704  a3835ef8           ldur     x3, [x29, #-0x18]
0x1ef708  d5ffff17           b        #0x1ef65c
0x1ef70c  a1835ff8           ldur     x1, [x29, #-8]
0x1ef710  e20304aa           mov      x2, x4
0x1ef714  23f040b8           ldur     w3, [x1, #0xf]
0x1ef718  63801c8b           add      x3, x3, x28, lsl #32
0x1ef71c  243041b8           ldur     w4, [x1, #0x13]
0x1ef720  84801c8b           add      x4, x4, x28, lsl #32
0x1ef724  257041b8           ldur     w5, [x1, #0x17]
0x1ef728  a5801c8b           add      x5, x5, x28, lsl #32
0x1ef72c  617c4193           sbfx     x1, x3, #1, #0x1f
0x1ef730  43000036           tbz      w3, #0, #0x1ef738
0x1ef734  617040f8           ldur     x1, [x3, #7]
0x1ef738  4300018b           add      x3, x2, x1
0x1ef73c  817c4193           sbfx     x1, x4, #1, #0x1f
0x1ef740  44000036           tbz      w4, #0, #0x1ef748
0x1ef744  817040f8           ldur     x1, [x4, #7]
0x1ef748  620001cb           sub      x2, x3, x1
0x1ef74c  a17040b8           ldur     w1, [x5, #7]
0x1ef750  237c4193           sbfx     x3, x1, #1, #0x1f
0x1ef754  4000038b           add      x0, x2, x3
0x1ef758  ef031daa           mov      x15, x29
0x1ef75c  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1ef760  c0035fd6           ret      
0x1ef764  a9cc0294           bl       #0x2a2a08
0x1ef768  abffff17           b        #0x1ef614
0x1ef76c  a7cc0294           bl       #0x2a2a08
0x1ef770  bfffff17           b        #0x1ef66c
# CFG: 0x1ef5f8->0x1ef614/ConditionalFalse 0x1ef5f8->0x1ef764/ConditionalTrue 0x1ef614->0x1ef65c/Fallthrough 0x1ef65c->0x1ef66c/ConditionalFalse 0x1ef65c->0x1ef76c/ConditionalTrue 0x1ef66c->0x1ef674/ConditionalFalse 0x1ef66c->0x1ef70c/ConditionalTrue 0x1ef674->0x1ef694/ConditionalFalse 0x1ef674->0x1ef6b4/ConditionalTrue 0x1ef694->0x1ef6b4/Fallthrough 0x1ef6b4->0x1ef6d4/ConditionalFalse 0x1ef6b4->0x1ef6dc/ConditionalTrue 0x1ef6d4->0x1ef6dc/Fallthrough 0x1ef6dc->0x1ef6f4/ConditionalFalse 0x1ef6dc->0x1ef6f8/ConditionalTrue 0x1ef6f4->0x1ef6f8/Fallthrough 0x1ef6f8->0x1ef65c/Branch 0x1ef70c->0x1ef734/ConditionalFalse 0x1ef70c->0x1ef738/ConditionalTrue 0x1ef734->0x1ef738/Fallthrough 0x1ef738->0x1ef744/ConditionalFalse 0x1ef738->0x1ef748/ConditionalTrue 0x1ef744->0x1ef748/Fallthrough 0x1ef764->0x1ef614/Branch 0x1ef76c->0x1ef66c/Branch

# top_level.e05NullFlow at 0x1ef7e0 (360 bytes)
0x1ef7e0  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1ef7e4  fd030faa           mov      x29, x15
0x1ef7e8  efa100d1           sub      x15, x15, #0x28
0x1ef7ec  e00301aa           mov      x0, x1
0x1ef7f0  a1831ff8           stur     x1, [x29, #-8]
0x1ef7f4  502740f9           ldr      x16, [x26, #0x48]
0x1ef7f8  ff0110eb           cmp      x15, x16
0x1ef7fc  290a0054           b.ls     #0x1ef940
0x1ef800  e10300aa           mov      x1, x0
0x1ef804  62234091           add      x2, x27, #8, lsl #12
0x1ef808  423444f9           ldr      x2, [x2, #0x868]  # pool[4363] = snapshotRef(870)
0x1ef80c  f9e6fc97           bl       #0x1293f0
0x1ef810  e10300aa           mov      x1, x0
0x1ef814  a0835ff8           ldur     x0, [x29, #-8]
0x1ef818  02f040b8           ldur     w2, [x0, #0xf]
0x1ef81c  42801c8b           add      x2, x2, x28, lsl #32
0x1ef820  5f00016b           cmp      w2, w1
0x1ef824  61000054           b.ne     #0x1ef830
0x1ef828  e40316aa           mov      x4, x22
0x1ef82c  02000014           b        #0x1ef834
0x1ef830  e40301aa           mov      x4, x1
0x1ef834  430080d2           mov      x3, #2
0x1ef838  e20303aa           mov      x2, x3
0x1ef83c  a4031ff8           stur     x4, [x29, #-0x10]
0x1ef840  e10316aa           mov      x1, x22
0x1ef844  2ecc0294           bl       #0x2a28fc
0x1ef848  a0831ef8           stur     x0, [x29, #-0x18]
0x1ef84c  70234091           add      x16, x27, #8, lsl #12
0x1ef850  104644f9           ldr      x16, [x16, #0x888]  # pool[4367] = snapshotRef(295)
0x1ef854  10f000b8           stur     w16, [x0, #0xf]
0x1ef858  61b743f9           ldr      x1, [x27, #0x768]  # pool[235] = snapshotRef(18261)
0x1ef85c  dcc70294           bl       #0x2a17cc
0x1ef860  e30300aa           mov      x3, x0
0x1ef864  a0835ef8           ldur     x0, [x29, #-0x18]
0x1ef868  a3031ef8           stur     x3, [x29, #-0x20]
0x1ef86c  60f000b8           stur     w0, [x3, #0xf]
0x1ef870  400080d2           mov      x0, #2
0x1ef874  60b000b8           stur     w0, [x3, #0xb]
0x1ef878  a4035ff8           ldur     x4, [x29, #-0x10]
0x1ef87c  9f00166b           cmp      w4, w22
0x1ef880  61000054           b.ne     #0x1ef88c
0x1ef884  e20316aa           mov      x2, x22
0x1ef888  09000014           b        #0x1ef8ac
0x1ef88c  80f05ff8           ldur     x0, [x4, #-1]
0x1ef890  007c4cd3           ubfx     x0, x0, #0xc, #0x14
0x1ef894  e10304aa           mov      x1, x4
0x1ef898  62db40f9           ldr      x2, [x27, #0x1b0]  # pool[52] = snapshotRef(903)
0x1ef89c  1e0440d1           sub      x30, x0, #1, lsl #12
0x1ef8a0  be7a7ef8           ldr      x30, [x21, x30, lsl #3]
0x1ef8a4  c0033fd6           blr      x30
0x1ef8a8  e20300aa           mov      x2, x0
0x1ef8ac  5f00166b           cmp      w2, w22
0x1ef8b0  60000054           b.eq     #0x1ef8bc
0x1ef8b4  a1035ef8           ldur     x1, [x29, #-0x20]
0x1ef8b8  b7e0fc97           bl       #0x127b94
0x1ef8bc  a0035ff8           ldur     x0, [x29, #-0x10]
0x1ef8c0  1f00166b           cmp      w0, w22
0x1ef8c4  01010054           b.ne     #0x1ef8e4
0x1ef8c8  70e36af9           ldr      x16, [x27, #0x55c0]  # pool[2742] = snapshotRef(272)
0x1ef8cc  f00100f9           str      x16, [x15]
0x1ef8d0  a1035ef8           ldur     x1, [x29, #-0x20]
0x1ef8d4  641342f9           ldr      x4, [x27, #0x420]  # pool[130] = snapshotRef(34387)
0x1ef8d8  203e0294           bl       #0x27f158
0x1ef8dc  e30300aa           mov      x3, x0
0x1ef8e0  02000014           b        #0x1ef8e8
0x1ef8e4  e30300aa           mov      x3, x0
0x1ef8e8  a0835ff8           ldur     x0, [x29, #-8]
0x1ef8ec  e10300aa           mov      x1, x0
0x1ef8f0  a3031ff8           stur     x3, [x29, #-0x10]
0x1ef8f4  62234091           add      x2, x27, #8, lsl #12
0x1ef8f8  426046f9           ldr      x2, [x2, #0xcc0]  # pool[4502] = "missing"
0x1ef8fc  bde6fc97           bl       #0x1293f0
0x1ef900  a1835ff8           ldur     x1, [x29, #-8]
0x1ef904  22f040b8           ldur     w2, [x1, #0xf]
0x1ef908  42801c8b           add      x2, x2, x28, lsl #32
0x1ef90c  5f00006b           cmp      w2, w0
0x1ef910  41000054           b.ne     #0x1ef918
0x1ef914  e00316aa           mov      x0, x22
0x1ef918  1f00166b           cmp      w0, w22
0x1ef91c  c1000054           b.ne     #0x1ef934
0x1ef920  a3035ff8           ldur     x3, [x29, #-0x10]
0x1ef924  62234091           add      x2, x27, #8, lsl #12
0x1ef928  426046f9           ldr      x2, [x2, #0xcc0]  # pool[4502] = "missing"
0x1ef92c  5ba00294           bl       #0x297a98
0x1ef930  a0035ff8           ldur     x0, [x29, #-0x10]
0x1ef934  ef031daa           mov      x15, x29
0x1ef938  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1ef93c  c0035fd6           ret      
0x1ef940  32cc0294           bl       #0x2a2a08
0x1ef944  afffff17           b        #0x1ef800
# CFG: 0x1ef7e0->0x1ef800/ConditionalFalse 0x1ef7e0->0x1ef940/ConditionalTrue 0x1ef800->0x1ef828/ConditionalFalse 0x1ef800->0x1ef830/ConditionalTrue 0x1ef828->0x1ef834/Branch 0x1ef830->0x1ef834/Fallthrough 0x1ef834->0x1ef884/ConditionalFalse 0x1ef834->0x1ef88c/ConditionalTrue 0x1ef884->0x1ef8ac/Branch 0x1ef88c->0x1ef8ac/Fallthrough 0x1ef8ac->0x1ef8b4/ConditionalFalse 0x1ef8ac->0x1ef8bc/ConditionalTrue 0x1ef8b4->0x1ef8bc/Fallthrough 0x1ef8bc->0x1ef8c8/ConditionalFalse 0x1ef8bc->0x1ef8e4/ConditionalTrue 0x1ef8c8->0x1ef8e8/Branch 0x1ef8e4->0x1ef8e8/Fallthrough 0x1ef8e8->0x1ef914/ConditionalFalse 0x1ef8e8->0x1ef918/ConditionalTrue 0x1ef914->0x1ef918/Fallthrough 0x1ef918->0x1ef920/ConditionalFalse 0x1ef918->0x1ef934/ConditionalTrue 0x1ef920->0x1ef934/Fallthrough 0x1ef940->0x1ef800/Branch

# top_level.e04BitTwiddle at 0x1ef948 (8 bytes)
0x1ef948  800880d2           mov      x0, #0x44
0x1ef94c  c0035fd6           ret      

# top_level.e02Cascade at 0x1ef950 (184 bytes)
0x1ef950  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1ef954  fd030faa           mov      x29, x15
0x1ef958  ef4100d1           sub      x15, x15, #0x10
0x1ef95c  e00301aa           mov      x0, x1
0x1ef960  a1831ff8           stur     x1, [x29, #-8]
0x1ef964  502740f9           ldr      x16, [x26, #0x48]
0x1ef968  ff0110eb           cmp      x15, x16
0x1ef96c  a9040054           b.ls     #0x1efa00
0x1ef970  61b743f9           ldr      x1, [x27, #0x768]  # pool[235] = snapshotRef(18261)
0x1ef974  020080d2           mov      x2, #0
0x1ef978  3fe3fc97           bl       #0x128674
0x1ef97c  e10300aa           mov      x1, x0
0x1ef980  a2835ff8           ldur     x2, [x29, #-8]
0x1ef984  a0831ff8           stur     x0, [x29, #-8]
0x1ef988  83e0fc97           bl       #0x127b94
0x1ef98c  a1835ff8           ldur     x1, [x29, #-8]
0x1ef990  640b42f9           ldr      x4, [x27, #0x410]  # pool[128] = snapshotRef(34439)
0x1ef994  b6b9fd97           bl       #0x15e06c
0x1ef998  a0835ff8           ldur     x0, [x29, #-8]
0x1ef99c  01b040b8           ldur     w1, [x0, #0xb]
0x1ef9a0  02f040b8           ldur     w2, [x0, #0xf]
0x1ef9a4  42801c8b           add      x2, x2, x28, lsl #32
0x1ef9a8  43b040b8           ldur     w3, [x2, #0xb]
0x1ef9ac  227c4193           sbfx     x2, x1, #1, #0x1f
0x1ef9b0  a2031ff8           stur     x2, [x29, #-0x10]
0x1ef9b4  617c4193           sbfx     x1, x3, #1, #0x1f
0x1ef9b8  5f0001eb           cmp      x2, x1
0x1ef9bc  61000054           b.ne     #0x1ef9c8
0x1ef9c0  e10300aa           mov      x1, x0
0x1ef9c4  d8ebfc97           bl       #0x12a924
0x1ef9c8  a0835ff8           ldur     x0, [x29, #-8]
0x1ef9cc  a1035ff8           ldur     x1, [x29, #-0x10]
0x1ef9d0  22040091           add      x2, x1, #1
0x1ef9d4  43f87fd3           lsl      x3, x2, #1
0x1ef9d8  03b000b8           stur     w3, [x0, #0xb]
0x1ef9dc  02f040b8           ldur     w2, [x0, #0xf]
0x1ef9e0  42801c8b           add      x2, x2, x28, lsl #32
0x1ef9e4  4308018b           add      x3, x2, x1, lsl #2
0x1ef9e8  70234091           add      x16, x27, #8, lsl #12
0x1ef9ec  106646f9           ldr      x16, [x16, #0xcc8]  # pool[4503] = "done"
0x1ef9f0  70f000b8           stur     w16, [x3, #0xf]
0x1ef9f4  ef031daa           mov      x15, x29
0x1ef9f8  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1ef9fc  c0035fd6           ret      
0x1efa00  02cc0294           bl       #0x2a2a08
0x1efa04  dbffff17           b        #0x1ef970
# CFG: 0x1ef950->0x1ef970/ConditionalFalse 0x1ef950->0x1efa00/ConditionalTrue 0x1ef970->0x1ef9c0/ConditionalFalse 0x1ef970->0x1ef9c8/ConditionalTrue 0x1ef9c0->0x1ef9c8/Fallthrough 0x1efa00->0x1ef970/Branch

# top_level.e01InterpChain at 0x1efa14 (244 bytes)
0x1efa14  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1efa18  fd030faa           mov      x29, x15
0x1efa1c  ef4100d1           sub      x15, x15, #0x10
0x1efa20  502740f9           ldr      x16, [x26, #0x48]
0x1efa24  ff0110eb           cmp      x15, x16
0x1efa28  c9060054           b.ls     #0x1efb00
0x1efa2c  e10316aa           mov      x1, x22
0x1efa30  020380d2           mov      x2, #0x18
0x1efa34  b2cb0294           bl       #0x2a28fc
0x1efa38  a0831ff8           stur     x0, [x29, #-8]
0x1efa3c  70234091           add      x16, x27, #8, lsl #12
0x1efa40  106a46f9           ldr      x16, [x16, #0xcd0]  # pool[4504] = "user="
0x1efa44  10f000b8           stur     w16, [x0, #0xf]
0x1efa48  70234091           add      x16, x27, #8, lsl #12
0x1efa4c  102e44f9           ldr      x16, [x16, #0x858]  # pool[4361] = snapshotRef(458)
0x1efa50  103001b8           stur     w16, [x0, #0x13]
0x1efa54  70234091           add      x16, x27, #8, lsl #12
0x1efa58  106e46f9           ldr      x16, [x16, #0xcd8]  # pool[4505] = " id="
0x1efa5c  107001b8           stur     w16, [x0, #0x17]
0x1efa60  d00080d2           mov      x16, #6
0x1efa64  10b001b8           stur     w16, [x0, #0x1b]
0x1efa68  70234091           add      x16, x27, #8, lsl #12
0x1efa6c  107246f9           ldr      x16, [x16, #0xce0]  # pool[4506] = " pct="
0x1efa70  10f001b8           stur     w16, [x0, #0x1f]
0x1efa74  61234091           add      x1, x27, #8, lsl #12
0x1efa78  217446f9           ldr      x1, [x1, #0xce8]  # pool[4507] = snapshotRef(15148)
0x1efa7c  220080d2           mov      x2, #1
0x1efa80  22000094           bl       #0x1efb08
0x1efa84  a1835ff8           ldur     x1, [x29, #-8]
0x1efa88  398c0091           add      x25, x1, #0x23
0x1efa8c  200300b9           str      w0, [x25]
0x1efa90  e0000036           tbz      w0, #0, #0x1efaac
0x1efa94  30f05f38           ldurb    w16, [x1, #-1]
0x1efa98  11f05f38           ldurb    w17, [x0, #-1]
0x1efa9c  300a508a           and      x16, x17, x16, lsr #2
0x1efaa0  1f825cea           tst      x16, x28, lsr #32
0x1efaa4  40000054           b.eq     #0x1efaac
0x1efaa8  1fc40294           bl       #0x2a0b24
0x1efaac  a0835ff8           ldur     x0, [x29, #-8]
0x1efab0  70234091           add      x16, x27, #8, lsl #12
0x1efab4  107a46f9           ldr      x16, [x16, #0xcf0]  # pool[4508] = "% nested=inner-"
0x1efab8  107002b8           stur     w16, [x0, #0x27]
0x1efabc  70234091           add      x16, x27, #8, lsl #12
0x1efac0  102e44f9           ldr      x16, [x16, #0x858]  # pool[4361] = snapshotRef(458)
0x1efac4  10b002b8           stur     w16, [x0, #0x2b]
0x1efac8  70234091           add      x16, x27, #8, lsl #12
0x1efacc  107e46f9           ldr      x16, [x16, #0xcf8]  # pool[4509] = " bool="
0x1efad0  10f002b8           stur     w16, [x0, #0x2f]
0x1efad4  d0c20091           add      x16, x22, #0x30
0x1efad8  103003b8           stur     w16, [x0, #0x33]
0x1efadc  70234091           add      x16, x27, #8, lsl #12
0x1efae0  108246f9           ldr      x16, [x16, #0xd00]  # pool[4510] = " nullish="
0x1efae4  107003b8           stur     w16, [x0, #0x37]
0x1efae8  16b003b8           stur     w22, [x0, #0x3b]
0x1efaec  e00100f9           str      x0, [x15]
0x1efaf0  7403fd97           bl       #0x1308c0
0x1efaf4  ef031daa           mov      x15, x29
0x1efaf8  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1efafc  c0035fd6           ret      
0x1efb00  c2cb0294           bl       #0x2a2a08
0x1efb04  caffff17           b        #0x1efa2c
# CFG: 0x1efa14->0x1efa2c/ConditionalFalse 0x1efa14->0x1efb00/ConditionalTrue 0x1efa2c->0x1efa94/ConditionalFalse 0x1efa2c->0x1efaac/ConditionalTrue 0x1efa94->0x1efaa8/ConditionalFalse 0x1efa94->0x1efaac/ConditionalTrue 0x1efaa8->0x1efaac/Fallthrough 0x1efb00->0x1efa2c/Branch

# ProbeApp.<anonymous closure> at 0x1efc20 (12 bytes)
0x1efc20  60234091           add      x0, x27, #8, lsl #12
0x1efc24  007844f9           ldr      x0, [x0, #0x8f0]  # pool[4380] = snapshotRef(427)
0x1efc28  c0035fd6           ret      

# ProbeApp.<anonymous closure> at 0x1efc2c (144 bytes)
0x1efc2c  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1efc30  fd030faa           mov      x29, x15
0x1efc34  ef8100d1           sub      x15, x15, #0x20
0x1efc38  a01340f9           ldr      x0, [x29, #0x20]
0x1efc3c  017041b8           ldur     w1, [x0, #0x17]
0x1efc40  21801c8b           add      x1, x1, x28, lsl #32
0x1efc44  a1831ff8           stur     x1, [x29, #-8]
0x1efc48  502740f9           ldr      x16, [x26, #0x48]
0x1efc4c  ff0110eb           cmp      x15, x16
0x1efc50  29030054           b.ls     #0x1efcb4
0x1efc54  a00b40f9           ldr      x0, [x29, #0x10]
0x1efc58  02f040b8           ldur     w2, [x0, #0xf]
0x1efc5c  42801c8b           add      x2, x2, x28, lsl #32
0x1efc60  5f00166b           cmp      w2, w22
0x1efc64  61000054           b.ne     #0x1efc70
0x1efc68  000080d2           mov      x0, #0
0x1efc6c  02000014           b        #0x1efc74
0x1efc70  e00302aa           mov      x0, x2
0x1efc74  e00100f9           str      x0, [x15]
0x1efc78  e503fd97           bl       #0x130c0c
0x1efc7c  e10300aa           mov      x1, x0
0x1efc80  a0835ff8           ldur     x0, [x29, #-8]
0x1efc84  a1831ef8           stur     x1, [x29, #-0x18]
0x1efc88  02f040b8           ldur     w2, [x0, #0xf]
0x1efc8c  42801c8b           add      x2, x2, x28, lsl #32
0x1efc90  a2031ff8           stur     x2, [x29, #-0x10]
0x1efc94  5dffff97           bl       #0x1efa08
0x1efc98  a1835ef8           ldur     x1, [x29, #-0x18]
0x1efc9c  01b000b8           stur     w1, [x0, #0xb]
0x1efca0  a1035ff8           ldur     x1, [x29, #-0x10]
0x1efca4  013003b8           stur     w1, [x0, #0x33]
0x1efca8  ef031daa           mov      x15, x29
0x1efcac  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1efcb0  c0035fd6           ret      
0x1efcb4  55cb0294           bl       #0x2a2a08
0x1efcb8  e7ffff17           b        #0x1efc54
# CFG: 0x1efc2c->0x1efc54/ConditionalFalse 0x1efc2c->0x1efcb4/ConditionalTrue 0x1efc54->0x1efc68/ConditionalFalse 0x1efc54->0x1efc70/ConditionalTrue 0x1efc68->0x1efc74/Branch 0x1efc70->0x1efc74/Fallthrough 0x1efcb4->0x1efc54/Branch

# E13Dynamic.noSuchMethod at 0x1f4720 (220 bytes)
0x1f4720  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x1f4724  fd030faa           mov      x29, x15
0x1f4728  ef4100d1           sub      x15, x15, #0x10
0x1f472c  502740f9           ldr      x16, [x26, #0x48]
0x1f4730  ff0110eb           cmp      x15, x16
0x1f4734  09060054           b.ls     #0x1f47f4
0x1f4738  e10316aa           mov      x1, x22
0x1f473c  020180d2           mov      x2, #8
0x1f4740  6fb80294           bl       #0x2a28fc
0x1f4744  a0831ff8           stur     x0, [x29, #-8]
0x1f4748  70274091           add      x16, x27, #9, lsl #12
0x1f474c  101643f9           ldr      x16, [x16, #0x628]  # pool[4803] = "unhandled:"
0x1f4750  10f000b8           stur     w16, [x0, #0xf]
0x1f4754  a10b40f9           ldr      x1, [x29, #0x10]
0x1f4758  08010094           bl       #0x1f4b78
0x1f475c  a1835ff8           ldur     x1, [x29, #-8]
0x1f4760  394c0091           add      x25, x1, #0x13
0x1f4764  200300b9           str      w0, [x25]
0x1f4768  e0000036           tbz      w0, #0, #0x1f4784
0x1f476c  30f05f38           ldurb    w16, [x1, #-1]
0x1f4770  11f05f38           ldurb    w17, [x0, #-1]
0x1f4774  300a508a           and      x16, x17, x16, lsr #2
0x1f4778  1f825cea           tst      x16, x28, lsr #32
0x1f477c  40000054           b.eq     #0x1f4784
0x1f4780  e9b00294           bl       #0x2a0b24
0x1f4784  a0835ff8           ldur     x0, [x29, #-8]
0x1f4788  70e351f9           ldr      x16, [x27, #0x23c0]  # pool[1142] = snapshotRef(758)
0x1f478c  107001b8           stur     w16, [x0, #0x17]
0x1f4790  a10b40f9           ldr      x1, [x29, #0x10]
0x1f4794  1a000094           bl       #0x1f47fc
0x1f4798  01f05ff8           ldur     x1, [x0, #-1]
0x1f479c  217c4cd3           ubfx     x1, x1, #0xc, #0x14
0x1f47a0  e00100f9           str      x0, [x15]
0x1f47a4  e00301aa           mov      x0, x1
0x1f47a8  1e103cd1           sub      x30, x0, #0xf04
0x1f47ac  be7a7ef8           ldr      x30, [x21, x30, lsl #3]
0x1f47b0  c0033fd6           blr      x30
0x1f47b4  a1835ff8           ldur     x1, [x29, #-8]
0x1f47b8  396c0091           add      x25, x1, #0x1b
0x1f47bc  200300b9           str      w0, [x25]
0x1f47c0  e0000036           tbz      w0, #0, #0x1f47dc
0x1f47c4  30f05f38           ldurb    w16, [x1, #-1]
0x1f47c8  11f05f38           ldurb    w17, [x0, #-1]
0x1f47cc  300a508a           and      x16, x17, x16, lsr #2
0x1f47d0  1f825cea           tst      x16, x28, lsr #32
0x1f47d4  40000054           b.eq     #0x1f47dc
0x1f47d8  d3b00294           bl       #0x2a0b24
0x1f47dc  b0835ff8           ldur     x16, [x29, #-8]
0x1f47e0  f00100f9           str      x16, [x15]
0x1f47e4  37f0fc97           bl       #0x1308c0
0x1f47e8  ef031daa           mov      x15, x29
0x1f47ec  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x1f47f0  c0035fd6           ret      
0x1f47f4  85b80294           bl       #0x2a2a08
0x1f47f8  d0ffff17           b        #0x1f4738
# CFG: 0x1f4720->0x1f4738/ConditionalFalse 0x1f4720->0x1f47f4/ConditionalTrue 0x1f4738->0x1f476c/ConditionalFalse 0x1f4738->0x1f4784/ConditionalTrue 0x1f476c->0x1f4780/ConditionalFalse 0x1f476c->0x1f4784/ConditionalTrue 0x1f4780->0x1f4784/Fallthrough 0x1f4784->0x1f47c4/ConditionalFalse 0x1f4784->0x1f47dc/ConditionalTrue 0x1f47c4->0x1f47d8/ConditionalFalse 0x1f47c4->0x1f47dc/ConditionalTrue 0x1f47d8->0x1f47dc/Fallthrough 0x1f47f4->0x1f4738/Branch

# E21Mode._enumToString at 0x211f58 (100 bytes)
0x211f58  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x211f5c  fd030faa           mov      x29, x15
0x211f60  ef4100d1           sub      x15, x15, #0x10
0x211f64  e00301aa           mov      x0, x1
0x211f68  a1831ff8           stur     x1, [x29, #-8]
0x211f6c  502740f9           ldr      x16, [x26, #0x48]
0x211f70  ff0110eb           cmp      x15, x16
0x211f74  09020054           b.ls     #0x211fb4
0x211f78  e10316aa           mov      x1, x22
0x211f7c  820080d2           mov      x2, #4
0x211f80  5f420294           bl       #0x2a28fc
0x211f84  70274091           add      x16, x27, #9, lsl #12
0x211f88  101243f9           ldr      x16, [x16, #0x620]  # pool[4802] = "E21Mode."
0x211f8c  10f000b8           stur     w16, [x0, #0xf]
0x211f90  a1835ff8           ldur     x1, [x29, #-8]
0x211f94  22f040b8           ldur     w2, [x1, #0xf]
0x211f98  42801c8b           add      x2, x2, x28, lsl #32
0x211f9c  023001b8           stur     w2, [x0, #0x13]
0x211fa0  e00100f9           str      x0, [x15]
0x211fa4  477afc97           bl       #0x1308c0
0x211fa8  ef031daa           mov      x15, x29
0x211fac  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x211fb0  c0035fd6           ret      
0x211fb4  95420294           bl       #0x2a2a08
0x211fb8  f0ffff17           b        #0x211f78
# CFG: 0x211f58->0x211f78/ConditionalFalse 0x211f58->0x211fb4/ConditionalTrue 0x211fb4->0x211f78/Branch

# E15Vec.get:hashCode at 0x237968 (56 bytes)
0x237968  e20140f9           ldr      x2, [x15]
0x23796c  437040f8           ldur     x3, [x2, #7]
0x237970  44f040f8           ldur     x4, [x2, #0xf]
0x237974  620004ca           eor      x2, x3, x4
0x237978  40787f93           sbfiz    x0, x2, #1, #0x1f
0x23797c  5f0480eb           cmp      x2, x0, asr #1
0x237980  e0000054           b.eq     #0x23799c
0x237984  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x237988  fd030faa           mov      x29, x15
0x23798c  7fac0194           bl       #0x2a2b88
0x237990  ef031daa           mov      x15, x29
0x237994  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x237998  027000f8           stur     x2, [x0, #7]
0x23799c  c0035fd6           ret      
# CFG: 0x237968->0x237984/ConditionalFalse 0x237968->0x23799c/ConditionalTrue 0x237984->0x23799c/Fallthrough

# E15Vec.== at 0x24c394 (88 bytes)
0x24c394  e10140f9           ldr      x1, [x15]
0x24c398  3f00166b           cmp      w1, w22
0x24c39c  61000054           b.ne     #0x24c3a8
0x24c3a0  c0c20091           add      x0, x22, #0x30
0x24c3a4  c0035fd6           ret      
0x24c3a8  820780d2           mov      x2, #0x3c
0x24c3ac  61000036           tbz      w1, #0, #0x24c3b8
0x24c3b0  22f05ff8           ldur     x2, [x1, #-1]
0x24c3b4  427c4cd3           ubfx     x2, x2, #0xc, #0x14
0x24c3b8  5fd00bf1           cmp      x2, #0x2f4
0x24c3bc  41010054           b.ne     #0x24c3e4
0x24c3c0  e20540f9           ldr      x2, [x15, #8]
0x24c3c4  437040f8           ldur     x3, [x2, #7]
0x24c3c8  227040f8           ldur     x2, [x1, #7]
0x24c3cc  7f0002eb           cmp      x3, x2
0x24c3d0  d0820091           add      x16, x22, #0x20
0x24c3d4  d1c20091           add      x17, x22, #0x30
0x24c3d8  0102919a           csel     x1, x16, x17, eq
0x24c3dc  e00301aa           mov      x0, x1
0x24c3e0  02000014           b        #0x24c3e8
0x24c3e4  c0c20091           add      x0, x22, #0x30
0x24c3e8  c0035fd6           ret      
# CFG: 0x24c394->0x24c3a0/ConditionalFalse 0x24c394->0x24c3a8/ConditionalTrue 0x24c3a8->0x24c3b0/ConditionalFalse 0x24c3a8->0x24c3b8/ConditionalTrue 0x24c3b0->0x24c3b8/Fallthrough 0x24c3b8->0x24c3c0/ConditionalFalse 0x24c3b8->0x24c3e4/ConditionalTrue 0x24c3c0->0x24c3e8/Branch 0x24c3e4->0x24c3e8/Fallthrough

# top_level.main at 0x2a4c70 (48 bytes)
0x2a4c70  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x2a4c74  fd030faa           mov      x29, x15
0x2a4c78  502740f9           ldr      x16, [x26, #0x48]
0x2a4c7c  ff0110eb           cmp      x15, x16
0x2a4c80  c9000054           b.ls     #0x2a4c98
0x2a4c84  07000094           bl       #0x2a4ca0
0x2a4c88  e00316aa           mov      x0, x22
0x2a4c8c  ef031daa           mov      x15, x29
0x2a4c90  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x2a4c94  c0035fd6           ret      
0x2a4c98  5cf7ff97           bl       #0x2a2a08
0x2a4c9c  faffff17           b        #0x2a4c84
# CFG: 0x2a4c70->0x2a4c84/ConditionalFalse 0x2a4c70->0x2a4c98/ConditionalTrue 0x2a4c98->0x2a4c84/Branch
