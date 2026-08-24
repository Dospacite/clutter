# Complete decoded machine-code evidence. Generated source intentionally omits this noise.

# E15Vec.compareTo at 0x18fd48 (304 bytes)
0x18fd48  00482de9           push     {fp, lr}
0x18fd4c  00b08de2           add      fp, sp, #0
0x18fd50  0cd04de2           sub      sp, sp, #0xc
0x18fd54  0140a0e1           mov      r4, r1
0x18fd58  0230a0e1           mov      r3, r2
0x18fd5c  04100be5           str      r1, [fp, #-4]
0x18fd60  08200be5           str      r2, [fp, #-8]
0x18fd64  24c09ae5           ldr      ip, [sl, #0x24]
0x18fd68  0c005de1           cmp      sp, ip
0x18fd6c  1224069b           blls     #0x318dbc
0x18fd70  0300a0e1           mov      r0, r3
0x18fd74  40209ae5           ldr      r2, [sl, #0x40]
0x18fd78  40109ae5           ldr      r1, [sl, #0x40]
0x18fd7c  010010e3           tst      r0, #1
0x18fd80  01401015           ldrne    r4, [r0, #-1]
0x18fd84  5446f317           ubfxne   r4, r4, #0xc, #0x14
0x18fd88  3c40a003           moveq    r4, #0x3c
0x18fd8c  bd0f54e3           cmp      r4, #0x2f4
0x18fd90  0400000a           beq      #0x18fda8
0x18fd94  018985e2           add      r8, r5, #0x4000
0x18fd98  3b8f98e5           ldr      r8, [r8, #0xf3b]  # pool[5069] = snapshotRef(15755)
0x18fd9c  013985e2           add      r3, r5, #0x4000
0x18fda0  3f3f93e5           ldr      r3, [r3, #0xf3f]  # pool[5070] = null
0x18fda4  dd1b06eb           bl       #0x316d20
0x18fda8  04001be5           ldr      r0, [fp, #-4]
0x18fdac  032090e5           ldr      r2, [r0, #3]
0x18fdb0  071090e5           ldr      r1, [r0, #7]
0x18fdb4  920106e0           mul      r6, r2, r1
0x18fdb8  916223e0           mla      r3, r1, r2, r6
0x18fdbc  924286e0           umull    r4, r6, r2, r2
0x18fdc0  063083e0           add      r3, r3, r6
0x18fdc4  0b2090e5           ldr      r2, [r0, #0xb]
0x18fdc8  0f1090e5           ldr      r1, [r0, #0xf]
0x18fdcc  920108e0           mul      r8, r2, r1
0x18fdd0  918220e0           mla      r0, r1, r2, r8
0x18fdd4  926288e0           umull    r6, r8, r2, r2
0x18fdd8  080080e0           add      r0, r0, r8
0x18fddc  068094e0           adds     r8, r4, r6
0x18fde0  0020b3e0           adcs     r2, r3, r0
0x18fde4  08001be5           ldr      r0, [fp, #-8]
0x18fde8  0c200be5           str      r2, [fp, #-0xc]
0x18fdec  033090e5           ldr      r3, [r0, #3]
0x18fdf0  071090e5           ldr      r1, [r0, #7]
0x18fdf4  930109e0           mul      sb, r3, r1
0x18fdf8  919324e0           mla      r4, r1, r3, sb
0x18fdfc  936389e0           umull    r6, sb, r3, r3
0x18fe00  094084e0           add      r4, r4, sb
0x18fe04  0b3090e5           ldr      r3, [r0, #0xb]
0x18fe08  0f1090e5           ldr      r1, [r0, #0xf]
0x18fe0c  930102e0           mul      r2, r3, r1
0x18fe10  912320e0           mla      r0, r1, r3, r2
0x18fe14  939382e0           umull    sb, r2, r3, r3
0x18fe18  020080e0           add      r0, r0, r2
0x18fe1c  093096e0           adds     r3, r6, sb
0x18fe20  0020b4e0           adcs     r2, r4, r0
0x18fe24  0c401be5           ldr      r4, [fp, #-0xc]
0x18fe28  8800a0e1           lsl      r0, r8, #1
0x18fe2c  c00058e1           cmp      r8, r0, asr #1
0x18fe30  c00f5401           cmpeq    r4, r0, asr #31
0x18fe34  0200000a           beq      #0x18fe44
0x18fe38  0b2406eb           bl       #0x318e6c
0x18fe3c  078080e5           str      r8, [r0, #7]
0x18fe40  0b4080e5           str      r4, [r0, #0xb]
0x18fe44  0040a0e1           mov      r4, r0
0x18fe48  8300a0e1           lsl      r0, r3, #1
0x18fe4c  c00053e1           cmp      r3, r0, asr #1
0x18fe50  c00f5201           cmpeq    r2, r0, asr #31
0x18fe54  0200000a           beq      #0x18fe64
0x18fe58  032406eb           bl       #0x318e6c
0x18fe5c  073080e5           str      r3, [r0, #7]
0x18fe60  0b2080e5           str      r2, [r0, #0xb]
0x18fe64  0410a0e1           mov      r1, r4
0x18fe68  0020a0e1           mov      r2, r0
0x18fe6c  2dca00eb           bl       #0x1c2728
0x18fe70  00d04be2           sub      sp, fp, #0
0x18fe74  0088bde8           pop      {fp, pc}
# CFG: 0x18fd48->0x18fd94/ConditionalFalse 0x18fd48->0x18fda8/ConditionalTrue 0x18fd94->0x18fda8/Fallthrough 0x18fda8->0x18fe38/ConditionalFalse 0x18fda8->0x18fe44/ConditionalTrue 0x18fe38->0x18fe44/Fallthrough 0x18fe44->0x18fe58/ConditionalFalse 0x18fe44->0x18fe64/ConditionalTrue 0x18fe58->0x18fe64/Fallthrough

# top_level.e19Ackermann at 0x18fe78 (380 bytes)
0x18fe78  00482de9           push     {fp, lr}
0x18fe7c  00b08de2           add      fp, sp, #0
0x18fe80  10d04de2           sub      sp, sp, #0x10
0x18fe84  0f0094e5           ldr      r0, [r4, #0xf]
0x18fe88  021040e2           sub      r1, r0, #2
0x18fe8c  81208be0           add      r2, fp, r1, lsl #1
0x18fe90  082092e5           ldr      r2, [r2, #8]
0x18fe94  020051e3           cmp      r1, #2
0x18fe98  020000ba           blt      #0x18fea8
0x18fe9c  81008be0           add      r0, fp, r1, lsl #1
0x18fea0  040090e5           ldr      r0, [r0, #4]
0x18fea4  000000ea           b        #0x18feac
0x18fea8  0400a0e3           mov      r0, #4
0x18feac  24c09ae5           ldr      ip, [sl, #0x24]
0x18feb0  0c005de1           cmp      sp, ip
0x18feb4  c023069b           blls     #0x318dbc
0x18feb8  c21fa0e1           asr      r1, r2, #0x1f
0x18febc  c230b0e1           asrs     r3, r2, #1
0x18fec0  0100003a           blo      #0x18fecc
0x18fec4  073092e5           ldr      r3, [r2, #7]
0x18fec8  0b1092e5           ldr      r1, [r2, #0xb]
0x18fecc  000053e3           cmp      r3, #0
0x18fed0  00005103           cmpeq    r1, #0
0x18fed4  0f00001a           bne      #0x18ff18
0x18fed8  c01fa0e1           asr      r1, r0, #0x1f
0x18fedc  c020b0e1           asrs     r2, r0, #1
0x18fee0  0100003a           blo      #0x18feec
0x18fee4  072090e5           ldr      r2, [r0, #7]
0x18fee8  0b1090e5           ldr      r1, [r0, #0xb]
0x18feec  014092e2           adds     r4, r2, #1
0x18fef0  0030b1e2           adcs     r3, r1, #0
0x18fef4  8400a0e1           lsl      r0, r4, #1
0x18fef8  c00054e1           cmp      r4, r0, asr #1
0x18fefc  c00f5301           cmpeq    r3, r0, asr #31
0x18ff00  0200000a           beq      #0x18ff10
0x18ff04  d82306eb           bl       #0x318e6c
0x18ff08  074080e5           str      r4, [r0, #7]
0x18ff0c  0b3080e5           str      r3, [r0, #0xb]
0x18ff10  00d04be2           sub      sp, fp, #0
0x18ff14  0088bde8           pop      {fp, pc}
0x18ff18  c04fa0e1           asr      r4, r0, #0x1f
0x18ff1c  c060b0e1           asrs     r6, r0, #1
0x18ff20  0100003a           blo      #0x18ff2c
0x18ff24  076090e5           ldr      r6, [r0, #7]
0x18ff28  0b4090e5           ldr      r4, [r0, #0xb]
0x18ff2c  000056e3           cmp      r6, #0
0x18ff30  00005403           cmpeq    r4, #0
0x18ff34  0d00001a           bne      #0x18ff70
0x18ff38  014053e2           subs     r4, r3, #1
0x18ff3c  0020d1e2           sbcs     r2, r1, #0
0x18ff40  8400a0e1           lsl      r0, r4, #1
0x18ff44  c00054e1           cmp      r4, r0, asr #1
0x18ff48  c00f5201           cmpeq    r2, r0, asr #31
0x18ff4c  0200000a           beq      #0x18ff5c
0x18ff50  c52306eb           bl       #0x318e6c
0x18ff54  074080e5           str      r4, [r0, #7]
0x18ff58  0b2080e5           str      r2, [r0, #0xb]
0x18ff5c  00008de5           str      r0, [sp]
0x18ff60  434195e5           ldr      r4, [r5, #0x143]  # pool[79] = snapshotRef(22)
0x18ff64  c3ffffeb           bl       #0x18fe78
0x18ff68  00d04be2           sub      sp, fp, #0
0x18ff6c  0088bde8           pop      {fp, pc}
0x18ff70  019053e2           subs     sb, r3, #1
0x18ff74  0080d1e2           sbcs     r8, r1, #0
0x18ff78  04900be5           str      sb, [fp, #-4]
0x18ff7c  08800be5           str      r8, [fp, #-8]
0x18ff80  010056e2           subs     r0, r6, #1
0x18ff84  0030d4e2           sbcs     r3, r4, #0
0x18ff88  0040a0e1           mov      r4, r0
0x18ff8c  8400a0e1           lsl      r0, r4, #1
0x18ff90  c00054e1           cmp      r4, r0, asr #1
0x18ff94  c00f5301           cmpeq    r3, r0, asr #31
0x18ff98  0200000a           beq      #0x18ffa8
0x18ff9c  b22306eb           bl       #0x318e6c
0x18ffa0  074080e5           str      r4, [r0, #7]
0x18ffa4  0b3080e5           str      r3, [r0, #0xb]
0x18ffa8  05008de8           stm      sp, {r0, r2}
0x18ffac  374195e5           ldr      r4, [r5, #0x137]  # pool[76] = snapshotRef(23)
0x18ffb0  b0ffffeb           bl       #0x18fe78
0x18ffb4  0040a0e1           mov      r4, r0
0x18ffb8  04301be5           ldr      r3, [fp, #-4]
0x18ffbc  08201be5           ldr      r2, [fp, #-8]
0x18ffc0  8300a0e1           lsl      r0, r3, #1
0x18ffc4  c00053e1           cmp      r3, r0, asr #1
0x18ffc8  c00f5201           cmpeq    r2, r0, asr #31
0x18ffcc  0200000a           beq      #0x18ffdc
0x18ffd0  a52306eb           bl       #0x318e6c
0x18ffd4  073080e5           str      r3, [r0, #7]
0x18ffd8  0b2080e5           str      r2, [r0, #0xb]
0x18ffdc  04008de5           str      r0, [sp, #4]
0x18ffe0  00408de5           str      r4, [sp]
0x18ffe4  374195e5           ldr      r4, [r5, #0x137]  # pool[76] = snapshotRef(23)
0x18ffe8  a2ffffeb           bl       #0x18fe78
0x18ffec  00d04be2           sub      sp, fp, #0
0x18fff0  0088bde8           pop      {fp, pc}
# CFG: 0x18fe78->0x18fe9c/ConditionalFalse 0x18fe78->0x18fea8/ConditionalTrue 0x18fe9c->0x18feac/Branch 0x18fea8->0x18feac/Fallthrough 0x18feac->0x18fec4/ConditionalFalse 0x18feac->0x18fecc/ConditionalTrue 0x18fec4->0x18fecc/Fallthrough 0x18fecc->0x18fed8/ConditionalFalse 0x18fecc->0x18ff18/ConditionalTrue 0x18fed8->0x18fee4/ConditionalFalse 0x18fed8->0x18feec/ConditionalTrue 0x18fee4->0x18feec/Fallthrough 0x18feec->0x18ff04/ConditionalFalse 0x18feec->0x18ff10/ConditionalTrue 0x18ff04->0x18ff10/Fallthrough 0x18ff18->0x18ff24/ConditionalFalse 0x18ff18->0x18ff2c/ConditionalTrue 0x18ff24->0x18ff2c/Fallthrough 0x18ff2c->0x18ff38/ConditionalFalse 0x18ff2c->0x18ff70/ConditionalTrue 0x18ff38->0x18ff50/ConditionalFalse 0x18ff38->0x18ff5c/ConditionalTrue 0x18ff50->0x18ff5c/Fallthrough 0x18ff70->0x18ff9c/ConditionalFalse 0x18ff70->0x18ffa8/ConditionalTrue 0x18ff9c->0x18ffa8/Fallthrough 0x18ffa8->0x18ffd0/ConditionalFalse 0x18ffa8->0x18ffdc/ConditionalTrue 0x18ffd0->0x18ffdc/Fallthrough

# top_level.e19Ackermann at 0x18fff4 (92 bytes)
0x18fff4  00482de9           push     {fp, lr}
0x18fff8  00b08de2           add      fp, sp, #0
0x18fffc  08d04de2           sub      sp, sp, #8
0x190000  0f0094e5           ldr      r0, [r4, #0xf]
0x190004  041040e2           sub      r1, r0, #4
0x190008  81008be0           add      r0, fp, r1, lsl #1
0x19000c  080090e5           ldr      r0, [r0, #8]
0x190010  020051e3           cmp      r1, #2
0x190014  030000ba           blt      #0x190028
0x190018  81208be0           add      r2, fp, r1, lsl #1
0x19001c  042092e5           ldr      r2, [r2, #4]
0x190020  0210a0e1           mov      r1, r2
0x190024  000000ea           b        #0x19002c
0x190028  0410a0e3           mov      r1, #4
0x19002c  24c09ae5           ldr      ip, [sl, #0x24]
0x190030  0c005de1           cmp      sp, ip
0x190034  6023069b           blls     #0x318dbc
0x190038  04008de5           str      r0, [sp, #4]
0x19003c  00108de5           str      r1, [sp]
0x190040  374195e5           ldr      r4, [r5, #0x137]  # pool[76] = snapshotRef(23)
0x190044  8bffffeb           bl       #0x18fe78
0x190048  00d04be2           sub      sp, fp, #0
0x19004c  0088bde8           pop      {fp, pc}
# CFG: 0x18fff4->0x190018/ConditionalFalse 0x18fff4->0x190028/ConditionalTrue 0x190018->0x19002c/Branch 0x190028->0x19002c/Fallthrough

# ProbeApp.build at 0x247ac8 (224 bytes)
0x247ac8  00482de9           push     {fp, lr}
0x247acc  00b08de2           add      fp, sp, #0
0x247ad0  08d04de2           sub      sp, sp, #8
0x247ad4  0130a0e1           mov      r3, r1
0x247ad8  0200a0e1           mov      r0, r2
0x247adc  011985e2           add      r1, r5, #0x4000
0x247ae0  471d91e5           ldr      r1, [r1, #0xd47]  # pool[4944] = ProbeApp.<anonymous closure>
0x247ae4  40209ae5           ldr      r2, [sl, #0x40]
0x247ae8  c34003eb           bl       #0x317dfc
0x247aec  04000be5           str      r0, [fp, #-4]
0x247af0  50effeeb           bl       #0x203838
0x247af4  0010a0e1           mov      r1, r0
0x247af8  04001be5           ldr      r0, [fp, #-4]
0x247afc  08100be5           str      r1, [fp, #-8]
0x247b00  070081e5           str      r0, [r1, #7]
0x247b04  2b0000eb           bl       #0x247bb8
0x247b08  0010a0e1           mov      r1, r0
0x247b0c  08001be5           ldr      r0, [fp, #-8]
0x247b10  04100be5           str      r1, [fp, #-4]
0x247b14  130081e5           str      r0, [r1, #0x13]
0x247b18  48009ae5           ldr      r0, [sl, #0x48]
0x247b1c  3f0081e5           str      r0, [r1, #0x3f]
0x247b20  4c209ae5           ldr      r2, [sl, #0x4c]
0x247b24  072081e5           str      r2, [r1, #7]
0x247b28  0b2081e5           str      r2, [r1, #0xb]
0x247b2c  1d0000eb           bl       #0x247ba8
0x247b30  04101be5           ldr      r1, [fp, #-4]
0x247b34  0f1080e5           str      r1, [r0, #0xf]
0x247b38  011985e2           add      r1, r5, #0x4000
0x247b3c  4b1d91e5           ldr      r1, [r1, #0xd4b]  # pool[4945] = snapshotRef(34549)
0x247b40  131080e5           str      r1, [r0, #0x13]
0x247b44  011985e2           add      r1, r5, #0x4000
0x247b48  4f1d91e5           ldr      r1, [r1, #0xd4f]  # pool[4946] = snapshotRef(34704)
0x247b4c  2b1080e5           str      r1, [r0, #0x2b]
0x247b50  011985e2           add      r1, r5, #0x4000
0x247b54  531d91e5           ldr      r1, [r1, #0xd53]  # pool[4947] = "clutter edge-case probe"
0x247b58  3b1080e5           str      r1, [r0, #0x3b]
0x247b5c  011985e2           add      r1, r5, #0x4000
0x247b60  571d91e5           ldr      r1, [r1, #0xd57]  # pool[4948] = snapshotInstance(ThemeMode)
0x247b64  531080e5           str      r1, [r0, #0x53]
0x247b68  011a85e2           add      r1, r5, #0x1000
0x247b6c  571691e5           ldr      r1, [r1, #0x657]  # pool[1428] = snapshotInstance(Duration)
0x247b70  571080e5           str      r1, [r0, #0x57]
0x247b74  011a85e2           add      r1, r5, #0x1000
0x247b78  8f1491e5           ldr      r1, [r1, #0x48f]  # pool[1314] = snapshotInstance(_Linear)
0x247b7c  5b1080e5           str      r1, [r0, #0x5b]
0x247b80  011985e2           add      r1, r5, #0x4000
0x247b84  5b1d91e5           ldr      r1, [r1, #0xd5b]  # pool[4949] = snapshotRef(34641) nestedStrings["US", "en"]
0x247b88  731080e5           str      r1, [r0, #0x73]
0x247b8c  4c109ae5           ldr      r1, [sl, #0x4c]
0x247b90  771080e5           str      r1, [r0, #0x77]
0x247b94  7b1080e5           str      r1, [r0, #0x7b]
0x247b98  48109ae5           ldr      r1, [sl, #0x48]
0x247b9c  7f1080e5           str      r1, [r0, #0x7f]
0x247ba0  00d04be2           sub      sp, fp, #0
0x247ba4  0088bde8           pop      {fp, pc}

# ProbeApp.<anonymous closure> at 0x247bc4 (4936 bytes)
0x247bc4  00482de9           push     {fp, lr}
0x247bc8  00b08de2           add      fp, sp, #0
0x247bcc  48d04de2           sub      sp, sp, #0x48
0x247bd0  0c009be5           ldr      r0, [fp, #0xc]
0x247bd4  131090e5           ldr      r1, [r0, #0x13]
0x247bd8  04100be5           str      r1, [fp, #-4]
0x247bdc  24c09ae5           ldr      ip, [sl, #0x24]
0x247be0  0c005de1           cmp      sp, ip
0x247be4  7444039b           blls     #0x318dbc
0x247be8  0210a0e3           mov      r1, #2
0x247bec  933f03eb           bl       #0x317a40
0x247bf0  0010a0e1           mov      r1, r0
0x247bf4  04001be5           ldr      r0, [fp, #-4]
0x247bf8  08100be5           str      r1, [fp, #-8]
0x247bfc  070081e5           str      r0, [r1, #7]
0x247c00  121b00eb           bl       #0x24e850
0x247c04  0030a0e1           mov      r3, r0
0x247c08  0120a0e1           mov      r2, r1
0x247c0c  0c300be5           str      r3, [fp, #-0xc]
0x247c10  10200be5           str      r2, [fp, #-0x10]
0x247c14  8300a0e1           lsl      r0, r3, #1
0x247c18  c00053e1           cmp      r3, r0, asr #1
0x247c1c  c00f5201           cmpeq    r2, r0, asr #31
0x247c20  0200000a           beq      #0x247c30
0x247c24  904403eb           bl       #0x318e6c
0x247c28  073080e5           str      r3, [r0, #7]
0x247c2c  0b2080e5           str      r2, [r0, #0xb]
0x247c30  0040a0e1           mov      r4, r0
0x247c34  08101be5           ldr      r1, [fp, #-8]
0x247c38  04400be5           str      r4, [fp, #-4]
0x247c3c  0b0081e5           str      r0, [r1, #0xb]
0x247c40  010010e3           tst      r0, #1
0x247c44  0500000a           beq      #0x247c60
0x247c48  01c051e5           ldrb     ip, [r1, #-1]
0x247c4c  01e050e5           ldrb     lr, [r0, #-1]
0x247c50  2cc10ee0           and      ip, lr, ip, lsr #2
0x247c54  28e09ae5           ldr      lr, [sl, #0x28]
0x247c58  0e001ce1           tst      ip, lr
0x247c5c  af3d031b           blne     #0x317320
0x247c60  c9f5fceb           bl       #0x18538c
0x247c64  0020a0e1           mov      r2, r0
0x247c68  04002de5           str      r0, [sp, #-4]!
0x247c6c  000bb7ee           vmovd    d0, #1.0
0x247c70  04009de4           pop      {r0}
0x247c74  18200be5           str      r2, [fp, #-0x18]
0x247c78  03c082e2           add      ip, r2, #3
0x247c7c  000b8ced           vstr     d0, [ip]
0x247c80  0200a0e1           mov      r0, r2
0x247c84  08301be5           ldr      r3, [fp, #-8]
0x247c88  0f0083e5           str      r0, [r3, #0xf]
0x247c8c  01c053e5           ldrb     ip, [r3, #-1]
0x247c90  01e050e5           ldrb     lr, [r0, #-1]
0x247c94  2cc10ee0           and      ip, lr, ip, lsr #2
0x247c98  28e09ae5           ldr      lr, [sl, #0x28]
0x247c9c  0e001ce1           tst      ip, lr
0x247ca0  aa3d031b           blne     #0x317350
0x247ca4  0c101be5           ldr      r1, [fp, #-0xc]
0x247ca8  10001be5           ldr      r0, [fp, #-0x10]
0x247cac  000050e3           cmp      r0, #0
0x247cb0  020000ba           blt      #0x247cc0
0x247cb4  050000ca           bgt      #0x247cd0
0x247cb8  000051e3           cmp      r1, #0
0x247cbc  0300002a           bhs      #0x247cd0
0x247cc0  006071e2           rsbs     r6, r1, #0
0x247cc4  0440c4e0           sbc      r4, r4, r4
0x247cc8  004044e0           sub      r4, r4, r0
0x247ccc  010000ea           b        #0x247cd8
0x247cd0  0160a0e1           mov      r6, r1
0x247cd4  0040a0e1           mov      r4, r0
0x247cd8  0c600be5           str      r6, [fp, #-0xc]
0x247cdc  10400be5           str      r4, [fp, #-0x10]
0x247ce0  8600a0e1           lsl      r0, r6, #1
0x247ce4  c00056e1           cmp      r6, r0, asr #1
0x247ce8  c00f5401           cmpeq    r4, r0, asr #31
0x247cec  0200000a           beq      #0x247cfc
0x247cf0  5d4403eb           bl       #0x318e6c
0x247cf4  076080e5           str      r6, [r0, #7]
0x247cf8  0b4080e5           str      r4, [r0, #0xb]
0x247cfc  14000be5           str      r0, [fp, #-0x14]
0x247d00  04008de5           str      r0, [sp, #4]
0x247d04  06e0a0e3           mov      lr, #6
0x247d08  00e08de5           str      lr, [sp]
0x247d0c  a92efeeb           bl       #0x1d37b8
0x247d10  c01fa0e1           asr      r1, r0, #0x1f
0x247d14  c020b0e1           asrs     r2, r0, #1
0x247d18  0100003a           blo      #0x247d24
0x247d1c  072090e5           ldr      r2, [r0, #7]
0x247d20  0b1090e5           ldr      r1, [r0, #0xb]
0x247d24  610092e2           adds     r0, r2, #0x61
0x247d28  0030b1e2           adcs     r3, r1, #0
0x247d2c  0020a0e1           mov      r2, r0
0x247d30  40109ae5           ldr      r1, [sl, #0x40]
0x247d34  d671fceb           bl       #0x164494
0x247d38  40109ae5           ldr      r1, [sl, #0x40]
0x247d3c  0420a0e3           mov      r2, #4
0x247d40  1c000be5           str      r0, [fp, #-0x1c]
0x247d44  d84303eb           bl       #0x318cac
0x247d48  0b2080e2           add      r2, r0, #0xb
0x247d4c  01c985e2           add      ip, r5, #0x4000
0x247d50  5fcd9ce5           ldr      ip, [ip, #0xd5f]  # pool[4950] = snapshotRef(130)
0x247d54  00c082e5           str      ip, [r2]
0x247d58  0c101be5           ldr      r1, [fp, #-0xc]
0x247d5c  10201be5           ldr      r2, [fp, #-0x10]
0x247d60  0130a0e1           mov      r3, r1
0x247d64  014003e2           and      r4, r3, #1
0x247d68  20400be5           str      r4, [fp, #-0x20]
0x247d6c  000054e3           cmp      r4, #0
0x247d70  0200001a           bne      #0x247d80
0x247d74  018985e2           add      r8, r5, #0x4000
0x247d78  638d98e5           ldr      r8, [r8, #0xd63]  # pool[4951] = "v v"
0x247d7c  000000ea           b        #0x247d84
0x247d80  40809ae5           ldr      r8, [sl, #0x40]
0x247d84  18301be5           ldr      r3, [fp, #-0x18]
0x247d88  14601be5           ldr      r6, [fp, #-0x14]
0x247d8c  0f1080e2           add      r1, r0, #0xf
0x247d90  008081e5           str      r8, [r1]
0x247d94  01e985e2           add      lr, r5, #0x4000
0x247d98  67ed9ee5           ldr      lr, [lr, #0xd67]  # pool[4952] = snapshotRef(17957)
0x247d9c  01408de8           stm      sp, {r0, lr}
0x247da0  d513fceb           bl       #0x14ccfc
0x247da4  40109ae5           ldr      r1, [sl, #0x40]
0x247da8  0420a0e3           mov      r2, #4
0x247dac  24000be5           str      r0, [fp, #-0x24]
0x247db0  bd4303eb           bl       #0x318cac
0x247db4  0b2080e2           add      r2, r0, #0xb
0x247db8  01c985e2           add      ip, r5, #0x4000
0x247dbc  6bcd9ce5           ldr      ip, [ip, #0xd6b]  # pool[4953] = snapshotRef(744)
0x247dc0  00c082e5           str      ip, [r2]
0x247dc4  14101be5           ldr      r1, [fp, #-0x14]
0x247dc8  0f3080e2           add      r3, r0, #0xf
0x247dcc  001083e5           str      r1, [r3]
0x247dd0  00008de5           str      r0, [sp]
0x247dd4  1b32fceb           bl       #0x154648
0x247dd8  28000be5           str      r0, [fp, #-0x28]
0x247ddc  14e01be5           ldr      lr, [fp, #-0x14]
0x247de0  c290a0e3           mov      sb, #0xc2
0x247de4  00428de8           stm      sp, {sb, lr}
0x247de8  722efeeb           bl       #0x1d37b8
0x247dec  2c000be5           str      r0, [fp, #-0x2c]
0x247df0  14e01be5           ldr      lr, [fp, #-0x14]
0x247df4  c890a0e3           mov      sb, #0xc8
0x247df8  00428de8           stm      sp, {sb, lr}
0x247dfc  6d2efeeb           bl       #0x1d37b8
0x247e00  40e09ae5           ldr      lr, [sl, #0x40]
0x247e04  01408de8           stm      sp, {r0, lr}
0x247e08  0c47fceb           bl       #0x159a40
0x247e0c  30000be5           str      r0, [fp, #-0x30]
0x247e10  40e09ae5           ldr      lr, [sl, #0x40]
0x247e14  c890a0e3           mov      sb, #0xc8
0x247e18  00428de8           stm      sp, {sb, lr}
0x247e1c  0747fceb           bl       #0x159a40
0x247e20  0010a0e1           mov      r1, r0
0x247e24  30001be5           ldr      r0, [fp, #-0x30]
0x247e28  03c080e2           add      ip, r0, #3
0x247e2c  010b9ced           vldr     d0, [ip, #4]
0x247e30  03c081e2           add      ip, r1, #3
0x247e34  012b9ced           vldr     d2, [ip, #4]
0x247e38  024b80ee           vdiv.f64 d4, d2
0x247e3c  2c001be5           ldr      r0, [fp, #-0x2c]
0x247e40  c03fa0e1           asr      r3, r0, #0x1f
0x247e44  c020b0e1           asrs     r2, r0, #1
0x247e48  0100003a           blo      #0x247e54
0x247e4c  072090e5           ldr      r2, [r0, #7]
0x247e50  0b3090e5           ldr      r3, [r0, #0xb]
0x247e54  28101be5           ldr      r1, [fp, #-0x28]
0x247e58  540124f2           vorr     q0, q2, q2
0x247e5c  a51900eb           bl       #0x24e4f8
0x247e60  28000be5           str      r0, [fp, #-0x28]
0x247e64  a01900eb           bl       #0x24e4ec
0x247e68  0030a0e1           mov      r3, r0
0x247e6c  28001be5           ldr      r0, [fp, #-0x28]
0x247e70  2c300be5           str      r3, [fp, #-0x2c]
0x247e74  070083e5           str      r0, [r3, #7]
0x247e78  18001be5           ldr      r0, [fp, #-0x18]
0x247e7c  2f0083e5           str      r0, [r3, #0x2f]
0x247e80  011985e2           add      r1, r5, #0x4000
0x247e84  6f1d91e5           ldr      r1, [r1, #0xd6f]  # pool[4954] = snapshotRef(18448)
0x247e88  3220a0e3           mov      r2, #0x32
0x247e8c  864303eb           bl       #0x318cac
0x247e90  0010a0e1           mov      r1, r0
0x247e94  2c001be5           ldr      r0, [fp, #-0x2c]
0x247e98  28100be5           str      r1, [fp, #-0x28]
0x247e9c  0b3081e2           add      r3, r1, #0xb
0x247ea0  000083e5           str      r0, [r3]
0x247ea4  14e01be5           ldr      lr, [fp, #-0x14]
0x247ea8  00e08de5           str      lr, [sp]
0x247eac  cc32fceb           bl       #0x1549e4
0x247eb0  40109ae5           ldr      r1, [sl, #0x40]
0x247eb4  0420a0e3           mov      r2, #4
0x247eb8  2c000be5           str      r0, [fp, #-0x2c]
0x247ebc  7a4303eb           bl       #0x318cac
0x247ec0  0010a0e1           mov      r1, r0
0x247ec4  2c001be5           ldr      r0, [fp, #-0x2c]
0x247ec8  30100be5           str      r1, [fp, #-0x30]
0x247ecc  0b3081e2           add      r3, r1, #0xb
0x247ed0  000083e5           str      r0, [r3]
0x247ed4  0f2081e2           add      r2, r1, #0xf
0x247ed8  01c985e2           add      ip, r5, #0x4000
0x247edc  73cd9ce5           ldr      ip, [ip, #0xd73]  # pool[4955] = snapshotRef(545)
0x247ee0  00c082e5           str      ip, [r2]
0x247ee4  b33395e5           ldr      r3, [r5, #0x3b3]  # pool[235] = snapshotRef(18337)
0x247ee8  c43e03eb           bl       #0x317a00
0x247eec  0010a0e1           mov      r1, r0
0x247ef0  30001be5           ldr      r0, [fp, #-0x30]
0x247ef4  0b0081e5           str      r0, [r1, #0xb]
0x247ef8  0420a0e3           mov      r2, #4
0x247efc  072081e5           str      r2, [r1, #7]
0x247f00  471900eb           bl       #0x24e424
0x247f04  00008de5           str      r0, [sp]
0x247f08  b532fceb           bl       #0x1549e4
0x247f0c  2c000be5           str      r0, [fp, #-0x2c]
0x247f10  751900eb           bl       #0x24e4ec
0x247f14  0010a0e1           mov      r1, r0
0x247f18  2c001be5           ldr      r0, [fp, #-0x2c]
0x247f1c  070081e5           str      r0, [r1, #7]
0x247f20  18201be5           ldr      r2, [fp, #-0x18]
0x247f24  2f2081e5           str      r2, [r1, #0x2f]
0x247f28  0100a0e1           mov      r0, r1
0x247f2c  28101be5           ldr      r1, [fp, #-0x28]
0x247f30  0f9081e2           add      sb, r1, #0xf
0x247f34  000089e5           str      r0, [sb]
0x247f38  010010e3           tst      r0, #1
0x247f3c  0500000a           beq      #0x247f58
0x247f40  01c051e5           ldrb     ip, [r1, #-1]
0x247f44  01e050e5           ldrb     lr, [r0, #-1]
0x247f48  2cc10ee0           and      ip, lr, ip, lsr #2
0x247f4c  28e09ae5           ldr      lr, [sl, #0x28]
0x247f50  0e001ce1           tst      ip, lr
0x247f54  553c031b           blne     #0x3170b0
0x247f58  01e985e2           add      lr, r5, #0x4000
0x247f5c  73ed9ee5           ldr      lr, [lr, #0xd73]  # pool[4955] = snapshotRef(545)
0x247f60  1c901be5           ldr      sb, [fp, #-0x1c]
0x247f64  00428de8           stm      sp, {sb, lr}
0x247f68  f5b502eb           bl       #0x2f5744
0x247f6c  0010a0e1           mov      r1, r0
0x247f70  48009ae5           ldr      r0, [sl, #0x48]
0x247f74  000051e1           cmp      r1, r0
0x247f78  0200001a           bne      #0x247f88
0x247f7c  013985e2           add      r3, r5, #0x4000
0x247f80  773d93e5           ldr      r3, [r3, #0xd77]  # pool[4956] = "alpha"
0x247f84  160000ea           b        #0x247fe4
0x247f88  01e985e2           add      lr, r5, #0x4000
0x247f8c  7bed9ee5           ldr      lr, [lr, #0xd7b]  # pool[4957] = snapshotRef(269)
0x247f90  1c901be5           ldr      sb, [fp, #-0x1c]
0x247f94  00428de8           stm      sp, {sb, lr}
0x247f98  e9b502eb           bl       #0x2f5744
0x247f9c  0010a0e1           mov      r1, r0
0x247fa0  48009ae5           ldr      r0, [sl, #0x48]
0x247fa4  000051e1           cmp      r1, r0
0x247fa8  0800000a           beq      #0x247fd0
0x247fac  03ea85e2           add      lr, r5, #0x3000
0x247fb0  6fe89ee5           ldr      lr, [lr, #0x86f]  # pool[3610] = snapshotRef(660)
0x247fb4  1c901be5           ldr      sb, [fp, #-0x1c]
0x247fb8  00428de8           stm      sp, {sb, lr}
0x247fbc  e0b502eb           bl       #0x2f5744
0x247fc0  0010a0e1           mov      r1, r0
0x247fc4  48009ae5           ldr      r0, [sl, #0x48]
0x247fc8  000051e1           cmp      r1, r0
0x247fcc  0200001a           bne      #0x247fdc
0x247fd0  013985e2           add      r3, r5, #0x4000
0x247fd4  7f3d93e5           ldr      r3, [r3, #0xd7f]  # pool[4958] = "beta-or-gamma"
0x247fd8  010000ea           b        #0x247fe4
0x247fdc  013985e2           add      r3, r5, #0x4000
0x247fe0  833d93e5           ldr      r3, [r3, #0xd83]  # pool[4959] = "other"
0x247fe4  18101be5           ldr      r1, [fp, #-0x18]
0x247fe8  04201be5           ldr      r2, [fp, #-4]
0x247fec  2c300be5           str      r3, [fp, #-0x2c]
0x247ff0  3d1900eb           bl       #0x24e4ec
0x247ff4  0010a0e1           mov      r1, r0
0x247ff8  2c001be5           ldr      r0, [fp, #-0x2c]
0x247ffc  070081e5           str      r0, [r1, #7]
0x248000  18301be5           ldr      r3, [fp, #-0x18]
0x248004  2f3081e5           str      r3, [r1, #0x2f]
0x248008  0100a0e1           mov      r0, r1
0x24800c  28101be5           ldr      r1, [fp, #-0x28]
0x248010  139081e2           add      sb, r1, #0x13
0x248014  000089e5           str      r0, [sb]
0x248018  010010e3           tst      r0, #1
0x24801c  0500000a           beq      #0x248038
0x248020  01c051e5           ldrb     ip, [r1, #-1]
0x248024  01e050e5           ldrb     lr, [r0, #-1]
0x248028  2cc10ee0           and      ip, lr, ip, lsr #2
0x24802c  28e09ae5           ldr      lr, [sl, #0x28]
0x248030  0e001ce1           tst      ip, lr
0x248034  1d3c031b           blne     #0x3170b0
0x248038  0c101be5           ldr      r1, [fp, #-0xc]
0x24803c  10201be5           ldr      r2, [fp, #-0x10]
0x248040  ab1800eb           bl       #0x24e2f4
0x248044  0030a0e1           mov      r3, r0
0x248048  0120a0e1           mov      r2, r1
0x24804c  8300a0e1           lsl      r0, r3, #1
0x248050  c00053e1           cmp      r3, r0, asr #1
0x248054  c00f5201           cmpeq    r2, r0, asr #31
0x248058  0200000a           beq      #0x248068
0x24805c  824303eb           bl       #0x318e6c
0x248060  073080e5           str      r3, [r0, #7]
0x248064  0b2080e5           str      r2, [r0, #0xb]
0x248068  00008de5           str      r0, [sp]
0x24806c  5c32fceb           bl       #0x1549e4
0x248070  2c000be5           str      r0, [fp, #-0x2c]
0x248074  1c1900eb           bl       #0x24e4ec
0x248078  0010a0e1           mov      r1, r0
0x24807c  2c001be5           ldr      r0, [fp, #-0x2c]
0x248080  070081e5           str      r0, [r1, #7]
0x248084  18201be5           ldr      r2, [fp, #-0x18]
0x248088  2f2081e5           str      r2, [r1, #0x2f]
0x24808c  0100a0e1           mov      r0, r1
0x248090  28101be5           ldr      r1, [fp, #-0x28]
0x248094  179081e2           add      sb, r1, #0x17
0x248098  000089e5           str      r0, [sb]
0x24809c  010010e3           tst      r0, #1
0x2480a0  0500000a           beq      #0x2480bc
0x2480a4  01c051e5           ldrb     ip, [r1, #-1]
0x2480a8  01e050e5           ldrb     lr, [r0, #-1]
0x2480ac  2cc10ee0           and      ip, lr, ip, lsr #2
0x2480b0  28e09ae5           ldr      lr, [sl, #0x28]
0x2480b4  0e001ce1           tst      ip, lr
0x2480b8  fc3b031b           blne     #0x3170b0
0x2480bc  24101be5           ldr      r1, [fp, #-0x24]
0x2480c0  2a1800eb           bl       #0x24e170
0x2480c4  00008de5           str      r0, [sp]
0x2480c8  4532fceb           bl       #0x1549e4
0x2480cc  24000be5           str      r0, [fp, #-0x24]
0x2480d0  051900eb           bl       #0x24e4ec
0x2480d4  0010a0e1           mov      r1, r0
0x2480d8  24001be5           ldr      r0, [fp, #-0x24]
0x2480dc  070081e5           str      r0, [r1, #7]
0x2480e0  18201be5           ldr      r2, [fp, #-0x18]
0x2480e4  2f2081e5           str      r2, [r1, #0x2f]
0x2480e8  0100a0e1           mov      r0, r1
0x2480ec  28101be5           ldr      r1, [fp, #-0x28]
0x2480f0  1b9081e2           add      sb, r1, #0x1b
0x2480f4  000089e5           str      r0, [sb]
0x2480f8  010010e3           tst      r0, #1
0x2480fc  0500000a           beq      #0x248118
0x248100  01c051e5           ldrb     ip, [r1, #-1]
0x248104  01e050e5           ldrb     lr, [r0, #-1]
0x248108  2cc10ee0           and      ip, lr, ip, lsr #2
0x24810c  28e09ae5           ldr      lr, [sl, #0x28]
0x248110  0e001ce1           tst      ip, lr
0x248114  e53b031b           blne     #0x3170b0
0x248118  14e01be5           ldr      lr, [fp, #-0x14]
0x24811c  1290a0e3           mov      sb, #0x12
0x248120  00428de8           stm      sp, {sb, lr}
0x248124  a32dfeeb           bl       #0x1d37b8
0x248128  24000be5           str      r0, [fp, #-0x24]
0x24812c  14e01be5           ldr      lr, [fp, #-0x14]
0x248130  0e90a0e3           mov      sb, #0xe
0x248134  00428de8           stm      sp, {sb, lr}
0x248138  9e2dfeeb           bl       #0x1d37b8
0x24813c  40109ae5           ldr      r1, [sl, #0x40]
0x248140  0420a0e3           mov      r2, #4
0x248144  2c000be5           str      r0, [fp, #-0x2c]
0x248148  d74203eb           bl       #0x318cac
0x24814c  0b2080e2           add      r2, r0, #0xb
0x248150  01c985e2           add      ip, r5, #0x4000
0x248154  87cd9ce5           ldr      ip, [ip, #0xd87]  # pool[4960] = snapshotRef(393)
0x248158  00c082e5           str      ip, [r2]
0x24815c  04101be5           ldr      r1, [fp, #-4]
0x248160  0f3080e2           add      r3, r0, #0xf
0x248164  001083e5           str      r1, [r3]
0x248168  00008de5           str      r0, [sp]
0x24816c  3531fceb           bl       #0x154648
0x248170  24201be5           ldr      r2, [fp, #-0x24]
0x248174  2c301be5           ldr      r3, [fp, #-0x2c]
0x248178  0040a0e1           mov      r4, r0
0x24817c  061000e3           movw     r1, #6
0x248180  021040e3           movt     r1, #2
0x248184  e03c03eb           bl       #0x31750c
0x248188  0010a0e1           mov      r1, r0
0x24818c  721700eb           bl       #0x24df5c
0x248190  0030a0e1           mov      r3, r0
0x248194  0120a0e1           mov      r2, r1
0x248198  8300a0e1           lsl      r0, r3, #1
0x24819c  c00053e1           cmp      r3, r0, asr #1
0x2481a0  c00f5201           cmpeq    r2, r0, asr #31
0x2481a4  0200000a           beq      #0x2481b4
0x2481a8  2f4303eb           bl       #0x318e6c
0x2481ac  073080e5           str      r3, [r0, #7]
0x2481b0  0b2080e5           str      r2, [r0, #0xb]
0x2481b4  00008de5           str      r0, [sp]
0x2481b8  0932fceb           bl       #0x1549e4
0x2481bc  24000be5           str      r0, [fp, #-0x24]
0x2481c0  c91800eb           bl       #0x24e4ec
0x2481c4  0010a0e1           mov      r1, r0
0x2481c8  24001be5           ldr      r0, [fp, #-0x24]
0x2481cc  070081e5           str      r0, [r1, #7]
0x2481d0  18301be5           ldr      r3, [fp, #-0x18]
0x2481d4  2f3081e5           str      r3, [r1, #0x2f]
0x2481d8  0100a0e1           mov      r0, r1
0x2481dc  28101be5           ldr      r1, [fp, #-0x28]
0x2481e0  1f9081e2           add      sb, r1, #0x1f
0x2481e4  000089e5           str      r0, [sb]
0x2481e8  010010e3           tst      r0, #1
0x2481ec  0500000a           beq      #0x248208
0x2481f0  01c051e5           ldrb     ip, [r1, #-1]
0x2481f4  01e050e5           ldrb     lr, [r0, #-1]
0x2481f8  2cc10ee0           and      ip, lr, ip, lsr #2
0x2481fc  28e09ae5           ldr      lr, [sl, #0x28]
0x248200  0e001ce1           tst      ip, lr
0x248204  a93b031b           blne     #0x3170b0
0x248208  40109ae5           ldr      r1, [sl, #0x40]
0x24820c  0420a0e3           mov      r2, #4
0x248210  a54203eb           bl       #0x318cac
0x248214  24000be5           str      r0, [fp, #-0x24]
0x248218  0b2080e2           add      r2, r0, #0xb
0x24821c  01c985e2           add      ip, r5, #0x4000
0x248220  8bcd9ce5           ldr      ip, [ip, #0xd8b]  # pool[4961] = snapshotRef(709)
0x248224  00c082e5           str      ip, [r2]
0x248228  14e01be5           ldr      lr, [fp, #-0x14]
0x24822c  1690a0e3           mov      sb, #0x16
0x248230  00428de8           stm      sp, {sb, lr}
0x248234  5f2dfeeb           bl       #0x1d37b8
0x248238  24101be5           ldr      r1, [fp, #-0x24]
0x24823c  0f9081e2           add      sb, r1, #0xf
0x248240  000089e5           str      r0, [sb]
0x248244  010010e3           tst      r0, #1
0x248248  0500000a           beq      #0x248264
0x24824c  01c051e5           ldrb     ip, [r1, #-1]
0x248250  01e050e5           ldrb     lr, [r0, #-1]
0x248254  2cc10ee0           and      ip, lr, ip, lsr #2
0x248258  28e09ae5           ldr      lr, [sl, #0x28]
0x24825c  0e001ce1           tst      ip, lr
0x248260  923b031b           blne     #0x3170b0
0x248264  01ea85e2           add      lr, r5, #0x1000
0x248268  17e29ee5           ldr      lr, [lr, #0x217]  # pool[1156] = snapshotRef(18010)
0x24826c  24901be5           ldr      sb, [fp, #-0x24]
0x248270  00428de8           stm      sp, {sb, lr}
0x248274  a012fceb           bl       #0x14ccfc
0x248278  02ea85e2           add      lr, r5, #0x2000
0x24827c  ebe09ee5           ldr      lr, [lr, #0xeb]  # pool[2105] = snapshotRef(17928)
0x248280  01408de8           stm      sp, {r0, lr}
0x248284  ab4e95e5           ldr      r4, [r5, #0xeab]  # pool[937] = snapshotRef(34604)
0x248288  ed1600eb           bl       #0x24de44
0x24828c  00008de5           str      r0, [sp]
0x248290  d331fceb           bl       #0x1549e4
0x248294  24000be5           str      r0, [fp, #-0x24]
0x248298  931800eb           bl       #0x24e4ec
0x24829c  0010a0e1           mov      r1, r0
0x2482a0  24001be5           ldr      r0, [fp, #-0x24]
0x2482a4  070081e5           str      r0, [r1, #7]
0x2482a8  18201be5           ldr      r2, [fp, #-0x18]
0x2482ac  2f2081e5           str      r2, [r1, #0x2f]
0x2482b0  0100a0e1           mov      r0, r1
0x2482b4  28101be5           ldr      r1, [fp, #-0x28]
0x2482b8  239081e2           add      sb, r1, #0x23
0x2482bc  000089e5           str      r0, [sb]
0x2482c0  010010e3           tst      r0, #1
0x2482c4  0500000a           beq      #0x2482e0
0x2482c8  01c051e5           ldrb     ip, [r1, #-1]
0x2482cc  01e050e5           ldrb     lr, [r0, #-1]
0x2482d0  2cc10ee0           and      ip, lr, ip, lsr #2
0x2482d4  28e09ae5           ldr      lr, [sl, #0x28]
0x2482d8  0e001ce1           tst      ip, lr
0x2482dc  733b031b           blne     #0x3170b0
0x2482e0  14e01be5           ldr      lr, [fp, #-0x14]
0x2482e4  0a90a0e3           mov      sb, #0xa
0x2482e8  00428de8           stm      sp, {sb, lr}
0x2482ec  312dfeeb           bl       #0x1d37b8
0x2482f0  c01fa0e1           asr      r1, r0, #0x1f
0x2482f4  c020b0e1           asrs     r2, r0, #1
0x2482f8  0100003a           blo      #0x248304
0x2482fc  072090e5           ldr      r2, [r0, #7]
0x248300  0b1090e5           ldr      r1, [r0, #0xb]
0x248304  3c100be5           str      r1, [fp, #-0x3c]
0x248308  40200be5           str      r2, [fp, #-0x40]
0x24830c  0060a0e3           mov      r6, #0
0x248310  0040a0e3           mov      r4, #0
0x248314  0030a0e3           mov      r3, #0
0x248318  0000a0e3           mov      r0, #0
0x24831c  24c09ae5           ldr      ip, [sl, #0x24]
0x248320  0c005de1           cmp      sp, ip
0x248324  a442039b           blls     #0x318dbc
0x248328  010050e1           cmp      r0, r1
0x24832c  020000ba           blt      #0x24833c
0x248330  420000ca           bgt      #0x248440
0x248334  020053e1           cmp      r3, r2
0x248338  4000002a           bhs      #0x248440
0x24833c  0690a0e1           mov      sb, r6
0x248340  0480a0e1           mov      r8, r4
0x248344  0060a0e3           mov      r6, #0
0x248348  0040a0e3           mov      r4, #0
0x24834c  34800be5           str      r8, [fp, #-0x34]
0x248350  38900be5           str      sb, [fp, #-0x38]
0x248354  24c09ae5           ldr      ip, [sl, #0x24]
0x248358  0c005de1           cmp      sp, ip
0x24835c  9642039b           blls     #0x318dbc
0x248360  010054e1           cmp      r4, r1
0x248364  020000ba           blt      #0x248374
0x248368  270000ca           bgt      #0x24840c
0x24836c  020056e1           cmp      r6, r2
0x248370  2500002a           bhs      #0x24840c
0x248374  930401e0           mul      r1, r3, r4
0x248378  901628e0           mla      r8, r0, r6, r1
0x24837c  939681e0           umull    sb, r1, r3, r6
0x248380  018088e0           add      r8, r8, r1
0x248384  000058e3           cmp      r8, #0
0x248388  1a0000ca           bgt      #0x2483f8
0x24838c  010000ba           blt      #0x248398
0x248390  060059e3           cmp      sb, #6
0x248394  1700008a           bhi      #0x2483f8
0x248398  3c101be5           ldr      r1, [fp, #-0x3c]
0x24839c  069093e0           adds     sb, r3, r6
0x2483a0  0480b0e0           adcs     r8, r0, r4
0x2483a4  020059e1           cmp      sb, r2
0x2483a8  01005801           cmpeq    r8, r1
0x2483ac  0c00000a           beq      #0x2483e4
0x2483b0  38801be5           ldr      r8, [fp, #-0x38]
0x2483b4  34901be5           ldr      sb, [fp, #-0x34]
0x2483b8  012098e2           adds     r2, r8, #1
0x2483bc  0010b9e2           adcs     r1, sb, #0
0x2483c0  019096e2           adds     sb, r6, #1
0x2483c4  0080b4e2           adcs     r8, r4, #0
0x2483c8  0960a0e1           mov      r6, sb
0x2483cc  0290a0e1           mov      sb, r2
0x2483d0  0840a0e1           mov      r4, r8
0x2483d4  0180a0e1           mov      r8, r1
0x2483d8  40201be5           ldr      r2, [fp, #-0x40]
0x2483dc  3c101be5           ldr      r1, [fp, #-0x3c]
0x2483e0  d9ffffea           b        #0x24834c
0x2483e4  38801be5           ldr      r8, [fp, #-0x38]
0x2483e8  34901be5           ldr      sb, [fp, #-0x34]
0x2483ec  0860a0e1           mov      r6, r8
0x2483f0  0940a0e1           mov      r4, sb
0x2483f4  110000ea           b        #0x248440
0x2483f8  38801be5           ldr      r8, [fp, #-0x38]
0x2483fc  34901be5           ldr      sb, [fp, #-0x34]
0x248400  0860a0e1           mov      r6, r8
0x248404  0940a0e1           mov      r4, sb
0x248408  050000ea           b        #0x248424
0x24840c  38801be5           ldr      r8, [fp, #-0x38]
0x248410  34901be5           ldr      sb, [fp, #-0x34]
0x248414  642098e2           adds     r2, r8, #0x64
0x248418  0010b9e2           adcs     r1, sb, #0
0x24841c  0260a0e1           mov      r6, r2
0x248420  0140a0e1           mov      r4, r1
0x248424  012093e2           adds     r2, r3, #1
0x248428  0010b0e2           adcs     r1, r0, #0
0x24842c  0230a0e1           mov      r3, r2
0x248430  0100a0e1           mov      r0, r1
0x248434  40201be5           ldr      r2, [fp, #-0x40]
0x248438  3c101be5           ldr      r1, [fp, #-0x3c]
0x24843c  b6ffffea           b        #0x24831c
0x248440  18201be5           ldr      r2, [fp, #-0x18]
0x248444  20301be5           ldr      r3, [fp, #-0x20]
0x248448  8600a0e1           lsl      r0, r6, #1
0x24844c  c00056e1           cmp      r6, r0, asr #1
0x248450  c00f5401           cmpeq    r4, r0, asr #31
0x248454  0200000a           beq      #0x248464
0x248458  834203eb           bl       #0x318e6c
0x24845c  076080e5           str      r6, [r0, #7]
0x248460  0b4080e5           str      r4, [r0, #0xb]
0x248464  00008de5           str      r0, [sp]
0x248468  5d31fceb           bl       #0x1549e4
0x24846c  24000be5           str      r0, [fp, #-0x24]
0x248470  1d1800eb           bl       #0x24e4ec
0x248474  0010a0e1           mov      r1, r0
0x248478  24001be5           ldr      r0, [fp, #-0x24]
0x24847c  070081e5           str      r0, [r1, #7]
0x248480  18201be5           ldr      r2, [fp, #-0x18]
0x248484  2f2081e5           str      r2, [r1, #0x2f]
0x248488  0100a0e1           mov      r0, r1
0x24848c  28101be5           ldr      r1, [fp, #-0x28]
0x248490  279081e2           add      sb, r1, #0x27
0x248494  000089e5           str      r0, [sb]
0x248498  010010e3           tst      r0, #1
0x24849c  0500000a           beq      #0x2484b8
0x2484a0  01c051e5           ldrb     ip, [r1, #-1]
0x2484a4  01e050e5           ldrb     lr, [r0, #-1]
0x2484a8  2cc10ee0           and      ip, lr, ip, lsr #2
0x2484ac  28e09ae5           ldr      lr, [sl, #0x28]
0x2484b0  0e001ce1           tst      ip, lr
0x2484b4  fd3a031b           blne     #0x3170b0
0x2484b8  20001be5           ldr      r0, [fp, #-0x20]
0x2484bc  000050e3           cmp      r0, #0
0x2484c0  0200001a           bne      #0x2484d0
0x2484c4  14101be5           ldr      r1, [fp, #-0x14]
0x2484c8  0200a0e1           mov      r0, r2
0x2484cc  040000ea           b        #0x2484e4
0x2484d0  14e01be5           ldr      lr, [fp, #-0x14]
0x2484d4  00e08de5           str      lr, [sp]
0x2484d8  4131fceb           bl       #0x1549e4
0x2484dc  0010a0e1           mov      r1, r0
0x2484e0  18001be5           ldr      r0, [fp, #-0x18]
0x2484e4  0c401be5           ldr      r4, [fp, #-0xc]
0x2484e8  10301be5           ldr      r3, [fp, #-0x10]
0x2484ec  14201be5           ldr      r2, [fp, #-0x14]
0x2484f0  df1500eb           bl       #0x24dc74
0x2484f4  24000be5           str      r0, [fp, #-0x24]
0x2484f8  fb1700eb           bl       #0x24e4ec
0x2484fc  0010a0e1           mov      r1, r0
0x248500  24001be5           ldr      r0, [fp, #-0x24]
0x248504  070081e5           str      r0, [r1, #7]
0x248508  18201be5           ldr      r2, [fp, #-0x18]
0x24850c  2f2081e5           str      r2, [r1, #0x2f]
0x248510  0100a0e1           mov      r0, r1
0x248514  28101be5           ldr      r1, [fp, #-0x28]
0x248518  2b9081e2           add      sb, r1, #0x2b
0x24851c  000089e5           str      r0, [sb]
0x248520  010010e3           tst      r0, #1
0x248524  0500000a           beq      #0x248540
0x248528  01c051e5           ldrb     ip, [r1, #-1]
0x24852c  01e050e5           ldrb     lr, [r0, #-1]
0x248530  2cc10ee0           and      ip, lr, ip, lsr #2
0x248534  28e09ae5           ldr      lr, [sl, #0x28]
0x248538  0e001ce1           tst      ip, lr
0x24853c  db3a031b           blne     #0x3170b0
0x248540  14e01be5           ldr      lr, [fp, #-0x14]
0x248544  0a90a0e3           mov      sb, #0xa
0x248548  00428de8           stm      sp, {sb, lr}
0x24854c  992cfeeb           bl       #0x1d37b8
0x248550  c02fa0e1           asr      r2, r0, #0x1f
0x248554  c010b0e1           asrs     r1, r0, #1
0x248558  0100003a           blo      #0x248564
0x24855c  071090e5           ldr      r1, [r0, #7]
0x248560  0b2090e5           ldr      r2, [r0, #0xb]
0x248564  6a1500eb           bl       #0x24db14
0x248568  073495e5           ldr      r3, [r5, #0x407]  # pool[256] = snapshotRef(18555)
0x24856c  24000be5           str      r0, [fp, #-0x24]
0x248570  641500eb           bl       #0x24db08
0x248574  0030a0e1           mov      r3, r0
0x248578  24001be5           ldr      r0, [fp, #-0x24]
0x24857c  2c300be5           str      r3, [fp, #-0x2c]
0x248580  0b0083e5           str      r0, [r3, #0xb]
0x248584  08201be5           ldr      r2, [fp, #-8]
0x248588  011985e2           add      r1, r5, #0x4000
0x24858c  8f1d91e5           ldr      r1, [r1, #0xd8f]  # pool[4962] = ProbeApp.<anonymous closure>
0x248590  193e03eb           bl       #0x317dfc
0x248594  0010a0e1           mov      r1, r0
0x248598  2c001be5           ldr      r0, [fp, #-0x2c]
0x24859c  0f1080e5           str      r1, [r0, #0xf]
0x2485a0  28101be5           ldr      r1, [fp, #-0x28]
0x2485a4  2f9081e2           add      sb, r1, #0x2f
0x2485a8  000089e5           str      r0, [sb]
0x2485ac  010010e3           tst      r0, #1
0x2485b0  0500000a           beq      #0x2485cc
0x2485b4  01c051e5           ldrb     ip, [r1, #-1]
0x2485b8  01e050e5           ldrb     lr, [r0, #-1]
0x2485bc  2cc10ee0           and      ip, lr, ip, lsr #2
0x2485c0  28e09ae5           ldr      lr, [sl, #0x28]
0x2485c4  0e001ce1           tst      ip, lr
0x2485c8  b83a031b           blne     #0x3170b0
0x2485cc  0c101be5           ldr      r1, [fp, #-0xc]
0x2485d0  10001be5           ldr      r0, [fp, #-0x10]
0x2485d4  0120a0e1           mov      r2, r1
0x2485d8  030002e2           and      r0, r2, #3
0x2485dc  0c000be5           str      r0, [fp, #-0xc]
0x2485e0  0010a0e1           mov      r1, r0
0x2485e4  022022e0           eor      r2, r2, r2
0x2485e8  8b1400eb           bl       #0x24d81c
0x2485ec  031090e5           ldr      r1, [r0, #3]
0x2485f0  0020a0e1           mov      r2, r0
0x2485f4  1801fceb           bl       #0x148a5c
0x2485f8  00008de5           str      r0, [sp]
0x2485fc  f830fceb           bl       #0x1549e4
0x248600  24000be5           str      r0, [fp, #-0x24]
0x248604  b81700eb           bl       #0x24e4ec
0x248608  0010a0e1           mov      r1, r0
0x24860c  24001be5           ldr      r0, [fp, #-0x24]
0x248610  070081e5           str      r0, [r1, #7]
0x248614  18201be5           ldr      r2, [fp, #-0x18]
0x248618  2f2081e5           str      r2, [r1, #0x2f]
0x24861c  0100a0e1           mov      r0, r1
0x248620  28101be5           ldr      r1, [fp, #-0x28]
0x248624  339081e2           add      sb, r1, #0x33
0x248628  000089e5           str      r0, [sb]
0x24862c  010010e3           tst      r0, #1
0x248630  0500000a           beq      #0x24864c
0x248634  01c051e5           ldrb     ip, [r1, #-1]
0x248638  01e050e5           ldrb     lr, [r0, #-1]
0x24863c  2cc10ee0           and      ip, lr, ip, lsr #2
0x248640  28e09ae5           ldr      lr, [sl, #0x28]
0x248644  0e001ce1           tst      ip, lr
0x248648  983a031b           blne     #0x3170b0
0x24864c  891200eb           bl       #0x24d078
0x248650  00008de5           str      r0, [sp]
0x248654  e230fceb           bl       #0x1549e4
0x248658  24000be5           str      r0, [fp, #-0x24]
0x24865c  a21700eb           bl       #0x24e4ec
0x248660  0010a0e1           mov      r1, r0
0x248664  24001be5           ldr      r0, [fp, #-0x24]
0x248668  070081e5           str      r0, [r1, #7]
0x24866c  18201be5           ldr      r2, [fp, #-0x18]
0x248670  2f2081e5           str      r2, [r1, #0x2f]
0x248674  0100a0e1           mov      r0, r1
0x248678  28101be5           ldr      r1, [fp, #-0x28]
0x24867c  379081e2           add      sb, r1, #0x37
0x248680  000089e5           str      r0, [sb]
0x248684  010010e3           tst      r0, #1
0x248688  0500000a           beq      #0x2486a4
0x24868c  01c051e5           ldrb     ip, [r1, #-1]
0x248690  01e050e5           ldrb     lr, [r0, #-1]
0x248694  2cc10ee0           and      ip, lr, ip, lsr #2
0x248698  28e09ae5           ldr      lr, [sl, #0x28]
0x24869c  0e001ce1           tst      ip, lr
0x2486a0  823a031b           blne     #0x3170b0
0x2486a4  701200eb           bl       #0x24d06c
0x2486a8  24000be5           str      r0, [fp, #-0x24]
0x2486ac  6e1200eb           bl       #0x24d06c
0x2486b0  24101be5           ldr      r1, [fp, #-0x24]
0x2486b4  0020a0e1           mov      r2, r0
0x2486b8  5c1200eb           bl       #0x24d030
0x2486bc  24000be5           str      r0, [fp, #-0x24]
0x2486c0  891700eb           bl       #0x24e4ec
0x2486c4  0010a0e1           mov      r1, r0
0x2486c8  24001be5           ldr      r0, [fp, #-0x24]
0x2486cc  070081e5           str      r0, [r1, #7]
0x2486d0  18201be5           ldr      r2, [fp, #-0x18]
0x2486d4  2f2081e5           str      r2, [r1, #0x2f]
0x2486d8  0100a0e1           mov      r0, r1
0x2486dc  28101be5           ldr      r1, [fp, #-0x28]
0x2486e0  3b9081e2           add      sb, r1, #0x3b
0x2486e4  000089e5           str      r0, [sb]
0x2486e8  010010e3           tst      r0, #1
0x2486ec  0500000a           beq      #0x248708
0x2486f0  01c051e5           ldrb     ip, [r1, #-1]
0x2486f4  01e050e5           ldrb     lr, [r0, #-1]
0x2486f8  2cc10ee0           and      ip, lr, ip, lsr #2
0x2486fc  28e09ae5           ldr      lr, [sl, #0x28]
0x248700  0e001ce1           tst      ip, lr
0x248704  693a031b           blne     #0x3170b0
0x248708  d61100eb           bl       #0x24ce68
0x24870c  0030a0e1           mov      r3, r0
0x248710  0120a0e1           mov      r2, r1
0x248714  8300a0e1           lsl      r0, r3, #1
0x248718  c00053e1           cmp      r3, r0, asr #1
0x24871c  c00f5201           cmpeq    r2, r0, asr #31
0x248720  0200000a           beq      #0x248730
0x248724  d04103eb           bl       #0x318e6c
0x248728  073080e5           str      r3, [r0, #7]
0x24872c  0b2080e5           str      r2, [r0, #0xb]
0x248730  00008de5           str      r0, [sp]
0x248734  aa30fceb           bl       #0x1549e4
0x248738  24000be5           str      r0, [fp, #-0x24]
0x24873c  6a1700eb           bl       #0x24e4ec
0x248740  0010a0e1           mov      r1, r0
0x248744  24001be5           ldr      r0, [fp, #-0x24]
0x248748  070081e5           str      r0, [r1, #7]
0x24874c  18201be5           ldr      r2, [fp, #-0x18]
0x248750  2f2081e5           str      r2, [r1, #0x2f]
0x248754  0100a0e1           mov      r0, r1
0x248758  28101be5           ldr      r1, [fp, #-0x28]
0x24875c  3f9081e2           add      sb, r1, #0x3f
0x248760  000089e5           str      r0, [sb]
0x248764  010010e3           tst      r0, #1
0x248768  0500000a           beq      #0x248784
0x24876c  01c051e5           ldrb     ip, [r1, #-1]
0x248770  01e050e5           ldrb     lr, [r0, #-1]
0x248774  2cc10ee0           and      ip, lr, ip, lsr #2
0x248778  28e09ae5           ldr      lr, [sl, #0x28]
0x24877c  0e001ce1           tst      ip, lr
0x248780  4a3a031b           blne     #0x3170b0
0x248784  14e01be5           ldr      lr, [fp, #-0x14]
0x248788  0a90a0e3           mov      sb, #0xa
0x24878c  00428de8           stm      sp, {sb, lr}
0x248790  082cfeeb           bl       #0x1d37b8
0x248794  c01fa0e1           asr      r1, r0, #0x1f
0x248798  c020b0e1           asrs     r2, r0, #1
0x24879c  0100003a           blo      #0x2487a8
0x2487a0  072090e5           ldr      r2, [r0, #7]
0x2487a4  0b1090e5           ldr      r1, [r0, #0xb]
0x2487a8  10200be5           str      r2, [fp, #-0x10]
0x2487ac  34100be5           str      r1, [fp, #-0x34]
0x2487b0  a91100eb           bl       #0x24ce5c
0x2487b4  0020a0e1           mov      r2, r0
0x2487b8  10101be5           ldr      r1, [fp, #-0x10]
0x2487bc  34001be5           ldr      r0, [fp, #-0x34]
0x2487c0  031082e5           str      r1, [r2, #3]
0x2487c4  070082e5           str      r0, [r2, #7]
0x2487c8  0400a0e3           mov      r0, #4
0x2487cc  0010a0e3           mov      r1, #0
0x2487d0  0b0082e5           str      r0, [r2, #0xb]
0x2487d4  0f1082e5           str      r1, [r2, #0xf]
0x2487d8  011985e2           add      r1, r5, #0x4000
0x2487dc  931d91e5           ldr      r1, [r1, #0xd93]  # pool[4963] = snapshotInstance(E15Vec)
0x2487e0  581dfdeb           bl       #0x18fd48
0x2487e4  0030a0e1           mov      r3, r0
0x2487e8  0120a0e1           mov      r2, r1
0x2487ec  8300a0e1           lsl      r0, r3, #1
0x2487f0  c00053e1           cmp      r3, r0, asr #1
0x2487f4  c00f5201           cmpeq    r2, r0, asr #31
0x2487f8  0200000a           beq      #0x248808
0x2487fc  9a4103eb           bl       #0x318e6c
0x248800  073080e5           str      r3, [r0, #7]
0x248804  0b2080e5           str      r2, [r0, #0xb]
0x248808  00008de5           str      r0, [sp]
0x24880c  7430fceb           bl       #0x1549e4
0x248810  24000be5           str      r0, [fp, #-0x24]
0x248814  341700eb           bl       #0x24e4ec
0x248818  0010a0e1           mov      r1, r0
0x24881c  24001be5           ldr      r0, [fp, #-0x24]
0x248820  070081e5           str      r0, [r1, #7]
0x248824  18201be5           ldr      r2, [fp, #-0x18]
0x248828  2f2081e5           str      r2, [r1, #0x2f]
0x24882c  0100a0e1           mov      r0, r1
0x248830  28101be5           ldr      r1, [fp, #-0x28]
0x248834  439081e2           add      sb, r1, #0x43
0x248838  000089e5           str      r0, [sb]
0x24883c  010010e3           tst      r0, #1
0x248840  0500000a           beq      #0x24885c
0x248844  01c051e5           ldrb     ip, [r1, #-1]
0x248848  01e050e5           ldrb     lr, [r0, #-1]
0x24884c  2cc10ee0           and      ip, lr, ip, lsr #2
0x248850  28e09ae5           ldr      lr, [sl, #0x28]
0x248854  0e001ce1           tst      ip, lr
0x248858  143a031b           blne     #0x3170b0
0x24885c  b33395e5           ldr      r3, [r5, #0x3b3]  # pool[235] = snapshotRef(18337)
0x248860  a829fceb           bl       #0x152f08
0x248864  0010a0e1           mov      r1, r0
0x248868  8f0595e5           ldr      r0, [r5, #0x58f]  # pool[354] = snapshotRef(51207)
0x24886c  24100be5           str      r1, [fp, #-0x24]
0x248870  170081e5           str      r0, [r1, #0x17]
0x248874  0000a0e3           mov      r0, #0
0x248878  070081e5           str      r0, [r1, #7]
0x24887c  932595e5           ldr      r2, [r5, #0x593]  # pool[355] = snapshotRef(47653)
0x248880  0b2081e5           str      r2, [r1, #0xb]
0x248884  0f0081e5           str      r0, [r1, #0xf]
0x248888  130081e5           str      r0, [r1, #0x13]
0x24888c  14e01be5           ldr      lr, [fp, #-0x14]
0x248890  00e08de5           str      lr, [sp]
0x248894  5230fceb           bl       #0x1549e4
0x248898  24101be5           ldr      r1, [fp, #-0x24]
0x24889c  0020a0e1           mov      r2, r0
0x2488a0  e6fc01eb           bl       #0x2c7c40
0x2488a4  24101be5           ldr      r1, [fp, #-0x24]
0x2488a8  012985e2           add      r2, r5, #0x4000
0x2488ac  732d92e5           ldr      r2, [r2, #0xd73]  # pool[4955] = snapshotRef(545)
0x2488b0  e2fc01eb           bl       #0x2c7c40
0x2488b4  24101be5           ldr      r1, [fp, #-0x24]
0x2488b8  3c0d00eb           bl       #0x24bdb0
0x2488bc  00008de5           str      r0, [sp]
0x2488c0  4730fceb           bl       #0x1549e4
0x2488c4  24000be5           str      r0, [fp, #-0x24]
0x2488c8  071700eb           bl       #0x24e4ec
0x2488cc  0010a0e1           mov      r1, r0
0x2488d0  24001be5           ldr      r0, [fp, #-0x24]
0x2488d4  070081e5           str      r0, [r1, #7]
0x2488d8  18301be5           ldr      r3, [fp, #-0x18]
0x2488dc  2f3081e5           str      r3, [r1, #0x2f]
0x2488e0  0100a0e1           mov      r0, r1
0x2488e4  28101be5           ldr      r1, [fp, #-0x28]
0x2488e8  479081e2           add      sb, r1, #0x47
0x2488ec  000089e5           str      r0, [sb]
0x2488f0  010010e3           tst      r0, #1
0x2488f4  0500000a           beq      #0x248910
0x2488f8  01c051e5           ldrb     ip, [r1, #-1]
0x2488fc  01e050e5           ldrb     lr, [r0, #-1]
0x248900  2cc10ee0           and      ip, lr, ip, lsr #2
0x248904  28e09ae5           ldr      lr, [sl, #0x28]
0x248908  0e001ce1           tst      ip, lr
0x24890c  e739031b           blne     #0x3170b0
0x248910  40109ae5           ldr      r1, [sl, #0x40]
0x248914  0620a0e3           mov      r2, #6
0x248918  e34003eb           bl       #0x318cac
0x24891c  0b2080e2           add      r2, r0, #0xb
0x248920  01c985e2           add      ip, r5, #0x4000
0x248924  97cd9ce5           ldr      ip, [ip, #0xd97]  # pool[4964] = "[{\"a\":"
0x248928  00c082e5           str      ip, [r2]
0x24892c  14101be5           ldr      r1, [fp, #-0x14]
0x248930  0f3080e2           add      r3, r0, #0xf
0x248934  001083e5           str      r1, [r3]
0x248938  133080e2           add      r3, r0, #0x13
0x24893c  01c985e2           add      ip, r5, #0x4000
0x248940  9bcd9ce5           ldr      ip, [ip, #0xd9b]  # pool[4965] = "},{\"b\":null}]"
0x248944  00c083e5           str      ip, [r3]
0x248948  00008de5           str      r0, [sp]
0x24894c  3d2ffceb           bl       #0x154648
0x248950  0010a0e1           mov      r1, r0
0x248954  b70c00eb           bl       #0x24bc38
0x248958  071090e5           ldr      r1, [r0, #7]
0x24895c  00108de5           str      r1, [sp]
0x248960  457901eb           bl       #0x2a6e7c
0x248964  24000be5           str      r0, [fp, #-0x24]
0x248968  df1600eb           bl       #0x24e4ec
0x24896c  0010a0e1           mov      r1, r0
0x248970  24001be5           ldr      r0, [fp, #-0x24]
0x248974  070081e5           str      r0, [r1, #7]
0x248978  18201be5           ldr      r2, [fp, #-0x18]
0x24897c  2f2081e5           str      r2, [r1, #0x2f]
0x248980  0100a0e1           mov      r0, r1
0x248984  28101be5           ldr      r1, [fp, #-0x28]
0x248988  4b9081e2           add      sb, r1, #0x4b
0x24898c  000089e5           str      r0, [sb]
0x248990  010010e3           tst      r0, #1
0x248994  0500000a           beq      #0x2489b0
0x248998  01c051e5           ldrb     ip, [r1, #-1]
0x24899c  01e050e5           ldrb     lr, [r0, #-1]
0x2489a0  2cc10ee0           and      ip, lr, ip, lsr #2
0x2489a4  28e09ae5           ldr      lr, [sl, #0x28]
0x2489a8  0e001ce1           tst      ip, lr
0x2489ac  bf39031b           blne     #0x3170b0
0x2489b0  14e01be5           ldr      lr, [fp, #-0x14]
0x2489b4  7d9ea0e3           mov      sb, #0x7d0
0x2489b8  00428de8           stm      sp, {sb, lr}
0x2489bc  7d2bfeeb           bl       #0x1d37b8
0x2489c0  40e09ae5           ldr      lr, [sl, #0x40]
0x2489c4  01408de8           stm      sp, {r0, lr}
0x2489c8  1c44fceb           bl       #0x159a40
0x2489cc  24000be5           str      r0, [fp, #-0x24]
0x2489d0  40e09ae5           ldr      lr, [sl, #0x40]
0x2489d4  1090a0e3           mov      sb, #0x10
0x2489d8  00428de8           stm      sp, {sb, lr}
0x2489dc  1744fceb           bl       #0x159a40
0x2489e0  0010a0e1           mov      r1, r0
0x2489e4  24001be5           ldr      r0, [fp, #-0x24]
0x2489e8  03c080e2           add      ip, r0, #3
0x2489ec  010b9ced           vldr     d0, [ip, #4]
0x2489f0  03c081e2           add      ip, r1, #3
0x2489f4  012b9ced           vldr     d2, [ip, #4]
0x2489f8  024b80ee           vdiv.f64 d4, d2
0x2489fc  540124f2           vorr     q0, q2, q2
0x248a00  ff0200eb           bl       #0x249604
0x248a04  24000be5           str      r0, [fp, #-0x24]
0x248a08  b71600eb           bl       #0x24e4ec
0x248a0c  0010a0e1           mov      r1, r0
0x248a10  24001be5           ldr      r0, [fp, #-0x24]
0x248a14  070081e5           str      r0, [r1, #7]
0x248a18  18201be5           ldr      r2, [fp, #-0x18]
0x248a1c  2f2081e5           str      r2, [r1, #0x2f]
0x248a20  0100a0e1           mov      r0, r1
0x248a24  28101be5           ldr      r1, [fp, #-0x28]
0x248a28  4f9081e2           add      sb, r1, #0x4f
0x248a2c  000089e5           str      r0, [sb]
0x248a30  010010e3           tst      r0, #1
0x248a34  0500000a           beq      #0x248a50
0x248a38  01c051e5           ldrb     ip, [r1, #-1]
0x248a3c  01e050e5           ldrb     lr, [r0, #-1]
0x248a40  2cc10ee0           and      ip, lr, ip, lsr #2
0x248a44  28e09ae5           ldr      lr, [sl, #0x28]
0x248a48  0e001ce1           tst      ip, lr
0x248a4c  9739031b           blne     #0x3170b0
0x248a50  14e01be5           ldr      lr, [fp, #-0x14]
0x248a54  0690a0e3           mov      sb, #6
0x248a58  00428de8           stm      sp, {sb, lr}
0x248a5c  552bfeeb           bl       #0x1d37b8
0x248a60  0010a0e1           mov      r1, r0
0x248a64  0c001be5           ldr      r0, [fp, #-0xc]
0x248a68  8020a0e1           lsl      r2, r0, #1
0x248a6c  04108de5           str      r1, [sp, #4]
0x248a70  00208de5           str      r2, [sp]
0x248a74  374195e5           ldr      r4, [r5, #0x137]  # pool[76] = snapshotRef(23)
0x248a78  fe1cfdeb           bl       #0x18fe78
0x248a7c  00008de5           str      r0, [sp]
0x248a80  d72ffceb           bl       #0x1549e4
0x248a84  24000be5           str      r0, [fp, #-0x24]
0x248a88  971600eb           bl       #0x24e4ec
0x248a8c  0010a0e1           mov      r1, r0
0x248a90  24001be5           ldr      r0, [fp, #-0x24]
0x248a94  070081e5           str      r0, [r1, #7]
0x248a98  18201be5           ldr      r2, [fp, #-0x18]
0x248a9c  2f2081e5           str      r2, [r1, #0x2f]
0x248aa0  0100a0e1           mov      r0, r1
0x248aa4  28101be5           ldr      r1, [fp, #-0x28]
0x248aa8  539081e2           add      sb, r1, #0x53
0x248aac  000089e5           str      r0, [sb]
0x248ab0  010010e3           tst      r0, #1
0x248ab4  0500000a           beq      #0x248ad0
0x248ab8  01c051e5           ldrb     ip, [r1, #-1]
0x248abc  01e050e5           ldrb     lr, [r0, #-1]
0x248ac0  2cc10ee0           and      ip, lr, ip, lsr #2
0x248ac4  28e09ae5           ldr      lr, [sl, #0x28]
0x248ac8  0e001ce1           tst      ip, lr
0x248acc  7739031b           blne     #0x3170b0
0x248ad0  c80200eb           bl       #0x2495f8
0x248ad4  0010a0e1           mov      r1, r0
0x248ad8  c30200eb           bl       #0x2495ec
0x248adc  24000be5           str      r0, [fp, #-0x24]
0x248ae0  811600eb           bl       #0x24e4ec
0x248ae4  0010a0e1           mov      r1, r0
0x248ae8  24001be5           ldr      r0, [fp, #-0x24]
0x248aec  070081e5           str      r0, [r1, #7]
0x248af0  18201be5           ldr      r2, [fp, #-0x18]
0x248af4  2f2081e5           str      r2, [r1, #0x2f]
0x248af8  0100a0e1           mov      r0, r1
0x248afc  28101be5           ldr      r1, [fp, #-0x28]
0x248b00  579081e2           add      sb, r1, #0x57
0x248b04  000089e5           str      r0, [sb]
0x248b08  010010e3           tst      r0, #1
0x248b0c  0500000a           beq      #0x248b28
0x248b10  01c051e5           ldrb     ip, [r1, #-1]
0x248b14  01e050e5           ldrb     lr, [r0, #-1]
0x248b18  2cc10ee0           and      ip, lr, ip, lsr #2
0x248b1c  28e09ae5           ldr      lr, [sl, #0x28]
0x248b20  0e001ce1           tst      ip, lr
0x248b24  6139031b           blne     #0x3170b0
0x248b28  1c101be5           ldr      r1, [fp, #-0x1c]
0x248b2c  6f0200eb           bl       #0x2494f0
0x248b30  01c985e2           add      ip, r5, #0x4000
0x248b34  9fcd9ce5           ldr      ip, [ip, #0xd9f]  # pool[4966] = snapshotInstance(E21Mode)
0x248b38  0c0050e1           cmp      r0, ip
0x248b3c  0600000a           beq      #0x248b5c
0x248b40  0f2090e5           ldr      r2, [r0, #0xf]
0x248b44  131090e5           ldr      r1, [r0, #0x13]
0x248b48  000051e3           cmp      r1, #0
0x248b4c  020000ba           blt      #0x248b5c
0x248b50  030000ca           bgt      #0x248b64
0x248b54  000052e3           cmp      r2, #0
0x248b58  0100002a           bhs      #0x248b64
0x248b5c  df4595e5           ldr      r4, [r5, #0x5df]  # pool[374] = snapshotRef(533)
0x248b60  000000ea           b        #0x248b68
0x248b64  e74595e5           ldr      r4, [r5, #0x5e7]  # pool[376] = snapshotRef(837)
0x248b68  18001be5           ldr      r0, [fp, #-0x18]
0x248b6c  20201be5           ldr      r2, [fp, #-0x20]
0x248b70  04301be5           ldr      r3, [fp, #-4]
0x248b74  14101be5           ldr      r1, [fp, #-0x14]
0x248b78  1c400be5           str      r4, [fp, #-0x1c]
0x248b7c  5a1600eb           bl       #0x24e4ec
0x248b80  0010a0e1           mov      r1, r0
0x248b84  1c001be5           ldr      r0, [fp, #-0x1c]
0x248b88  070081e5           str      r0, [r1, #7]
0x248b8c  18201be5           ldr      r2, [fp, #-0x18]
0x248b90  2f2081e5           str      r2, [r1, #0x2f]
0x248b94  0100a0e1           mov      r0, r1
0x248b98  28101be5           ldr      r1, [fp, #-0x28]
0x248b9c  5b9081e2           add      sb, r1, #0x5b
0x248ba0  000089e5           str      r0, [sb]
0x248ba4  010010e3           tst      r0, #1
0x248ba8  0500000a           beq      #0x248bc4
0x248bac  01c051e5           ldrb     ip, [r1, #-1]
0x248bb0  01e050e5           ldrb     lr, [r0, #-1]
0x248bb4  2cc10ee0           and      ip, lr, ip, lsr #2
0x248bb8  28e09ae5           ldr      lr, [sl, #0x28]
0x248bbc  0e001ce1           tst      ip, lr
0x248bc0  3a39031b           blne     #0x3170b0
0x248bc4  02e0a0e3           mov      lr, #2
0x248bc8  00e08de5           str      lr, [sp]
0x248bcc  aa7801eb           bl       #0x2a6e7c
0x248bd0  1c000be5           str      r0, [fp, #-0x1c]
0x248bd4  441600eb           bl       #0x24e4ec
0x248bd8  0010a0e1           mov      r1, r0
0x248bdc  1c001be5           ldr      r0, [fp, #-0x1c]
0x248be0  070081e5           str      r0, [r1, #7]
0x248be4  18301be5           ldr      r3, [fp, #-0x18]
0x248be8  2f3081e5           str      r3, [r1, #0x2f]
0x248bec  0100a0e1           mov      r0, r1
0x248bf0  28101be5           ldr      r1, [fp, #-0x28]
0x248bf4  5f9081e2           add      sb, r1, #0x5f
0x248bf8  000089e5           str      r0, [sb]
0x248bfc  010010e3           tst      r0, #1
0x248c00  0500000a           beq      #0x248c1c
0x248c04  01c051e5           ldrb     ip, [r1, #-1]
0x248c08  01e050e5           ldrb     lr, [r0, #-1]
0x248c0c  2cc10ee0           and      ip, lr, ip, lsr #2
0x248c10  28e09ae5           ldr      lr, [sl, #0x28]
0x248c14  0e001ce1           tst      ip, lr
0x248c18  2439031b           blne     #0x3170b0
0x248c1c  40109ae5           ldr      r1, [sl, #0x40]
0x248c20  0220a0e3           mov      r2, #2
0x248c24  204003eb           bl       #0x318cac
0x248c28  0010a0e1           mov      r1, r0
0x248c2c  14001be5           ldr      r0, [fp, #-0x14]
0x248c30  1c100be5           str      r1, [fp, #-0x1c]
0x248c34  0b3081e2           add      r3, r1, #0xb
0x248c38  000083e5           str      r0, [r3]
0x248c3c  40309ae5           ldr      r3, [sl, #0x40]
0x248c40  6e3b03eb           bl       #0x317a00
0x248c44  0030a0e1           mov      r3, r0
0x248c48  1c001be5           ldr      r0, [fp, #-0x1c]
0x248c4c  24300be5           str      r3, [fp, #-0x24]
0x248c50  0b0083e5           str      r0, [r3, #0xb]
0x248c54  0200a0e3           mov      r0, #2
0x248c58  070083e5           str      r0, [r3, #7]
0x248c5c  08201be5           ldr      r2, [fp, #-8]
0x248c60  011985e2           add      r1, r5, #0x4000
0x248c64  a31d91e5           ldr      r1, [r1, #0xda3]  # pool[4967] = ProbeApp.<anonymous closure>
0x248c68  633c03eb           bl       #0x317dfc
0x248c6c  0010a0e1           mov      r1, r0
0x248c70  24201be5           ldr      r2, [fp, #-0x24]
0x248c74  200100eb           bl       #0x2490fc
0x248c78  08000be5           str      r0, [fp, #-8]
0x248c7c  1a1600eb           bl       #0x24e4ec
0x248c80  0010a0e1           mov      r1, r0
0x248c84  08001be5           ldr      r0, [fp, #-8]
0x248c88  070081e5           str      r0, [r1, #7]
0x248c8c  18201be5           ldr      r2, [fp, #-0x18]
0x248c90  2f2081e5           str      r2, [r1, #0x2f]
0x248c94  0100a0e1           mov      r0, r1
0x248c98  28101be5           ldr      r1, [fp, #-0x28]
0x248c9c  639081e2           add      sb, r1, #0x63
0x248ca0  000089e5           str      r0, [sb]
0x248ca4  010010e3           tst      r0, #1
0x248ca8  0500000a           beq      #0x248cc4
0x248cac  01c051e5           ldrb     ip, [r1, #-1]
0x248cb0  01e050e5           ldrb     lr, [r0, #-1]
0x248cb4  2cc10ee0           and      ip, lr, ip, lsr #2
0x248cb8  28e09ae5           ldr      lr, [sl, #0x28]
0x248cbc  0e001ce1           tst      ip, lr
0x248cc0  fa38031b           blne     #0x3170b0
0x248cc4  20001be5           ldr      r0, [fp, #-0x20]
0x248cc8  000050e3           cmp      r0, #0
0x248ccc  48109a05           ldreq    r1, [sl, #0x48]
0x248cd0  4c109a15           ldrne    r1, [sl, #0x4c]
0x248cd4  08100be5           str      r1, [fp, #-8]
0x248cd8  14e01be5           ldr      lr, [fp, #-0x14]
0x248cdc  0690a0e3           mov      sb, #6
0x248ce0  00428de8           stm      sp, {sb, lr}
0x248ce4  b32afeeb           bl       #0x1d37b8
0x248ce8  0010a0e1           mov      r1, r0
0x248cec  0000a0e3           mov      r0, #0
0x248cf0  000051e1           cmp      r1, r0
0x248cf4  48209a05           ldreq    r2, [sl, #0x48]
0x248cf8  4c209a15           ldrne    r2, [sl, #0x4c]
0x248cfc  1c200be5           str      r2, [fp, #-0x1c]
0x248d00  14e01be5           ldr      lr, [fp, #-0x14]
0x248d04  2290a0e3           mov      sb, #0x22
0x248d08  00428de8           stm      sp, {sb, lr}
0x248d0c  a92afeeb           bl       #0x1d37b8
0x248d10  c08fa0e1           asr      r8, r0, #0x1f
0x248d14  c030b0e1           asrs     r3, r0, #1
0x248d18  0100003a           blo      #0x248d24
0x248d1c  073090e5           ldr      r3, [r0, #7]
0x248d20  0b8090e5           ldr      r8, [r0, #0xb]
0x248d24  08101be5           ldr      r1, [fp, #-8]
0x248d28  1c201be5           ldr      r2, [fp, #-0x1c]
0x248d2c  b70000eb           bl       #0x249010
0x248d30  0030a0e1           mov      r3, r0
0x248d34  0120a0e1           mov      r2, r1
0x248d38  8300a0e1           lsl      r0, r3, #1
0x248d3c  c00053e1           cmp      r3, r0, asr #1
0x248d40  c00f5201           cmpeq    r2, r0, asr #31
0x248d44  0200000a           beq      #0x248d54
0x248d48  474003eb           bl       #0x318e6c
0x248d4c  073080e5           str      r3, [r0, #7]
0x248d50  0b2080e5           str      r2, [r0, #0xb]
0x248d54  00008de5           str      r0, [sp]
0x248d58  212ffceb           bl       #0x1549e4
0x248d5c  08000be5           str      r0, [fp, #-8]
0x248d60  e11500eb           bl       #0x24e4ec
0x248d64  0010a0e1           mov      r1, r0
0x248d68  08001be5           ldr      r0, [fp, #-8]
0x248d6c  070081e5           str      r0, [r1, #7]
0x248d70  18301be5           ldr      r3, [fp, #-0x18]
0x248d74  2f3081e5           str      r3, [r1, #0x2f]
0x248d78  0100a0e1           mov      r0, r1
0x248d7c  28101be5           ldr      r1, [fp, #-0x28]
0x248d80  679081e2           add      sb, r1, #0x67
0x248d84  000089e5           str      r0, [sb]
0x248d88  010010e3           tst      r0, #1
0x248d8c  0500000a           beq      #0x248da8
0x248d90  01c051e5           ldrb     ip, [r1, #-1]
0x248d94  01e050e5           ldrb     lr, [r0, #-1]
0x248d98  2cc10ee0           and      ip, lr, ip, lsr #2
0x248d9c  28e09ae5           ldr      lr, [sl, #0x28]
0x248da0  0e001ce1           tst      ip, lr
0x248da4  c138031b           blne     #0x3170b0
0x248da8  40109ae5           ldr      r1, [sl, #0x40]
0x248dac  0420a0e3           mov      r2, #4
0x248db0  bd3f03eb           bl       #0x318cac
0x248db4  0b2080e2           add      r2, r0, #0xb
0x248db8  01c985e2           add      ip, r5, #0x4000
0x248dbc  5fcd9ce5           ldr      ip, [ip, #0xd5f]  # pool[4950] = snapshotRef(130)
0x248dc0  00c082e5           str      ip, [r2]
0x248dc4  04101be5           ldr      r1, [fp, #-4]
0x248dc8  0f3080e2           add      r3, r0, #0xf
0x248dcc  001083e5           str      r1, [r3]
0x248dd0  00008de5           str      r0, [sp]
0x248dd4  1b2efceb           bl       #0x154648
0x248dd8  0010a0e1           mov      r1, r0
0x248ddc  500000eb           bl       #0x248f24
0x248de0  0010a0e1           mov      r1, r0
0x248de4  48009ae5           ldr      r0, [sl, #0x48]
0x248de8  000051e1           cmp      r1, r0
0x248dec  0100001a           bne      #0x248df8
0x248df0  df2595e5           ldr      r2, [r5, #0x5df]  # pool[374] = snapshotRef(533)
0x248df4  000000ea           b        #0x248dfc
0x248df8  e72595e5           ldr      r2, [r5, #0x5e7]  # pool[376] = snapshotRef(837)
0x248dfc  18001be5           ldr      r0, [fp, #-0x18]
0x248e00  28101be5           ldr      r1, [fp, #-0x28]
0x248e04  04200be5           str      r2, [fp, #-4]
0x248e08  b71500eb           bl       #0x24e4ec
0x248e0c  0010a0e1           mov      r1, r0
0x248e10  04001be5           ldr      r0, [fp, #-4]
0x248e14  070081e5           str      r0, [r1, #7]
0x248e18  18001be5           ldr      r0, [fp, #-0x18]
0x248e1c  2f0081e5           str      r0, [r1, #0x2f]
0x248e20  0100a0e1           mov      r0, r1
0x248e24  28101be5           ldr      r1, [fp, #-0x28]
0x248e28  6b9081e2           add      sb, r1, #0x6b
0x248e2c  000089e5           str      r0, [sb]
0x248e30  010010e3           tst      r0, #1
0x248e34  0500000a           beq      #0x248e50
0x248e38  01c051e5           ldrb     ip, [r1, #-1]
0x248e3c  01e050e5           ldrb     lr, [r0, #-1]
0x248e40  2cc10ee0           and      ip, lr, ip, lsr #2
0x248e44  28e09ae5           ldr      lr, [sl, #0x28]
0x248e48  0e001ce1           tst      ip, lr
0x248e4c  9738031b           blne     #0x3170b0
0x248e50  013985e2           add      r3, r5, #0x4000
0x248e54  6f3d93e5           ldr      r3, [r3, #0xd6f]  # pool[4954] = snapshotRef(18448)
0x248e58  e83a03eb           bl       #0x317a00
0x248e5c  0010a0e1           mov      r1, r0
0x248e60  28001be5           ldr      r0, [fp, #-0x28]
0x248e64  04100be5           str      r1, [fp, #-4]
0x248e68  0b0081e5           str      r0, [r1, #0xb]
0x248e6c  3200a0e3           mov      r0, #0x32
0x248e70  070081e5           str      r0, [r1, #7]
0x248e74  270000eb           bl       #0x248f18
0x248e78  0010a0e1           mov      r1, r0
0x248e7c  010985e2           add      r0, r5, #0x4000
0x248e80  a70d90e5           ldr      r0, [r0, #0xda7]  # pool[4968] = snapshotInstance(Axis)
0x248e84  08100be5           str      r1, [fp, #-8]
0x248e88  0b0081e5           str      r0, [r1, #0xb]
0x248e8c  010985e2           add      r0, r5, #0x4000
0x248e90  ab0d90e5           ldr      r0, [r0, #0xdab]  # pool[4969] = snapshotInstance(MainAxisAlignment)
0x248e94  0f0081e5           str      r0, [r1, #0xf]
0x248e98  010985e2           add      r0, r5, #0x4000
0x248e9c  af0d90e5           ldr      r0, [r0, #0xdaf]  # pool[4970] = snapshotInstance(MainAxisSize)
0x248ea0  130081e5           str      r0, [r1, #0x13]
0x248ea4  010985e2           add      r0, r5, #0x4000
0x248ea8  b30d90e5           ldr      r0, [r0, #0xdb3]  # pool[4971] = snapshotInstance(CrossAxisAlignment)
0x248eac  170081e5           str      r0, [r1, #0x17]
0x248eb0  010985e2           add      r0, r5, #0x4000
0x248eb4  b70d90e5           ldr      r0, [r0, #0xdb7]  # pool[4972] = snapshotInstance(VerticalDirection)
0x248eb8  1f0081e5           str      r0, [r1, #0x1f]
0x248ebc  010985e2           add      r0, r5, #0x4000
0x248ec0  bb0d90e5           ldr      r0, [r0, #0xdbb]  # pool[4973] = snapshotInstance(Clip)
0x248ec4  270081e5           str      r0, [r1, #0x27]
0x248ec8  04002de5           str      r0, [sp, #-4]!
0x248ecc  03cc04e3           movw     ip, #0x4c03
0x248ed0  0cc085e0           add      ip, r5, ip
0x248ed4  6f0b9ced           vldr     d0, [ip, #0x1bc]
0x248ed8  04009de4           pop      {r0}
0x248edc  03c081e2           add      ip, r1, #3
0x248ee0  0a0b8ced           vstr     d0, [ip, #0x28]
0x248ee4  04001be5           ldr      r0, [fp, #-4]
0x248ee8  070081e5           str      r0, [r1, #7]
0x248eec  060000eb           bl       #0x248f0c
0x248ef0  011985e2           add      r1, r5, #0x4000
0x248ef4  c71d91e5           ldr      r1, [r1, #0xdc7]  # pool[4976] = snapshotInstance(Alignment)
0x248ef8  0b1080e5           str      r1, [r0, #0xb]
0x248efc  08101be5           ldr      r1, [fp, #-8]
0x248f00  071080e5           str      r1, [r0, #7]
0x248f04  00d04be2           sub      sp, fp, #0
0x248f08  0088bde8           pop      {fp, pc}
# CFG: 0x247bc4->0x247c24/ConditionalFalse 0x247bc4->0x247c30/ConditionalTrue 0x247c24->0x247c30/Fallthrough 0x247c30->0x247c48/ConditionalFalse 0x247c30->0x247c60/ConditionalTrue 0x247c48->0x247c60/Fallthrough 0x247c60->0x247cb4/ConditionalFalse 0x247c60->0x247cc0/ConditionalTrue 0x247cb4->0x247cb8/ConditionalFalse 0x247cb4->0x247cd0/ConditionalTrue 0x247cb8->0x247cc0/ConditionalFalse 0x247cb8->0x247cd0/ConditionalTrue 0x247cc0->0x247cd8/Branch 0x247cd0->0x247cd8/Fallthrough 0x247cd8->0x247cf0/ConditionalFalse 0x247cd8->0x247cfc/ConditionalTrue 0x247cf0->0x247cfc/Fallthrough 0x247cfc->0x247d1c/ConditionalFalse 0x247cfc->0x247d24/ConditionalTrue 0x247d1c->0x247d24/Fallthrough 0x247d24->0x247d74/ConditionalFalse 0x247d24->0x247d80/ConditionalTrue 0x247d74->0x247d84/Branch 0x247d80->0x247d84/Fallthrough 0x247d84->0x247e4c/ConditionalFalse 0x247d84->0x247e54/ConditionalTrue 0x247e4c->0x247e54/Fallthrough 0x247e54->0x247f40/ConditionalFalse 0x247e54->0x247f58/ConditionalTrue 0x247f40->0x247f58/Fallthrough 0x247f58->0x247f7c/ConditionalFalse 0x247f58->0x247f88/ConditionalTrue 0x247f7c->0x247fe4/Branch 0x247f88->0x247fac/ConditionalFalse 0x247f88->0x247fd0/ConditionalTrue 0x247fac->0x247fd0/ConditionalFalse 0x247fac->0x247fdc/ConditionalTrue 0x247fd0->0x247fe4/Branch 0x247fdc->0x247fe4/Fallthrough 0x247fe4->0x248020/ConditionalFalse 0x247fe4->0x248038/ConditionalTrue 0x248020->0x248038/Fallthrough 0x248038->0x24805c/ConditionalFalse 0x248038->0x248068/ConditionalTrue 0x24805c->0x248068/Fallthrough 0x248068->0x2480a4/ConditionalFalse 0x248068->0x2480bc/ConditionalTrue 0x2480a4->0x2480bc/Fallthrough 0x2480bc->0x248100/ConditionalFalse 0x2480bc->0x248118/ConditionalTrue 0x248100->0x248118/Fallthrough 0x248118->0x2481a8/ConditionalFalse 0x248118->0x2481b4/ConditionalTrue 0x2481a8->0x2481b4/Fallthrough 0x2481b4->0x2481f0/ConditionalFalse 0x2481b4->0x248208/ConditionalTrue 0x2481f0->0x248208/Fallthrough 0x248208->0x24824c/ConditionalFalse 0x248208->0x248264/ConditionalTrue 0x24824c->0x248264/Fallthrough 0x248264->0x2482c8/ConditionalFalse 0x248264->0x2482e0/ConditionalTrue 0x2482c8->0x2482e0/Fallthrough 0x2482e0->0x2482fc/ConditionalFalse 0x2482e0->0x248304/ConditionalTrue 0x2482fc->0x248304/Fallthrough 0x248304->0x24831c/Fallthrough 0x24831c->0x248330/ConditionalFalse 0x24831c->0x24833c/ConditionalTrue 0x248330->0x248334/ConditionalFalse 0x248330->0x248440/ConditionalTrue 0x248334->0x24833c/ConditionalFalse 0x248334->0x248440/ConditionalTrue 0x24833c->0x24834c/Fallthrough 0x24834c->0x248368/ConditionalFalse 0x24834c->0x248374/ConditionalTrue 0x248368->0x24836c/ConditionalFalse 0x248368->0x24840c/ConditionalTrue 0x24836c->0x248374/ConditionalFalse 0x24836c->0x24840c/ConditionalTrue 0x248374->0x24838c/ConditionalFalse 0x248374->0x2483f8/ConditionalTrue 0x24838c->0x248390/ConditionalFalse 0x24838c->0x248398/ConditionalTrue 0x248390->0x248398/ConditionalFalse 0x248390->0x2483f8/ConditionalTrue 0x248398->0x2483b0/ConditionalFalse 0x248398->0x2483e4/ConditionalTrue 0x2483b0->0x24834c/Branch 0x2483e4->0x248440/Branch 0x2483f8->0x248424/Branch 0x24840c->0x248424/Fallthrough 0x248424->0x24831c/Branch 0x248440->0x248458/ConditionalFalse 0x248440->0x248464/ConditionalTrue 0x248458->0x248464/Fallthrough 0x248464->0x2484a0/ConditionalFalse 0x248464->0x2484b8/ConditionalTrue 0x2484a0->0x2484b8/Fallthrough 0x2484b8->0x2484c4/ConditionalFalse 0x2484b8->0x2484d0/ConditionalTrue 0x2484c4->0x2484e4/Branch 0x2484d0->0x2484e4/Fallthrough 0x2484e4->0x248528/ConditionalFalse 0x2484e4->0x248540/ConditionalTrue 0x248528->0x248540/Fallthrough 0x248540->0x24855c/ConditionalFalse 0x248540->0x248564/ConditionalTrue 0x24855c->0x248564/Fallthrough 0x248564->0x2485b4/ConditionalFalse 0x248564->0x2485cc/ConditionalTrue 0x2485b4->0x2485cc/Fallthrough 0x2485cc->0x248634/ConditionalFalse 0x2485cc->0x24864c/ConditionalTrue 0x248634->0x24864c/Fallthrough 0x24864c->0x24868c/ConditionalFalse 0x24864c->0x2486a4/ConditionalTrue 0x24868c->0x2486a4/Fallthrough 0x2486a4->0x2486f0/ConditionalFalse 0x2486a4->0x248708/ConditionalTrue 0x2486f0->0x248708/Fallthrough 0x248708->0x248724/ConditionalFalse 0x248708->0x248730/ConditionalTrue 0x248724->0x248730/Fallthrough 0x248730->0x24876c/ConditionalFalse 0x248730->0x248784/ConditionalTrue 0x24876c->0x248784/Fallthrough 0x248784->0x2487a0/ConditionalFalse 0x248784->0x2487a8/ConditionalTrue 0x2487a0->0x2487a8/Fallthrough 0x2487a8->0x2487fc/ConditionalFalse 0x2487a8->0x248808/ConditionalTrue 0x2487fc->0x248808/Fallthrough 0x248808->0x248844/ConditionalFalse 0x248808->0x24885c/ConditionalTrue 0x248844->0x24885c/Fallthrough 0x24885c->0x2488f8/ConditionalFalse 0x24885c->0x248910/ConditionalTrue 0x2488f8->0x248910/Fallthrough 0x248910->0x248998/ConditionalFalse 0x248910->0x2489b0/ConditionalTrue 0x248998->0x2489b0/Fallthrough 0x2489b0->0x248a38/ConditionalFalse 0x2489b0->0x248a50/ConditionalTrue 0x248a38->0x248a50/Fallthrough 0x248a50->0x248ab8/ConditionalFalse 0x248a50->0x248ad0/ConditionalTrue 0x248ab8->0x248ad0/Fallthrough 0x248ad0->0x248b10/ConditionalFalse 0x248ad0->0x248b28/ConditionalTrue 0x248b10->0x248b28/Fallthrough 0x248b28->0x248b40/ConditionalFalse 0x248b28->0x248b5c/ConditionalTrue 0x248b40->0x248b50/ConditionalFalse 0x248b40->0x248b5c/ConditionalTrue 0x248b50->0x248b54/ConditionalFalse 0x248b50->0x248b64/ConditionalTrue 0x248b54->0x248b5c/ConditionalFalse 0x248b54->0x248b64/ConditionalTrue 0x248b5c->0x248b68/Branch 0x248b64->0x248b68/Fallthrough 0x248b68->0x248bac/ConditionalFalse 0x248b68->0x248bc4/ConditionalTrue 0x248bac->0x248bc4/Fallthrough 0x248bc4->0x248c04/ConditionalFalse 0x248bc4->0x248c1c/ConditionalTrue 0x248c04->0x248c1c/Fallthrough 0x248c1c->0x248cac/ConditionalFalse 0x248c1c->0x248cc4/ConditionalTrue 0x248cac->0x248cc4/Fallthrough 0x248cc4->0x248d1c/ConditionalFalse 0x248cc4->0x248d24/ConditionalTrue 0x248d1c->0x248d24/Fallthrough 0x248d24->0x248d48/ConditionalFalse 0x248d24->0x248d54/ConditionalTrue 0x248d48->0x248d54/Fallthrough 0x248d54->0x248d90/ConditionalFalse 0x248d54->0x248da8/ConditionalTrue 0x248d90->0x248da8/Fallthrough 0x248da8->0x248df0/ConditionalFalse 0x248da8->0x248df8/ConditionalTrue 0x248df0->0x248dfc/Branch 0x248df8->0x248dfc/Fallthrough 0x248dfc->0x248e38/ConditionalFalse 0x248dfc->0x248e50/ConditionalTrue 0x248e38->0x248e50/Fallthrough

# top_level.e25Intrinsics at 0x248f24 (236 bytes)
0x248f24  00482de9           push     {fp, lr}
0x248f28  00b08de2           add      fp, sp, #0
0x248f2c  14d04de2           sub      sp, sp, #0x14
0x248f30  04100be5           str      r1, [fp, #-4]
0x248f34  24c09ae5           ldr      ip, [sl, #0x24]
0x248f38  0c005de1           cmp      sp, ip
0x248f3c  9e3f039b           blls     #0x318dbc
0x248f40  010985e2           add      r0, r5, #0x4000
0x248f44  cf0d90e5           ldr      r0, [r0, #0xdcf]  # pool[4978] = "ey"
0x248f48  03008de8           stm      sp, {r0, r1}
0x248f4c  852efceb           bl       #0x154968
0x248f50  0c000be5           str      r0, [fp, #-0xc]
0x248f54  073090e5           ldr      r3, [r0, #7]
0x248f58  08300be5           str      r3, [fp, #-8]
0x248f5c  0040a0e3           mov      r4, #0
0x248f60  040053e1           cmp      r3, r4
0x248f64  0200001a           bne      #0x248f74
0x248f68  4c009ae5           ldr      r0, [sl, #0x4c]
0x248f6c  00d04be2           sub      sp, fp, #0
0x248f70  0088bde8           pop      {fp, pc}
0x248f74  0010a0e1           mov      r1, r0
0x248f78  012985e2           add      r2, r5, #0x4000
0x248f7c  5f2d92e5           ldr      r2, [r2, #0xd5f]  # pool[4950] = snapshotRef(130)
0x248f80  9f4095e5           ldr      r4, [r5, #0x9f]  # pool[38] = snapshotRef(34599)
0x248f84  df42fceb           bl       #0x159b08
0x248f88  080010e3           tst      r0, #8
0x248f8c  0200000a           beq      #0x248f9c
0x248f90  4c009ae5           ldr      r0, [sl, #0x4c]
0x248f94  00d04be2           sub      sp, fp, #0
0x248f98  0088bde8           pop      {fp, pc}
0x248f9c  0c201be5           ldr      r2, [fp, #-0xc]
0x248fa0  08001be5           ldr      r0, [fp, #-8]
0x248fa4  0010a0e3           mov      r1, #0
0x248fa8  000051e1           cmp      r1, r0
0x248fac  1600002a           bhs      #0x24900c
0x248fb0  011012e5           ldr      r1, [r2, #-1]
0x248fb4  5116f3e7           ubfx     r1, r1, #0xc, #0x14
0x248fb8  8110a0e1           lsl      r1, r1, #1
0x248fbc  bc0051e3           cmp      r1, #0xbc
0x248fc0  0300001a           bne      #0x248fd4
0x248fc4  0b10d2e5           ldrb     r1, [r2, #0xb]
0x248fc8  6b0051e3           cmp      r1, #0x6b
0x248fcc  0b00001a           bne      #0x249000
0x248fd0  020000ea           b        #0x248fe0
0x248fd4  bb10d2e1           ldrh     r1, [r2, #0xb]
0x248fd8  6b0051e3           cmp      r1, #0x6b
0x248fdc  0700001a           bne      #0x249000
0x248fe0  04101be5           ldr      r1, [fp, #-4]
0x248fe4  012985e2           add      r2, r5, #0x4000
0x248fe8  cf2d92e5           ldr      r2, [r2, #0xdcf]  # pool[4978] = "ey"
0x248fec  020051e1           cmp      r1, r2
0x248ff0  48309a05           ldreq    r3, [sl, #0x48]
0x248ff4  4c309a15           ldrne    r3, [sl, #0x4c]
0x248ff8  0300a0e1           mov      r0, r3
0x248ffc  000000ea           b        #0x249004
0x249000  4c009ae5           ldr      r0, [sl, #0x4c]
0x249004  00d04be2           sub      sp, fp, #0
0x249008  0088bde8           pop      {fp, pc}
0x24900c  1e4003eb           bl       #0x31908c
# CFG: 0x248f24->0x248f68/ConditionalFalse 0x248f24->0x248f74/ConditionalTrue 0x248f74->0x248f90/ConditionalFalse 0x248f74->0x248f9c/ConditionalTrue 0x248f9c->0x248fb0/ConditionalFalse 0x248f9c->0x24900c/ConditionalTrue 0x248fb0->0x248fc4/ConditionalFalse 0x248fb0->0x248fd4/ConditionalTrue 0x248fc4->0x248fd0/ConditionalFalse 0x248fc4->0x249000/ConditionalTrue 0x248fd0->0x248fe0/Branch 0x248fd4->0x248fe0/ConditionalFalse 0x248fd4->0x249000/ConditionalTrue 0x248fe0->0x249004/Branch 0x249000->0x249004/Fallthrough

# top_level.e24Knot at 0x249010 (236 bytes)
0x249010  00482de9           push     {fp, lr}
0x249014  00b08de2           add      fp, sp, #0
0x249018  48409ae5           ldr      r4, [sl, #0x48]
0x24901c  040051e1           cmp      r1, r4
0x249020  0900001a           bne      #0x24904c
0x249024  040052e1           cmp      r2, r4
0x249028  0300001a           bne      #0x24903c
0x24902c  0100a0e3           mov      r0, #1
0x249030  0010a0e3           mov      r1, #0
0x249034  00d04be2           sub      sp, fp, #0
0x249038  0088bde8           pop      {fp, pc}
0x24903c  0200a0e3           mov      r0, #2
0x249040  0010a0e3           mov      r1, #0
0x249044  00d04be2           sub      sp, fp, #0
0x249048  0088bde8           pop      {fp, pc}
0x24904c  000058e3           cmp      r8, #0
0x249050  020000ba           blt      #0x249060
0x249054  140000ca           bgt      #0x2490ac
0x249058  000053e3           cmp      r3, #0
0x24905c  1200002a           bhs      #0x2490ac
0x249060  0340a0e1           mov      r4, r3
0x249064  0820a0e1           mov      r2, r8
0x249068  24c09ae5           ldr      ip, [sl, #0x24]
0x24906c  0c005de1           cmp      sp, ip
0x249070  513f039b           blls     #0x318dbc
0x249074  0460a0e1           mov      r6, r4
0x249078  019006e2           and      sb, r6, #1
0x24907c  000059e3           cmp      sb, #0
0x249080  0500000a           beq      #0x24909c
0x249084  001074e2           rsbs     r1, r4, #0
0x249088  0000c0e0           sbc      r0, r0, r0
0x24908c  020040e0           sub      r0, r0, r2
0x249090  0140a0e1           mov      r4, r1
0x249094  0020a0e1           mov      r2, r0
0x249098  f2ffffea           b        #0x249068
0x24909c  0300a0e3           mov      r0, #3
0x2490a0  0010a0e3           mov      r1, #0
0x2490a4  00d04be2           sub      sp, fp, #0
0x2490a8  0088bde8           pop      {fp, pc}
0x2490ac  0820a0e1           mov      r2, r8
0x2490b0  24c09ae5           ldr      ip, [sl, #0x24]
0x2490b4  0c005de1           cmp      sp, ip
0x2490b8  3f3f039b           blls     #0x318dbc
0x2490bc  821fa0e1           lsl      r1, r2, #0x1f
0x2490c0  a31081e1           orr      r1, r1, r3, lsr #1
0x2490c4  c200a0e1           asr      r0, r2, #1
0x2490c8  000050e3           cmp      r0, #0
0x2490cc  020000ca           bgt      #0x2490dc
0x2490d0  040000ba           blt      #0x2490e8
0x2490d4  040051e3           cmp      r1, #4
0x2490d8  0200009a           bls      #0x2490e8
0x2490dc  0130a0e1           mov      r3, r1
0x2490e0  0020a0e1           mov      r2, r0
0x2490e4  f1ffffea           b        #0x2490b0
0x2490e8  00c0a0e1           mov      ip, r0
0x2490ec  0100a0e1           mov      r0, r1
0x2490f0  0c10a0e1           mov      r1, ip
0x2490f4  00d04be2           sub      sp, fp, #0
0x2490f8  0088bde8           pop      {fp, pc}
# CFG: 0x249010->0x249024/ConditionalFalse 0x249010->0x24904c/ConditionalTrue 0x249024->0x24902c/ConditionalFalse 0x249024->0x24903c/ConditionalTrue 0x24904c->0x249054/ConditionalFalse 0x24904c->0x249060/ConditionalTrue 0x249054->0x249058/ConditionalFalse 0x249054->0x2490ac/ConditionalTrue 0x249058->0x249060/ConditionalFalse 0x249058->0x2490ac/ConditionalTrue 0x249060->0x249068/Fallthrough 0x249068->0x249084/ConditionalFalse 0x249068->0x24909c/ConditionalTrue 0x249084->0x249068/Branch 0x2490ac->0x2490b0/Fallthrough 0x2490b0->0x2490d0/ConditionalFalse 0x2490b0->0x2490dc/ConditionalTrue 0x2490d0->0x2490d4/ConditionalFalse 0x2490d0->0x2490e8/ConditionalTrue 0x2490d4->0x2490dc/ConditionalFalse 0x2490d4->0x2490e8/ConditionalTrue 0x2490dc->0x2490b0/Branch

# top_level.e23DynamicApply at 0x2490fc (80 bytes)
0x2490fc  00482de9           push     {fp, lr}
0x249100  00b08de2           add      fp, sp, #0
0x249104  04d04de2           sub      sp, sp, #4
0x249108  24c09ae5           ldr      ip, [sl, #0x24]
0x24910c  0c005de1           cmp      sp, ip
0x249110  293f039b           blls     #0x318dbc
0x249114  0c0000eb           bl       #0x24914c
0x249118  010010e3           tst      r0, #1
0x24911c  01101015           ldrne    r1, [r0, #-1]
0x249120  5116f317           ubfxne   r1, r1, #0xc, #0x14
0x249124  3c10a003           moveq    r1, #0x3c
0x249128  00008de5           str      r0, [sp]
0x24912c  0100a0e1           mov      r0, r1
0x249130  434195e5           ldr      r4, [r5, #0x143]  # pool[79] = snapshotRef(22)
0x249134  00e187e0           add      lr, r7, r0, lsl #2
0x249138  05ea8ee2           add      lr, lr, #0x5000
0x24913c  28e79ee5           ldr      lr, [lr, #0x728]
0x249140  3eff2fe1           blx      lr
0x249144  00d04be2           sub      sp, fp, #0
0x249148  0088bde8           pop      {fp, pc}

# E21Mode.parse at 0x2494f0 (52 bytes)
0x2494f0  00482de9           push     {fp, lr}
0x2494f4  00b08de2           add      fp, sp, #0
0x2494f8  08d04de2           sub      sp, sp, #8
0x2494fc  24c09ae5           ldr      ip, [sl, #0x24]
0x249500  0c005de1           cmp      sp, ip
0x249504  2c3e039b           blls     #0x318dbc
0x249508  01e985e2           add      lr, r5, #0x4000
0x24950c  ebed9ee5           ldr      lr, [lr, #0xdeb]  # pool[4985] = snapshotRef(18131)
0x249510  02408de8           stm      sp, {r1, lr}
0x249514  0b4495e5           ldr      r4, [r5, #0x40b]  # pool[257] = snapshotRef(54)
0x249518  010000eb           bl       #0x249524
0x24951c  00d04be2           sub      sp, fp, #0
0x249520  0088bde8           pop      {fp, pc}

# E20Combo.greet at 0x2495ec (12 bytes)
0x2495ec  010985e2           add      r0, r5, #0x4000
0x2495f0  fb0d90e5           ldr      r0, [r0, #0xdfb]  # pool[4989] = "base+combo"
0x2495f4  1eff2fe1           bx       lr

# package:edge_probe/probe_code.dart.E20Combo at 0x2495f8 (12 bytes)
0x2495f8  1c2100e3           movw     r2, #0x11c
0x2495fc  492040e3           movt     r2, #0x49
0x249600  ea3803ea           b        #0x3179b0

# top_level.e18NumericEdges at 0x249604 (560 bytes)
0x249604  00482de9           push     {fp, lr}
0x249608  00b08de2           add      fp, sp, #0
0x24960c  14d04de2           sub      sp, sp, #0x14
0x249610  508120f2           vorr     q4, q0, q0
0x249614  040b0bed           vstr     d0, [fp, #-0x10]
0x249618  24c09ae5           ldr      ip, [sl, #0x24]
0x24961c  0c005de1           cmp      sp, ip
0x249620  f33d039b           blls     #0x318df4
0x249624  488bb4ee           vcmpd    d8, d8
0x249628  10faf1ee           vmrs     apsr_nzcv, fpscr
0x24962c  0300007a           bvc      #0x249640
0x249630  010985e2           add      r0, r5, #0x4000
0x249634  ff0d90e5           ldr      r0, [r0, #0xdff]  # pool[4990] = "nan"
0x249638  00d04be2           sub      sp, fp, #0
0x24963c  0088bde8           pop      {fp, pc}
0x249640  580128f2           vorr     q0, q4, q4
0x249644  00482de9           push     {fp, lr}
0x249648  00b08de2           add      fp, sp, #0
0x24964c  07d0cde3           bic      sp, sp, #7
0x249650  100b51ec           vmov     r0, r1, d0
0x249654  112b53ec           vmov     r2, r3, d1
0x249658  80c39ae5           ldr      ip, [sl, #0x380]
0x24965c  30c38ae5           str      ip, [sl, #0x330]
0x249660  3cff2fe1           blx      ip
0x249664  08c0a0e3           mov      ip, #8
0x249668  30c38ae5           str      ip, [sl, #0x330]
0x24966c  100b41ec           vmov     d0, r0, r1
0x249670  112b43ec           vmov     d1, r2, r3
0x249674  00d04be2           sub      sp, fp, #0
0x249678  0048bde8           pop      {fp, lr}
0x24967c  408bb4ee           vcmpd    d8, d0
0x249680  10faf1ee           vmrs     apsr_nzcv, fpscr
0x249684  1500001a           bne      #0x2496e0
0x249688  40109ae5           ldr      r1, [sl, #0x40]
0x24968c  0420a0e3           mov      r2, #4
0x249690  853d03eb           bl       #0x318cac
0x249694  0b2080e2           add      r2, r0, #0xb
0x249698  01c985e2           add      ip, r5, #0x4000
0x24969c  03ce9ce5           ldr      ip, [ip, #0xe03]  # pool[4991] = "integral:"
0x2496a0  00c082e5           str      ip, [r2]
0x2496a4  042b1bed           vldr     d2, [fp, #-0x10]
0x2496a8  422bb4ee           vcmpd    d2, d2
0x2496ac  10faf1ee           vmrs     apsr_nzcv, fpscr
0x2496b0  4c00006a           bvs      #0x2497e8
0x2496b4  c2ebbdee           vcvt.fi  d14, d2
0x2496b8  101a1eee           vmov     r1, s28
0x2496bc  030151e3           cmp      r1, #0xc0000000
0x2496c0  4800004a           bmi      #0x2497e8
0x2496c4  8110a0e1           lsl      r1, r1, #1
0x2496c8  0f3080e2           add      r3, r0, #0xf
0x2496cc  001083e5           str      r1, [r3]
0x2496d0  00008de5           str      r0, [sp]
0x2496d4  db2bfceb           bl       #0x154648
0x2496d8  00d04be2           sub      sp, fp, #0
0x2496dc  0088bde8           pop      {fp, pc}
0x2496e0  582128f2           vorr     q1, q4, q4
0x2496e4  520122f2           vorr     q0, q1, q1
0x2496e8  40109ae5           ldr      r1, [sl, #0x40]
0x2496ec  f80100eb           bl       #0x249ed4
0x2496f0  40109ae5           ldr      r1, [sl, #0x40]
0x2496f4  0820a0e3           mov      r2, #8
0x2496f8  04000be5           str      r0, [fp, #-4]
0x2496fc  6a3d03eb           bl       #0x318cac
0x249700  08000be5           str      r0, [fp, #-8]
0x249704  0b2080e2           add      r2, r0, #0xb
0x249708  01c985e2           add      ip, r5, #0x4000
0x24970c  07ce9ce5           ldr      ip, [ip, #0xe07]  # pool[4992] = "frac:"
0x249710  00c082e5           str      ip, [r2]
0x249714  04101be5           ldr      r1, [fp, #-4]
0x249718  1020a0e3           mov      r2, #0x10
0x24971c  0030a0e3           mov      r3, #0
0x249720  910000eb           bl       #0x24996c
0x249724  08101be5           ldr      r1, [fp, #-8]
0x249728  0f9081e2           add      sb, r1, #0xf
0x24972c  000089e5           str      r0, [sb]
0x249730  010010e3           tst      r0, #1
0x249734  0500000a           beq      #0x249750
0x249738  01c051e5           ldrb     ip, [r1, #-1]
0x24973c  01e050e5           ldrb     lr, [r0, #-1]
0x249740  2cc10ee0           and      ip, lr, ip, lsr #2
0x249744  28e09ae5           ldr      lr, [sl, #0x28]
0x249748  0e001ce1           tst      ip, lr
0x24974c  5736031b           blne     #0x3170b0
0x249750  08001be5           ldr      r0, [fp, #-8]
0x249754  132080e2           add      r2, r0, #0x13
0x249758  01ca85e2           add      ip, r5, #0x1000
0x24975c  1fc29ce5           ldr      ip, [ip, #0x21f]  # pool[1158] = snapshotRef(244)
0x249760  00c082e5           str      ip, [r2]
0x249764  040b1bed           vldr     d0, [fp, #-0x10]
0x249768  2c109ae5           ldr      r1, [sl, #0x2c]
0x24976c  101081e2           add      r1, r1, #0x10
0x249770  30c09ae5           ldr      ip, [sl, #0x30]
0x249774  01005ce1           cmp      ip, r1
0x249778  2600009a           bls      #0x249818
0x24977c  2c108ae5           str      r1, [sl, #0x2c]
0x249780  0f1041e2           sub      r1, r1, #0xf
0x249784  9c220ee3           movw     r2, #0xe29c
0x249788  032040e3           movt     r2, #3
0x24978c  012001e5           str      r2, [r1, #-1]
0x249790  5af07ff5           dmb      ishst
0x249794  03c081e2           add      ip, r1, #3
0x249798  010b8ced           vstr     d0, [ip, #4]
0x24979c  0220a0e3           mov      r2, #2
0x2497a0  0030a0e3           mov      r3, #0
0x2497a4  220000eb           bl       #0x249834
0x2497a8  08101be5           ldr      r1, [fp, #-8]
0x2497ac  179081e2           add      sb, r1, #0x17
0x2497b0  000089e5           str      r0, [sb]
0x2497b4  010010e3           tst      r0, #1
0x2497b8  0500000a           beq      #0x2497d4
0x2497bc  01c051e5           ldrb     ip, [r1, #-1]
0x2497c0  01e050e5           ldrb     lr, [r0, #-1]
0x2497c4  2cc10ee0           and      ip, lr, ip, lsr #2
0x2497c8  28e09ae5           ldr      lr, [sl, #0x28]
0x2497cc  0e001ce1           tst      ip, lr
0x2497d0  3636031b           blne     #0x3170b0
0x2497d4  08e01be5           ldr      lr, [fp, #-8]
0x2497d8  00e08de5           str      lr, [sp]
0x2497dc  992bfceb           bl       #0x154648
0x2497e0  00d04be2           sub      sp, fp, #0
0x2497e4  0088bde8           pop      {fp, pc}
0x2497e8  042b2ded           vpush    {d2, d3}
0x2497ec  01002de9           stmdb    sp!, {r0}
0x2497f0  420bb0ee           vmov.f64 d0, d2
0x2497f4  4c00a0e3           mov      r0, #0x4c
0x2497f8  01ea85e2           add      lr, r5, #0x1000
0x2497fc  6be39ee5           ldr      lr, [lr, #0x36b]  # pool[1241] = snapshotRef(951)
0x249800  03e09ee5           ldr      lr, [lr, #3]
0x249804  3eff2fe1           blx      lr
0x249808  0010a0e1           mov      r1, r0
0x24980c  0100bde8           ldm      sp!, {r0}
0x249810  042bbdec           vpop     {d2, d3}
0x249814  abffffea           b        #0x2496c8
0x249818  040b2ded           vpush    {d0, d1}
0x24981c  01002de9           stmdb    sp!, {r0}
0x249820  f13c03eb           bl       #0x318bec
0x249824  0010a0e1           mov      r1, r0
0x249828  0100bde8           ldm      sp!, {r0}
0x24982c  040bbdec           vpop     {d0, d1}
0x249830  d7ffffea           b        #0x249794
# CFG: 0x249604->0x249630/ConditionalFalse 0x249604->0x249640/ConditionalTrue 0x249640->0x249688/ConditionalFalse 0x249640->0x2496e0/ConditionalTrue 0x249688->0x2496b4/ConditionalFalse 0x249688->0x2497e8/ConditionalTrue 0x2496b4->0x2496c4/ConditionalFalse 0x2496b4->0x2497e8/ConditionalTrue 0x2496c4->0x2496c8/Fallthrough 0x2496e0->0x249738/ConditionalFalse 0x2496e0->0x249750/ConditionalTrue 0x249738->0x249750/Fallthrough 0x249750->0x24977c/ConditionalFalse 0x249750->0x249818/ConditionalTrue 0x24977c->0x249794/Fallthrough 0x249794->0x2497bc/ConditionalFalse 0x249794->0x2497d4/ConditionalTrue 0x2497bc->0x2497d4/Fallthrough 0x2497e8->0x2496c8/Branch 0x249818->0x249794/Branch

# top_level.e17JsonRoundTrip at 0x24bc38 (240 bytes)
0x24bc38  00482de9           push     {fp, lr}
0x24bc3c  00b08de2           add      fp, sp, #0
0x24bc40  10d04de2           sub      sp, sp, #0x10
0x24bc44  24c09ae5           ldr      ip, [sl, #0x24]
0x24bc48  0c005de1           cmp      sp, ip
0x24bc4c  5a34039b           blls     #0x318dbc
0x24bc50  340000eb           bl       #0x24bd28
0x24bc54  0030a0e1           mov      r3, r0
0x24bc58  40209ae5           ldr      r2, [sl, #0x40]
0x24bc5c  40109ae5           ldr      r1, [sl, #0x40]
0x24bc60  04300be5           str      r3, [fp, #-4]
0x24bc64  010010e3           tst      r0, #1
0x24bc68  01401015           ldrne    r4, [r0, #-1]
0x24bc6c  5446f317           ubfxne   r4, r4, #0xc, #0x14
0x24bc70  3c40a003           moveq    r4, #0x3c
0x24bc74  5a4044e2           sub      r4, r4, #0x5a
0x24bc78  020054e3           cmp      r4, #2
0x24bc7c  0d00009a           bls      #0x24bcb8
0x24bc80  164044e2           sub      r4, r4, #0x16
0x24bc84  370054e3           cmp      r4, #0x37
0x24bc88  0a00009a           bls      #0x24bcb8
0x24bc8c  fec700e3           movw     ip, #0x7fe
0x24bc90  0c0054e1           cmp      r4, ip
0x24bc94  0700000a           beq      #0x24bcb8
0x24bc98  39c800e3           movw     ip, #0x839
0x24bc9c  0c0054e1           cmp      r4, ip
0x24bca0  0400000a           beq      #0x24bcb8
0x24bca4  038a85e2           add      r8, r5, #0x3000
0x24bca8  1b8198e5           ldr      r8, [r8, #0x11b]  # pool[3141] = snapshotRef(17279)
0x24bcac  013985e2           add      r3, r5, #0x4000
0x24bcb0  573e93e5           ldr      r3, [r3, #0xe57]  # pool[5012] = null
0x24bcb4  192c03eb           bl       #0x316d20
0x24bcb8  04001be5           ldr      r0, [fp, #-4]
0x24bcbc  011010e5           ldr      r1, [r0, #-1]
0x24bcc0  5116f3e7           ubfx     r1, r1, #0xc, #0x14
0x24bcc4  03ea85e2           add      lr, r5, #0x3000
0x24bcc8  c7e79ee5           ldr      lr, [lr, #0x7c7]  # pool[3568] = snapshotRef(18192)
0x24bccc  01408de8           stm      sp, {r0, lr}
0x24bcd0  0100a0e1           mov      r0, r1
0x24bcd4  0b4495e5           ldr      r4, [r5, #0x40b]  # pool[257] = snapshotRef(54)
0x24bcd8  00e187e0           add      lr, r7, r0, lsl #2
0x24bcdc  0fea8ee2           add      lr, lr, #0xf000
0x24bce0  a8ef9ee5           ldr      lr, [lr, #0xfa8]
0x24bce4  3eff2fe1           blx      lr
0x24bce8  011985e2           add      r1, r5, #0x4000
0x24bcec  5f1e91e5           ldr      r1, [r1, #0xe5f]  # pool[5014] = <anonymous closure>
0x24bcf0  40209ae5           ldr      r2, [sl, #0x40]
0x24bcf4  04000be5           str      r0, [fp, #-4]
0x24bcf8  3f3003eb           bl       #0x317dfc
0x24bcfc  01e985e2           add      lr, r5, #0x4000
0x24bd00  63ee9ee5           ldr      lr, [lr, #0xe63]  # pool[5015] = snapshotRef(18001)
0x24bd04  04901be5           ldr      sb, [fp, #-4]
0x24bd08  01428de8           stm      sp, {r0, sb, lr}
0x24bd0c  1f4095e5           ldr      r4, [r5, #0x1f]  # pool[6] = snapshotRef(55)
0x24bd10  a519ffeb           bl       #0x2123ac
0x24bd14  031090e5           ldr      r1, [r0, #3]
0x24bd18  0020a0e1           mov      r2, r0
0x24bd1c  4ef3fbeb           bl       #0x148a5c
0x24bd20  00d04be2           sub      sp, fp, #0
0x24bd24  0088bde8           pop      {fp, pc}
# CFG: 0x24bc38->0x24bc80/ConditionalFalse 0x24bc38->0x24bcb8/ConditionalTrue 0x24bc80->0x24bc8c/ConditionalFalse 0x24bc80->0x24bcb8/ConditionalTrue 0x24bc8c->0x24bc98/ConditionalFalse 0x24bc8c->0x24bcb8/ConditionalTrue 0x24bc98->0x24bca4/ConditionalFalse 0x24bc98->0x24bcb8/ConditionalTrue 0x24bca4->0x24bcb8/Fallthrough

# top_level.<anonymous closure> at 0x24bd64 (76 bytes)
0x24bd64  00482de9           push     {fp, lr}
0x24bd68  00b08de2           add      fp, sp, #0
0x24bd6c  08d04de2           sub      sp, sp, #8
0x24bd70  24c09ae5           ldr      ip, [sl, #0x24]
0x24bd74  0c005de1           cmp      sp, ip
0x24bd78  0f34039b           blls     #0x318dbc
0x24bd7c  08009be5           ldr      r0, [fp, #8]
0x24bd80  011010e5           ldr      r1, [r0, #-1]
0x24bd84  5116f3e7           ubfx     r1, r1, #0xc, #0x14
0x24bd88  03ea85e2           add      lr, r5, #0x3000
0x24bd8c  c3ec9ee5           ldr      lr, [lr, #0xcc3]  # pool[3887] = snapshotRef(17946)
0x24bd90  01408de8           stm      sp, {r0, lr}
0x24bd94  0100a0e1           mov      r0, r1
0x24bd98  ab4e95e5           ldr      r4, [r5, #0xeab]  # pool[937] = snapshotRef(34604)
0x24bd9c  00e187e0           add      lr, r7, r0, lsl #2
0x24bda0  f8eb9ee5           ldr      lr, [lr, #0xbf8]
0x24bda4  3eff2fe1           blx      lr
0x24bda8  00d04be2           sub      sp, fp, #0
0x24bdac  0088bde8           pop      {fp, pc}

# top_level.e16SortedCopy at 0x24bdb0 (96 bytes)
0x24bdb0  00482de9           push     {fp, lr}
0x24bdb4  00b08de2           add      fp, sp, #0
0x24bdb8  08d04de2           sub      sp, sp, #8
0x24bdbc  0120a0e1           mov      r2, r1
0x24bdc0  04100be5           str      r1, [fp, #-4]
0x24bdc4  24c09ae5           ldr      ip, [sl, #0x24]
0x24bdc8  0c005de1           cmp      sp, ip
0x24bdcc  fa33039b           blls     #0x318dbc
0x24bdd0  013985e2           add      r3, r5, #0x4000
0x24bdd4  6b3e93e5           ldr      r3, [r3, #0xe6b]  # pool[5017] = snapshotRef(18569)
0x24bdd8  1c0400eb           bl       #0x24ce50
0x24bddc  0010a0e1           mov      r1, r0
0x24bde0  40209ae5           ldr      r2, [sl, #0x40]
0x24bde4  08000be5           str      r0, [fp, #-8]
0x24bde8  9c0300eb           bl       #0x24cc60
0x24bdec  08101be5           ldr      r1, [fp, #-8]
0x24bdf0  04201be5           ldr      r2, [fp, #-4]
0x24bdf4  050000eb           bl       #0x24be10
0x24bdf8  08201be5           ldr      r2, [fp, #-8]
0x24bdfc  011985e2           add      r1, r5, #0x4000
0x24be00  6b1e91e5           ldr      r1, [r1, #0xe6b]  # pool[5017] = snapshotRef(18569)
0x24be04  86cbfceb           bl       #0x17ec24
0x24be08  00d04be2           sub      sp, fp, #0
0x24be0c  0088bde8           pop      {fp, pc}

# package:edge_probe/probe_code.dart.E15Vec at 0x24ce5c (12 bytes)
0x24ce5c  1c2304e3           movw     r2, #0x431c
0x24ce60  2f2040e3           movt     r2, #0x2f
0x24ce64  d12a03ea           b        #0x3179b0

# E14Statics.bump at 0x24ce68 (196 bytes)
0x24ce68  00482de9           push     {fp, lr}
0x24ce6c  00b08de2           add      fp, sp, #0
0x24ce70  08d04de2           sub      sp, sp, #8
0x24ce74  24c09ae5           ldr      ip, [sl, #0x24]
0x24ce78  0c005de1           cmp      sp, ip
0x24ce7c  ce2f039b           blls     #0x318dbc
0x24ce80  38009ae5           ldr      r0, [sl, #0x38]
0x24ce84  d40390e5           ldr      r0, [r0, #0x3d4]
0x24ce88  c01fa0e1           asr      r1, r0, #0x1f
0x24ce8c  c020b0e1           asrs     r2, r0, #1
0x24ce90  0100003a           blo      #0x24ce9c
0x24ce94  072090e5           ldr      r2, [r0, #7]
0x24ce98  0b1090e5           ldr      r1, [r0, #0xb]
0x24ce9c  014092e2           adds     r4, r2, #1
0x24cea0  0030b1e2           adcs     r3, r1, #0
0x24cea4  04400be5           str      r4, [fp, #-4]
0x24cea8  08300be5           str      r3, [fp, #-8]
0x24ceac  8400a0e1           lsl      r0, r4, #1
0x24ceb0  c00054e1           cmp      r4, r0, asr #1
0x24ceb4  c00f5301           cmpeq    r3, r0, asr #31
0x24ceb8  0200000a           beq      #0x24cec8
0x24cebc  ea2f03eb           bl       #0x318e6c
0x24cec0  074080e5           str      r4, [r0, #7]
0x24cec4  0b3080e5           str      r3, [r0, #0xb]
0x24cec8  0020a0e1           mov      r2, r0
0x24cecc  38009ae5           ldr      r0, [sl, #0x38]
0x24ced0  d42380e5           str      r2, [r0, #0x3d4]
0x24ced4  38009ae5           ldr      r0, [sl, #0x38]
0x24ced8  d00390e5           ldr      r0, [r0, #0x3d0]
0x24cedc  44c09ae5           ldr      ip, [sl, #0x44]
0x24cee0  0c0050e1           cmp      r0, ip
0x24cee4  0200001a           bne      #0x24cef4
0x24cee8  012985e2           add      r2, r5, #0x4000
0x24ceec  472f92e5           ldr      r2, [r2, #0xf47]  # pool[5072] = E14Statics.stamp
0x24cef0  112803eb           bl       #0x316f3c
0x24cef4  c020b0e1           asrs     r2, r0, #1
0x24cef8  0000003a           blo      #0x24cf00
0x24cefc  072090e5           ldr      r2, [r0, #7]
0x24cf00  013002e2           and      r3, r2, #1
0x24cf04  0340a0e1           mov      r4, r3
0x24cf08  022022e0           eor      r2, r2, r2
0x24cf0c  04601be5           ldr      r6, [fp, #-4]
0x24cf10  08301be5           ldr      r3, [fp, #-8]
0x24cf14  049096e0           adds     sb, r6, r4
0x24cf18  0280b3e0           adcs     r8, r3, r2
0x24cf1c  030099e2           adds     r0, sb, #3
0x24cf20  0010b8e2           adcs     r1, r8, #0
0x24cf24  00d04be2           sub      sp, fp, #0
0x24cf28  0088bde8           pop      {fp, pc}
# CFG: 0x24ce68->0x24ce94/ConditionalFalse 0x24ce68->0x24ce9c/ConditionalTrue 0x24ce94->0x24ce9c/Fallthrough 0x24ce9c->0x24cebc/ConditionalFalse 0x24ce9c->0x24cec8/ConditionalTrue 0x24cebc->0x24cec8/Fallthrough 0x24cec8->0x24cee8/ConditionalFalse 0x24cec8->0x24cef4/ConditionalTrue 0x24cee8->0x24cef4/Fallthrough 0x24cef4->0x24cefc/ConditionalFalse 0x24cef4->0x24cf00/ConditionalTrue 0x24cefc->0x24cf00/Fallthrough

# E14Statics.init:stamp at 0x24cf2c (32 bytes)
0x24cf2c  00482de9           push     {fp, lr}
0x24cf30  00b08de2           add      fp, sp, #0
0x24cf34  24c09ae5           ldr      ip, [sl, #0x24]
0x24cf38  0c005de1           cmp      sp, ip
0x24cf3c  9e2f039b           blls     #0x318dbc
0x24cf40  1a0000eb           bl       #0x24cfb0
0x24cf44  00d04be2           sub      sp, fp, #0
0x24cf48  0088bde8           pop      {fp, pc}

# E13Dynamic.probe at 0x24d030 (60 bytes)
0x24d030  00482de9           push     {fp, lr}
0x24d034  00b08de2           add      fp, sp, #0
0x24d038  04d04de2           sub      sp, sp, #4
0x24d03c  24c09ae5           ldr      ip, [sl, #0x24]
0x24d040  0c005de1           cmp      sp, ip
0x24d044  5c2f039b           blls     #0x318dbc
0x24d048  00208de5           str      r2, [sp]
0x24d04c  00009de5           ldr      r0, [sp]
0x24d050  01e985e2           add      lr, r5, #0x4000
0x24d054  5bef9ee5           ldr      lr, [lr, #0xf5b]  # pool[5077] = resetPoolEntry(5077)
0x24d058  019985e2           add      sb, r5, #0x4000
0x24d05c  5f9f99e5           ldr      sb, [sb, #0xf5f]  # pool[5078] = dynamicCall("dyn:get:unknownMember")
0x24d060  3eff2fe1           blx      lr
0x24d064  00d04be2           sub      sp, fp, #0
0x24d068  0088bde8           pop      {fp, pc}

# package:edge_probe/probe_code.dart.E13Dynamic at 0x24d06c (12 bytes)
0x24d06c  1c2106e3           movw     r2, #0x611c
0x24d070  2f2040e3           movt     r2, #0x2f
0x24d074  4d2a03ea           b        #0x3179b0

# top_level.e12TearOffs at 0x24d078 (308 bytes)
0x24d078  00482de9           push     {fp, lr}
0x24d07c  00b08de2           add      fp, sp, #0
0x24d080  0cd04de2           sub      sp, sp, #0xc
0x24d084  0400a0e3           mov      r0, #4
0x24d088  24c09ae5           ldr      ip, [sl, #0x24]
0x24d08c  0c005de1           cmp      sp, ip
0x24d090  492f039b           blls     #0x318dbc
0x24d094  0020a0e1           mov      r2, r0
0x24d098  40109ae5           ldr      r1, [sl, #0x40]
0x24d09c  022f03eb           bl       #0x318cac
0x24d0a0  04000be5           str      r0, [fp, #-4]
0x24d0a4  0b2080e2           add      r2, r0, #0xb
0x24d0a8  06c0a0e3           mov      ip, #6
0x24d0ac  00c082e5           str      ip, [r2]
0x24d0b0  0f2080e2           add      r2, r0, #0xf
0x24d0b4  02c0a0e3           mov      ip, #2
0x24d0b8  00c082e5           str      ip, [r2]
0x24d0bc  073495e5           ldr      r3, [r5, #0x407]  # pool[256] = snapshotRef(18555)
0x24d0c0  4e2a03eb           bl       #0x317a00
0x24d0c4  0020a0e1           mov      r2, r0
0x24d0c8  04001be5           ldr      r0, [fp, #-4]
0x24d0cc  08200be5           str      r2, [fp, #-8]
0x24d0d0  0b0082e5           str      r0, [r2, #0xb]
0x24d0d4  0400a0e3           mov      r0, #4
0x24d0d8  070082e5           str      r0, [r2, #7]
0x24d0dc  0210a0e1           mov      r1, r2
0x24d0e0  c7d2ffeb           bl       #0x241c04
0x24d0e4  40109ae5           ldr      r1, [sl, #0x40]
0x24d0e8  0e20a0e3           mov      r2, #0xe
0x24d0ec  04000be5           str      r0, [fp, #-4]
0x24d0f0  ed2e03eb           bl       #0x318cac
0x24d0f4  0c000be5           str      r0, [fp, #-0xc]
0x24d0f8  0b2080e2           add      r2, r0, #0xb
0x24d0fc  04c0a0e3           mov      ip, #4
0x24d100  00c082e5           str      ip, [r2]
0x24d104  04101be5           ldr      r1, [fp, #-4]
0x24d108  0f3080e2           add      r3, r0, #0xf
0x24d10c  001083e5           str      r1, [r3]
0x24d110  08201be5           ldr      r2, [fp, #-8]
0x24d114  011985e2           add      r1, r5, #0x4000
0x24d118  631f91e5           ldr      r1, [r1, #0xf63]  # pool[5079] = ListBase.sort
0x24d11c  073495e5           ldr      r3, [r5, #0x407]  # pool[256] = snapshotRef(18555)
0x24d120  be2a03eb           bl       #0x317c20
0x24d124  0010a0e1           mov      r1, r0
0x24d128  0c001be5           ldr      r0, [fp, #-0xc]
0x24d12c  133080e2           add      r3, r0, #0x13
0x24d130  001083e5           str      r1, [r3]
0x24d134  08201be5           ldr      r2, [fp, #-8]
0x24d138  011985e2           add      r1, r5, #0x4000
0x24d13c  671f91e5           ldr      r1, [r1, #0xf67]  # pool[5080] = _GrowableList.removeLast
0x24d140  073495e5           ldr      r3, [r5, #0x407]  # pool[256] = snapshotRef(18555)
0x24d144  b52a03eb           bl       #0x317c20
0x24d148  0010a0e1           mov      r1, r0
0x24d14c  0c001be5           ldr      r0, [fp, #-0xc]
0x24d150  173080e2           add      r3, r0, #0x17
0x24d154  001083e5           str      r1, [r3]
0x24d158  1b2080e2           add      r2, r0, #0x1b
0x24d15c  01c985e2           add      ip, r5, #0x4000
0x24d160  6bcf9ce5           ldr      ip, [ip, #0xf6b]  # pool[5081] = snapshotRef(33623)
0x24d164  00c082e5           str      ip, [r2]
0x24d168  1f2080e2           add      r2, r0, #0x1f
0x24d16c  01c985e2           add      ip, r5, #0x4000
0x24d170  6fcf9ce5           ldr      ip, [ip, #0xf6f]  # pool[5082] = snapshotRef(33625)
0x24d174  00c082e5           str      ip, [r2]
0x24d178  232080e2           add      r2, r0, #0x23
0x24d17c  01c985e2           add      ip, r5, #0x4000
0x24d180  73cf9ce5           ldr      ip, [ip, #0xf73]  # pool[5083] = snapshotRef(33624)
0x24d184  00c082e5           str      ip, [r2]
0x24d188  013a85e2           add      r3, r5, #0x1000
0x24d18c  373393e5           ldr      r3, [r3, #0x337]  # pool[1228] = snapshotRef(18149)
0x24d190  1a2a03eb           bl       #0x317a00
0x24d194  0c101be5           ldr      r1, [fp, #-0xc]
0x24d198  0b1080e5           str      r1, [r0, #0xb]
0x24d19c  0e10a0e3           mov      r1, #0xe
0x24d1a0  071080e5           str      r1, [r0, #7]
0x24d1a4  00d04be2           sub      sp, fp, #0
0x24d1a8  0088bde8           pop      {fp, pc}

# top_level.e11SyncGen at 0x24d81c (324 bytes)
0x24d81c  00482de9           push     {fp, lr}
0x24d820  00b08de2           add      fp, sp, #0
0x24d824  18d04de2           sub      sp, sp, #0x18
0x24d828  40009ae5           ldr      r0, [sl, #0x40]
0x24d82c  04000be5           str      r0, [fp, #-4]
0x24d830  08200be5           str      r2, [fp, #-8]
0x24d834  0c100be5           str      r1, [fp, #-0xc]
0x24d838  24c09ae5           ldr      ip, [sl, #0x24]
0x24d83c  0c005de1           cmp      sp, ip
0x24d840  5d2d039b           blls     #0x318dbc
0x24d844  070495e5           ldr      r0, [r5, #0x407]  # pool[256] = snapshotRef(18555)
0x24d848  2b5ffeeb           bl       #0x1e54fc
0x24d84c  40009ae5           ldr      r0, [sl, #0x40]
0x24d850  be5efeeb           bl       #0x1e5350
0x24d854  0080a0e3           mov      r8, #0
0x24d858  0060a0e3           mov      r6, #0
0x24d85c  0c201be5           ldr      r2, [fp, #-0xc]
0x24d860  08301be5           ldr      r3, [fp, #-8]
0x24d864  0040a0e3           mov      r4, #0
0x24d868  10800be5           str      r8, [fp, #-0x10]
0x24d86c  14600be5           str      r6, [fp, #-0x14]
0x24d870  24c09ae5           ldr      ip, [sl, #0x24]
0x24d874  0c005de1           cmp      sp, ip
0x24d878  4f2d039b           blls     #0x318dbc
0x24d87c  030056e1           cmp      r6, r3
0x24d880  020000ba           blt      #0x24d890
0x24d884  320000ca           bgt      #0x24d954
0x24d888  020058e1           cmp      r8, r2
0x24d88c  3000002a           bhs      #0x24d954
0x24d890  0800a0e1           mov      r0, r8
0x24d894  011000e2           and      r1, r0, #1
0x24d898  000051e3           cmp      r1, #0
0x24d89c  1500001a           bne      #0x24d8f8
0x24d8a0  84008be0           add      r0, fp, r4, lsl #1
0x24d8a4  040010e5           ldr      r0, [r0, #-4]
0x24d8a8  0b9090e5           ldr      sb, [r0, #0xb]
0x24d8ac  8800a0e1           lsl      r0, r8, #1
0x24d8b0  c00058e1           cmp      r8, r0, asr #1
0x24d8b4  c00f5601           cmpeq    r6, r0, asr #31
0x24d8b8  0200000a           beq      #0x24d8c8
0x24d8bc  6a2d03eb           bl       #0x318e6c
0x24d8c0  078080e5           str      r8, [r0, #7]
0x24d8c4  0b6080e5           str      r6, [r0, #0xb]
0x24d8c8  130089e5           str      r0, [sb, #0x13]
0x24d8cc  010010e3           tst      r0, #1
0x24d8d0  0500000a           beq      #0x24d8ec
0x24d8d4  01c059e5           ldrb     ip, [sb, #-1]
0x24d8d8  01e050e5           ldrb     lr, [r0, #-1]
0x24d8dc  2cc10ee0           and      ip, lr, ip, lsr #2
0x24d8e0  28e09ae5           ldr      lr, [sl, #0x28]
0x24d8e4  0e001ce1           tst      ip, lr
0x24d8e8  b026031b           blne     #0x3173b0
0x24d8ec  48009ae5           ldr      r0, [sl, #0x48]
0x24d8f0  1a0000eb           bl       #0x24d960
0x24d8f4  110000ea           b        #0x24d940
0x24d8f8  0400a0e1           mov      r0, r4
0x24d8fc  80108be0           add      r1, fp, r0, lsl #1
0x24d900  041011e5           ldr      r1, [r1, #-4]
0x24d904  0b3091e5           ldr      r3, [r1, #0xb]
0x24d908  10101be5           ldr      r1, [fp, #-0x10]
0x24d90c  14201be5           ldr      r2, [fp, #-0x14]
0x24d910  18300be5           str      r3, [fp, #-0x18]
0x24d914  c0ffffeb           bl       #0x24d81c
0x24d918  18101be5           ldr      r1, [fp, #-0x18]
0x24d91c  170081e5           str      r0, [r1, #0x17]
0x24d920  01c051e5           ldrb     ip, [r1, #-1]
0x24d924  01e050e5           ldrb     lr, [r0, #-1]
0x24d928  2cc10ee0           and      ip, lr, ip, lsr #2
0x24d92c  28e09ae5           ldr      lr, [sl, #0x28]
0x24d930  0e001ce1           tst      ip, lr
0x24d934  7926031b           blne     #0x317320
0x24d938  48009ae5           ldr      r0, [sl, #0x48]
0x24d93c  070000eb           bl       #0x24d960
0x24d940  10201be5           ldr      r2, [fp, #-0x10]
0x24d944  14101be5           ldr      r1, [fp, #-0x14]
0x24d948  018092e2           adds     r8, r2, #1
0x24d94c  0060b1e2           adcs     r6, r1, #0
0x24d950  c1ffffea           b        #0x24d85c
0x24d954  4c009ae5           ldr      r0, [sl, #0x4c]
0x24d958  00d04be2           sub      sp, fp, #0
0x24d95c  0088bde8           pop      {fp, pc}
# CFG: 0x24d81c->0x24d85c/Fallthrough 0x24d85c->0x24d884/ConditionalFalse 0x24d85c->0x24d890/ConditionalTrue 0x24d884->0x24d888/ConditionalFalse 0x24d884->0x24d954/ConditionalTrue 0x24d888->0x24d890/ConditionalFalse 0x24d888->0x24d954/ConditionalTrue 0x24d890->0x24d8a0/ConditionalFalse 0x24d890->0x24d8f8/ConditionalTrue 0x24d8a0->0x24d8bc/ConditionalFalse 0x24d8a0->0x24d8c8/ConditionalTrue 0x24d8bc->0x24d8c8/Fallthrough 0x24d8c8->0x24d8d4/ConditionalFalse 0x24d8c8->0x24d8ec/ConditionalTrue 0x24d8d4->0x24d8ec/Fallthrough 0x24d8ec->0x24d940/Branch 0x24d8f8->0x24d940/Fallthrough 0x24d940->0x24d85c/Branch

# top_level.e10AsyncLoop at 0x24db14 (352 bytes)
0x24db14  00482de9           push     {fp, lr}
0x24db18  00b08de2           add      fp, sp, #0
0x24db1c  24d04de2           sub      sp, sp, #0x24
0x24db20  40009ae5           ldr      r0, [sl, #0x40]
0x24db24  04000be5           str      r0, [fp, #-4]
0x24db28  08200be5           str      r2, [fp, #-8]
0x24db2c  0c100be5           str      r1, [fp, #-0xc]
0x24db30  24c09ae5           ldr      ip, [sl, #0x24]
0x24db34  0c005de1           cmp      sp, ip
0x24db38  9f2c039b           blls     #0x318dbc
0x24db3c  070495e5           ldr      r0, [r5, #0x407]  # pool[256] = snapshotRef(18555)
0x24db40  3a9afceb           bl       #0x174430
0x24db44  0060a0e3           mov      r6, #0
0x24db48  0040a0e3           mov      r4, #0
0x24db4c  0020a0e3           mov      r2, #0
0x24db50  0c001be5           ldr      r0, [fp, #-0xc]
0x24db54  08101be5           ldr      r1, [fp, #-8]
0x24db58  10600be5           str      r6, [fp, #-0x10]
0x24db5c  14400be5           str      r4, [fp, #-0x14]
0x24db60  18200be5           str      r2, [fp, #-0x18]
0x24db64  24c09ae5           ldr      ip, [sl, #0x24]
0x24db68  0c005de1           cmp      sp, ip
0x24db6c  922c039b           blls     #0x318dbc
0x24db70  010052e1           cmp      r2, r1
0x24db74  020000ba           blt      #0x24db84
0x24db78  350000ca           bgt      #0x24dc54
0x24db7c  000054e1           cmp      r4, r0
0x24db80  3300002a           bhs      #0x24dc54
0x24db84  073495e5           ldr      r3, [r5, #0x407]  # pool[256] = snapshotRef(18555)
0x24db88  0b9afceb           bl       #0x1743bc
0x24db8c  0020a0e1           mov      r2, r0
0x24db90  0000a0e3           mov      r0, #0
0x24db94  0010a0e3           mov      r1, #0
0x24db98  1c200be5           str      r2, [fp, #-0x1c]
0x24db9c  070082e5           str      r0, [r2, #7]
0x24dba0  0b1082e5           str      r1, [r2, #0xb]
0x24dba4  38009ae5           ldr      r0, [sl, #0x38]
0x24dba8  540390e5           ldr      r0, [r0, #0x354]
0x24dbac  44c09ae5           ldr      ip, [sl, #0x44]
0x24dbb0  0c0050e1           cmp      r0, ip
0x24dbb4  0100001a           bne      #0x24dbc0
0x24dbb8  772095e5           ldr      r2, [r5, #0x77]  # pool[28] = Zone._current
0x24dbbc  f82403eb           bl       #0x316fa4
0x24dbc0  1c301be5           ldr      r3, [fp, #-0x1c]
0x24dbc4  0f0083e5           str      r0, [r3, #0xf]
0x24dbc8  14601be5           ldr      r6, [fp, #-0x14]
0x24dbcc  18401be5           ldr      r4, [fp, #-0x18]
0x24dbd0  8600a0e1           lsl      r0, r6, #1
0x24dbd4  c00056e1           cmp      r6, r0, asr #1
0x24dbd8  c00f5401           cmpeq    r4, r0, asr #31
0x24dbdc  0200000a           beq      #0x24dbec
0x24dbe0  a12c03eb           bl       #0x318e6c
0x24dbe4  076080e5           str      r6, [r0, #7]
0x24dbe8  0b4080e5           str      r4, [r0, #0xb]
0x24dbec  0310a0e1           mov      r1, r3
0x24dbf0  0020a0e1           mov      r2, r0
0x24dbf4  ed89fceb           bl       #0x1703b0
0x24dbf8  1c001be5           ldr      r0, [fp, #-0x1c]
0x24dbfc  7199fceb           bl       #0x1741c8
0x24dc00  10e01be5           ldr      lr, [fp, #-0x10]
0x24dc04  01408de8           stm      sp, {r0, lr}
0x24dc08  7f2303eb           bl       #0x316a0c
0x24dc0c  c01fa0e1           asr      r1, r0, #0x1f
0x24dc10  c020b0e1           asrs     r2, r0, #1
0x24dc14  0100003a           blo      #0x24dc20
0x24dc18  072090e5           ldr      r2, [r0, #7]
0x24dc1c  0b1090e5           ldr      r1, [r0, #0xb]
0x24dc20  000051e3           cmp      r1, #0
0x24dc24  090000ca           bgt      #0x24dc50
0x24dc28  010000ba           blt      #0x24dc34
0x24dc2c  640052e3           cmp      r2, #0x64
0x24dc30  0600008a           bhi      #0x24dc50
0x24dc34  14201be5           ldr      r2, [fp, #-0x14]
0x24dc38  18101be5           ldr      r1, [fp, #-0x18]
0x24dc3c  014092e2           adds     r4, r2, #1
0x24dc40  0030b1e2           adcs     r3, r1, #0
0x24dc44  0060a0e1           mov      r6, r0
0x24dc48  0320a0e1           mov      r2, r3
0x24dc4c  bfffffea           b        #0x24db50
0x24dc50  c398fcea           b        #0x173f64
0x24dc54  671195e5           ldr      r1, [r5, #0x167]  # pool[88] = snapshotRef(18524)
0x24dc58  672f95e5           ldr      r2, [r5, #0xf67]  # pool[984] = snapshotInstance(Duration)
0x24dc5c  22c7fdeb           bl       #0x1bf8ec
0x24dc60  0010a0e1           mov      r1, r0
0x24dc64  1c100be5           str      r1, [fp, #-0x1c]
0x24dc68  5699fceb           bl       #0x1741c8
0x24dc6c  10001be5           ldr      r0, [fp, #-0x10]
0x24dc70  bb98fcea           b        #0x173f64
# CFG: 0x24db14->0x24db50/Fallthrough 0x24db50->0x24db78/ConditionalFalse 0x24db50->0x24db84/ConditionalTrue 0x24db78->0x24db7c/ConditionalFalse 0x24db78->0x24dc54/ConditionalTrue 0x24db7c->0x24db84/ConditionalFalse 0x24db7c->0x24dc54/ConditionalTrue 0x24db84->0x24dbb8/ConditionalFalse 0x24db84->0x24dbc0/ConditionalTrue 0x24dbb8->0x24dbc0/Fallthrough 0x24dbc0->0x24dbe0/ConditionalFalse 0x24dbc0->0x24dbec/ConditionalTrue 0x24dbe0->0x24dbec/Fallthrough 0x24dbec->0x24dc18/ConditionalFalse 0x24dbec->0x24dc20/ConditionalTrue 0x24dc18->0x24dc20/Fallthrough 0x24dc20->0x24dc28/ConditionalFalse 0x24dc20->0x24dc50/ConditionalTrue 0x24dc28->0x24dc2c/ConditionalFalse 0x24dc28->0x24dc34/ConditionalTrue 0x24dc2c->0x24dc34/ConditionalFalse 0x24dc2c->0x24dc50/ConditionalTrue 0x24dc34->0x24db50/Branch

# top_level.e09TryRethrow at 0x24dc74 (464 bytes)
0x24dc74  00482de9           push     {fp, lr}
0x24dc78  00b08de2           add      fp, sp, #0
0x24dc7c  4cd04de2           sub      sp, sp, #0x4c
0x24dc80  38100be5           str      r1, [fp, #-0x38]
0x24dc84  24c09ae5           ldr      ip, [sl, #0x24]
0x24dc88  0c005de1           cmp      sp, ip
0x24dc8c  4a2c039b           blls     #0x318dbc
0x24dc90  d7e095e5           ldr      lr, [r5, #0xd7]  # pool[52] = snapshotRef(101)
0x24dc94  019985e2           add      sb, r5, #0x4000
0x24dc98  879d99e5           ldr      sb, [sb, #0xd87]  # pool[4960] = snapshotRef(393)
0x24dc9c  00428de8           stm      sp, {sb, lr}
0x24dca0  301bfceb           bl       #0x154968
0x24dca4  0030a0e1           mov      r3, r0
0x24dca8  38001be5           ldr      r0, [fp, #-0x38]
0x24dcac  3c300be5           str      r3, [fp, #-0x3c]
0x24dcb0  010010e3           tst      r0, #1
0x24dcb4  01101015           ldrne    r1, [r0, #-1]
0x24dcb8  5116f317           ubfxne   r1, r1, #0xc, #0x14
0x24dcbc  3c10a003           moveq    r1, #0x3c
0x24dcc0  3c2041e2           sub      r2, r1, #0x3c
0x24dcc4  010052e3           cmp      r2, #1
0x24dcc8  3d00008a           bhi      #0x24ddc4
0x24dccc  40109ae5           ldr      r1, [sl, #0x40]
0x24dcd0  0420a0e3           mov      r2, #4
0x24dcd4  f42b03eb           bl       #0x318cac
0x24dcd8  0b2080e2           add      r2, r0, #0xb
0x24dcdc  01c985e2           add      ip, r5, #0x4000
0x24dce0  bfcf9ce5           ldr      ip, [ip, #0xfbf]  # pool[5102] = "ok:"
0x24dce4  00c082e5           str      ip, [r2]
0x24dce8  38101be5           ldr      r1, [fp, #-0x38]
0x24dcec  0f3080e2           add      r3, r0, #0xf
0x24dcf0  001083e5           str      r1, [r3]
0x24dcf4  00008de5           str      r0, [sp]
0x24dcf8  521afceb           bl       #0x154648
0x24dcfc  38000be5           str      r0, [fp, #-0x38]
0x24dd00  3ce01be5           ldr      lr, [fp, #-0x3c]
0x24dd04  019985e2           add      sb, r5, #0x4000
0x24dd08  c39f99e5           ldr      sb, [sb, #0xfc3]  # pool[5103] = snapshotRef(894)
0x24dd0c  00428de8           stm      sp, {sb, lr}
0x24dd10  141bfceb           bl       #0x154968
0x24dd14  38001be5           ldr      r0, [fp, #-0x38]
0x24dd18  00d04be2           sub      sp, fp, #0
0x24dd1c  0088bde8           pop      {fp, pc}
0x24dd20  4cd04be2           sub      sp, fp, #0x4c
0x24dd24  0030a0e1           mov      r3, r0
0x24dd28  38000be5           str      r0, [fp, #-0x38]
0x24dd2c  0100a0e1           mov      r0, r1
0x24dd30  40100be5           str      r1, [fp, #-0x40]
0x24dd34  010013e3           tst      r3, #1
0x24dd38  01101315           ldrne    r1, [r3, #-1]
0x24dd3c  5116f317           ubfxne   r1, r1, #0xc, #0x14
0x24dd40  3c10a003           moveq    r1, #0x3c
0x24dd44  c2c800e3           movw     ip, #0x8c2
0x24dd48  0c0051e1           cmp      r1, ip
0x24dd4c  2500000a           beq      #0x24dde8
0x24dd50  40109ae5           ldr      r1, [sl, #0x40]
0x24dd54  0420a0e3           mov      r2, #4
0x24dd58  d32b03eb           bl       #0x318cac
0x24dd5c  44000be5           str      r0, [fp, #-0x44]
0x24dd60  0b2080e2           add      r2, r0, #0xb
0x24dd64  01c985e2           add      ip, r5, #0x4000
0x24dd68  c7cf9ce5           ldr      ip, [ip, #0xfc7]  # pool[5104] = "fallback"
0x24dd6c  00c082e5           str      ip, [r2]
0x24dd70  40e01be5           ldr      lr, [fp, #-0x40]
0x24dd74  00e08de5           str      lr, [sp]
0x24dd78  59b7fdeb           bl       #0x1bbae4
0x24dd7c  0010a0e1           mov      r1, r0
0x24dd80  44001be5           ldr      r0, [fp, #-0x44]
0x24dd84  0f3080e2           add      r3, r0, #0xf
0x24dd88  001083e5           str      r1, [r3]
0x24dd8c  00008de5           str      r0, [sp]
0x24dd90  2c1afceb           bl       #0x154648
0x24dd94  34e01be5           ldr      lr, [fp, #-0x34]
0x24dd98  01408de8           stm      sp, {r0, lr}
0x24dd9c  f11afceb           bl       #0x154968
0x24dda0  44000be5           str      r0, [fp, #-0x44]
0x24dda4  04008de5           str      r0, [sp, #4]
0x24dda8  01e985e2           add      lr, r5, #0x4000
0x24ddac  c3ef9ee5           ldr      lr, [lr, #0xfc3]  # pool[5103] = snapshotRef(894)
0x24ddb0  00e08de5           str      lr, [sp]
0x24ddb4  eb1afceb           bl       #0x154968
0x24ddb8  44001be5           ldr      r0, [fp, #-0x44]
0x24ddbc  00d04be2           sub      sp, fp, #0
0x24ddc0  0088bde8           pop      {fp, pc}
0x24ddc4  0822fceb           bl       #0x1565ec
0x24ddc8  0010a0e1           mov      r1, r0
0x24ddcc  010985e2           add      r0, r5, #0x4000
0x24ddd0  cb0f90e5           ldr      r0, [r0, #0xfcb]  # pool[5105] = "bad"
0x24ddd4  44100be5           str      r1, [fp, #-0x44]
0x24ddd8  030081e5           str      r0, [r1, #3]
0x24dddc  0100a0e1           mov      r0, r1
0x24dde0  a82403eb           bl       #0x317088
0x24dde4  700020e1           bkpt     #0
0x24dde8  0300a0e1           mov      r0, r3
0x24ddec  031090e5           ldr      r1, [r0, #3]
0x24ddf0  34e01be5           ldr      lr, [fp, #-0x34]
0x24ddf4  02408de8           stm      sp, {r1, lr}
0x24ddf8  da1afceb           bl       #0x154968
0x24ddfc  0020a0e1           mov      r2, r0
0x24de00  38001be5           ldr      r0, [fp, #-0x38]
0x24de04  40101be5           ldr      r1, [fp, #-0x40]
0x24de08  3c200be5           str      r2, [fp, #-0x3c]
0x24de0c  902403eb           bl       #0x317054
0x24de10  700020e1           bkpt     #0
0x24de14  4cd04be2           sub      sp, fp, #0x4c
0x24de18  38000be5           str      r0, [fp, #-0x38]
0x24de1c  3c100be5           str      r1, [fp, #-0x3c]
0x24de20  34e01be5           ldr      lr, [fp, #-0x34]
0x24de24  019985e2           add      sb, r5, #0x4000
0x24de28  c39f99e5           ldr      sb, [sb, #0xfc3]  # pool[5103] = snapshotRef(894)
0x24de2c  00428de8           stm      sp, {sb, lr}
0x24de30  cc1afceb           bl       #0x154968
0x24de34  38001be5           ldr      r0, [fp, #-0x38]
0x24de38  3c101be5           ldr      r1, [fp, #-0x3c]
0x24de3c  842403eb           bl       #0x317054
0x24de40  700020e1           bkpt     #0
# CFG: 0x24dc74->0x24dccc/ConditionalFalse 0x24dc74->0x24ddc4/ConditionalTrue 0x24dd20->0x24dd50/ConditionalFalse 0x24dd20->0x24dde8/ConditionalTrue 0x24ddc4->0x24dde8/Fallthrough

# top_level.e07GenericBound at 0x24de44 (156 bytes)
0x24de44  00482de9           push     {fp, lr}
0x24de48  00b08de2           add      fp, sp, #0
0x24de4c  10d04de2           sub      sp, sp, #0x10
0x24de50  0000a0e3           mov      r0, #0
0x24de54  0b1094e5           ldr      r1, [r4, #0xb]
0x24de58  000051e1           cmp      r1, r0
0x24de5c  0100001a           bne      #0x24de68
0x24de60  40209ae5           ldr      r2, [sl, #0x40]
0x24de64  030000ea           b        #0x24de78
0x24de68  132094e5           ldr      r2, [r4, #0x13]
0x24de6c  82308be0           add      r3, fp, r2, lsl #1
0x24de70  083093e5           ldr      r3, [r3, #8]
0x24de74  0320a0e1           mov      r2, r3
0x24de78  24c09ae5           ldr      ip, [sl, #0x24]
0x24de7c  0c005de1           cmp      sp, ip
0x24de80  cd2b039b           blls     #0x318dbc
0x24de84  000051e1           cmp      r1, r0
0x24de88  0200001a           bne      #0x24de98
0x24de8c  010985e2           add      r0, r5, #0x4000
0x24de90  cf0f90e5           ldr      r0, [r0, #0xfcf]  # pool[5106] = snapshotRef(18246)
0x24de94  000000ea           b        #0x24de9c
0x24de98  0200a0e1           mov      r0, r2
0x24de9c  04000be5           str      r0, [fp, #-4]
0x24dea0  011985e2           add      r1, r5, #0x4000
0x24dea4  d31f91e5           ldr      r1, [r1, #0xfd3]  # pool[5107] = <anonymous closure>
0x24dea8  40209ae5           ldr      r2, [sl, #0x40]
0x24deac  d22703eb           bl       #0x317dfc
0x24deb0  0010a0e1           mov      r1, r0
0x24deb4  04001be5           ldr      r0, [fp, #-4]
0x24deb8  070081e5           str      r0, [r1, #7]
0x24debc  01e985e2           add      lr, r5, #0x4000
0x24dec0  d7ef9ee5           ldr      lr, [lr, #0xfd7]  # pool[5108] = snapshotRef(18413)
0x24dec4  08909be5           ldr      sb, [fp, #8]
0x24dec8  02428de8           stm      sp, {r1, sb, lr}
0x24decc  034a85e2           add      r4, r5, #0x3000
0x24ded0  5f4d94e5           ldr      r4, [r4, #0xd5f]  # pool[3926] = snapshotRef(34714)
0x24ded4  b37d02eb           bl       #0x2ed5a8
0x24ded8  00d04be2           sub      sp, fp, #0
0x24dedc  0088bde8           pop      {fp, pc}
# CFG: 0x24de44->0x24de60/ConditionalFalse 0x24de44->0x24de68/ConditionalTrue 0x24de60->0x24de78/Branch 0x24de68->0x24de78/Fallthrough 0x24de78->0x24de8c/ConditionalFalse 0x24de78->0x24de98/ConditionalTrue 0x24de8c->0x24de9c/Branch 0x24de98->0x24de9c/Fallthrough

# top_level.<anonymous closure> at 0x24dee0 (112 bytes)
0x24dee0  00482de9           push     {fp, lr}
0x24dee4  00b08de2           add      fp, sp, #0
0x24dee8  0cd04de2           sub      sp, sp, #0xc
0x24deec  24c09ae5           ldr      ip, [sl, #0x24]
0x24def0  0c005de1           cmp      sp, ip
0x24def4  b02b039b           blls     #0x318dbc
0x24def8  08009be5           ldr      r0, [fp, #8]
0x24defc  010010e3           tst      r0, #1
0x24df00  01101015           ldrne    r1, [r0, #-1]
0x24df04  5116f317           ubfxne   r1, r1, #0xc, #0x14
0x24df08  3c10a003           moveq    r1, #0x3c
0x24df0c  04008de5           str      r0, [sp, #4]
0x24df10  02e0a0e3           mov      lr, #2
0x24df14  00e08de5           str      lr, [sp]
0x24df18  0100a0e1           mov      r0, r1
0x24df1c  00e187e0           add      lr, r7, r0, lsl #2
0x24df20  f0ef1ee5           ldr      lr, [lr, #-0xff0]
0x24df24  3eff2fe1           blx      lr
0x24df28  013985e2           add      r3, r5, #0x4000
0x24df2c  d73f93e5           ldr      r3, [r3, #0xfd7]  # pool[5108] = snapshotRef(18413)
0x24df30  04000be5           str      r0, [fp, #-4]
0x24df34  050000eb           bl       #0x24df50
0x24df38  0c109be5           ldr      r1, [fp, #0xc]
0x24df3c  071080e5           str      r1, [r0, #7]
0x24df40  04101be5           ldr      r1, [fp, #-4]
0x24df44  0b1080e5           str      r1, [r0, #0xb]
0x24df48  00d04be2           sub      sp, fp, #0
0x24df4c  0088bde8           pop      {fp, pc}

# top_level.e06RecordDestructure at 0x24df5c (424 bytes)
0x24df5c  00482de9           push     {fp, lr}
0x24df60  00b08de2           add      fp, sp, #0
0x24df64  20d04de2           sub      sp, sp, #0x20
0x24df68  04100be5           str      r1, [fp, #-4]
0x24df6c  24c09ae5           ldr      ip, [sl, #0x24]
0x24df70  0c005de1           cmp      sp, ip
0x24df74  902b039b           blls     #0x318dbc
0x24df78  0220a0e3           mov      r2, #2
0x24df7c  0430a0e3           mov      r3, #4
0x24df80  f92503eb           bl       #0x31776c
0x24df84  40109ae5           ldr      r1, [sl, #0x40]
0x24df88  0420a0e3           mov      r2, #4
0x24df8c  08000be5           str      r0, [fp, #-8]
0x24df90  452b03eb           bl       #0x318cac
0x24df94  0010a0e1           mov      r1, r0
0x24df98  08001be5           ldr      r0, [fp, #-8]
0x24df9c  0c100be5           str      r1, [fp, #-0xc]
0x24dfa0  0b3081e2           add      r3, r1, #0xb
0x24dfa4  000083e5           str      r0, [r3]
0x24dfa8  0620a0e3           mov      r2, #6
0x24dfac  0830a0e3           mov      r3, #8
0x24dfb0  ed2503eb           bl       #0x31776c
0x24dfb4  0c301be5           ldr      r3, [fp, #-0xc]
0x24dfb8  0f2083e2           add      r2, r3, #0xf
0x24dfbc  000082e5           str      r0, [r2]
0x24dfc0  0080a0e3           mov      r8, #0
0x24dfc4  0060a0e3           mov      r6, #0
0x24dfc8  0000a0e3           mov      r0, #0
0x24dfcc  40409ae5           ldr      r4, [sl, #0x40]
0x24dfd0  14800be5           str      r8, [fp, #-0x14]
0x24dfd4  18600be5           str      r6, [fp, #-0x18]
0x24dfd8  24c09ae5           ldr      ip, [sl, #0x24]
0x24dfdc  0c005de1           cmp      sp, ip
0x24dfe0  752b039b           blls     #0x318dbc
0x24dfe4  020050e3           cmp      r0, #2
0x24dfe8  2a0000aa           bge      #0x24e098
0x24dfec  8010a0e1           lsl      r1, r0, #1
0x24dff0  81c083e0           add      ip, r3, r1, lsl #1
0x24dff4  0b909ce5           ldr      sb, [ip, #0xb]
0x24dff8  10900be5           str      sb, [fp, #-0x10]
0x24dffc  011080e2           add      r1, r0, #1
0x24e000  08100be5           str      r1, [fp, #-8]
0x24e004  040059e1           cmp      sb, r4
0x24e008  0700001a           bne      #0x24e02c
0x24e00c  0900a0e1           mov      r0, sb
0x24e010  0420a0e1           mov      r2, r4
0x24e014  0410a0e1           mov      r1, r4
0x24e018  018985e2           add      r8, r5, #0x4000
0x24e01c  fb8f98e5           ldr      r8, [r8, #0xffb]  # pool[5117] = snapshotRef(34538)
0x24e020  013985e2           add      r3, r5, #0x4000
0x24e024  ff3f93e5           ldr      r3, [r3, #0xfff]  # pool[5118] = null
0x24e028  350000eb           bl       #0x24e104
0x24e02c  14201be5           ldr      r2, [fp, #-0x14]
0x24e030  18101be5           ldr      r1, [fp, #-0x18]
0x24e034  10001be5           ldr      r0, [fp, #-0x10]
0x24e038  073090e5           ldr      r3, [r0, #7]
0x24e03c  0b4090e5           ldr      r4, [r0, #0xb]
0x24e040  010013e3           tst      r3, #1
0x24e044  01001315           ldrne    r0, [r3, #-1]
0x24e048  5006f317           ubfxne   r0, r0, #0xc, #0x14
0x24e04c  3c00a003           moveq    r0, #0x3c
0x24e050  04308de5           str      r3, [sp, #4]
0x24e054  00408de5           str      r4, [sp]
0x24e058  00e187e0           add      lr, r7, r0, lsl #2
0x24e05c  bcef1ee5           ldr      lr, [lr, #-0xfbc]
0x24e060  3eff2fe1           blx      lr
0x24e064  c02fa0e1           asr      r2, r0, #0x1f
0x24e068  c030b0e1           asrs     r3, r0, #1
0x24e06c  0100003a           blo      #0x24e078
0x24e070  073090e5           ldr      r3, [r0, #7]
0x24e074  0b2090e5           ldr      r2, [r0, #0xb]
0x24e078  14601be5           ldr      r6, [fp, #-0x14]
0x24e07c  18401be5           ldr      r4, [fp, #-0x18]
0x24e080  038096e0           adds     r8, r6, r3
0x24e084  0210b4e0           adcs     r1, r4, r2
0x24e088  0160a0e1           mov      r6, r1
0x24e08c  08001be5           ldr      r0, [fp, #-8]
0x24e090  0c301be5           ldr      r3, [fp, #-0xc]
0x24e094  ccffffea           b        #0x24dfcc
0x24e098  04201be5           ldr      r2, [fp, #-4]
0x24e09c  0640a0e1           mov      r4, r6
0x24e0a0  0860a0e1           mov      r6, r8
0x24e0a4  073092e5           ldr      r3, [r2, #7]
0x24e0a8  0b8092e5           ldr      r8, [r2, #0xb]
0x24e0ac  0f9092e5           ldr      sb, [r2, #0xf]
0x24e0b0  c32fa0e1           asr      r2, r3, #0x1f
0x24e0b4  c300b0e1           asrs     r0, r3, #1
0x24e0b8  0100003a           blo      #0x24e0c4
0x24e0bc  070093e5           ldr      r0, [r3, #7]
0x24e0c0  0b2093e5           ldr      r2, [r3, #0xb]
0x24e0c4  001096e0           adds     r1, r6, r0
0x24e0c8  0230b4e0           adcs     r3, r4, r2
0x24e0cc  c82fa0e1           asr      r2, r8, #0x1f
0x24e0d0  c840b0e1           asrs     r4, r8, #1
0x24e0d4  0100003a           blo      #0x24e0e0
0x24e0d8  074098e5           ldr      r4, [r8, #7]
0x24e0dc  0b2098e5           ldr      r2, [r8, #0xb]
0x24e0e0  048051e0           subs     r8, r1, r4
0x24e0e4  0260d3e0           sbcs     r6, r3, r2
0x24e0e8  072099e5           ldr      r2, [sb, #7]
0x24e0ec  c240a0e1           asr      r4, r2, #1
0x24e0f0  c43fa0e1           asr      r3, r4, #0x1f
0x24e0f4  040098e0           adds     r0, r8, r4
0x24e0f8  0310b6e0           adcs     r1, r6, r3
0x24e0fc  00d04be2           sub      sp, fp, #0
0x24e100  0088bde8           pop      {fp, pc}
# CFG: 0x24df5c->0x24dfcc/Fallthrough 0x24dfcc->0x24dfec/ConditionalFalse 0x24dfcc->0x24e098/ConditionalTrue 0x24dfec->0x24e00c/ConditionalFalse 0x24dfec->0x24e02c/ConditionalTrue 0x24e00c->0x24e02c/Fallthrough 0x24e02c->0x24e070/ConditionalFalse 0x24e02c->0x24e078/ConditionalTrue 0x24e070->0x24e078/Fallthrough 0x24e078->0x24dfcc/Branch 0x24e098->0x24e0bc/ConditionalFalse 0x24e098->0x24e0c4/ConditionalTrue 0x24e0bc->0x24e0c4/Fallthrough 0x24e0c4->0x24e0d8/ConditionalFalse 0x24e0c4->0x24e0e0/ConditionalTrue 0x24e0d8->0x24e0e0/Fallthrough

# top_level.e05NullFlow at 0x24e170 (388 bytes)
0x24e170  00482de9           push     {fp, lr}
0x24e174  00b08de2           add      fp, sp, #0
0x24e178  14d04de2           sub      sp, sp, #0x14
0x24e17c  0100a0e1           mov      r0, r1
0x24e180  04100be5           str      r1, [fp, #-4]
0x24e184  24c09ae5           ldr      ip, [sl, #0x24]
0x24e188  0c005de1           cmp      sp, ip
0x24e18c  0a2b039b           blls     #0x318dbc
0x24e190  0010a0e1           mov      r1, r0
0x24e194  012985e2           add      r2, r5, #0x4000
0x24e198  5f2d92e5           ldr      r2, [r2, #0xd5f]  # pool[4950] = snapshotRef(130)
0x24e19c  daeefbeb           bl       #0x149d0c
0x24e1a0  0010a0e1           mov      r1, r0
0x24e1a4  04001be5           ldr      r0, [fp, #-4]
0x24e1a8  0b2090e5           ldr      r2, [r0, #0xb]
0x24e1ac  010052e1           cmp      r2, r1
0x24e1b0  0100001a           bne      #0x24e1bc
0x24e1b4  40609ae5           ldr      r6, [sl, #0x40]
0x24e1b8  000000ea           b        #0x24e1c0
0x24e1bc  0160a0e1           mov      r6, r1
0x24e1c0  40409ae5           ldr      r4, [sl, #0x40]
0x24e1c4  0230a0e3           mov      r3, #2
0x24e1c8  0410a0e1           mov      r1, r4
0x24e1cc  0320a0e1           mov      r2, r3
0x24e1d0  08600be5           str      r6, [fp, #-8]
0x24e1d4  b42a03eb           bl       #0x318cac
0x24e1d8  0c000be5           str      r0, [fp, #-0xc]
0x24e1dc  0b2080e2           add      r2, r0, #0xb
0x24e1e0  01c985e2           add      ip, r5, #0x4000
0x24e1e4  8bcd9ce5           ldr      ip, [ip, #0xd8b]  # pool[4961] = snapshotRef(709)
0x24e1e8  00c082e5           str      ip, [r2]
0x24e1ec  b33395e5           ldr      r3, [r5, #0x3b3]  # pool[235] = snapshotRef(18337)
0x24e1f0  022603eb           bl       #0x317a00
0x24e1f4  0030a0e1           mov      r3, r0
0x24e1f8  0c001be5           ldr      r0, [fp, #-0xc]
0x24e1fc  10300be5           str      r3, [fp, #-0x10]
0x24e200  0b0083e5           str      r0, [r3, #0xb]
0x24e204  0200a0e3           mov      r0, #2
0x24e208  070083e5           str      r0, [r3, #7]
0x24e20c  08601be5           ldr      r6, [fp, #-8]
0x24e210  40409ae5           ldr      r4, [sl, #0x40]
0x24e214  040056e1           cmp      r6, r4
0x24e218  0200001a           bne      #0x24e228
0x24e21c  0400a0e1           mov      r0, r4
0x24e220  40209ae5           ldr      r2, [sl, #0x40]
0x24e224  080000ea           b        #0x24e24c
0x24e228  010016e5           ldr      r0, [r6, #-1]
0x24e22c  5006f3e7           ubfx     r0, r0, #0xc, #0x14
0x24e230  0610a0e1           mov      r1, r6
0x24e234  d72095e5           ldr      r2, [r5, #0xd7]  # pool[52] = snapshotRef(101)
0x24e238  00e187e0           add      lr, r7, r0, lsl #2
0x24e23c  fcef1ee5           ldr      lr, [lr, #-0xffc]
0x24e240  3eff2fe1           blx      lr
0x24e244  0020a0e1           mov      r2, r0
0x24e248  40009ae5           ldr      r0, [sl, #0x40]
0x24e24c  000052e1           cmp      r2, r0
0x24e250  0100000a           beq      #0x24e25c
0x24e254  10101be5           ldr      r1, [fp, #-0x10]
0x24e258  dce7fbeb           bl       #0x1481d0
0x24e25c  08101be5           ldr      r1, [fp, #-8]
0x24e260  40009ae5           ldr      r0, [sl, #0x40]
0x24e264  000051e1           cmp      r1, r0
0x24e268  0700001a           bne      #0x24e28c
0x24e26c  03ea85e2           add      lr, r5, #0x3000
0x24e270  0fe39ee5           ldr      lr, [lr, #0x30f]  # pool[3266] = snapshotRef(732)
0x24e274  00e08de5           str      lr, [sp]
0x24e278  10101be5           ldr      r1, [fp, #-0x10]
0x24e27c  134295e5           ldr      r4, [r5, #0x213]  # pool[131] = snapshotRef(34581)
0x24e280  518a02eb           bl       #0x2f0bcc
0x24e284  0030a0e1           mov      r3, r0
0x24e288  000000ea           b        #0x24e290
0x24e28c  0130a0e1           mov      r3, r1
0x24e290  04001be5           ldr      r0, [fp, #-4]
0x24e294  0010a0e1           mov      r1, r0
0x24e298  08300be5           str      r3, [fp, #-8]
0x24e29c  052a85e2           add      r2, r5, #0x5000
0x24e2a0  0b2092e5           ldr      r2, [r2, #0xb]  # pool[5121] = "missing"
0x24e2a4  98eefbeb           bl       #0x149d0c
0x24e2a8  04101be5           ldr      r1, [fp, #-4]
0x24e2ac  0b2091e5           ldr      r2, [r1, #0xb]
0x24e2b0  000052e1           cmp      r2, r0
0x24e2b4  0100001a           bne      #0x24e2c0
0x24e2b8  40209ae5           ldr      r2, [sl, #0x40]
0x24e2bc  000000ea           b        #0x24e2c4
0x24e2c0  0020a0e1           mov      r2, r0
0x24e2c4  40009ae5           ldr      r0, [sl, #0x40]
0x24e2c8  000052e1           cmp      r2, r0
0x24e2cc  0500001a           bne      #0x24e2e8
0x24e2d0  08301be5           ldr      r3, [fp, #-8]
0x24e2d4  052a85e2           add      r2, r5, #0x5000
0x24e2d8  0b2092e5           ldr      r2, [r2, #0xb]  # pool[5121] = "missing"
0x24e2dc  9c8802eb           bl       #0x2f0554
0x24e2e0  08001be5           ldr      r0, [fp, #-8]
0x24e2e4  000000ea           b        #0x24e2ec
0x24e2e8  0200a0e1           mov      r0, r2
0x24e2ec  00d04be2           sub      sp, fp, #0
0x24e2f0  0088bde8           pop      {fp, pc}
# CFG: 0x24e170->0x24e1b4/ConditionalFalse 0x24e170->0x24e1bc/ConditionalTrue 0x24e1b4->0x24e1c0/Branch 0x24e1bc->0x24e1c0/Fallthrough 0x24e1c0->0x24e21c/ConditionalFalse 0x24e1c0->0x24e228/ConditionalTrue 0x24e21c->0x24e24c/Branch 0x24e228->0x24e24c/Fallthrough 0x24e24c->0x24e254/ConditionalFalse 0x24e24c->0x24e25c/ConditionalTrue 0x24e254->0x24e25c/Fallthrough 0x24e25c->0x24e26c/ConditionalFalse 0x24e25c->0x24e28c/ConditionalTrue 0x24e26c->0x24e290/Branch 0x24e28c->0x24e290/Fallthrough 0x24e290->0x24e2b8/ConditionalFalse 0x24e290->0x24e2c0/ConditionalTrue 0x24e2b8->0x24e2c4/Branch 0x24e2c0->0x24e2c4/Fallthrough 0x24e2c4->0x24e2d0/ConditionalFalse 0x24e2c4->0x24e2e8/ConditionalTrue 0x24e2d0->0x24e2ec/Branch 0x24e2e8->0x24e2ec/Fallthrough

# top_level.e04BitTwiddle at 0x24e2f4 (304 bytes)
0x24e2f4  00482de9           push     {fp, lr}
0x24e2f8  00b08de2           add      fp, sp, #0
0x24e2fc  24d04de2           sub      sp, sp, #0x24
0x24e300  0230a0e1           mov      r3, r2
0x24e304  14200be5           str      r2, [fp, #-0x14]
0x24e308  0120a0e1           mov      r2, r1
0x24e30c  18100be5           str      r1, [fp, #-0x18]
0x24e310  24c09ae5           ldr      ip, [sl, #0x24]
0x24e314  0c005de1           cmp      sp, ip
0x24e318  a72a039b           blls     #0x318dbc
0x24e31c  0200a0e1           mov      r0, r2
0x24e320  8011a0e1           lsl      r1, r0, #3
0x24e324  ff4001e2           and      r4, r1, #0xff
0x24e328  04400be5           str      r4, [fp, #-4]
0x24e32c  031fa0e1           lsl      r1, r3, #0x1e
0x24e330  221181e1           orr      r1, r1, r2, lsr #2
0x24e334  4301a0e1           asr      r0, r3, #2
0x24e338  0260a0e1           mov      r6, r2
0x24e33c  0f8006e2           and      r8, r6, #0xf
0x24e340  0890a0e1           mov      sb, r8
0x24e344  066026e0           eor      r6, r6, r6
0x24e348  094081e1           orr      r4, r1, sb
0x24e34c  068080e1           orr      r8, r0, r6
0x24e350  08400be5           str      r4, [fp, #-8]
0x24e354  10800be5           str      r8, [fp, #-0x10]
0x24e358  8200a0e1           lsl      r0, r2, #1
0x24e35c  c00052e1           cmp      r2, r0, asr #1
0x24e360  c00f5301           cmpeq    r3, r0, asr #31
0x24e364  0200000a           beq      #0x24e374
0x24e368  bf2a03eb           bl       #0x318e6c
0x24e36c  072080e5           str      r2, [r0, #7]
0x24e370  0b3080e5           str      r3, [r0, #0xb]
0x24e374  0c000be5           str      r0, [fp, #-0xc]
0x24e378  04008de5           str      r0, [sp, #4]
0x24e37c  0ee0a0e3           mov      lr, #0xe
0x24e380  00e08de5           str      lr, [sp]
0x24e384  c610fceb           bl       #0x1526a4
0x24e388  1c000be5           str      r0, [fp, #-0x1c]
0x24e38c  0ce01be5           ldr      lr, [fp, #-0xc]
0x24e390  0590e0e3           mvn      sb, #5
0x24e394  00428de8           stm      sp, {sb, lr}
0x24e398  c110fceb           bl       #0x1526a4
0x24e39c  c02fa0e1           asr      r2, r0, #0x1f
0x24e3a0  c030b0e1           asrs     r3, r0, #1
0x24e3a4  0100003a           blo      #0x24e3b0
0x24e3a8  073090e5           ldr      r3, [r0, #7]
0x24e3ac  0b2090e5           ldr      r2, [r0, #0xb]
0x24e3b0  0240e0e3           mvn      r4, #2
0x24e3b4  0060e0e3           mvn      r6, #0
0x24e3b8  930600e0           mul      r0, r3, r6
0x24e3bc  920428e0           mla      r8, r2, r4, r0
0x24e3c0  939480e0           umull    sb, r0, r3, r4
0x24e3c4  008088e0           add      r8, r8, r0
0x24e3c8  18201be5           ldr      r2, [fp, #-0x18]
0x24e3cc  14301be5           ldr      r3, [fp, #-0x14]
0x24e3d0  096052e0           subs     r6, r2, sb
0x24e3d4  0840d3e0           sbcs     r4, r3, r8
0x24e3d8  04201be5           ldr      r2, [fp, #-4]
0x24e3dc  0280a0e1           mov      r8, r2
0x24e3e0  033023e0           eor      r3, r3, r3
0x24e3e4  08901be5           ldr      sb, [fp, #-8]
0x24e3e8  10201be5           ldr      r2, [fp, #-0x10]
0x24e3ec  091028e0           eor      r1, r8, sb
0x24e3f0  020023e0           eor      r0, r3, r2
0x24e3f4  1c201be5           ldr      r2, [fp, #-0x1c]
0x24e3f8  c23fa0e1           asr      r3, r2, #0x1f
0x24e3fc  c280b0e1           asrs     r8, r2, #1
0x24e400  0100003a           blo      #0x24e40c
0x24e404  078092e5           ldr      r8, [r2, #7]
0x24e408  0b3092e5           ldr      r3, [r2, #0xb]
0x24e40c  089021e0           eor      sb, r1, r8
0x24e410  032020e0           eor      r2, r0, r3
0x24e414  060029e0           eor      r0, sb, r6
0x24e418  041022e0           eor      r1, r2, r4
0x24e41c  00d04be2           sub      sp, fp, #0
0x24e420  0088bde8           pop      {fp, pc}
# CFG: 0x24e2f4->0x24e368/ConditionalFalse 0x24e2f4->0x24e374/ConditionalTrue 0x24e368->0x24e374/Fallthrough 0x24e374->0x24e3a8/ConditionalFalse 0x24e374->0x24e3b0/ConditionalTrue 0x24e3a8->0x24e3b0/Fallthrough 0x24e3b0->0x24e404/ConditionalFalse 0x24e3b0->0x24e40c/ConditionalTrue 0x24e404->0x24e40c/Fallthrough

# top_level.e02Cascade at 0x24e424 (200 bytes)
0x24e424  00482de9           push     {fp, lr}
0x24e428  00b08de2           add      fp, sp, #0
0x24e42c  10d04de2           sub      sp, sp, #0x10
0x24e430  0100a0e1           mov      r0, r1
0x24e434  04100be5           str      r1, [fp, #-4]
0x24e438  24c09ae5           ldr      ip, [sl, #0x24]
0x24e43c  0c005de1           cmp      sp, ip
0x24e440  5d2a039b           blls     #0x318dbc
0x24e444  b31395e5           ldr      r1, [r5, #0x3b3]  # pool[235] = snapshotRef(18337)
0x24e448  0020a0e3           mov      r2, #0
0x24e44c  0030a0e3           mov      r3, #0
0x24e450  3aeafbeb           bl       #0x148d40
0x24e454  0010a0e1           mov      r1, r0
0x24e458  04201be5           ldr      r2, [fp, #-4]
0x24e45c  04000be5           str      r0, [fp, #-4]
0x24e460  5ae7fbeb           bl       #0x1481d0
0x24e464  04101be5           ldr      r1, [fp, #-4]
0x24e468  0b4295e5           ldr      r4, [r5, #0x20b]  # pool[129] = snapshotRef(34634)
0x24e46c  6ef2fceb           bl       #0x18ae2c
0x24e470  04001be5           ldr      r0, [fp, #-4]
0x24e474  072090e5           ldr      r2, [r0, #7]
0x24e478  10200be5           str      r2, [fp, #-0x10]
0x24e47c  0b1090e5           ldr      r1, [r0, #0xb]
0x24e480  073091e5           ldr      r3, [r1, #7]
0x24e484  c260a0e1           asr      r6, r2, #1
0x24e488  c64fa0e1           asr      r4, r6, #0x1f
0x24e48c  08600be5           str      r6, [fp, #-8]
0x24e490  0c400be5           str      r4, [fp, #-0xc]
0x24e494  0610a0e1           mov      r1, r6
0x24e498  c380a0e1           asr      r8, r3, #1
0x24e49c  080051e1           cmp      r1, r8
0x24e4a0  0100001a           bne      #0x24e4ac
0x24e4a4  0010a0e1           mov      r1, r0
0x24e4a8  40f4fbeb           bl       #0x14b5b0
0x24e4ac  04001be5           ldr      r0, [fp, #-4]
0x24e4b0  08201be5           ldr      r2, [fp, #-8]
0x24e4b4  0c101be5           ldr      r1, [fp, #-0xc]
0x24e4b8  0230a0e1           mov      r3, r2
0x24e4bc  011083e2           add      r1, r3, #1
0x24e4c0  8120a0e1           lsl      r2, r1, #1
0x24e4c4  072080e5           str      r2, [r0, #7]
0x24e4c8  0b1090e5           ldr      r1, [r0, #0xb]
0x24e4cc  10401be5           ldr      r4, [fp, #-0x10]
0x24e4d0  843081e0           add      r3, r1, r4, lsl #1
0x24e4d4  0b3083e2           add      r3, r3, #0xb
0x24e4d8  05ca85e2           add      ip, r5, #0x5000
0x24e4dc  0fc09ce5           ldr      ip, [ip, #0xf]  # pool[5122] = "done"
0x24e4e0  00c083e5           str      ip, [r3]
0x24e4e4  00d04be2           sub      sp, fp, #0
0x24e4e8  0088bde8           pop      {fp, pc}
# CFG: 0x24e424->0x24e4a4/ConditionalFalse 0x24e424->0x24e4ac/ConditionalTrue 0x24e4a4->0x24e4ac/Fallthrough

# top_level.e01InterpChain at 0x24e4f8 (560 bytes)
0x24e4f8  00482de9           push     {fp, lr}
0x24e4fc  00b08de2           add      fp, sp, #0
0x24e500  1cd04de2           sub      sp, sp, #0x1c
0x24e504  0140a0e1           mov      r4, r1
0x24e508  0200a0e1           mov      r0, r2
0x24e50c  04100be5           str      r1, [fp, #-4]
0x24e510  08300be5           str      r3, [fp, #-8]
0x24e514  0c200be5           str      r2, [fp, #-0xc]
0x24e518  060b0bed           vstr     d0, [fp, #-0x18]
0x24e51c  24c09ae5           ldr      ip, [sl, #0x24]
0x24e520  0c005de1           cmp      sp, ip
0x24e524  322a039b           blls     #0x318df4
0x24e528  40109ae5           ldr      r1, [sl, #0x40]
0x24e52c  1820a0e3           mov      r2, #0x18
0x24e530  dd2903eb           bl       #0x318cac
0x24e534  0040a0e1           mov      r4, r0
0x24e538  10400be5           str      r4, [fp, #-0x10]
0x24e53c  0b1084e2           add      r1, r4, #0xb
0x24e540  05ca85e2           add      ip, r5, #0x5000
0x24e544  13c09ce5           ldr      ip, [ip, #0x13]  # pool[5123] = "user="
0x24e548  00c081e5           str      ip, [r1]
0x24e54c  04601be5           ldr      r6, [fp, #-4]
0x24e550  0f1084e2           add      r1, r4, #0xf
0x24e554  006081e5           str      r6, [r1]
0x24e558  131084e2           add      r1, r4, #0x13
0x24e55c  05ca85e2           add      ip, r5, #0x5000
0x24e560  17c09ce5           ldr      ip, [ip, #0x17]  # pool[5124] = " id="
0x24e564  00c081e5           str      ip, [r1]
0x24e568  0c801be5           ldr      r8, [fp, #-0xc]
0x24e56c  08201be5           ldr      r2, [fp, #-8]
0x24e570  019098e2           adds     sb, r8, #1
0x24e574  0030b2e2           adcs     r3, r2, #0
0x24e578  8900a0e1           lsl      r0, sb, #1
0x24e57c  c00059e1           cmp      sb, r0, asr #1
0x24e580  c00f5301           cmpeq    r3, r0, asr #31
0x24e584  0200000a           beq      #0x24e594
0x24e588  372a03eb           bl       #0x318e6c
0x24e58c  079080e5           str      sb, [r0, #7]
0x24e590  0b3080e5           str      r3, [r0, #0xb]
0x24e594  0410a0e1           mov      r1, r4
0x24e598  179081e2           add      sb, r1, #0x17
0x24e59c  000089e5           str      r0, [sb]
0x24e5a0  010010e3           tst      r0, #1
0x24e5a4  0500000a           beq      #0x24e5c0
0x24e5a8  01c051e5           ldrb     ip, [r1, #-1]
0x24e5ac  01e050e5           ldrb     lr, [r0, #-1]
0x24e5b0  2cc10ee0           and      ip, lr, ip, lsr #2
0x24e5b4  28e09ae5           ldr      lr, [sl, #0x28]
0x24e5b8  0e001ce1           tst      ip, lr
0x24e5bc  bb22031b           blne     #0x3170b0
0x24e5c0  1b1084e2           add      r1, r4, #0x1b
0x24e5c4  05ca85e2           add      ip, r5, #0x5000
0x24e5c8  1bc09ce5           ldr      ip, [ip, #0x1b]  # pool[5125] = " pct="
0x24e5cc  00c081e5           str      ip, [r1]
0x24e5d0  062b1bed           vldr     d2, [fp, #-0x18]
0x24e5d4  04002de5           str      r0, [sp, #-4]!
0x24e5d8  03c005e3           movw     ip, #0x5003
0x24e5dc  0cc085e0           add      ip, r5, ip
0x24e5e0  070b9ced           vldr     d0, [ip, #0x1c]
0x24e5e4  04009de4           pop      {r0}
0x24e5e8  004b22ee           vmul.f64 d4, d0
0x24e5ec  2c109ae5           ldr      r1, [sl, #0x2c]
0x24e5f0  101081e2           add      r1, r1, #0x10
0x24e5f4  30c09ae5           ldr      ip, [sl, #0x30]
0x24e5f8  01005ce1           cmp      ip, r1
0x24e5fc  4200009a           bls      #0x24e70c
0x24e600  2c108ae5           str      r1, [sl, #0x2c]
0x24e604  0f1041e2           sub      r1, r1, #0xf
0x24e608  9c020ee3           movw     r0, #0xe29c
0x24e60c  030040e3           movt     r0, #3
0x24e610  010001e5           str      r0, [r1, #-1]
0x24e614  5af07ff5           dmb      ishst
0x24e618  03c081e2           add      ip, r1, #3
0x24e61c  014b8ced           vstr     d4, [ip, #4]
0x24e620  0200a0e1           mov      r0, r2
0x24e624  0120a0e3           mov      r2, #1
0x24e628  0030a0e3           mov      r3, #0
0x24e62c  3d0000eb           bl       #0x24e728
0x24e630  10101be5           ldr      r1, [fp, #-0x10]
0x24e634  1f9081e2           add      sb, r1, #0x1f
0x24e638  000089e5           str      r0, [sb]
0x24e63c  010010e3           tst      r0, #1
0x24e640  0500000a           beq      #0x24e65c
0x24e644  01c051e5           ldrb     ip, [r1, #-1]
0x24e648  01e050e5           ldrb     lr, [r0, #-1]
0x24e64c  2cc10ee0           and      ip, lr, ip, lsr #2
0x24e650  28e09ae5           ldr      lr, [sl, #0x28]
0x24e654  0e001ce1           tst      ip, lr
0x24e658  9422031b           blne     #0x3170b0
0x24e65c  10201be5           ldr      r2, [fp, #-0x10]
0x24e660  231082e2           add      r1, r2, #0x23
0x24e664  05ca85e2           add      ip, r5, #0x5000
0x24e668  27c09ce5           ldr      ip, [ip, #0x27]  # pool[5128] = "% nested=inner-"
0x24e66c  00c081e5           str      ip, [r1]
0x24e670  0210a0e1           mov      r1, r2
0x24e674  04001be5           ldr      r0, [fp, #-4]
0x24e678  279081e2           add      sb, r1, #0x27
0x24e67c  000089e5           str      r0, [sb]
0x24e680  010010e3           tst      r0, #1
0x24e684  0500000a           beq      #0x24e6a0
0x24e688  01c051e5           ldrb     ip, [r1, #-1]
0x24e68c  01e050e5           ldrb     lr, [r0, #-1]
0x24e690  2cc10ee0           and      ip, lr, ip, lsr #2
0x24e694  28e09ae5           ldr      lr, [sl, #0x28]
0x24e698  0e001ce1           tst      ip, lr
0x24e69c  8322031b           blne     #0x3170b0
0x24e6a0  2b1082e2           add      r1, r2, #0x2b
0x24e6a4  05ca85e2           add      ip, r5, #0x5000
0x24e6a8  2bc09ce5           ldr      ip, [ip, #0x2b]  # pool[5129] = " bool="
0x24e6ac  00c081e5           str      ip, [r1]
0x24e6b0  0c001be5           ldr      r0, [fp, #-0xc]
0x24e6b4  08101be5           ldr      r1, [fp, #-8]
0x24e6b8  000051e3           cmp      r1, #0
0x24e6bc  040000ca           bgt      #0x24e6d4
0x24e6c0  010000ba           blt      #0x24e6cc
0x24e6c4  0a0050e3           cmp      r0, #0xa
0x24e6c8  0100008a           bhi      #0x24e6d4
0x24e6cc  4c309ae5           ldr      r3, [sl, #0x4c]
0x24e6d0  000000ea           b        #0x24e6d8
0x24e6d4  48309ae5           ldr      r3, [sl, #0x48]
0x24e6d8  2f1082e2           add      r1, r2, #0x2f
0x24e6dc  003081e5           str      r3, [r1]
0x24e6e0  331082e2           add      r1, r2, #0x33
0x24e6e4  05ca85e2           add      ip, r5, #0x5000
0x24e6e8  2fc09ce5           ldr      ip, [ip, #0x2f]  # pool[5130] = " nullish="
0x24e6ec  00c081e5           str      ip, [r1]
0x24e6f0  371082e2           add      r1, r2, #0x37
0x24e6f4  40c09ae5           ldr      ip, [sl, #0x40]
0x24e6f8  00c081e5           str      ip, [r1]
0x24e6fc  00208de5           str      r2, [sp]
0x24e700  d017fceb           bl       #0x154648
0x24e704  00d04be2           sub      sp, fp, #0
0x24e708  0088bde8           pop      {fp, pc}
0x24e70c  044b2ded           vpush    {d4, d5}
0x24e710  54012de9           push     {r2, r4, r6, r8}
0x24e714  342903eb           bl       #0x318bec
0x24e718  0010a0e1           mov      r1, r0
0x24e71c  5401bde8           pop      {r2, r4, r6, r8}
0x24e720  044bbdec           vpop     {d4, d5}
0x24e724  bbffffea           b        #0x24e618
# CFG: 0x24e4f8->0x24e588/ConditionalFalse 0x24e4f8->0x24e594/ConditionalTrue 0x24e588->0x24e594/Fallthrough 0x24e594->0x24e5a8/ConditionalFalse 0x24e594->0x24e5c0/ConditionalTrue 0x24e5a8->0x24e5c0/Fallthrough 0x24e5c0->0x24e600/ConditionalFalse 0x24e5c0->0x24e70c/ConditionalTrue 0x24e600->0x24e618/Fallthrough 0x24e618->0x24e644/ConditionalFalse 0x24e618->0x24e65c/ConditionalTrue 0x24e644->0x24e65c/Fallthrough 0x24e65c->0x24e688/ConditionalFalse 0x24e65c->0x24e6a0/ConditionalTrue 0x24e688->0x24e6a0/Fallthrough 0x24e6a0->0x24e6c0/ConditionalFalse 0x24e6a0->0x24e6d4/ConditionalTrue 0x24e6c0->0x24e6c4/ConditionalFalse 0x24e6c0->0x24e6cc/ConditionalTrue 0x24e6c4->0x24e6cc/ConditionalFalse 0x24e6c4->0x24e6d4/ConditionalTrue 0x24e6cc->0x24e6d8/Branch 0x24e6d4->0x24e6d8/Fallthrough 0x24e70c->0x24e618/Branch

# top_level.seedNow at 0x24e850 (56 bytes)
0x24e850  00482de9           push     {fp, lr}
0x24e854  00b08de2           add      fp, sp, #0
0x24e858  24c09ae5           ldr      ip, [sl, #0x24]
0x24e85c  0c005de1           cmp      sp, ip
0x24e860  5529039b           blls     #0x318dbc
0x24e864  d1f9ffeb           bl       #0x24cfb0
0x24e868  c01fa0e1           asr      r1, r0, #0x1f
0x24e86c  c020b0e1           asrs     r2, r0, #1
0x24e870  0100003a           blo      #0x24e87c
0x24e874  072090e5           ldr      r2, [r0, #7]
0x24e878  0b1090e5           ldr      r1, [r0, #0xb]
0x24e87c  0200a0e1           mov      r0, r2
0x24e880  00d04be2           sub      sp, fp, #0
0x24e884  0088bde8           pop      {fp, pc}
# CFG: 0x24e850->0x24e874/ConditionalFalse 0x24e850->0x24e87c/ConditionalTrue 0x24e874->0x24e87c/Fallthrough

# top_level.main at 0x24e888 (36 bytes)
0x24e888  00482de9           push     {fp, lr}
0x24e88c  00b08de2           add      fp, sp, #0
0x24e890  24c09ae5           ldr      ip, [sl, #0x24]
0x24e894  0c005de1           cmp      sp, ip
0x24e898  4729039b           blls     #0x318dbc
0x24e89c  020000eb           bl       #0x24e8ac
0x24e8a0  40009ae5           ldr      r0, [sl, #0x40]
0x24e8a4  00d04be2           sub      sp, fp, #0
0x24e8a8  0088bde8           pop      {fp, pc}

# ProbeApp.<anonymous closure> at 0x274274 (96 bytes)
0x274274  00482de9           push     {fp, lr}
0x274278  00b08de2           add      fp, sp, #0
0x27427c  08d04de2           sub      sp, sp, #8
0x274280  08009be5           ldr      r0, [fp, #8]
0x274284  133090e5           ldr      r3, [r0, #0x13]
0x274288  04300be5           str      r3, [fp, #-4]
0x27428c  24c09ae5           ldr      ip, [sl, #0x24]
0x274290  0c005de1           cmp      sp, ip
0x274294  c892029b           blls     #0x318dbc
0x274298  40109ae5           ldr      r1, [sl, #0x40]
0x27429c  0420a0e3           mov      r2, #4
0x2742a0  819202eb           bl       #0x318cac
0x2742a4  0b2080e2           add      r2, r0, #0xb
0x2742a8  01c985e2           add      ip, r5, #0x4000
0x2742ac  cbcd9ce5           ldr      ip, [ip, #0xdcb]  # pool[4977] = snapshotRef(576)
0x2742b0  00c082e5           str      ip, [r2]
0x2742b4  04101be5           ldr      r1, [fp, #-4]
0x2742b8  0b2091e5           ldr      r2, [r1, #0xb]
0x2742bc  0f3080e2           add      r3, r0, #0xf
0x2742c0  002083e5           str      r2, [r3]
0x2742c4  00008de5           str      r0, [sp]
0x2742c8  de80fbeb           bl       #0x154648
0x2742cc  00d04be2           sub      sp, fp, #0
0x2742d0  0088bde8           pop      {fp, pc}

# ProbeApp.<anonymous closure> at 0x2742d4 (124 bytes)
0x2742d4  00482de9           push     {fp, lr}
0x2742d8  00b08de2           add      fp, sp, #0
0x2742dc  10d04de2           sub      sp, sp, #0x10
0x2742e0  10009be5           ldr      r0, [fp, #0x10]
0x2742e4  131090e5           ldr      r1, [r0, #0x13]
0x2742e8  04100be5           str      r1, [fp, #-4]
0x2742ec  24c09ae5           ldr      ip, [sl, #0x24]
0x2742f0  0c005de1           cmp      sp, ip
0x2742f4  b092029b           blls     #0x318dbc
0x2742f8  08009be5           ldr      r0, [fp, #8]
0x2742fc  0b2090e5           ldr      r2, [r0, #0xb]
0x274300  40c09ae5           ldr      ip, [sl, #0x40]
0x274304  0c0052e1           cmp      r2, ip
0x274308  0100001a           bne      #0x274314
0x27430c  0000a0e3           mov      r0, #0
0x274310  000000ea           b        #0x274318
0x274314  0200a0e1           mov      r0, r2
0x274318  00008de5           str      r0, [sp]
0x27431c  b081fbeb           bl       #0x1549e4
0x274320  0010a0e1           mov      r1, r0
0x274324  04001be5           ldr      r0, [fp, #-4]
0x274328  0c100be5           str      r1, [fp, #-0xc]
0x27432c  0f2090e5           ldr      r2, [r0, #0xf]
0x274330  08200be5           str      r2, [fp, #-8]
0x274334  6c68ffeb           bl       #0x24e4ec
0x274338  0c101be5           ldr      r1, [fp, #-0xc]
0x27433c  071080e5           str      r1, [r0, #7]
0x274340  08101be5           ldr      r1, [fp, #-8]
0x274344  2f1080e5           str      r1, [r0, #0x2f]
0x274348  00d04be2           sub      sp, fp, #0
0x27434c  0088bde8           pop      {fp, pc}
# CFG: 0x2742d4->0x27430c/ConditionalFalse 0x2742d4->0x274314/ConditionalTrue 0x27430c->0x274318/Branch 0x274314->0x274318/Fallthrough

# E13Dynamic.noSuchMethod at 0x27a578 (232 bytes)
0x27a578  00482de9           push     {fp, lr}
0x27a57c  00b08de2           add      fp, sp, #0
0x27a580  08d04de2           sub      sp, sp, #8
0x27a584  24c09ae5           ldr      ip, [sl, #0x24]
0x27a588  0c005de1           cmp      sp, ip
0x27a58c  0a7a029b           blls     #0x318dbc
0x27a590  40109ae5           ldr      r1, [sl, #0x40]
0x27a594  0820a0e3           mov      r2, #8
0x27a598  c37902eb           bl       #0x318cac
0x27a59c  04000be5           str      r0, [fp, #-4]
0x27a5a0  0b2080e2           add      r2, r0, #0xb
0x27a5a4  05ca85e2           add      ip, r5, #0x5000
0x27a5a8  f3c49ce5           ldr      ip, [ip, #0x4f3]  # pool[5435] = "unhandled:"
0x27a5ac  00c082e5           str      ip, [r2]
0x27a5b0  08109be5           ldr      r1, [fp, #8]
0x27a5b4  1c0100eb           bl       #0x27aa2c
0x27a5b8  04101be5           ldr      r1, [fp, #-4]
0x27a5bc  0f9081e2           add      sb, r1, #0xf
0x27a5c0  000089e5           str      r0, [sb]
0x27a5c4  010010e3           tst      r0, #1
0x27a5c8  0500000a           beq      #0x27a5e4
0x27a5cc  01c051e5           ldrb     ip, [r1, #-1]
0x27a5d0  01e050e5           ldrb     lr, [r0, #-1]
0x27a5d4  2cc10ee0           and      ip, lr, ip, lsr #2
0x27a5d8  28e09ae5           ldr      lr, [sl, #0x28]
0x27a5dc  0e001ce1           tst      ip, lr
0x27a5e0  b272021b           blne     #0x3170b0
0x27a5e4  04001be5           ldr      r0, [fp, #-4]
0x27a5e8  132080e2           add      r2, r0, #0x13
0x27a5ec  01ca85e2           add      ip, r5, #0x1000
0x27a5f0  1fc29ce5           ldr      ip, [ip, #0x21f]  # pool[1158] = snapshotRef(244)
0x27a5f4  00c082e5           str      ip, [r2]
0x27a5f8  08109be5           ldr      r1, [fp, #8]
0x27a5fc  170000eb           bl       #0x27a660
0x27a600  011010e5           ldr      r1, [r0, #-1]
0x27a604  5116f3e7           ubfx     r1, r1, #0xc, #0x14
0x27a608  00008de5           str      r0, [sp]
0x27a60c  0100a0e1           mov      r0, r1
0x27a610  00e187e0           add      lr, r7, r0, lsl #2
0x27a614  0fea8ee2           add      lr, lr, #0xf000
0x27a618  c4e29ee5           ldr      lr, [lr, #0x2c4]
0x27a61c  3eff2fe1           blx      lr
0x27a620  04101be5           ldr      r1, [fp, #-4]
0x27a624  179081e2           add      sb, r1, #0x17
0x27a628  000089e5           str      r0, [sb]
0x27a62c  010010e3           tst      r0, #1
0x27a630  0500000a           beq      #0x27a64c
0x27a634  01c051e5           ldrb     ip, [r1, #-1]
0x27a638  01e050e5           ldrb     lr, [r0, #-1]
0x27a63c  2cc10ee0           and      ip, lr, ip, lsr #2
0x27a640  28e09ae5           ldr      lr, [sl, #0x28]
0x27a644  0e001ce1           tst      ip, lr
0x27a648  9872021b           blne     #0x3170b0
0x27a64c  04e01be5           ldr      lr, [fp, #-4]
0x27a650  00e08de5           str      lr, [sp]
0x27a654  fb67fbeb           bl       #0x154648
0x27a658  00d04be2           sub      sp, fp, #0
0x27a65c  0088bde8           pop      {fp, pc}
# CFG: 0x27a578->0x27a5cc/ConditionalFalse 0x27a578->0x27a5e4/ConditionalTrue 0x27a5cc->0x27a5e4/Fallthrough 0x27a5e4->0x27a634/ConditionalFalse 0x27a5e4->0x27a64c/ConditionalTrue 0x27a634->0x27a64c/Fallthrough

# E15Vec.get:hashCode at 0x2c0800 (76 bytes)
0x2c0800  00209de5           ldr      r2, [sp]
0x2c0804  034092e5           ldr      r4, [r2, #3]
0x2c0808  073092e5           ldr      r3, [r2, #7]
0x2c080c  0b8092e5           ldr      r8, [r2, #0xb]
0x2c0810  0f6092e5           ldr      r6, [r2, #0xf]
0x2c0814  089024e0           eor      sb, r4, r8
0x2c0818  062023e0           eor      r2, r3, r6
0x2c081c  8900a0e1           lsl      r0, sb, #1
0x2c0820  c00059e1           cmp      sb, r0, asr #1
0x2c0824  c00f5201           cmpeq    r2, r0, asr #31
0x2c0828  0600000a           beq      #0x2c0848
0x2c082c  00482de9           push     {fp, lr}
0x2c0830  00b08de2           add      fp, sp, #0
0x2c0834  8c6101eb           bl       #0x318e6c
0x2c0838  00d04be2           sub      sp, fp, #0
0x2c083c  0048bde8           pop      {fp, lr}
0x2c0840  079080e5           str      sb, [r0, #7]
0x2c0844  0b2080e5           str      r2, [r0, #0xb]
0x2c0848  1eff2fe1           bx       lr
# CFG: 0x2c0800->0x2c082c/ConditionalFalse 0x2c0800->0x2c0848/ConditionalTrue 0x2c082c->0x2c0848/Fallthrough

# E21Mode._enumToString at 0x2c74f8 (92 bytes)
0x2c74f8  00482de9           push     {fp, lr}
0x2c74fc  00b08de2           add      fp, sp, #0
0x2c7500  08d04de2           sub      sp, sp, #8
0x2c7504  0100a0e1           mov      r0, r1
0x2c7508  04100be5           str      r1, [fp, #-4]
0x2c750c  24c09ae5           ldr      ip, [sl, #0x24]
0x2c7510  0c005de1           cmp      sp, ip
0x2c7514  2846019b           blls     #0x318dbc
0x2c7518  40109ae5           ldr      r1, [sl, #0x40]
0x2c751c  0420a0e3           mov      r2, #4
0x2c7520  e14501eb           bl       #0x318cac
0x2c7524  0b2080e2           add      r2, r0, #0xb
0x2c7528  05ca85e2           add      ip, r5, #0x5000
0x2c752c  efc49ce5           ldr      ip, [ip, #0x4ef]  # pool[5434] = "E21Mode."
0x2c7530  00c082e5           str      ip, [r2]
0x2c7534  04101be5           ldr      r1, [fp, #-4]
0x2c7538  0b2091e5           ldr      r2, [r1, #0xb]
0x2c753c  0f3080e2           add      r3, r0, #0xf
0x2c7540  002083e5           str      r2, [r3]
0x2c7544  00008de5           str      r0, [sp]
0x2c7548  3e34faeb           bl       #0x154648
0x2c754c  00d04be2           sub      sp, fp, #0
0x2c7550  0088bde8           pop      {fp, pc}

# E15Vec.== at 0x2f0458 (100 bytes)
0x2f0458  00109de5           ldr      r1, [sp]
0x2f045c  40c09ae5           ldr      ip, [sl, #0x40]
0x2f0460  0c0051e1           cmp      r1, ip
0x2f0464  0100001a           bne      #0x2f0470
0x2f0468  4c009ae5           ldr      r0, [sl, #0x4c]
0x2f046c  1eff2fe1           bx       lr
0x2f0470  010011e3           tst      r1, #1
0x2f0474  01201115           ldrne    r2, [r1, #-1]
0x2f0478  5226f317           ubfxne   r2, r2, #0xc, #0x14
0x2f047c  3c20a003           moveq    r2, #0x3c
0x2f0480  bd0f52e3           cmp      r2, #0x2f4
0x2f0484  0a00001a           bne      #0x2f04b4
0x2f0488  04209de5           ldr      r2, [sp, #4]
0x2f048c  034092e5           ldr      r4, [r2, #3]
0x2f0490  073092e5           ldr      r3, [r2, #7]
0x2f0494  036091e5           ldr      r6, [r1, #3]
0x2f0498  072091e5           ldr      r2, [r1, #7]
0x2f049c  060054e1           cmp      r4, r6
0x2f04a0  02005301           cmpeq    r3, r2
0x2f04a4  48109a05           ldreq    r1, [sl, #0x48]
0x2f04a8  4c109a15           ldrne    r1, [sl, #0x4c]
0x2f04ac  0100a0e1           mov      r0, r1
0x2f04b0  000000ea           b        #0x2f04b8
0x2f04b4  4c009ae5           ldr      r0, [sl, #0x4c]
0x2f04b8  1eff2fe1           bx       lr
# CFG: 0x2f0458->0x2f0468/ConditionalFalse 0x2f0458->0x2f0470/ConditionalTrue 0x2f0470->0x2f0488/ConditionalFalse 0x2f0470->0x2f04b4/ConditionalTrue 0x2f0488->0x2f04b8/Branch 0x2f04b4->0x2f04b8/Fallthrough
