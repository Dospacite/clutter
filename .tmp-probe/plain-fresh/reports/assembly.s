# Complete decoded machine-code evidence. Generated source intentionally omits this noise.

# ShopDemoApp.build at 0x2a3ce0 (208 bytes)
0x2a3ce0  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x2a3ce4  fd030faa           mov      x29, x15
0x2a3ce8  ef4100d1           sub      x15, x15, #0x10
0x2a3cec  502740f9           ldr      x16, [x26, #0x48]
0x2a3cf0  ff0110eb           cmp      x15, x16
0x2a3cf4  a9050054           b.ls     #0x2a3da8
0x2a3cf8  e10316aa           mov      x1, x22
0x2a3cfc  622f4091           add      x2, x27, #0xb, lsl #12
0x2a3d00  420843f9           ldr      x2, [x2, #0x610]  # pool[5824] = snapshotInstance(MaterialColor)
0x2a3d04  64a340f9           ldr      x4, [x27, #0x140]  # pool[38] = snapshotRef(48150)
0x2a3d08  a497fd97           bl       #0x209b98
0x2a3d0c  e00100f9           str      x0, [x15]
0x2a3d10  e10316aa           mov      x1, x22
0x2a3d14  64bb70f9           ldr      x4, [x27, #0x6170]  # pool[3116] = snapshotRef(48234) nestedStrings["colorScheme"]
0x2a3d18  158efd97           bl       #0x20756c
0x2a3d1c  a0831ff8           stur     x0, [x29, #-8]
0x2a3d20  24000094           bl       #0x2a3db0
0x2a3d24  612f4091           add      x1, x27, #0xb, lsl #12
0x2a3d28  210c43f9           ldr      x1, [x1, #0x618]  # pool[5825] = snapshotInstance(CatalogPage)
0x2a3d2c  013001b8           stur     w1, [x0, #0x13]
0x2a3d30  612f4091           add      x1, x27, #0xb, lsl #12
0x2a3d34  211043f9           ldr      x1, [x1, #0x620]  # pool[5826] = snapshotRef(48094)
0x2a3d38  017001b8           stur     w1, [x0, #0x17]
0x2a3d3c  612f4091           add      x1, x27, #0xb, lsl #12
0x2a3d40  211443f9           ldr      x1, [x1, #0x628]  # pool[5827] = snapshotRef(48334)
0x2a3d44  01f002b8           stur     w1, [x0, #0x2f]
0x2a3d48  612f4091           add      x1, x27, #0xb, lsl #12
0x2a3d4c  211843f9           ldr      x1, [x1, #0x630]  # pool[5828] = "Clutter Shop Demo"
0x2a3d50  01f003b8           stur     w1, [x0, #0x3f]
0x2a3d54  a1835ff8           ldur     x1, [x29, #-8]
0x2a3d58  017004b8           stur     w1, [x0, #0x47]
0x2a3d5c  612f4091           add      x1, x27, #0xb, lsl #12
0x2a3d60  211c43f9           ldr      x1, [x1, #0x638]  # pool[5829] = snapshotInstance(ThemeMode)
0x2a3d64  017005b8           stur     w1, [x0, #0x57]
0x2a3d68  612f4091           add      x1, x27, #0xb, lsl #12
0x2a3d6c  212043f9           ldr      x1, [x1, #0x640]  # pool[5830] = snapshotInstance(Duration)
0x2a3d70  01b005b8           stur     w1, [x0, #0x5b]
0x2a3d74  618b6ef9           ldr      x1, [x27, #0x5d10]  # pool[2976] = snapshotInstance(_Linear)
0x2a3d78  01f005b8           stur     w1, [x0, #0x5f]
0x2a3d7c  612f4091           add      x1, x27, #0xb, lsl #12
0x2a3d80  212443f9           ldr      x1, [x1, #0x648]  # pool[5831] = snapshotRef(48246) nestedStrings["US", "en"]
0x2a3d84  017007b8           stur     w1, [x0, #0x77]
0x2a3d88  c1c20091           add      x1, x22, #0x30
0x2a3d8c  01b007b8           stur     w1, [x0, #0x7b]
0x2a3d90  01f007b8           stur     w1, [x0, #0x7f]
0x2a3d94  c1820091           add      x1, x22, #0x20
0x2a3d98  013008b8           stur     w1, [x0, #0x83]
0x2a3d9c  ef031daa           mov      x15, x29
0x2a3da0  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x2a3da4  c0035fd6           ret      
0x2a3da8  a0a90594           bl       #0x40e428
0x2a3dac  d3ffff17           b        #0x2a3cf8
# CFG: 0x2a3ce0->0x2a3cf8/ConditionalFalse 0x2a3ce0->0x2a3da8/ConditionalTrue 0x2a3da8->0x2a3cf8/Branch

# _CatalogPageState.build at 0x2fe714 (1548 bytes)
0x2fe714  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x2fe718  fd030faa           mov      x29, x15
0x2fe71c  efa101d1           sub      x15, x15, #0x68
0x2fe720  e00301aa           mov      x0, x1
0x2fe724  a1831ff8           stur     x1, [x29, #-8]
0x2fe728  e10302aa           mov      x1, x2
0x2fe72c  a2031ff8           stur     x2, [x29, #-0x10]
0x2fe730  502740f9           ldr      x16, [x26, #0x48]
0x2fe734  ff0110eb           cmp      x15, x16
0x2fe738  092f0054           b.ls     #0x2fed18
0x2fe73c  410080d2           mov      x1, #2
0x2fe740  bb3a0494           bl       #0x40d22c
0x2fe744  e20300aa           mov      x2, x0
0x2fe748  a0835ff8           ldur     x0, [x29, #-8]
0x2fe74c  a2831ef8           stur     x2, [x29, #-0x18]
0x2fe750  40f000b8           stur     w0, [x2, #0xf]
0x2fe754  e10300aa           mov      x1, x0
0x2fe758  e3020094           bl       #0x2ff2e4
0x2fe75c  e40300aa           mov      x4, x0
0x2fe760  a3835ef8           ldur     x3, [x29, #-0x18]
0x2fe764  a4031ef8           stur     x4, [x29, #-0x20]
0x2fe768  603001b8           stur     w0, [x3, #0x13]
0x2fe76c  70f05f38           ldurb    w16, [x3, #-1]
0x2fe770  11f05f38           ldurb    w17, [x0, #-1]
0x2fe774  300a508a           and      x16, x17, x16, lsr #2
0x2fe778  1f825cea           tst      x16, x28, lsr #32
0x2fe77c  40000054           b.eq     #0x2fe784
0x2fe780  92380494           bl       #0x40c9c8
0x2fe784  612f4091           add      x1, x27, #0xb, lsl #12
0x2fe788  21cc47f9           ldr      x1, [x1, #0xf98]  # pool[6129] = _CatalogPageState.<anonymous closure>
0x2fe78c  e20316aa           mov      x2, x22
0x2fe790  9c3b0494           bl       #0x40d600
0x2fe794  702f4091           add      x16, x27, #0xb, lsl #12
0x2fe798  10d247f9           ldr      x16, [x16, #0xfa0]  # pool[6130] = snapshotRef(23385)
0x2fe79c  be035ef8           ldur     x30, [x29, #-0x20]
0x2fe7a0  fec100a9           stp      x30, x16, [x15, #8]
0x2fe7a4  e00100f9           str      x0, [x15]
0x2fe7a8  642340f9           ldr      x4, [x27, #0x40]  # pool[6] = snapshotRef(55)
0x2fe7ac  96020094           bl       #0x2ff204
0x2fe7b0  a0831df8           stur     x0, [x29, #-0x28]
0x2fe7b4  91020094           bl       #0x2ff1f8
0x2fe7b8  e10300aa           mov      x1, x0
0x2fe7bc  a0031df8           stur     x0, [x29, #-0x30]
0x2fe7c0  66020094           bl       #0x2ff158
0x2fe7c4  60020094           bl       #0x2ff144
0x2fe7c8  e30300aa           mov      x3, x0
0x2fe7cc  604b79f9           ldr      x0, [x27, #0x7290]  # pool[3664] = snapshotRef(19935)
0x2fe7d0  a3831cf8           stur     x3, [x29, #-0x38]
0x2fe7d4  60f000b8           stur     w0, [x3, #0xf]
0x2fe7d8  602f4091           add      x0, x27, #0xb, lsl #12
0x2fe7dc  00d447f9           ldr      x0, [x0, #0xfa8]  # pool[6131] = snapshotInstance(InputDecoration)
0x2fe7e0  60b001b8           stur     w0, [x3, #0x1b]
0x2fe7e4  60db7cf9           ldr      x0, [x27, #0x79b0]  # pool[3892] = snapshotInstance(TextCapitalization)
0x2fe7e8  607002b8           stur     w0, [x3, #0x27]
0x2fe7ec  607b6bf9           ldr      x0, [x27, #0x56f0]  # pool[2780] = snapshotInstance(TextAlign)
0x2fe7f0  603003b8           stur     w0, [x3, #0x33]
0x2fe7f4  c0c20091           add      x0, x22, #0x30
0x2fe7f8  60f006b8           stur     w0, [x3, #0x6f]
0x2fe7fc  60f003b8           stur     w0, [x3, #0x3f]
0x2fe800  612f4091           add      x1, x27, #0xb, lsl #12
0x2fe804  21d847f9           ldr      x1, [x1, #0xfb0]  # pool[6132] = "•"
0x2fe808  617004b8           stur     w1, [x3, #0x47]
0x2fe80c  60b004b8           stur     w0, [x3, #0x4b]
0x2fe810  c4820091           add      x4, x22, #0x20
0x2fe814  64b005b8           stur     w4, [x3, #0x5b]
0x2fe818  250080d2           mov      x5, #1
0x2fe81c  65f005f8           stur     x5, [x3, #0x5f]
0x2fe820  60b006b8           stur     w0, [x3, #0x6b]
0x2fe824  a2835ef8           ldur     x2, [x29, #-0x18]
0x2fe828  612f4091           add      x1, x27, #0xb, lsl #12
0x2fe82c  21dc47f9           ldr      x1, [x1, #0xfb8]  # pool[6133] = _CatalogPageState.<anonymous closure>
0x2fe830  743b0494           bl       #0x40d600
0x2fe834  e10300aa           mov      x1, x0
0x2fe838  a0835cf8           ldur     x0, [x29, #-0x38]
0x2fe83c  013008b8           stur     w1, [x0, #0x83]
0x2fe840  0010601e           fmov     d0, #2.00000000
0x2fe844  00f009fc           stur     d0, [x0, #0x9f]
0x2fe848  61b779f9           ldr      x1, [x27, #0x7368]  # pool[3691] = snapshotInstance(EdgeInsets)
0x2fe84c  01300cb8           stur     w1, [x0, #0xc3]
0x2fe850  614379f9           ldr      x1, [x27, #0x7280]  # pool[3662] = snapshotInstance(DragStartBehavior)
0x2fe854  01300db8           stur     w1, [x0, #0xd3]
0x2fe858  c1c20091           add      x1, x22, #0x30
0x2fe85c  01b00db8           stur     w1, [x0, #0xdb]
0x2fe860  62d77cf9           ldr      x2, [x27, #0x79a8]  # pool[3891] = snapshotRef(48280)
0x2fe864  02700fb8           stur     w2, [x0, #0xf7]
0x2fe868  622f4091           add      x2, x27, #0xb, lsl #12
0x2fe86c  42e445f9           ldr      x2, [x2, #0xbc8]  # pool[6007] = snapshotInstance(Clip)
0x2fe870  02b00fb8           stur     w2, [x0, #0xfb]
0x2fe874  c2820091           add      x2, x22, #0x20
0x2fe878  712080d2           mov      x17, #0x103
0x2fe87c  026831b8           str      w2, [x0, x17]
0x2fe880  f12080d2           mov      x17, #0x107
0x2fe884  026831b8           str      w2, [x0, x17]
0x2fe888  712180d2           mov      x17, #0x10b
0x2fe88c  026831b8           str      w2, [x0, x17]
0x2fe890  639f7af9           ldr      x3, [x27, #0x7538]  # pool[3749] = snapshotRef(46906)
0x2fe894  f12280d2           mov      x17, #0x117
0x2fe898  036831b8           str      w3, [x0, x17]
0x2fe89c  712380d2           mov      x17, #0x11b
0x2fe8a0  026831b8           str      w2, [x0, x17]
0x2fe8a4  63df7cf9           ldr      x3, [x27, #0x79b8]  # pool[3893] = snapshotInstance(SmartDashesType)
0x2fe8a8  033005b8           stur     w3, [x0, #0x53]
0x2fe8ac  632f4091           add      x3, x27, #0xb, lsl #12
0x2fe8b0  63e047f9           ldr      x3, [x3, #0xfc0]  # pool[6134] = snapshotInstance(SmartQuotesType)
0x2fe8b4  037005b8           stur     w3, [x0, #0x57]
0x2fe8b8  632f4091           add      x3, x27, #0xb, lsl #12
0x2fe8bc  63e447f9           ldr      x3, [x3, #0xfc8]  # pool[6135] = snapshotInstance(TextInputType)
0x2fe8c0  03f001b8           stur     w3, [x0, #0x1f]
0x2fe8c4  02700cb8           stur     w2, [x0, #0xc7]
0x2fe8c8  b85bfe97           bl       #0x2957a8
0x2fe8cc  e20300aa           mov      x2, x0
0x2fe8d0  602f4091           add      x0, x27, #0xb, lsl #12
0x2fe8d4  00e847f9           ldr      x0, [x0, #0xfd0]  # pool[6136] = snapshotInstance(EdgeInsets)
0x2fe8d8  a2031cf8           stur     x2, [x29, #-0x40]
0x2fe8dc  40f000b8           stur     w0, [x2, #0xf]
0x2fe8e0  a0835cf8           ldur     x0, [x29, #-0x38]
0x2fe8e4  40b000b8           stur     w0, [x2, #0xb]
0x2fe8e8  a0835ff8           ldur     x0, [x29, #-8]
0x2fe8ec  03b041b8           ldur     w3, [x0, #0x1b]
0x2fe8f0  63801c8b           add      x3, x3, x28, lsl #32
0x2fe8f4  a1035ff8           ldur     x1, [x29, #-0x10]
0x2fe8f8  a3831cf8           stur     x3, [x29, #-0x38]
0x2fe8fc  ef19fc97           bl       #0x2050b8
0x2fe900  017048b8           ldur     w1, [x0, #0x87]
0x2fe904  21801c8b           add      x1, x1, x28, lsl #32
0x2fe908  203042b8           ldur     w0, [x1, #0x23]
0x2fe90c  00801c8b           add      x0, x0, x28, lsl #32
0x2fe910  a0031ff8           stur     x0, [x29, #-0x10]
0x2fe914  826efe97           bl       #0x29a31c
0x2fe918  e30300aa           mov      x3, x0
0x2fe91c  a0835cf8           ldur     x0, [x29, #-0x38]
0x2fe920  a3831bf8           stur     x3, [x29, #-0x48]
0x2fe924  60b000b8           stur     w0, [x3, #0xb]
0x2fe928  a0035ff8           ldur     x0, [x29, #-0x10]
0x2fe92c  603001b8           stur     w0, [x3, #0x13]
0x2fe930  e10316aa           mov      x1, x22
0x2fe934  820080d2           mov      x2, #4
0x2fe938  793e0494           bl       #0x40e31c
0x2fe93c  e20300aa           mov      x2, x0
0x2fe940  a0035cf8           ldur     x0, [x29, #-0x40]
0x2fe944  a2031ff8           stur     x2, [x29, #-0x10]
0x2fe948  40f000b8           stur     w0, [x2, #0xf]
0x2fe94c  a0835bf8           ldur     x0, [x29, #-0x48]
0x2fe950  403001b8           stur     w0, [x2, #0x13]
0x2fe954  611370f9           ldr      x1, [x27, #0x6020]  # pool[3074] = snapshotRef(23352)
0x2fe958  253a0494           bl       #0x40d1ec
0x2fe95c  e30300aa           mov      x3, x0
0x2fe960  a0035ff8           ldur     x0, [x29, #-0x10]
0x2fe964  a3831cf8           stur     x3, [x29, #-0x38]
0x2fe968  60f000b8           stur     w0, [x3, #0xf]
0x2fe96c  800080d2           mov      x0, #4
0x2fe970  60b000b8           stur     w0, [x3, #0xb]
0x2fe974  a4835df8           ldur     x4, [x29, #-0x28]
0x2fe978  9f00166b           cmp      w4, w22
0x2fe97c  60080054           b.eq     #0x2fea88
0x2fe980  e10316aa           mov      x1, x22
0x2fe984  020180d2           mov      x2, #8
0x2fe988  653e0494           bl       #0x40e31c
0x2fe98c  a0031ff8           stur     x0, [x29, #-0x10]
0x2fe990  702f4091           add      x16, x27, #0xb, lsl #12
0x2fe994  10ee47f9           ldr      x16, [x16, #0xfd8]  # pool[6137] = "Deal: "
0x2fe998  10f000b8           stur     w16, [x0, #0xf]
0x2fe99c  a1835df8           ldur     x1, [x29, #-0x28]
0x2fe9a0  22b040b8           ldur     w2, [x1, #0xb]
0x2fe9a4  42801c8b           add      x2, x2, x28, lsl #32
0x2fe9a8  023001b8           stur     w2, [x0, #0x13]
0x2fe9ac  702f4091           add      x16, x27, #0xb, lsl #12
0x2fe9b0  10f247f9           ldr      x16, [x16, #0xfe0]  # pool[6138] = " at "
0x2fe9b4  107001b8           stur     w16, [x0, #0x17]
0x2fe9b8  20f040fc           ldur     d0, [x1, #0xf]
0x2fe9bc  67010094           bl       #0x2fef58
0x2fe9c0  a1035ff8           ldur     x1, [x29, #-0x10]
0x2fe9c4  396c0091           add      x25, x1, #0x1b
0x2fe9c8  200300b9           str      w0, [x25]
0x2fe9cc  e0000036           tbz      w0, #0, #0x2fe9e8
0x2fe9d0  30f05f38           ldurb    w16, [x1, #-1]
0x2fe9d4  11f05f38           ldurb    w17, [x0, #-1]
0x2fe9d8  300a508a           and      x16, x17, x16, lsr #2
0x2fe9dc  1f825cea           tst      x16, x28, lsr #32
0x2fe9e0  40000054           b.eq     #0x2fe9e8
0x2fe9e4  d8360494           bl       #0x40c544
0x2fe9e8  b0035ff8           ldur     x16, [x29, #-0x10]
0x2fe9ec  f00100f9           str      x16, [x15]
0x2fe9f0  99befa97           bl       #0x1ae454
0x2fe9f4  a0031ff8           stur     x0, [x29, #-0x10]
0x2fe9f8  496efe97           bl       #0x29a31c
0x2fe9fc  e20300aa           mov      x2, x0
0x2fea00  a0035ff8           ldur     x0, [x29, #-0x10]
0x2fea04  a2831df8           stur     x2, [x29, #-0x28]
0x2fea08  40b000b8           stur     w0, [x2, #0xb]
0x2fea0c  a0835cf8           ldur     x0, [x29, #-0x38]
0x2fea10  01b040b8           ldur     w1, [x0, #0xb]
0x2fea14  03f040b8           ldur     w3, [x0, #0xf]
0x2fea18  63801c8b           add      x3, x3, x28, lsl #32
0x2fea1c  64b040b8           ldur     w4, [x3, #0xb]
0x2fea20  237c4193           sbfx     x3, x1, #1, #0x1f
0x2fea24  a3031bf8           stur     x3, [x29, #-0x50]
0x2fea28  817c4193           sbfx     x1, x4, #1, #0x1f
0x2fea2c  7f0001eb           cmp      x3, x1
0x2fea30  61000054           b.ne     #0x2fea3c
0x2fea34  e10300aa           mov      x1, x0
0x2fea38  23a1fa97           bl       #0x1a6ec4
0x2fea3c  a2835cf8           ldur     x2, [x29, #-0x38]
0x2fea40  a3035bf8           ldur     x3, [x29, #-0x50]
0x2fea44  60040091           add      x0, x3, #1
0x2fea48  01f87fd3           lsl      x1, x0, #1
0x2fea4c  41b000b8           stur     w1, [x2, #0xb]
0x2fea50  41f040b8           ldur     w1, [x2, #0xf]
0x2fea54  21801c8b           add      x1, x1, x28, lsl #32
0x2fea58  a0835df8           ldur     x0, [x29, #-0x28]
0x2fea5c  3908038b           add      x25, x1, x3, lsl #2
0x2fea60  393f0091           add      x25, x25, #0xf
0x2fea64  200300b9           str      w0, [x25]
0x2fea68  e0000036           tbz      w0, #0, #0x2fea84
0x2fea6c  30f05f38           ldurb    w16, [x1, #-1]
0x2fea70  11f05f38           ldurb    w17, [x0, #-1]
0x2fea74  300a508a           and      x16, x17, x16, lsr #2
0x2fea78  1f825cea           tst      x16, x28, lsr #32
0x2fea7c  40000054           b.eq     #0x2fea84
0x2fea80  b1360494           bl       #0x40c544
0x2fea84  02000014           b        #0x2fea8c
0x2fea88  e20303aa           mov      x2, x3
0x2fea8c  a0035ef8           ldur     x0, [x29, #-0x20]
0x2fea90  01f05ff8           ldur     x1, [x0, #-1]
0x2fea94  217c4cd3           ubfx     x1, x1, #0xc, #0x14
0x2fea98  e00100f9           str      x0, [x15]
0x2fea9c  e00301aa           mov      x0, x1
0x2feaa0  1e9836d1           sub      x30, x0, #0xda6
0x2feaa4  be7a7ef8           ldr      x30, [x21, x30, lsl #3]
0x2feaa8  c0033fd6           blr      x30
0x2feaac  037c4193           sbfx     x3, x0, #1, #0x1f
0x2feab0  a2835ef8           ldur     x2, [x29, #-0x18]
0x2feab4  a3031bf8           stur     x3, [x29, #-0x50]
0x2feab8  612f4091           add      x1, x27, #0xb, lsl #12
0x2feabc  21f447f9           ldr      x1, [x1, #0xfe8]  # pool[6139] = _CatalogPageState.<anonymous closure>
0x2feac0  d03a0494           bl       #0x40d600
0x2feac4  a0031ff8           stur     x0, [x29, #-0x10]
0x2feac8  21010094           bl       #0x2fef4c
0x2feacc  e10300aa           mov      x1, x0
0x2fead0  a2035ff8           ldur     x2, [x29, #-0x10]
0x2fead4  a3035bf8           ldur     x3, [x29, #-0x50]
0x2fead8  a0031ff8           stur     x0, [x29, #-0x10]
0x2feadc  e5000094           bl       #0x2fee70
0x2feae0  612f4091           add      x1, x27, #0xb, lsl #12
0x2feae4  21f847f9           ldr      x1, [x1, #0xff0]  # pool[6140] = snapshotRef(22966)
0x2feae8  df000094           bl       #0x2fee64
0x2feaec  e20300aa           mov      x2, x0
0x2feaf0  200080d2           mov      x0, #1
0x2feaf4  a2031ef8           stur     x2, [x29, #-0x20]
0x2feaf8  403001f8           stur     x0, [x2, #0x13]
0x2feafc  602f4091           add      x0, x27, #0xb, lsl #12
0x2feb00  00fc47f9           ldr      x0, [x0, #0xff8]  # pool[6141] = snapshotInstance(FlexFit)
0x2feb04  40b001b8           stur     w0, [x2, #0x1b]
0x2feb08  a0035ff8           ldur     x0, [x29, #-0x10]
0x2feb0c  40b000b8           stur     w0, [x2, #0xb]
0x2feb10  a0835cf8           ldur     x0, [x29, #-0x38]
0x2feb14  01b040b8           ldur     w1, [x0, #0xb]
0x2feb18  03f040b8           ldur     w3, [x0, #0xf]
0x2feb1c  63801c8b           add      x3, x3, x28, lsl #32
0x2feb20  64b040b8           ldur     w4, [x3, #0xb]
0x2feb24  237c4193           sbfx     x3, x1, #1, #0x1f
0x2feb28  a3031bf8           stur     x3, [x29, #-0x50]
0x2feb2c  817c4193           sbfx     x1, x4, #1, #0x1f
0x2feb30  7f0001eb           cmp      x3, x1
0x2feb34  61000054           b.ne     #0x2feb40
0x2feb38  e10300aa           mov      x1, x0
0x2feb3c  e2a0fa97           bl       #0x1a6ec4
0x2feb40  a4835ff8           ldur     x4, [x29, #-8]
0x2feb44  a5035df8           ldur     x5, [x29, #-0x30]
0x2feb48  a2835cf8           ldur     x2, [x29, #-0x38]
0x2feb4c  a3035bf8           ldur     x3, [x29, #-0x50]
0x2feb50  60040091           add      x0, x3, #1
0x2feb54  01f87fd3           lsl      x1, x0, #1
0x2feb58  41b000b8           stur     w1, [x2, #0xb]
0x2feb5c  41f040b8           ldur     w1, [x2, #0xf]
0x2feb60  21801c8b           add      x1, x1, x28, lsl #32
0x2feb64  a0035ef8           ldur     x0, [x29, #-0x20]
0x2feb68  3908038b           add      x25, x1, x3, lsl #2
0x2feb6c  393f0091           add      x25, x25, #0xf
0x2feb70  200300b9           str      w0, [x25]
0x2feb74  e0000036           tbz      w0, #0, #0x2feb90
0x2feb78  30f05f38           ldurb    w16, [x1, #-1]
0x2feb7c  11f05f38           ldurb    w17, [x0, #-1]
0x2feb80  300a508a           and      x16, x17, x16, lsr #2
0x2feb84  1f825cea           tst      x16, x28, lsr #32
0x2feb88  40000054           b.eq     #0x2feb90
0x2feb8c  6e360494           bl       #0x40c544
0x2feb90  e85ffe97           bl       #0x296b30
0x2feb94  e30300aa           mov      x3, x0
0x2feb98  60f37df9           ldr      x0, [x27, #0x7be0]  # pool[3962] = snapshotInstance(Axis)
0x2feb9c  a3031ff8           stur     x3, [x29, #-0x10]
0x2feba0  60f000b8           stur     w0, [x3, #0xf]
0x2feba4  602f4091           add      x0, x27, #0xb, lsl #12
0x2feba8  008845f9           ldr      x0, [x0, #0xb10]  # pool[5984] = snapshotInstance(MainAxisAlignment)
0x2febac  603001b8           stur     w0, [x3, #0x13]
0x2febb0  60334091           add      x0, x27, #0xc, lsl #12
0x2febb4  000040f9           ldr      x0, [x0]  # pool[6142] = snapshotInstance(MainAxisSize)
0x2febb8  607001b8           stur     w0, [x3, #0x17]
0x2febbc  602f4091           add      x0, x27, #0xb, lsl #12
0x2febc0  009045f9           ldr      x0, [x0, #0xb20]  # pool[5986] = snapshotInstance(CrossAxisAlignment)
0x2febc4  60b001b8           stur     w0, [x3, #0x1b]
0x2febc8  602f4091           add      x0, x27, #0xb, lsl #12
0x2febcc  009445f9           ldr      x0, [x0, #0xb28]  # pool[5987] = snapshotInstance(VerticalDirection)
0x2febd0  603002b8           stur     w0, [x3, #0x23]
0x2febd4  602f4091           add      x0, x27, #0xb, lsl #12
0x2febd8  009845f9           ldr      x0, [x0, #0xb30]  # pool[5988] = snapshotInstance(Clip)
0x2febdc  60b002b8           stur     w0, [x3, #0x2b]
0x2febe0  7ff002f8           stur     xzr, [x3, #0x2f]
0x2febe4  a1835cf8           ldur     x1, [x29, #-0x38]
0x2febe8  61b000b8           stur     w1, [x3, #0xb]
0x2febec  e10316aa           mov      x1, x22
0x2febf0  820080d2           mov      x2, #4
0x2febf4  ca3d0494           bl       #0x40e31c
0x2febf8  a0031ef8           stur     x0, [x29, #-0x20]
0x2febfc  70334091           add      x16, x27, #0xc, lsl #12
0x2fec00  100640f9           ldr      x16, [x16, #8]  # pool[6143] = "Pay "
0x2fec04  10f000b8           stur     w16, [x0, #0xf]
0x2fec08  a1835ff8           ldur     x1, [x29, #-8]
0x2fec0c  223041b8           ldur     w2, [x1, #0x13]
0x2fec10  42801c8b           add      x2, x2, x28, lsl #32
0x2fec14  e10302aa           mov      x1, x2
0x2fec18  48000094           bl       #0x2fed38
0x2fec1c  cf000094           bl       #0x2fef58
0x2fec20  a1035ef8           ldur     x1, [x29, #-0x20]
0x2fec24  394c0091           add      x25, x1, #0x13
0x2fec28  200300b9           str      w0, [x25]
0x2fec2c  e0000036           tbz      w0, #0, #0x2fec48
0x2fec30  30f05f38           ldurb    w16, [x1, #-1]
0x2fec34  11f05f38           ldurb    w17, [x0, #-1]
0x2fec38  300a508a           and      x16, x17, x16, lsr #2
0x2fec3c  1f825cea           tst      x16, x28, lsr #32
0x2fec40  40000054           b.eq     #0x2fec48
0x2fec44  40360494           bl       #0x40c544
0x2fec48  b0035ef8           ldur     x16, [x29, #-0x20]
0x2fec4c  f00100f9           str      x16, [x15]
0x2fec50  01befa97           bl       #0x1ae454
0x2fec54  a0831ff8           stur     x0, [x29, #-8]
0x2fec58  b16dfe97           bl       #0x29a31c
0x2fec5c  e10300aa           mov      x1, x0
0x2fec60  a0835ff8           ldur     x0, [x29, #-8]
0x2fec64  a1031ef8           stur     x1, [x29, #-0x20]
0x2fec68  20b000b8           stur     w0, [x1, #0xb]
0x2fec6c  30000094           bl       #0x2fed2c
0x2fec70  e30300aa           mov      x3, x0
0x2fec74  60334091           add      x0, x27, #0xc, lsl #12
0x2fec78  000840f9           ldr      x0, [x0, #0x10]  # pool[6144] = snapshotInstance(_DefaultHeroTag)
0x2fec7c  a3831ff8           stur     x3, [x29, #-8]
0x2fec80  607002b8           stur     w0, [x3, #0x27]
0x2fec84  a2835ef8           ldur     x2, [x29, #-0x18]
0x2fec88  61334091           add      x1, x27, #0xc, lsl #12
0x2fec8c  210c40f9           ldr      x1, [x1, #0x18]  # pool[6145] = _CatalogPageState.<anonymous closure>
0x2fec90  5c3a0494           bl       #0x40d600
0x2fec94  e10300aa           mov      x1, x0
0x2fec98  a0835ff8           ldur     x0, [x29, #-8]
0x2fec9c  01b002b8           stur     w1, [x0, #0x2b]
0x2feca0  c1820091           add      x1, x22, #0x20
0x2feca4  01f004b8           stur     w1, [x0, #0x4f]
0x2feca8  622f4091           add      x2, x27, #0xb, lsl #12
0x2fecac  429845f9           ldr      x2, [x2, #0xb30]  # pool[5988] = snapshotInstance(Clip)
0x2fecb0  02b004b8           stur     w2, [x0, #0x4b]
0x2fecb4  c2c20091           add      x2, x22, #0x30
0x2fecb8  027005b8           stur     w2, [x0, #0x57]
0x2fecbc  63334091           add      x3, x27, #0xc, lsl #12
0x2fecc0  631040f9           ldr      x3, [x3, #0x20]  # pool[6146] = snapshotInstance(_FloatingActionButtonType)
0x2fecc4  03f006b8           stur     w3, [x0, #0x6f]
0x2fecc8  63334091           add      x3, x27, #0xc, lsl #12
0x2feccc  631440f9           ldr      x3, [x3, #0x28]  # pool[6147] = snapshotInstance(Icon)
0x2fecd0  03b000b8           stur     w3, [x0, #0xb]
0x2fecd4  a3035ef8           ldur     x3, [x29, #-0x20]
0x2fecd8  033007b8           stur     w3, [x0, #0x73]
0x2fecdc  11000094           bl       #0x2fed20
0x2fece0  a1035df8           ldur     x1, [x29, #-0x30]
0x2fece4  013001b8           stur     w1, [x0, #0x13]
0x2fece8  a1035ff8           ldur     x1, [x29, #-0x10]
0x2fecec  017001b8           stur     w1, [x0, #0x17]
0x2fecf0  a1835ff8           ldur     x1, [x29, #-8]
0x2fecf4  01b001b8           stur     w1, [x0, #0x1b]
0x2fecf8  c1820091           add      x1, x22, #0x20
0x2fecfc  013004b8           stur     w1, [x0, #0x43]
0x2fed00  c1c20091           add      x1, x22, #0x30
0x2fed04  01b000b8           stur     w1, [x0, #0xb]
0x2fed08  01f000b8           stur     w1, [x0, #0xf]
0x2fed0c  ef031daa           mov      x15, x29
0x2fed10  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x2fed14  c0035fd6           ret      
0x2fed18  c43d0494           bl       #0x40e428
0x2fed1c  88feff17           b        #0x2fe73c
# CFG: 0x2fe714->0x2fe73c/ConditionalFalse 0x2fe714->0x2fed18/ConditionalTrue 0x2fe73c->0x2fe780/ConditionalFalse 0x2fe73c->0x2fe784/ConditionalTrue 0x2fe780->0x2fe784/Fallthrough 0x2fe784->0x2fe980/ConditionalFalse 0x2fe784->0x2fea88/ConditionalTrue 0x2fe980->0x2fe9d0/ConditionalFalse 0x2fe980->0x2fe9e8/ConditionalTrue 0x2fe9d0->0x2fe9e4/ConditionalFalse 0x2fe9d0->0x2fe9e8/ConditionalTrue 0x2fe9e4->0x2fe9e8/Fallthrough 0x2fe9e8->0x2fea34/ConditionalFalse 0x2fe9e8->0x2fea3c/ConditionalTrue 0x2fea34->0x2fea3c/Fallthrough 0x2fea3c->0x2fea6c/ConditionalFalse 0x2fea3c->0x2fea84/ConditionalTrue 0x2fea6c->0x2fea80/ConditionalFalse 0x2fea6c->0x2fea84/ConditionalTrue 0x2fea80->0x2fea84/Fallthrough 0x2fea84->0x2fea8c/Branch 0x2fea88->0x2fea8c/Fallthrough 0x2fea8c->0x2feb38/ConditionalFalse 0x2fea8c->0x2feb40/ConditionalTrue 0x2feb38->0x2feb40/Fallthrough 0x2feb40->0x2feb78/ConditionalFalse 0x2feb40->0x2feb90/ConditionalTrue 0x2feb78->0x2feb8c/ConditionalFalse 0x2feb78->0x2feb90/ConditionalTrue 0x2feb8c->0x2feb90/Fallthrough 0x2feb90->0x2fec30/ConditionalFalse 0x2feb90->0x2fec48/ConditionalTrue 0x2fec30->0x2fec44/ConditionalFalse 0x2fec30->0x2fec48/ConditionalTrue 0x2fec44->0x2fec48/Fallthrough 0x2fed18->0x2fe73c/Branch

# Cart.get:subtotal at 0x2fed38 (268 bytes)
0x2fed38  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x2fed3c  fd030faa           mov      x29, x15
0x2fed40  ef8100d1           sub      x15, x15, #0x20
0x2fed44  502740f9           ldr      x16, [x26, #0x48]
0x2fed48  ff0110eb           cmp      x15, x16
0x2fed4c  49070054           b.ls     #0x2fee34
0x2fed50  207040b8           ldur     w0, [x1, #7]
0x2fed54  00801c8b           add      x0, x0, x28, lsl #32
0x2fed58  a0831ff8           stur     x0, [x29, #-8]
0x2fed5c  61334091           add      x1, x27, #0xc, lsl #12
0x2fed60  213440f9           ldr      x1, [x1, #0x68]  # pool[6155] = snapshotRef(23048)
0x2fed64  b7e8fb97           bl       #0x1f9040
0x2fed68  e10300aa           mov      x1, x0
0x2fed6c  a0835ff8           ldur     x0, [x29, #-8]
0x2fed70  20b000b8           stur     w0, [x1, #0xb]
0x2fed74  05810194           bl       #0x35f188
0x2fed78  a0031ff8           stur     x0, [x29, #-0x10]
0x2fed7c  027040b8           ldur     w2, [x0, #7]
0x2fed80  42801c8b           add      x2, x2, x28, lsl #32
0x2fed84  a2831ff8           stur     x2, [x29, #-8]
0x2fed88  001c206e           eor      v0.16b, v0.16b, v0.16b
0x2fed8c  a0031efc           stur     d0, [x29, #-0x20]
0x2fed90  502740f9           ldr      x16, [x26, #0x48]
0x2fed94  ff0110eb           cmp      x15, x16
0x2fed98  29050054           b.ls     #0x2fee3c
0x2fed9c  e10300aa           mov      x1, x0
0x2feda0  2b5a0394           bl       #0x3d564c
0x2feda4  00042037           tbnz     w0, #4, #0x2fee24
0x2feda8  a3035ff8           ldur     x3, [x29, #-0x10]
0x2fedac  643043b8           ldur     w4, [x3, #0x33]
0x2fedb0  84801c8b           add      x4, x4, x28, lsl #32
0x2fedb4  a4831ef8           stur     x4, [x29, #-0x18]
0x2fedb8  9f00166b           cmp      w4, w22
0x2fedbc  a1010054           b.ne     #0x2fedf0
0x2fedc0  e00304aa           mov      x0, x4
0x2fedc4  a2835ff8           ldur     x2, [x29, #-8]
0x2fedc8  e10316aa           mov      x1, x22
0x2fedcc  5f00166b           cmp      w2, w22
0x2fedd0  00010054           b.eq     #0x2fedf0
0x2fedd4  447041b8           ldur     w4, [x2, #0x17]
0x2fedd8  84801c8b           add      x4, x4, x28, lsl #32
0x2feddc  683740f9           ldr      x8, [x27, #0x68]  # pool[11] = snapshotRef(19356)
0x2fede0  897040f8           ldur     x9, [x4, #7]
0x2fede4  63334091           add      x3, x27, #0xc, lsl #12
0x2fede8  63a440f9           ldr      x3, [x3, #0x148]  # pool[6183] = null
0x2fedec  20013fd6           blr      x9
0x2fedf0  a0035efc           ldur     d0, [x29, #-0x20]
0x2fedf4  a0835ef8           ldur     x0, [x29, #-0x18]
0x2fedf8  017040b8           ldur     w1, [x0, #7]
0x2fedfc  21801c8b           add      x1, x1, x28, lsl #32
0x2fee00  21f040fc           ldur     d1, [x1, #0xf]
0x2fee04  01b040f8           ldur     x1, [x0, #0xb]
0x2fee08  2200629e           scvtf    d2, x1
0x2fee0c  2308621e           fmul     d3, d1, d2
0x2fee10  0128631e           fadd     d1, d0, d3
0x2fee14  201ca14e           mov      v0.16b, v1.16b
0x2fee18  a0035ff8           ldur     x0, [x29, #-0x10]
0x2fee1c  a2835ff8           ldur     x2, [x29, #-8]
0x2fee20  dbffff17           b        #0x2fed8c
0x2fee24  a0035efc           ldur     d0, [x29, #-0x20]
0x2fee28  ef031daa           mov      x15, x29
0x2fee2c  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x2fee30  c0035fd6           ret      
0x2fee34  7d3d0494           bl       #0x40e428
0x2fee38  c6ffff17           b        #0x2fed50
0x2fee3c  9b3d0494           bl       #0x40e4a8
0x2fee40  d7ffff17           b        #0x2fed9c
# CFG: 0x2fed38->0x2fed50/ConditionalFalse 0x2fed38->0x2fee34/ConditionalTrue 0x2fed50->0x2fed8c/Fallthrough 0x2fed8c->0x2fed9c/ConditionalFalse 0x2fed8c->0x2fee3c/ConditionalTrue 0x2fed9c->0x2feda8/ConditionalFalse 0x2fed9c->0x2fee24/ConditionalTrue 0x2feda8->0x2fedc0/ConditionalFalse 0x2feda8->0x2fedf0/ConditionalTrue 0x2fedc0->0x2fedd4/ConditionalFalse 0x2fedc0->0x2fedf0/ConditionalTrue 0x2fedd4->0x2fedf0/Fallthrough 0x2fedf0->0x2fed8c/Branch 0x2fee34->0x2fed50/Branch 0x2fee3c->0x2fed9c/Branch

# top_level.formatPrice at 0x2fef58 (212 bytes)
0x2fef58  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x2fef5c  fd030faa           mov      x29, x15
0x2fef60  ef6100d1           sub      x15, x15, #0x18
0x2fef64  a0031ffc           stur     d0, [x29, #-0x10]
0x2fef68  502740f9           ldr      x16, [x26, #0x48]
0x2fef6c  ff0110eb           cmp      x15, x16
0x2fef70  c9040054           b.ls     #0x2ff008
0x2fef74  e10316aa           mov      x1, x22
0x2fef78  820080d2           mov      x2, #4
0x2fef7c  e83c0494           bl       #0x40e31c
0x2fef80  a0831ff8           stur     x0, [x29, #-8]
0x2fef84  70334091           add      x16, x27, #0xc, lsl #12
0x2fef88  10ba40f9           ldr      x16, [x16, #0x170]  # pool[6188] = snapshotRef(371)
0x2fef8c  10f000b8           stur     w16, [x0, #0xf]
0x2fef90  a0035ffc           ldur     d0, [x29, #-0x10]
0x2fef94  410b46a9           ldp      x1, x2, [x26, #0x60]
0x2fef98  21400091           add      x1, x1, #0x10
0x2fef9c  5f0001eb           cmp      x2, x1
0x2fefa0  89030054           b.ls     #0x2ff010
0x2fefa4  413300f9           str      x1, [x26, #0x60]
0x2fefa8  213c00d1           sub      x1, x1, #0xf
0x2fefac  82339cd2           mov      x2, #0xe19c
0x2fefb0  6200a0f2           movk     x2, #3, lsl #16
0x2fefb4  22f01ff8           stur     x2, [x1, #-1]
0x2fefb8  bf3a03d5           dmb      ishst
0x2fefbc  207000fc           stur     d0, [x1, #7]
0x2fefc0  420080d2           mov      x2, #2
0x2fefc4  1a000094           bl       #0x2ff02c
0x2fefc8  a1835ff8           ldur     x1, [x29, #-8]
0x2fefcc  394c0091           add      x25, x1, #0x13
0x2fefd0  200300b9           str      w0, [x25]
0x2fefd4  e0000036           tbz      w0, #0, #0x2feff0
0x2fefd8  30f05f38           ldurb    w16, [x1, #-1]
0x2fefdc  11f05f38           ldurb    w17, [x0, #-1]
0x2fefe0  300a508a           and      x16, x17, x16, lsr #2
0x2fefe4  1f825cea           tst      x16, x28, lsr #32
0x2fefe8  40000054           b.eq     #0x2feff0
0x2fefec  56350494           bl       #0x40c544
0x2feff0  b0835ff8           ldur     x16, [x29, #-8]
0x2feff4  f00100f9           str      x16, [x15]
0x2feff8  17bdfa97           bl       #0x1ae454
0x2feffc  ef031daa           mov      x15, x29
0x2ff000  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x2ff004  c0035fd6           ret      
0x2ff008  283d0494           bl       #0x40e4a8
0x2ff00c  daffff17           b        #0x2fef74
0x2ff010  e00d9f3c           str      q0, [x15, #-0x10]!
0x2ff014  e08d1ff8           str      x0, [x15, #-8]!
0x2ff018  953c0494           bl       #0x40e26c
0x2ff01c  e10300aa           mov      x1, x0
0x2ff020  e08540f8           ldr      x0, [x15], #8
0x2ff024  e005c13c           ldr      q0, [x15], #0x10
0x2ff028  e5ffff17           b        #0x2fefbc
# CFG: 0x2fef58->0x2fef74/ConditionalFalse 0x2fef58->0x2ff008/ConditionalTrue 0x2fef74->0x2fefa4/ConditionalFalse 0x2fef74->0x2ff010/ConditionalTrue 0x2fefa4->0x2fefbc/Fallthrough 0x2fefbc->0x2fefd8/ConditionalFalse 0x2fefbc->0x2feff0/ConditionalTrue 0x2fefd8->0x2fefec/ConditionalFalse 0x2fefd8->0x2feff0/ConditionalTrue 0x2fefec->0x2feff0/Fallthrough 0x2ff008->0x2fef74/Branch 0x2ff010->0x2fefbc/Branch

# top_level.firstWhereOrNull at 0x2ff204 (224 bytes)
0x2ff204  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x2ff208  fd030faa           mov      x29, x15
0x2ff20c  ef8100d1           sub      x15, x15, #0x20
0x2ff210  502740f9           ldr      x16, [x26, #0x48]
0x2ff214  ff0110eb           cmp      x15, x16
0x2ff218  e9050054           b.ls     #0x2ff2d4
0x2ff21c  a10f40f9           ldr      x1, [x29, #0x18]
0x2ff220  20f05ff8           ldur     x0, [x1, #-1]
0x2ff224  007c4cd3           ubfx     x0, x0, #0xc, #0x14
0x2ff228  1ec02d91           add      x30, x0, #0xb70
0x2ff22c  be7a7ef8           ldr      x30, [x21, x30, lsl #3]
0x2ff230  c0033fd6           blr      x30
0x2ff234  e20300aa           mov      x2, x0
0x2ff238  a2831ff8           stur     x2, [x29, #-8]
0x2ff23c  502740f9           ldr      x16, [x26, #0x48]
0x2ff240  ff0110eb           cmp      x15, x16
0x2ff244  c9040054           b.ls     #0x2ff2dc
0x2ff248  40f05ff8           ldur     x0, [x2, #-1]
0x2ff24c  007c4cd3           ubfx     x0, x0, #0xc, #0x14
0x2ff250  e10302aa           mov      x1, x2
0x2ff254  1e0440d1           sub      x30, x0, #1, lsl #12
0x2ff258  be7a7ef8           ldr      x30, [x21, x30, lsl #3]
0x2ff25c  c0033fd6           blr      x30
0x2ff260  20032037           tbnz     w0, #4, #0x2ff2c4
0x2ff264  a2835ff8           ldur     x2, [x29, #-8]
0x2ff268  40f05ff8           ldur     x0, [x2, #-1]
0x2ff26c  007c4cd3           ubfx     x0, x0, #0xc, #0x14
0x2ff270  e10302aa           mov      x1, x2
0x2ff274  1e403fd1           sub      x30, x0, #0xfd0
0x2ff278  be7a7ef8           ldr      x30, [x21, x30, lsl #3]
0x2ff27c  c0033fd6           blr      x30
0x2ff280  e10300aa           mov      x1, x0
0x2ff284  a1031ff8           stur     x1, [x29, #-0x10]
0x2ff288  b00b40f9           ldr      x16, [x29, #0x10]
0x2ff28c  e14100a9           stp      x1, x16, [x15]
0x2ff290  a00b40f9           ldr      x0, [x29, #0x10]
0x2ff294  643b41f9           ldr      x4, [x27, #0x270]  # pool[76] = snapshotRef(23)
0x2ff298  02f041f8           ldur     x2, [x0, #0x1f]
0x2ff29c  40003fd6           blr      x2
0x2ff2a0  d0820091           add      x16, x22, #0x20
0x2ff2a4  1f00106b           cmp      w0, w16
0x2ff2a8  60000054           b.eq     #0x2ff2b4
0x2ff2ac  a2835ff8           ldur     x2, [x29, #-8]
0x2ff2b0  e3ffff17           b        #0x2ff23c
0x2ff2b4  a0035ff8           ldur     x0, [x29, #-0x10]
0x2ff2b8  ef031daa           mov      x15, x29
0x2ff2bc  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x2ff2c0  c0035fd6           ret      
0x2ff2c4  e00316aa           mov      x0, x22
0x2ff2c8  ef031daa           mov      x15, x29
0x2ff2cc  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x2ff2d0  c0035fd6           ret      
0x2ff2d4  553c0494           bl       #0x40e428
0x2ff2d8  d1ffff17           b        #0x2ff21c
0x2ff2dc  533c0494           bl       #0x40e428
0x2ff2e0  daffff17           b        #0x2ff248
# CFG: 0x2ff204->0x2ff21c/ConditionalFalse 0x2ff204->0x2ff2d4/ConditionalTrue 0x2ff21c->0x2ff23c/Fallthrough 0x2ff23c->0x2ff248/ConditionalFalse 0x2ff23c->0x2ff2dc/ConditionalTrue 0x2ff248->0x2ff264/ConditionalFalse 0x2ff248->0x2ff2c4/ConditionalTrue 0x2ff264->0x2ff2ac/ConditionalFalse 0x2ff264->0x2ff2b4/ConditionalTrue 0x2ff2ac->0x2ff23c/Branch 0x2ff2d4->0x2ff21c/Branch 0x2ff2dc->0x2ff248/Branch

# _CatalogPageState.get:_filteredProducts at 0x2ff2e4 (180 bytes)
0x2ff2e4  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x2ff2e8  fd030faa           mov      x29, x15
0x2ff2ec  ef4100d1           sub      x15, x15, #0x10
0x2ff2f0  502740f9           ldr      x16, [x26, #0x48]
0x2ff2f4  ff0110eb           cmp      x15, x16
0x2ff2f8  c9040054           b.ls     #0x2ff390
0x2ff2fc  207041b8           ldur     w0, [x1, #0x17]
0x2ff300  00801c8b           add      x0, x0, x28, lsl #32
0x2ff304  017040b8           ldur     w1, [x0, #7]
0x2ff308  c1000035           cbnz     w1, #0x2ff320
0x2ff30c  60334091           add      x0, x27, #0xc, lsl #12
0x2ff310  00c840f9           ldr      x0, [x0, #0x190]  # pool[6192] = snapshotRef(48403) nestedStrings["Aged Cheese", "Apple", "Avocado", "Sourdough Bread", "Whole Milk", "apple", "avocado", "bakery", "bread", "cheese", "dairy", "milk", "produce"]
0x2ff314  ef031daa           mov      x15, x29
0x2ff318  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x2ff31c  c0035fd6           ret      
0x2ff320  01f05ff8           ldur     x1, [x0, #-1]
0x2ff324  217c4cd3           ubfx     x1, x1, #0xc, #0x14
0x2ff328  e00100f9           str      x0, [x15]
0x2ff32c  e00301aa           mov      x0, x1
0x2ff330  1ee43fd1           sub      x30, x0, #0xff9
0x2ff334  be7a7ef8           ldr      x30, [x21, x30, lsl #3]
0x2ff338  c0033fd6           blr      x30
0x2ff33c  a0831ff8           stur     x0, [x29, #-8]
0x2ff340  210080d2           mov      x1, #1
0x2ff344  ba370494           bl       #0x40d22c
0x2ff348  e10300aa           mov      x1, x0
0x2ff34c  a0835ff8           ldur     x0, [x29, #-8]
0x2ff350  20f000b8           stur     w0, [x1, #0xf]
0x2ff354  e20301aa           mov      x2, x1
0x2ff358  61334091           add      x1, x27, #0xc, lsl #12
0x2ff35c  21cc40f9           ldr      x1, [x1, #0x198]  # pool[6193] = _CatalogPageState.<anonymous closure>
0x2ff360  a8380494           bl       #0x40d600
0x2ff364  e20300aa           mov      x2, x0
0x2ff368  61334091           add      x1, x27, #0xc, lsl #12
0x2ff36c  21c840f9           ldr      x1, [x1, #0x190]  # pool[6192] = snapshotRef(48403) nestedStrings["Aged Cheese", "Apple", "Avocado", "Sourdough Bread", "Whole Milk", "apple", "avocado", "bakery", "bread", "cheese", "dairy", "milk", "produce"]
0x2ff370  e6320094           bl       #0x30bf08
0x2ff374  017040b8           ldur     w1, [x0, #7]
0x2ff378  21801c8b           add      x1, x1, x28, lsl #32
0x2ff37c  e20300aa           mov      x2, x0
0x2ff380  1d9efa97           bl       #0x1a6bf4
0x2ff384  ef031daa           mov      x15, x29
0x2ff388  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x2ff38c  c0035fd6           ret      
0x2ff390  263c0494           bl       #0x40e428
0x2ff394  daffff17           b        #0x2ff2fc
# CFG: 0x2ff2e4->0x2ff2fc/ConditionalFalse 0x2ff2e4->0x2ff390/ConditionalTrue 0x2ff2fc->0x2ff30c/ConditionalFalse 0x2ff2fc->0x2ff320/ConditionalTrue 0x2ff390->0x2ff2fc/Branch

# _CatalogPageState.<anonymous closure> at 0x2ff398 (380 bytes)
0x2ff398  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x2ff39c  fd030faa           mov      x29, x15
0x2ff3a0  efa100d1           sub      x15, x15, #0x28
0x2ff3a4  a00f40f9           ldr      x0, [x29, #0x18]
0x2ff3a8  017041b8           ldur     w1, [x0, #0x17]
0x2ff3ac  21801c8b           add      x1, x1, x28, lsl #32
0x2ff3b0  a1031ef8           stur     x1, [x29, #-0x20]
0x2ff3b4  502740f9           ldr      x16, [x26, #0x48]
0x2ff3b8  ff0110eb           cmp      x15, x16
0x2ff3bc  a9090054           b.ls     #0x2ff4f0
0x2ff3c0  a00b40f9           ldr      x0, [x29, #0x10]
0x2ff3c4  02b040b8           ldur     w2, [x0, #0xb]
0x2ff3c8  42801c8b           add      x2, x2, x28, lsl #32
0x2ff3cc  a2831ef8           stur     x2, [x29, #-0x18]
0x2ff3d0  407040b8           ldur     w0, [x2, #7]
0x2ff3d4  037c4193           sbfx     x3, x0, #1, #0x1f
0x2ff3d8  a3031ff8           stur     x3, [x29, #-0x10]
0x2ff3dc  050080d2           mov      x5, #0
0x2ff3e0  64274091           add      x4, x27, #9, lsl #12
0x2ff3e4  849447f9           ldr      x4, [x4, #0xf28]  # pool[5091] = "\0\u{1}\u{2}\u{3}\u{4}\u{5}\u{6}\u{7}\u{8}\t\n\u{b}\u{c}\r\u{e}\u{f}\u{10}\u{11}\u{12}\u{13}\u{14}\u{15}\u{16}\u{17}\u{18}\u{19}\u{1a}\u{1b}\u{1c}\u{1d}\u{1e}\u{1f} !\"#$%&'()*+,-./0123456789:;<=>?@abcdefghijklmnopqrstuvwxyz[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~\u{7f}\u{80}\u{81}\u{82}\u{83}\u{84}\u{85}\u{86}\u{87}\u{88}\u{89}\u{8a}\u{8b}\u{8c}\u{8d}\u{8e}\u{8f}\u{90}\u{91}\u{92}\u{93}\u{94}\u{95}\u{96}\u{97}\u{98}\u{99}\u{9a}\u{9b}\u{9c}\u{9d}\u{9e}\u{9f}"…
0x2ff3e8  a5831ff8           stur     x5, [x29, #-8]
0x2ff3ec  502740f9           ldr      x16, [x26, #0x48]
0x2ff3f0  ff0110eb           cmp      x15, x16
0x2ff3f4  29080054           b.ls     #0x2ff4f8
0x2ff3f8  bf0003eb           cmp      x5, x3
0x2ff3fc  6a060054           b.ge     #0x2ff4c8
0x2ff400  5000058b           add      x16, x2, x5
0x2ff404  063e4039           ldrb     w6, [x16, #0xf]
0x2ff408  9000068b           add      x16, x4, x6
0x2ff40c  073e4039           ldrb     w7, [x16, #0xf]
0x2ff410  df0007eb           cmp      x6, x7
0x2ff414  81000054           b.ne     #0x2ff424
0x2ff418  a6040091           add      x6, x5, #1
0x2ff41c  e50306aa           mov      x5, x6
0x2ff420  f2ffff17           b        #0x2ff3e8
0x2ff424  e00100f9           str      x0, [x15]
0x2ff428  6fbdfa97           bl       #0x1ae9e4
0x2ff42c  e20300aa           mov      x2, x0
0x2ff430  a3835ef8           ldur     x3, [x29, #-0x18]
0x2ff434  a4835ff8           ldur     x4, [x29, #-8]
0x2ff438  050080d2           mov      x5, #0
0x2ff43c  502740f9           ldr      x16, [x26, #0x48]
0x2ff440  ff0110eb           cmp      x15, x16
0x2ff444  e9050054           b.ls     #0x2ff500
0x2ff448  bf0004eb           cmp      x5, x4
0x2ff44c  8a010054           b.ge     #0x2ff47c
0x2ff450  a0035ff8           ldur     x0, [x29, #-0x10]
0x2ff454  e10305aa           mov      x1, x5
0x2ff458  3f0000eb           cmp      x1, x0
0x2ff45c  62050054           b.hs     #0x2ff508
0x2ff460  7000058b           add      x16, x3, x5
0x2ff464  003e4039           ldrb     w0, [x16, #0xf]
0x2ff468  4100058b           add      x1, x2, x5
0x2ff46c  203c0039           strb     w0, [x1, #0xf]
0x2ff470  a0040091           add      x0, x5, #1
0x2ff474  e50300aa           mov      x5, x0
0x2ff478  f1ffff17           b        #0x2ff43c
0x2ff47c  a0035ff8           ldur     x0, [x29, #-0x10]
0x2ff480  61274091           add      x1, x27, #9, lsl #12
0x2ff484  219447f9           ldr      x1, [x1, #0xf28]  # pool[5091] = "\0\u{1}\u{2}\u{3}\u{4}\u{5}\u{6}\u{7}\u{8}\t\n\u{b}\u{c}\r\u{e}\u{f}\u{10}\u{11}\u{12}\u{13}\u{14}\u{15}\u{16}\u{17}\u{18}\u{19}\u{1a}\u{1b}\u{1c}\u{1d}\u{1e}\u{1f} !\"#$%&'()*+,-./0123456789:;<=>?@abcdefghijklmnopqrstuvwxyz[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~\u{7f}\u{80}\u{81}\u{82}\u{83}\u{84}\u{85}\u{86}\u{87}\u{88}\u{89}\u{8a}\u{8b}\u{8c}\u{8d}\u{8e}\u{8f}\u{90}\u{91}\u{92}\u{93}\u{94}\u{95}\u{96}\u{97}\u{98}\u{99}\u{9a}\u{9b}\u{9c}\u{9d}\u{9e}\u{9f}"…
0x2ff488  502740f9           ldr      x16, [x26, #0x48]
0x2ff48c  ff0110eb           cmp      x15, x16
0x2ff490  e9030054           b.ls     #0x2ff50c
0x2ff494  9f0000eb           cmp      x4, x0
0x2ff498  4a010054           b.ge     #0x2ff4c0
0x2ff49c  7000048b           add      x16, x3, x4
0x2ff4a0  053e4039           ldrb     w5, [x16, #0xf]
0x2ff4a4  3000058b           add      x16, x1, x5
0x2ff4a8  063e4039           ldrb     w6, [x16, #0xf]
0x2ff4ac  4500048b           add      x5, x2, x4
0x2ff4b0  a63c0039           strb     w6, [x5, #0xf]
0x2ff4b4  85040091           add      x5, x4, #1
0x2ff4b8  e40305aa           mov      x4, x5
0x2ff4bc  f3ffff17           b        #0x2ff488
0x2ff4c0  e10302aa           mov      x1, x2
0x2ff4c4  03000014           b        #0x2ff4d0
0x2ff4c8  e30302aa           mov      x3, x2
0x2ff4cc  e10303aa           mov      x1, x3
0x2ff4d0  a0035ef8           ldur     x0, [x29, #-0x20]
0x2ff4d4  02f040b8           ldur     w2, [x0, #0xf]
0x2ff4d8  42801c8b           add      x2, x2, x28, lsl #32
0x2ff4dc  64a340f9           ldr      x4, [x27, #0x140]  # pool[38] = snapshotRef(48150)
0x2ff4e0  b71a0494           bl       #0x405fbc
0x2ff4e4  ef031daa           mov      x15, x29
0x2ff4e8  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x2ff4ec  c0035fd6           ret      
0x2ff4f0  ce3b0494           bl       #0x40e428
0x2ff4f4  b3ffff17           b        #0x2ff3c0
0x2ff4f8  cc3b0494           bl       #0x40e428
0x2ff4fc  bfffff17           b        #0x2ff3f8
0x2ff500  ca3b0494           bl       #0x40e428
0x2ff504  d1ffff17           b        #0x2ff448
0x2ff508  143d0494           bl       #0x40e958
0x2ff50c  c73b0494           bl       #0x40e428
0x2ff510  e1ffff17           b        #0x2ff494
# CFG: 0x2ff398->0x2ff3c0/ConditionalFalse 0x2ff398->0x2ff4f0/ConditionalTrue 0x2ff3c0->0x2ff3e8/Fallthrough 0x2ff3e8->0x2ff3f8/ConditionalFalse 0x2ff3e8->0x2ff4f8/ConditionalTrue 0x2ff3f8->0x2ff400/ConditionalFalse 0x2ff3f8->0x2ff4c8/ConditionalTrue 0x2ff400->0x2ff418/ConditionalFalse 0x2ff400->0x2ff424/ConditionalTrue 0x2ff418->0x2ff3e8/Branch 0x2ff424->0x2ff43c/Fallthrough 0x2ff43c->0x2ff448/ConditionalFalse 0x2ff43c->0x2ff500/ConditionalTrue 0x2ff448->0x2ff450/ConditionalFalse 0x2ff448->0x2ff47c/ConditionalTrue 0x2ff450->0x2ff460/ConditionalFalse 0x2ff450->0x2ff508/ConditionalTrue 0x2ff460->0x2ff43c/Branch 0x2ff47c->0x2ff488/Fallthrough 0x2ff488->0x2ff494/ConditionalFalse 0x2ff488->0x2ff50c/ConditionalTrue 0x2ff494->0x2ff49c/ConditionalFalse 0x2ff494->0x2ff4c0/ConditionalTrue 0x2ff49c->0x2ff488/Branch 0x2ff4c0->0x2ff4d0/Branch 0x2ff4c8->0x2ff4d0/Fallthrough 0x2ff4f0->0x2ff3c0/Branch 0x2ff4f8->0x2ff3f8/Branch 0x2ff500->0x2ff448/Branch 0x2ff508->0x2ff50c/Fallthrough 0x2ff50c->0x2ff494/Branch

# _CatalogPageState.<anonymous closure> at 0x2ff514 (68 bytes)
0x2ff514  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x2ff518  fd030faa           mov      x29, x15
0x2ff51c  a00b40f9           ldr      x0, [x29, #0x10]
0x2ff520  017041b8           ldur     w1, [x0, #0x17]
0x2ff524  21801c8b           add      x1, x1, x28, lsl #32
0x2ff528  502740f9           ldr      x16, [x26, #0x48]
0x2ff52c  ff0110eb           cmp      x15, x16
0x2ff530  09010054           b.ls     #0x2ff550
0x2ff534  20f040b8           ldur     w0, [x1, #0xf]
0x2ff538  00801c8b           add      x0, x0, x28, lsl #32
0x2ff53c  e10300aa           mov      x1, x0
0x2ff540  06000094           bl       #0x2ff558
0x2ff544  ef031daa           mov      x15, x29
0x2ff548  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x2ff54c  c0035fd6           ret      
0x2ff550  b63b0494           bl       #0x40e428
0x2ff554  f8ffff17           b        #0x2ff534
# CFG: 0x2ff514->0x2ff534/ConditionalFalse 0x2ff514->0x2ff550/ConditionalTrue 0x2ff550->0x2ff534/Branch

# _CatalogPageState._checkout at 0x2ff558 (304 bytes)
0x2ff558  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x2ff55c  fd030faa           mov      x29, x15
0x2ff560  efa100d1           sub      x15, x15, #0x28
0x2ff564  b6831ff8           stur     x22, [x29, #-8]
0x2ff568  a1031ff8           stur     x1, [x29, #-0x10]
0x2ff56c  502740f9           ldr      x16, [x26, #0x48]
0x2ff570  ff0110eb           cmp      x15, x16
0x2ff574  69080054           b.ls     #0x2ff680
0x2ff578  410080d2           mov      x1, #2
0x2ff57c  2c370494           bl       #0x40d22c
0x2ff580  e20300aa           mov      x2, x0
0x2ff584  a1035ff8           ldur     x1, [x29, #-0x10]
0x2ff588  a2831ef8           stur     x2, [x29, #-0x18]
0x2ff58c  41f000b8           stur     w1, [x2, #0xf]
0x2ff590  606b41f9           ldr      x0, [x27, #0x2d0]  # pool[88] = snapshotRef(23591)
0x2ff594  b922fb97           bl       #0x1c8078
0x2ff598  a0035ff8           ldur     x0, [x29, #-0x10]
0x2ff59c  023041b8           ldur     w2, [x0, #0x13]
0x2ff5a0  42801c8b           add      x2, x2, x28, lsl #32
0x2ff5a4  e10302aa           mov      x1, x2
0x2ff5a8  a2031ef8           stur     x2, [x29, #-0x20]
0x2ff5ac  37000094           bl       #0x2ff688
0x2ff5b0  40012037           tbnz     w0, #4, #0x2ff5d8
0x2ff5b4  a2835ef8           ldur     x2, [x29, #-0x18]
0x2ff5b8  61334091           add      x1, x27, #0xc, lsl #12
0x2ff5bc  211840f9           ldr      x1, [x1, #0x30]  # pool[6148] = _CatalogPageState.<anonymous closure>
0x2ff5c0  10380494           bl       #0x40d600
0x2ff5c4  a1035ff8           ldur     x1, [x29, #-0x10]
0x2ff5c8  e20300aa           mov      x2, x0
0x2ff5cc  88a5fa97           bl       #0x1a8bec
0x2ff5d0  e00316aa           mov      x0, x22
0x2ff5d4  6b21fb17           b        #0x1c7b80
0x2ff5d8  a0035ff8           ldur     x0, [x29, #-0x10]
0x2ff5dc  a2835ef8           ldur     x2, [x29, #-0x18]
0x2ff5e0  61334091           add      x1, x27, #0xc, lsl #12
0x2ff5e4  211c40f9           ldr      x1, [x1, #0x38]  # pool[6149] = _CatalogPageState.<anonymous closure>
0x2ff5e8  06380494           bl       #0x40d600
0x2ff5ec  a1035ff8           ldur     x1, [x29, #-0x10]
0x2ff5f0  e20300aa           mov      x2, x0
0x2ff5f4  7ea5fa97           bl       #0x1a8bec
0x2ff5f8  616b41f9           ldr      x1, [x27, #0x2d0]  # pool[88] = snapshotRef(23591)
0x2ff5fc  62334091           add      x2, x27, #0xc, lsl #12
0x2ff600  422040f9           ldr      x2, [x2, #0x40]  # pool[6150] = snapshotInstance(Duration)
0x2ff604  ff6bfc97           bl       #0x21a600
0x2ff608  e10300aa           mov      x1, x0
0x2ff60c  a1831df8           stur     x1, [x29, #-0x28]
0x2ff610  0922fb97           bl       #0x1c7e34
0x2ff614  a0035ff8           ldur     x0, [x29, #-0x10]
0x2ff618  01f040b8           ldur     w1, [x0, #0xf]
0x2ff61c  21801c8b           add      x1, x1, x28, lsl #32
0x2ff620  3f00166b           cmp      w1, w22
0x2ff624  61000054           b.ne     #0x2ff630
0x2ff628  e00316aa           mov      x0, x22
0x2ff62c  5521fb17           b        #0x1c7b80
0x2ff630  a2835ef8           ldur     x2, [x29, #-0x18]
0x2ff634  a1035ef8           ldur     x1, [x29, #-0x20]
0x2ff638  c0fdff97           bl       #0x2fed38
0x2ff63c  47feff97           bl       #0x2fef58
0x2ff640  a2835ef8           ldur     x2, [x29, #-0x18]
0x2ff644  403001b8           stur     w0, [x2, #0x13]
0x2ff648  50f05f38           ldurb    w16, [x2, #-1]
0x2ff64c  11f05f38           ldurb    w17, [x0, #-1]
0x2ff650  300a508a           and      x16, x17, x16, lsr #2
0x2ff654  1f825cea           tst      x16, x28, lsr #32
0x2ff658  40000054           b.eq     #0x2ff660
0x2ff65c  d3340494           bl       #0x40c9a8
0x2ff660  61334091           add      x1, x27, #0xc, lsl #12
0x2ff664  212440f9           ldr      x1, [x1, #0x48]  # pool[6151] = _CatalogPageState.<anonymous closure>
0x2ff668  e6370494           bl       #0x40d600
0x2ff66c  a1035ff8           ldur     x1, [x29, #-0x10]
0x2ff670  e20300aa           mov      x2, x0
0x2ff674  5ea5fa97           bl       #0x1a8bec
0x2ff678  e00316aa           mov      x0, x22
0x2ff67c  4121fb17           b        #0x1c7b80
0x2ff680  6a3b0494           bl       #0x40e428
0x2ff684  bdffff17           b        #0x2ff578
# CFG: 0x2ff558->0x2ff578/ConditionalFalse 0x2ff558->0x2ff680/ConditionalTrue 0x2ff578->0x2ff5b4/ConditionalFalse 0x2ff578->0x2ff5d8/ConditionalTrue 0x2ff5d8->0x2ff628/ConditionalFalse 0x2ff5d8->0x2ff630/ConditionalTrue 0x2ff630->0x2ff65c/ConditionalFalse 0x2ff630->0x2ff660/ConditionalTrue 0x2ff65c->0x2ff660/Fallthrough 0x2ff680->0x2ff578/Branch

# Cart.get:isEmpty at 0x2ff688 (68 bytes)
0x2ff688  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x2ff68c  fd030faa           mov      x29, x15
0x2ff690  227040b8           ldur     w2, [x1, #7]
0x2ff694  42801c8b           add      x2, x2, x28, lsl #32
0x2ff698  413041b8           ldur     w1, [x2, #0x13]
0x2ff69c  237c4193           sbfx     x3, x1, #1, #0x1f
0x2ff6a0  61fc4193           asr      x1, x3, #1
0x2ff6a4  437041b8           ldur     w3, [x2, #0x17]
0x2ff6a8  627c4193           sbfx     x2, x3, #1, #0x1f
0x2ff6ac  230002cb           sub      x3, x1, x2
0x2ff6b0  630000b4           cbz      x3, #0x2ff6bc
0x2ff6b4  c0c20091           add      x0, x22, #0x30
0x2ff6b8  02000014           b        #0x2ff6c0
0x2ff6bc  c0820091           add      x0, x22, #0x20
0x2ff6c0  ef031daa           mov      x15, x29
0x2ff6c4  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x2ff6c8  c0035fd6           ret      
# CFG: 0x2ff688->0x2ff6b4/ConditionalFalse 0x2ff688->0x2ff6bc/ConditionalTrue 0x2ff6b4->0x2ff6c0/Branch 0x2ff6bc->0x2ff6c0/Fallthrough

# _CatalogPageState.<anonymous closure> at 0x2ff6cc (296 bytes)
0x2ff6cc  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x2ff6d0  fd030faa           mov      x29, x15
0x2ff6d4  ef8100d1           sub      x15, x15, #0x20
0x2ff6d8  a00b40f9           ldr      x0, [x29, #0x10]
0x2ff6dc  037041b8           ldur     w3, [x0, #0x17]
0x2ff6e0  63801c8b           add      x3, x3, x28, lsl #32
0x2ff6e4  a3031ff8           stur     x3, [x29, #-0x10]
0x2ff6e8  502740f9           ldr      x16, [x26, #0x48]
0x2ff6ec  ff0110eb           cmp      x15, x16
0x2ff6f0  e9070054           b.ls     #0x2ff7ec
0x2ff6f4  60f040b8           ldur     w0, [x3, #0xf]
0x2ff6f8  00801c8b           add      x0, x0, x28, lsl #32
0x2ff6fc  a0831ff8           stur     x0, [x29, #-8]
0x2ff700  e10316aa           mov      x1, x22
0x2ff704  420180d2           mov      x2, #0xa
0x2ff708  053b0494           bl       #0x40e31c
0x2ff70c  a0831ef8           stur     x0, [x29, #-0x18]
0x2ff710  70334091           add      x16, x27, #0xc, lsl #12
0x2ff714  102a40f9           ldr      x16, [x16, #0x50]  # pool[6152] = "Paid "
0x2ff718  10f000b8           stur     w16, [x0, #0xf]
0x2ff71c  a2035ff8           ldur     x2, [x29, #-0x10]
0x2ff720  413041b8           ldur     w1, [x2, #0x13]
0x2ff724  21801c8b           add      x1, x1, x28, lsl #32
0x2ff728  013001b8           stur     w1, [x0, #0x13]
0x2ff72c  70334091           add      x16, x27, #0xc, lsl #12
0x2ff730  102e40f9           ldr      x16, [x16, #0x58]  # pool[6153] = " for "
0x2ff734  107001b8           stur     w16, [x0, #0x17]
0x2ff738  a3835ff8           ldur     x3, [x29, #-8]
0x2ff73c  613041b8           ldur     w1, [x3, #0x13]
0x2ff740  21801c8b           add      x1, x1, x28, lsl #32
0x2ff744  3b000094           bl       #0x2ff830
0x2ff748  e20300aa           mov      x2, x0
0x2ff74c  40787f93           sbfiz    x0, x2, #1, #0x1f
0x2ff750  5f0480eb           cmp      x2, x0, asr #1
0x2ff754  60000054           b.eq     #0x2ff760
0x2ff758  943b0494           bl       #0x40e5a8
0x2ff75c  027000f8           stur     x2, [x0, #7]
0x2ff760  a1835ef8           ldur     x1, [x29, #-0x18]
0x2ff764  396c0091           add      x25, x1, #0x1b
0x2ff768  200300b9           str      w0, [x25]
0x2ff76c  e0000036           tbz      w0, #0, #0x2ff788
0x2ff770  30f05f38           ldurb    w16, [x1, #-1]
0x2ff774  11f05f38           ldurb    w17, [x0, #-1]
0x2ff778  300a508a           and      x16, x17, x16, lsr #2
0x2ff77c  1f825cea           tst      x16, x28, lsr #32
0x2ff780  40000054           b.eq     #0x2ff788
0x2ff784  70330494           bl       #0x40c544
0x2ff788  a0835ef8           ldur     x0, [x29, #-0x18]
0x2ff78c  70334091           add      x16, x27, #0xc, lsl #12
0x2ff790  103240f9           ldr      x16, [x16, #0x60]  # pool[6154] = " items"
0x2ff794  10f001b8           stur     w16, [x0, #0x1f]
0x2ff798  e00100f9           str      x0, [x15]
0x2ff79c  2ebbfa97           bl       #0x1ae454
0x2ff7a0  a1835ff8           ldur     x1, [x29, #-8]
0x2ff7a4  20b001b8           stur     w0, [x1, #0x1b]
0x2ff7a8  30f05f38           ldurb    w16, [x1, #-1]
0x2ff7ac  11f05f38           ldurb    w17, [x0, #-1]
0x2ff7b0  300a508a           and      x16, x17, x16, lsr #2
0x2ff7b4  1f825cea           tst      x16, x28, lsr #32
0x2ff7b8  40000054           b.eq     #0x2ff7c0
0x2ff7bc  73340494           bl       #0x40c988
0x2ff7c0  a0035ff8           ldur     x0, [x29, #-0x10]
0x2ff7c4  01f040b8           ldur     w1, [x0, #0xf]
0x2ff7c8  21801c8b           add      x1, x1, x28, lsl #32
0x2ff7cc  203041b8           ldur     w0, [x1, #0x13]
0x2ff7d0  00801c8b           add      x0, x0, x28, lsl #32
0x2ff7d4  e10300aa           mov      x1, x0
0x2ff7d8  07000094           bl       #0x2ff7f4
0x2ff7dc  e00316aa           mov      x0, x22
0x2ff7e0  ef031daa           mov      x15, x29
0x2ff7e4  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x2ff7e8  c0035fd6           ret      
0x2ff7ec  0f3b0494           bl       #0x40e428
0x2ff7f0  c1ffff17           b        #0x2ff6f4
# CFG: 0x2ff6cc->0x2ff6f4/ConditionalFalse 0x2ff6cc->0x2ff7ec/ConditionalTrue 0x2ff6f4->0x2ff758/ConditionalFalse 0x2ff6f4->0x2ff760/ConditionalTrue 0x2ff758->0x2ff760/Fallthrough 0x2ff760->0x2ff770/ConditionalFalse 0x2ff760->0x2ff788/ConditionalTrue 0x2ff770->0x2ff784/ConditionalFalse 0x2ff770->0x2ff788/ConditionalTrue 0x2ff784->0x2ff788/Fallthrough 0x2ff788->0x2ff7bc/ConditionalFalse 0x2ff788->0x2ff7c0/ConditionalTrue 0x2ff7bc->0x2ff7c0/Fallthrough 0x2ff7ec->0x2ff6f4/Branch

# Cart.clear at 0x2ff7f4 (60 bytes)
0x2ff7f4  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x2ff7f8  fd030faa           mov      x29, x15
0x2ff7fc  502740f9           ldr      x16, [x26, #0x48]
0x2ff800  ff0110eb           cmp      x15, x16
0x2ff804  29010054           b.ls     #0x2ff828
0x2ff808  207040b8           ldur     w0, [x1, #7]
0x2ff80c  00801c8b           add      x0, x0, x28, lsl #32
0x2ff810  e10300aa           mov      x1, x0
0x2ff814  beb6fa97           bl       #0x1ad30c
0x2ff818  e00316aa           mov      x0, x22
0x2ff81c  ef031daa           mov      x15, x29
0x2ff820  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x2ff824  c0035fd6           ret      
0x2ff828  003b0494           bl       #0x40e428
0x2ff82c  f7ffff17           b        #0x2ff808
# CFG: 0x2ff7f4->0x2ff808/ConditionalFalse 0x2ff7f4->0x2ff828/ConditionalTrue 0x2ff828->0x2ff808/Branch

# Cart.get:itemCount at 0x2ff830 (244 bytes)
0x2ff830  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x2ff834  fd030faa           mov      x29, x15
0x2ff838  ef8100d1           sub      x15, x15, #0x20
0x2ff83c  502740f9           ldr      x16, [x26, #0x48]
0x2ff840  ff0110eb           cmp      x15, x16
0x2ff844  89060054           b.ls     #0x2ff914
0x2ff848  207040b8           ldur     w0, [x1, #7]
0x2ff84c  00801c8b           add      x0, x0, x28, lsl #32
0x2ff850  a0831ff8           stur     x0, [x29, #-8]
0x2ff854  61334091           add      x1, x27, #0xc, lsl #12
0x2ff858  213440f9           ldr      x1, [x1, #0x68]  # pool[6155] = snapshotRef(23048)
0x2ff85c  f9e5fb97           bl       #0x1f9040
0x2ff860  e10300aa           mov      x1, x0
0x2ff864  a0835ff8           ldur     x0, [x29, #-8]
0x2ff868  20b000b8           stur     w0, [x1, #0xb]
0x2ff86c  477e0194           bl       #0x35f188
0x2ff870  a0831ef8           stur     x0, [x29, #-0x18]
0x2ff874  027040b8           ldur     w2, [x0, #7]
0x2ff878  42801c8b           add      x2, x2, x28, lsl #32
0x2ff87c  a2831ff8           stur     x2, [x29, #-8]
0x2ff880  030080d2           mov      x3, #0
0x2ff884  a3031ff8           stur     x3, [x29, #-0x10]
0x2ff888  502740f9           ldr      x16, [x26, #0x48]
0x2ff88c  ff0110eb           cmp      x15, x16
0x2ff890  69040054           b.ls     #0x2ff91c
0x2ff894  e10300aa           mov      x1, x0
0x2ff898  6d570394           bl       #0x3d564c
0x2ff89c  40032037           tbnz     w0, #4, #0x2ff904
0x2ff8a0  a3835ef8           ldur     x3, [x29, #-0x18]
0x2ff8a4  643043b8           ldur     w4, [x3, #0x33]
0x2ff8a8  84801c8b           add      x4, x4, x28, lsl #32
0x2ff8ac  a4031ef8           stur     x4, [x29, #-0x20]
0x2ff8b0  9f00166b           cmp      w4, w22
0x2ff8b4  a1010054           b.ne     #0x2ff8e8
0x2ff8b8  e00304aa           mov      x0, x4
0x2ff8bc  a2835ff8           ldur     x2, [x29, #-8]
0x2ff8c0  e10316aa           mov      x1, x22
0x2ff8c4  5f00166b           cmp      w2, w22
0x2ff8c8  00010054           b.eq     #0x2ff8e8
0x2ff8cc  447041b8           ldur     w4, [x2, #0x17]
0x2ff8d0  84801c8b           add      x4, x4, x28, lsl #32
0x2ff8d4  683740f9           ldr      x8, [x27, #0x68]  # pool[11] = snapshotRef(19356)
0x2ff8d8  897040f8           ldur     x9, [x4, #7]
0x2ff8dc  63334091           add      x3, x27, #0xc, lsl #12
0x2ff8e0  633840f9           ldr      x3, [x3, #0x70]  # pool[6156] = null
0x2ff8e4  20013fd6           blr      x9
0x2ff8e8  a0035ff8           ldur     x0, [x29, #-0x10]
0x2ff8ec  a1035ef8           ldur     x1, [x29, #-0x20]
0x2ff8f0  22b040f8           ldur     x2, [x1, #0xb]
0x2ff8f4  0300028b           add      x3, x0, x2
0x2ff8f8  a0835ef8           ldur     x0, [x29, #-0x18]
0x2ff8fc  a2835ff8           ldur     x2, [x29, #-8]
0x2ff900  e1ffff17           b        #0x2ff884
0x2ff904  a0035ff8           ldur     x0, [x29, #-0x10]
0x2ff908  ef031daa           mov      x15, x29
0x2ff90c  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x2ff910  c0035fd6           ret      
0x2ff914  c53a0494           bl       #0x40e428
0x2ff918  ccffff17           b        #0x2ff848
0x2ff91c  c33a0494           bl       #0x40e428
0x2ff920  ddffff17           b        #0x2ff894
# CFG: 0x2ff830->0x2ff848/ConditionalFalse 0x2ff830->0x2ff914/ConditionalTrue 0x2ff848->0x2ff884/Fallthrough 0x2ff884->0x2ff894/ConditionalFalse 0x2ff884->0x2ff91c/ConditionalTrue 0x2ff894->0x2ff8a0/ConditionalFalse 0x2ff894->0x2ff904/ConditionalTrue 0x2ff8a0->0x2ff8b8/ConditionalFalse 0x2ff8a0->0x2ff8e8/ConditionalTrue 0x2ff8b8->0x2ff8cc/ConditionalFalse 0x2ff8b8->0x2ff8e8/ConditionalTrue 0x2ff8cc->0x2ff8e8/Fallthrough 0x2ff8e8->0x2ff884/Branch 0x2ff914->0x2ff848/Branch 0x2ff91c->0x2ff894/Branch

# _CatalogPageState.<anonymous closure> at 0x2ff924 (40 bytes)
0x2ff924  61334091           add      x1, x27, #0xc, lsl #12
0x2ff928  214040f9           ldr      x1, [x1, #0x80]  # pool[6158] = "Processing..."
0x2ff92c  e20140f9           ldr      x2, [x15]
0x2ff930  437041b8           ldur     w3, [x2, #0x17]
0x2ff934  63801c8b           add      x3, x3, x28, lsl #32
0x2ff938  62f040b8           ldur     w2, [x3, #0xf]
0x2ff93c  42801c8b           add      x2, x2, x28, lsl #32
0x2ff940  41b001b8           stur     w1, [x2, #0x1b]
0x2ff944  e00316aa           mov      x0, x22
0x2ff948  c0035fd6           ret      

# _CatalogPageState.<anonymous closure> at 0x2ff94c (40 bytes)
0x2ff94c  61334091           add      x1, x27, #0xc, lsl #12
0x2ff950  214440f9           ldr      x1, [x1, #0x88]  # pool[6159] = "Cart is empty"
0x2ff954  e20140f9           ldr      x2, [x15]
0x2ff958  437041b8           ldur     w3, [x2, #0x17]
0x2ff95c  63801c8b           add      x3, x3, x28, lsl #32
0x2ff960  62f040b8           ldur     w2, [x3, #0xf]
0x2ff964  42801c8b           add      x2, x2, x28, lsl #32
0x2ff968  41b001b8           stur     w1, [x2, #0x1b]
0x2ff96c  e00316aa           mov      x0, x22
0x2ff970  c0035fd6           ret      

# _CatalogPageState.<anonymous closure> at 0x2ff974 (360 bytes)
0x2ff974  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x2ff978  fd030faa           mov      x29, x15
0x2ff97c  efc100d1           sub      x15, x15, #0x30
0x2ff980  a01340f9           ldr      x0, [x29, #0x20]
0x2ff984  017041b8           ldur     w1, [x0, #0x17]
0x2ff988  21801c8b           add      x1, x1, x28, lsl #32
0x2ff98c  a1831ff8           stur     x1, [x29, #-8]
0x2ff990  502740f9           ldr      x16, [x26, #0x48]
0x2ff994  ff0110eb           cmp      x15, x16
0x2ff998  e9090054           b.ls     #0x2ffad4
0x2ff99c  210080d2           mov      x1, #1
0x2ff9a0  23360494           bl       #0x40d22c
0x2ff9a4  e10300aa           mov      x1, x0
0x2ff9a8  a0835ff8           ldur     x0, [x29, #-8]
0x2ff9ac  a1031ff8           stur     x1, [x29, #-0x10]
0x2ff9b0  20b000b8           stur     w0, [x1, #0xb]
0x2ff9b4  023041b8           ldur     w2, [x0, #0x13]
0x2ff9b8  42801c8b           add      x2, x2, x28, lsl #32
0x2ff9bc  40f05ff8           ldur     x0, [x2, #-1]
0x2ff9c0  007c4cd3           ubfx     x0, x0, #0xc, #0x14
0x2ff9c4  b00b40f9           ldr      x16, [x29, #0x10]
0x2ff9c8  f00900a9           stp      x16, x2, [x15]
0x2ff9cc  1ee83fd1           sub      x30, x0, #0xffa
0x2ff9d0  be7a7ef8           ldr      x30, [x21, x30, lsl #3]
0x2ff9d4  c0033fd6           blr      x30
0x2ff9d8  e10300aa           mov      x1, x0
0x2ff9dc  a2035ff8           ldur     x2, [x29, #-0x10]
0x2ff9e0  a1831ff8           stur     x1, [x29, #-8]
0x2ff9e4  40f000b8           stur     w0, [x2, #0xf]
0x2ff9e8  50f05f38           ldurb    w16, [x2, #-1]
0x2ff9ec  11f05f38           ldurb    w17, [x0, #-1]
0x2ff9f0  300a508a           and      x16, x17, x16, lsr #2
0x2ff9f4  1f825cea           tst      x16, x28, lsr #32
0x2ff9f8  40000054           b.eq     #0x2ffa00
0x2ff9fc  eb330494           bl       #0x40c9a8
0x2ffa00  e10100f9           str      x1, [x15]
0x2ffa04  ffc00094           bl       #0x32fe00
0x2ffa08  a0831ef8           stur     x0, [x29, #-0x18]
0x2ffa0c  446afe97           bl       #0x29a31c
0x2ffa10  e10300aa           mov      x1, x0
0x2ffa14  a0835ef8           ldur     x0, [x29, #-0x18]
0x2ffa18  a1031ef8           stur     x1, [x29, #-0x20]
0x2ffa1c  20b000b8           stur     w0, [x1, #0xb]
0x2ffa20  a0835ff8           ldur     x0, [x29, #-8]
0x2ffa24  027041b8           ldur     w2, [x0, #0x17]
0x2ffa28  42801c8b           add      x2, x2, x28, lsl #32
0x2ffa2c  40f040b8           ldur     w0, [x2, #0xf]
0x2ffa30  00801c8b           add      x0, x0, x28, lsl #32
0x2ffa34  a0831ff8           stur     x0, [x29, #-8]
0x2ffa38  396afe97           bl       #0x29a31c
0x2ffa3c  e30300aa           mov      x3, x0
0x2ffa40  a0835ff8           ldur     x0, [x29, #-8]
0x2ffa44  a3831ef8           stur     x3, [x29, #-0x18]
0x2ffa48  60b000b8           stur     w0, [x3, #0xb]
0x2ffa4c  a2035ff8           ldur     x2, [x29, #-0x10]
0x2ffa50  61334091           add      x1, x27, #0xc, lsl #12
0x2ffa54  216840f9           ldr      x1, [x1, #0xd0]  # pool[6168] = _CatalogPageState.<anonymous closure>
0x2ffa58  ea360494           bl       #0x40d600
0x2ffa5c  a0831ff8           stur     x0, [x29, #-8]
0x2ffa60  155efe97           bl       #0x2972b4
0x2ffa64  e10300aa           mov      x1, x0
0x2ffa68  a0835ff8           ldur     x0, [x29, #-8]
0x2ffa6c  a1031ff8           stur     x1, [x29, #-0x10]
0x2ffa70  20b003b8           stur     w0, [x1, #0x3b]
0x2ffa74  c0c20091           add      x0, x22, #0x30
0x2ffa78  20f004b8           stur     w0, [x1, #0x4f]
0x2ffa7c  62334091           add      x2, x27, #0xc, lsl #12
0x2ffa80  426c40f9           ldr      x2, [x2, #0xd8]  # pool[6169] = snapshotInstance(Icon)
0x2ffa84  22f001b8           stur     w2, [x1, #0x1f]
0x2ffa88  62334091           add      x2, x27, #0xc, lsl #12
0x2ffa8c  427040f9           ldr      x2, [x2, #0xe0]  # pool[6170] = snapshotInstance(_IconButtonVariant)
0x2ffa90  22f006b8           stur     w2, [x1, #0x6f]
0x2ffa94  12000094           bl       #0x2ffadc
0x2ffa98  a1035ef8           ldur     x1, [x29, #-0x20]
0x2ffa9c  01f000b8           stur     w1, [x0, #0xf]
0x2ffaa0  a1835ef8           ldur     x1, [x29, #-0x18]
0x2ffaa4  013001b8           stur     w1, [x0, #0x13]
0x2ffaa8  a1035ff8           ldur     x1, [x29, #-0x10]
0x2ffaac  017001b8           stur     w1, [x0, #0x17]
0x2ffab0  c1820091           add      x1, x22, #0x20
0x2ffab4  01b004b8           stur     w1, [x0, #0x4b]
0x2ffab8  c2c20091           add      x2, x22, #0x30
0x2ffabc  02f005b8           stur     w2, [x0, #0x5f]
0x2ffac0  023007b8           stur     w2, [x0, #0x73]
0x2ffac4  017009b8           stur     w1, [x0, #0x97]
0x2ffac8  ef031daa           mov      x15, x29
0x2ffacc  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x2ffad0  c0035fd6           ret      
0x2ffad4  553a0494           bl       #0x40e428
0x2ffad8  b1ffff17           b        #0x2ff99c
# CFG: 0x2ff974->0x2ff99c/ConditionalFalse 0x2ff974->0x2ffad4/ConditionalTrue 0x2ff99c->0x2ff9fc/ConditionalFalse 0x2ff99c->0x2ffa00/ConditionalTrue 0x2ff9fc->0x2ffa00/Fallthrough 0x2ffad4->0x2ff99c/Branch

# _CatalogPageState.<anonymous closure> at 0x2ffae8 (92 bytes)
0x2ffae8  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x2ffaec  fd030faa           mov      x29, x15
0x2ffaf0  a00b40f9           ldr      x0, [x29, #0x10]
0x2ffaf4  017041b8           ldur     w1, [x0, #0x17]
0x2ffaf8  21801c8b           add      x1, x1, x28, lsl #32
0x2ffafc  502740f9           ldr      x16, [x26, #0x48]
0x2ffb00  ff0110eb           cmp      x15, x16
0x2ffb04  c9010054           b.ls     #0x2ffb3c
0x2ffb08  20b040b8           ldur     w0, [x1, #0xb]
0x2ffb0c  00801c8b           add      x0, x0, x28, lsl #32
0x2ffb10  02f040b8           ldur     w2, [x0, #0xf]
0x2ffb14  42801c8b           add      x2, x2, x28, lsl #32
0x2ffb18  20f040b8           ldur     w0, [x1, #0xf]
0x2ffb1c  00801c8b           add      x0, x0, x28, lsl #32
0x2ffb20  e10302aa           mov      x1, x2
0x2ffb24  e20300aa           mov      x2, x0
0x2ffb28  07000094           bl       #0x2ffb44
0x2ffb2c  e00316aa           mov      x0, x22
0x2ffb30  ef031daa           mov      x15, x29
0x2ffb34  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x2ffb38  c0035fd6           ret      
0x2ffb3c  3b3a0494           bl       #0x40e428
0x2ffb40  f2ffff17           b        #0x2ffb08
# CFG: 0x2ffae8->0x2ffb08/ConditionalFalse 0x2ffae8->0x2ffb3c/ConditionalTrue 0x2ffb3c->0x2ffb08/Branch

# _CatalogPageState._add at 0x2ffb44 (112 bytes)
0x2ffb44  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x2ffb48  fd030faa           mov      x29, x15
0x2ffb4c  ef4100d1           sub      x15, x15, #0x10
0x2ffb50  a1831ff8           stur     x1, [x29, #-8]
0x2ffb54  a2031ff8           stur     x2, [x29, #-0x10]
0x2ffb58  502740f9           ldr      x16, [x26, #0x48]
0x2ffb5c  ff0110eb           cmp      x15, x16
0x2ffb60  69020054           b.ls     #0x2ffbac
0x2ffb64  410080d2           mov      x1, #2
0x2ffb68  b1350494           bl       #0x40d22c
0x2ffb6c  e10300aa           mov      x1, x0
0x2ffb70  a0835ff8           ldur     x0, [x29, #-8]
0x2ffb74  20f000b8           stur     w0, [x1, #0xf]
0x2ffb78  a2035ff8           ldur     x2, [x29, #-0x10]
0x2ffb7c  223001b8           stur     w2, [x1, #0x13]
0x2ffb80  e20301aa           mov      x2, x1
0x2ffb84  61334091           add      x1, x27, #0xc, lsl #12
0x2ffb88  217440f9           ldr      x1, [x1, #0xe8]  # pool[6171] = _CatalogPageState.<anonymous closure>
0x2ffb8c  9d360494           bl       #0x40d600
0x2ffb90  a1835ff8           ldur     x1, [x29, #-8]
0x2ffb94  e20300aa           mov      x2, x0
0x2ffb98  15a4fa97           bl       #0x1a8bec
0x2ffb9c  e00316aa           mov      x0, x22
0x2ffba0  ef031daa           mov      x15, x29
0x2ffba4  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x2ffba8  c0035fd6           ret      
0x2ffbac  1f3a0494           bl       #0x40e428
0x2ffbb0  edffff17           b        #0x2ffb64
# CFG: 0x2ffb44->0x2ffb64/ConditionalFalse 0x2ffb44->0x2ffbac/ConditionalTrue 0x2ffbac->0x2ffb64/Branch

# _CatalogPageState.<anonymous closure> at 0x2ffbb4 (196 bytes)
0x2ffbb4  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x2ffbb8  fd030faa           mov      x29, x15
0x2ffbbc  ef6100d1           sub      x15, x15, #0x18
0x2ffbc0  a00b40f9           ldr      x0, [x29, #0x10]
0x2ffbc4  037041b8           ldur     w3, [x0, #0x17]
0x2ffbc8  63801c8b           add      x3, x3, x28, lsl #32
0x2ffbcc  a3831ff8           stur     x3, [x29, #-8]
0x2ffbd0  502740f9           ldr      x16, [x26, #0x48]
0x2ffbd4  ff0110eb           cmp      x15, x16
0x2ffbd8  c9040054           b.ls     #0x2ffc70
0x2ffbdc  60f040b8           ldur     w0, [x3, #0xf]
0x2ffbe0  00801c8b           add      x0, x0, x28, lsl #32
0x2ffbe4  013041b8           ldur     w1, [x0, #0x13]
0x2ffbe8  21801c8b           add      x1, x1, x28, lsl #32
0x2ffbec  623041b8           ldur     w2, [x3, #0x13]
0x2ffbf0  42801c8b           add      x2, x2, x28, lsl #32
0x2ffbf4  21000094           bl       #0x2ffc78
0x2ffbf8  a0835ff8           ldur     x0, [x29, #-8]
0x2ffbfc  03f040b8           ldur     w3, [x0, #0xf]
0x2ffc00  63801c8b           add      x3, x3, x28, lsl #32
0x2ffc04  a3031ff8           stur     x3, [x29, #-0x10]
0x2ffc08  e10316aa           mov      x1, x22
0x2ffc0c  820080d2           mov      x2, #4
0x2ffc10  c3390494           bl       #0x40e31c
0x2ffc14  70334091           add      x16, x27, #0xc, lsl #12
0x2ffc18  107a40f9           ldr      x16, [x16, #0xf0]  # pool[6172] = "Added "
0x2ffc1c  10f000b8           stur     w16, [x0, #0xf]
0x2ffc20  a1835ff8           ldur     x1, [x29, #-8]
0x2ffc24  223041b8           ldur     w2, [x1, #0x13]
0x2ffc28  42801c8b           add      x2, x2, x28, lsl #32
0x2ffc2c  41b040b8           ldur     w1, [x2, #0xb]
0x2ffc30  21801c8b           add      x1, x1, x28, lsl #32
0x2ffc34  013001b8           stur     w1, [x0, #0x13]
0x2ffc38  e00100f9           str      x0, [x15]
0x2ffc3c  06bafa97           bl       #0x1ae454
0x2ffc40  a1035ff8           ldur     x1, [x29, #-0x10]
0x2ffc44  20b001b8           stur     w0, [x1, #0x1b]
0x2ffc48  30f05f38           ldurb    w16, [x1, #-1]
0x2ffc4c  11f05f38           ldurb    w17, [x0, #-1]
0x2ffc50  300a508a           and      x16, x17, x16, lsr #2
0x2ffc54  1f825cea           tst      x16, x28, lsr #32
0x2ffc58  40000054           b.eq     #0x2ffc60
0x2ffc5c  4b330494           bl       #0x40c988
0x2ffc60  e00316aa           mov      x0, x22
0x2ffc64  ef031daa           mov      x15, x29
0x2ffc68  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x2ffc6c  c0035fd6           ret      
0x2ffc70  ee390494           bl       #0x40e428
0x2ffc74  daffff17           b        #0x2ffbdc
# CFG: 0x2ffbb4->0x2ffbdc/ConditionalFalse 0x2ffbb4->0x2ffc70/ConditionalTrue 0x2ffbdc->0x2ffc5c/ConditionalFalse 0x2ffbdc->0x2ffc60/ConditionalTrue 0x2ffc5c->0x2ffc60/Fallthrough 0x2ffc70->0x2ffbdc/Branch

# Cart.add at 0x2ffc78 (224 bytes)
0x2ffc78  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x2ffc7c  fd030faa           mov      x29, x15
0x2ffc80  ef8100d1           sub      x15, x15, #0x20
0x2ffc84  e00302aa           mov      x0, x2
0x2ffc88  a2831ef8           stur     x2, [x29, #-0x18]
0x2ffc8c  502740f9           ldr      x16, [x26, #0x48]
0x2ffc90  ff0110eb           cmp      x15, x16
0x2ffc94  e9050054           b.ls     #0x2ffd50
0x2ffc98  237040b8           ldur     w3, [x1, #7]
0x2ffc9c  63801c8b           add      x3, x3, x28, lsl #32
0x2ffca0  a3031ff8           stur     x3, [x29, #-0x10]
0x2ffca4  047040b8           ldur     w4, [x0, #7]
0x2ffca8  84801c8b           add      x4, x4, x28, lsl #32
0x2ffcac  e10303aa           mov      x1, x3
0x2ffcb0  e20304aa           mov      x2, x4
0x2ffcb4  a4831ff8           stur     x4, [x29, #-8]
0x2ffcb8  dcc7fa97           bl       #0x1b1c28
0x2ffcbc  a1035ff8           ldur     x1, [x29, #-0x10]
0x2ffcc0  22f040b8           ldur     w2, [x1, #0xf]
0x2ffcc4  42801c8b           add      x2, x2, x28, lsl #32
0x2ffcc8  5f00006b           cmp      w2, w0
0x2ffccc  41000054           b.ne     #0x2ffcd4
0x2ffcd0  e00316aa           mov      x0, x22
0x2ffcd4  1f00166b           cmp      w0, w22
0x2ffcd8  81010054           b.ne     #0x2ffd08
0x2ffcdc  a0835ef8           ldur     x0, [x29, #-0x18]
0x2ffce0  1e000094           bl       #0x2ffd58
0x2ffce4  a1835ef8           ldur     x1, [x29, #-0x18]
0x2ffce8  017000b8           stur     w1, [x0, #7]
0x2ffcec  210080d2           mov      x1, #1
0x2ffcf0  01b000f8           stur     x1, [x0, #0xb]
0x2ffcf4  a1035ff8           ldur     x1, [x29, #-0x10]
0x2ffcf8  a2835ff8           ldur     x2, [x29, #-8]
0x2ffcfc  e30300aa           mov      x3, x0
0x2ffd00  4f0b0494           bl       #0x402a3c
0x2ffd04  0f000014           b        #0x2ffd40
0x2ffd08  a1835ef8           ldur     x1, [x29, #-0x18]
0x2ffd0c  02b040f8           ldur     x2, [x0, #0xb]
0x2ffd10  40040091           add      x0, x2, #1
0x2ffd14  a0031ef8           stur     x0, [x29, #-0x20]
0x2ffd18  10000094           bl       #0x2ffd58
0x2ffd1c  e10300aa           mov      x1, x0
0x2ffd20  a0835ef8           ldur     x0, [x29, #-0x18]
0x2ffd24  207000b8           stur     w0, [x1, #7]
0x2ffd28  a0035ef8           ldur     x0, [x29, #-0x20]
0x2ffd2c  20b000f8           stur     x0, [x1, #0xb]
0x2ffd30  e30301aa           mov      x3, x1
0x2ffd34  a1035ff8           ldur     x1, [x29, #-0x10]
0x2ffd38  a2835ff8           ldur     x2, [x29, #-8]
0x2ffd3c  400b0494           bl       #0x402a3c
0x2ffd40  e00316aa           mov      x0, x22
0x2ffd44  ef031daa           mov      x15, x29
0x2ffd48  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x2ffd4c  c0035fd6           ret      
0x2ffd50  b6390494           bl       #0x40e428
0x2ffd54  d1ffff17           b        #0x2ffc98
# CFG: 0x2ffc78->0x2ffc98/ConditionalFalse 0x2ffc78->0x2ffd50/ConditionalTrue 0x2ffc98->0x2ffcd0/ConditionalFalse 0x2ffc98->0x2ffcd4/ConditionalTrue 0x2ffcd0->0x2ffcd4/Fallthrough 0x2ffcd4->0x2ffcdc/ConditionalFalse 0x2ffcd4->0x2ffd08/ConditionalTrue 0x2ffcdc->0x2ffd40/Branch 0x2ffd08->0x2ffd40/Fallthrough 0x2ffd50->0x2ffc98/Branch

# package:simple_app/models.dart.CartLine at 0x2ffd58 (12 bytes)
0x2ffd58  824388d2           mov      x2, #0x421c
0x2ffd5c  6201a0f2           movk     x2, #0xb, lsl #16
0x2ffd60  0f350414           b        #0x40d19c

# _CatalogPageState.<anonymous closure> at 0x2ffd64 (132 bytes)
0x2ffd64  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x2ffd68  fd030faa           mov      x29, x15
0x2ffd6c  ef4100d1           sub      x15, x15, #0x10
0x2ffd70  a00f40f9           ldr      x0, [x29, #0x18]
0x2ffd74  017041b8           ldur     w1, [x0, #0x17]
0x2ffd78  21801c8b           add      x1, x1, x28, lsl #32
0x2ffd7c  a1831ff8           stur     x1, [x29, #-8]
0x2ffd80  502740f9           ldr      x16, [x26, #0x48]
0x2ffd84  ff0110eb           cmp      x15, x16
0x2ffd88  c9020054           b.ls     #0x2ffde0
0x2ffd8c  210080d2           mov      x1, #1
0x2ffd90  27350494           bl       #0x40d22c
0x2ffd94  e10300aa           mov      x1, x0
0x2ffd98  a0835ff8           ldur     x0, [x29, #-8]
0x2ffd9c  20b000b8           stur     w0, [x1, #0xb]
0x2ffda0  a20b40f9           ldr      x2, [x29, #0x10]
0x2ffda4  22f000b8           stur     w2, [x1, #0xf]
0x2ffda8  03f040b8           ldur     w3, [x0, #0xf]
0x2ffdac  63801c8b           add      x3, x3, x28, lsl #32
0x2ffdb0  e20301aa           mov      x2, x1
0x2ffdb4  a3031ff8           stur     x3, [x29, #-0x10]
0x2ffdb8  61334091           add      x1, x27, #0xc, lsl #12
0x2ffdbc  21a040f9           ldr      x1, [x1, #0x140]  # pool[6182] = _CatalogPageState.<anonymous closure>
0x2ffdc0  10360494           bl       #0x40d600
0x2ffdc4  a1035ff8           ldur     x1, [x29, #-0x10]
0x2ffdc8  e20300aa           mov      x2, x0
0x2ffdcc  88a3fa97           bl       #0x1a8bec
0x2ffdd0  e00316aa           mov      x0, x22
0x2ffdd4  ef031daa           mov      x15, x29
0x2ffdd8  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x2ffddc  c0035fd6           ret      
0x2ffde0  92390494           bl       #0x40e428
0x2ffde4  eaffff17           b        #0x2ffd8c
# CFG: 0x2ffd64->0x2ffd8c/ConditionalFalse 0x2ffd64->0x2ffde0/ConditionalTrue 0x2ffde0->0x2ffd8c/Branch

# _CatalogPageState.<anonymous closure> at 0x2ffde8 (80 bytes)
0x2ffde8  e10140f9           ldr      x1, [x15]
0x2ffdec  227041b8           ldur     w2, [x1, #0x17]
0x2ffdf0  42801c8b           add      x2, x2, x28, lsl #32
0x2ffdf4  41b040b8           ldur     w1, [x2, #0xb]
0x2ffdf8  21801c8b           add      x1, x1, x28, lsl #32
0x2ffdfc  23f040b8           ldur     w3, [x1, #0xf]
0x2ffe00  63801c8b           add      x3, x3, x28, lsl #32
0x2ffe04  40f040b8           ldur     w0, [x2, #0xf]
0x2ffe08  00801c8b           add      x0, x0, x28, lsl #32
0x2ffe0c  607001b8           stur     w0, [x3, #0x17]
0x2ffe10  70f05f38           ldurb    w16, [x3, #-1]
0x2ffe14  11f05f38           ldurb    w17, [x0, #-1]
0x2ffe18  300a508a           and      x16, x17, x16, lsr #2
0x2ffe1c  1f825cea           tst      x16, x28, lsr #32
0x2ffe20  80000054           b.eq     #0x2ffe30
0x2ffe24  fe8d1ff8           str      x30, [x15, #-8]!
0x2ffe28  e8320494           bl       #0x40c9c8
0x2ffe2c  fe8540f8           ldr      x30, [x15], #8
0x2ffe30  e00316aa           mov      x0, x22
0x2ffe34  c0035fd6           ret      
# CFG: 0x2ffde8->0x2ffe24/ConditionalFalse 0x2ffde8->0x2ffe30/ConditionalTrue 0x2ffe24->0x2ffe30/Fallthrough

# _CatalogPageState.<anonymous closure> at 0x2ffe38 (36 bytes)
0x2ffe38  0090641e           fmov     d0, #10.00000000
0x2ffe3c  e10140f9           ldr      x1, [x15]
0x2ffe40  21f040fc           ldur     d1, [x1, #0xf]
0x2ffe44  2020601e           fcmp     d1, d0
0x2ffe48  d0820091           add      x16, x22, #0x20
0x2ffe4c  d1c20091           add      x17, x22, #0x30
0x2ffe50  01a2919a           csel     x1, x16, x17, ge
0x2ffe54  20007cd2           eor      x0, x1, #0x10
0x2ffe58  c0035fd6           ret      

# CatalogPage.createState at 0x306aec (156 bytes)
0x306aec  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x306af0  fd030faa           mov      x29, x15
0x306af4  ef8100d1           sub      x15, x15, #0x20
0x306af8  502740f9           ldr      x16, [x26, #0x48]
0x306afc  ff0110eb           cmp      x15, x16
0x306b00  09040054           b.ls     #0x306b80
0x306b04  612f4091           add      x1, x27, #0xb, lsl #12
0x306b08  21f444f9           ldr      x1, [x1, #0x9e8]  # pool[5947] = snapshotRef(23425)
0x306b0c  22000094           bl       #0x306b94
0x306b10  e10300aa           mov      x1, x0
0x306b14  60db40f9           ldr      x0, [x27, #0x1b0]  # pool[52] = snapshotRef(903)
0x306b18  a1831ff8           stur     x1, [x29, #-8]
0x306b1c  207001b8           stur     w0, [x1, #0x17]
0x306b20  602f4091           add      x0, x27, #0xb, lsl #12
0x306b24  00f844f9           ldr      x0, [x0, #0x9f0]  # pool[5948] = "Ready"
0x306b28  20b001b8           stur     w0, [x1, #0x1b]
0x306b2c  702f4091           add      x16, x27, #0xb, lsl #12
0x306b30  10fe44f9           ldr      x16, [x16, #0x9f8]  # pool[5949] = snapshotRef(23225)
0x306b34  5e5740f9           ldr      x30, [x26, #0xa8]
0x306b38  fe4100a9           stp      x30, x16, [x15]
0x306b3c  c98bfa97           bl       #0x1a9a60
0x306b40  a0031ff8           stur     x0, [x29, #-0x10]
0x306b44  11000094           bl       #0x306b88
0x306b48  a1035ff8           ldur     x1, [x29, #-0x10]
0x306b4c  017000b8           stur     w1, [x0, #7]
0x306b50  a1835ff8           ldur     x1, [x29, #-8]
0x306b54  203001b8           stur     w0, [x1, #0x13]
0x306b58  30f05f38           ldurb    w16, [x1, #-1]
0x306b5c  11f05f38           ldurb    w17, [x0, #-1]
0x306b60  300a508a           and      x16, x17, x16, lsr #2
0x306b64  1f825cea           tst      x16, x28, lsr #32
0x306b68  40000054           b.eq     #0x306b70
0x306b6c  87170494           bl       #0x40c988
0x306b70  e00301aa           mov      x0, x1
0x306b74  ef031daa           mov      x15, x29
0x306b78  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x306b7c  c0035fd6           ret      
0x306b80  2a1e0494           bl       #0x40e428
0x306b84  e0ffff17           b        #0x306b04
# CFG: 0x306aec->0x306b04/ConditionalFalse 0x306aec->0x306b80/ConditionalTrue 0x306b04->0x306b6c/ConditionalFalse 0x306b04->0x306b70/ConditionalTrue 0x306b6c->0x306b70/Fallthrough 0x306b80->0x306b04/Branch

# package:simple_app/models.dart.Cart at 0x306b88 (12 bytes)
0x306b88  822386d2           mov      x2, #0x311c
0x306b8c  6201a0f2           movk     x2, #0xb, lsl #16
0x306b90  83190414           b        #0x40d19c

# package:simple_app/main.dart._CatalogPageState at 0x306b94 (12 bytes)
0x306b94  82438cd2           mov      x2, #0x621c
0x306b98  620da0f2           movk     x2, #0x6b, lsl #16
0x306b9c  66190414           b        #0x40d134

# Product.toString at 0x32fe00 (264 bytes)
0x32fe00  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x32fe04  fd030faa           mov      x29, x15
0x32fe08  ef6100d1           sub      x15, x15, #0x18
0x32fe0c  502740f9           ldr      x16, [x26, #0x48]
0x32fe10  ff0110eb           cmp      x15, x16
0x32fe14  89060054           b.ls     #0x32fee4
0x32fe18  a00b40f9           ldr      x0, [x29, #0x10]
0x32fe1c  03b040b8           ldur     w3, [x0, #0xb]
0x32fe20  63801c8b           add      x3, x3, x28, lsl #32
0x32fe24  a3831ff8           stur     x3, [x29, #-8]
0x32fe28  e10316aa           mov      x1, x22
0x32fe2c  420180d2           mov      x2, #0xa
0x32fe30  3b790394           bl       #0x40e31c
0x32fe34  e30300aa           mov      x3, x0
0x32fe38  a0835ff8           ldur     x0, [x29, #-8]
0x32fe3c  a3031ff8           stur     x3, [x29, #-0x10]
0x32fe40  60f000b8           stur     w0, [x3, #0xf]
0x32fe44  70334091           add      x16, x27, #0xc, lsl #12
0x32fe48  107e40f9           ldr      x16, [x16, #0xf8]  # pool[6173] = " (#"
0x32fe4c  703001b8           stur     w16, [x3, #0x13]
0x32fe50  a00b40f9           ldr      x0, [x29, #0x10]
0x32fe54  017040b8           ldur     w1, [x0, #7]
0x32fe58  21801c8b           add      x1, x1, x28, lsl #32
0x32fe5c  617001b8           stur     w1, [x3, #0x17]
0x32fe60  70334091           add      x16, x27, #0xc, lsl #12
0x32fe64  108240f9           ldr      x16, [x16, #0x100]  # pool[6174] = ") $"
0x32fe68  70b001b8           stur     w16, [x3, #0x1b]
0x32fe6c  00f040fc           ldur     d0, [x0, #0xf]
0x32fe70  410346a9           ldp      x1, x0, [x26, #0x60]
0x32fe74  21400091           add      x1, x1, #0x10
0x32fe78  1f0001eb           cmp      x0, x1
0x32fe7c  89030054           b.ls     #0x32feec
0x32fe80  413300f9           str      x1, [x26, #0x60]
0x32fe84  213c00d1           sub      x1, x1, #0xf
0x32fe88  80339cd2           mov      x0, #0xe19c
0x32fe8c  6000a0f2           movk     x0, #3, lsl #16
0x32fe90  20f01ff8           stur     x0, [x1, #-1]
0x32fe94  bf3a03d5           dmb      ishst
0x32fe98  207000fc           stur     d0, [x1, #7]
0x32fe9c  420080d2           mov      x2, #2
0x32fea0  633cff97           bl       #0x2ff02c
0x32fea4  a1035ff8           ldur     x1, [x29, #-0x10]
0x32fea8  397c0091           add      x25, x1, #0x1f
0x32feac  200300b9           str      w0, [x25]
0x32feb0  e0000036           tbz      w0, #0, #0x32fecc
0x32feb4  30f05f38           ldurb    w16, [x1, #-1]
0x32feb8  11f05f38           ldurb    w17, [x0, #-1]
0x32febc  300a508a           and      x16, x17, x16, lsr #2
0x32fec0  1f825cea           tst      x16, x28, lsr #32
0x32fec4  40000054           b.eq     #0x32fecc
0x32fec8  9f710394           bl       #0x40c544
0x32fecc  b0035ff8           ldur     x16, [x29, #-0x10]
0x32fed0  f00100f9           str      x16, [x15]
0x32fed4  60f9f997           bl       #0x1ae454
0x32fed8  ef031daa           mov      x15, x29
0x32fedc  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x32fee0  c0035fd6           ret      
0x32fee4  51790394           bl       #0x40e428
0x32fee8  ccffff17           b        #0x32fe18
0x32feec  e00d9f3c           str      q0, [x15, #-0x10]!
0x32fef0  e38d1ff8           str      x3, [x15, #-8]!
0x32fef4  de780394           bl       #0x40e26c
0x32fef8  e10300aa           mov      x1, x0
0x32fefc  e38540f8           ldr      x3, [x15], #8
0x32ff00  e005c13c           ldr      q0, [x15], #0x10
0x32ff04  e5ffff17           b        #0x32fe98
# CFG: 0x32fe00->0x32fe18/ConditionalFalse 0x32fe00->0x32fee4/ConditionalTrue 0x32fe18->0x32fe80/ConditionalFalse 0x32fe18->0x32feec/ConditionalTrue 0x32fe80->0x32fe98/Fallthrough 0x32fe98->0x32feb4/ConditionalFalse 0x32fe98->0x32fecc/ConditionalTrue 0x32feb4->0x32fec8/ConditionalFalse 0x32feb4->0x32fecc/ConditionalTrue 0x32fec8->0x32fecc/Fallthrough 0x32fee4->0x32fe18/Branch 0x32feec->0x32fe98/Branch

# Category._enumToString at 0x34d4f8 (100 bytes)
0x34d4f8  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x34d4fc  fd030faa           mov      x29, x15
0x34d500  ef4100d1           sub      x15, x15, #0x10
0x34d504  e00301aa           mov      x0, x1
0x34d508  a1831ff8           stur     x1, [x29, #-8]
0x34d50c  502740f9           ldr      x16, [x26, #0x48]
0x34d510  ff0110eb           cmp      x15, x16
0x34d514  09020054           b.ls     #0x34d554
0x34d518  e10316aa           mov      x1, x22
0x34d51c  820080d2           mov      x2, #4
0x34d520  7f030394           bl       #0x40e31c
0x34d524  70334091           add      x16, x27, #0xc, lsl #12
0x34d528  107247f9           ldr      x16, [x16, #0xee0]  # pool[6618] = "Category."
0x34d52c  10f000b8           stur     w16, [x0, #0xf]
0x34d530  a1835ff8           ldur     x1, [x29, #-8]
0x34d534  22f040b8           ldur     w2, [x1, #0xf]
0x34d538  42801c8b           add      x2, x2, x28, lsl #32
0x34d53c  023001b8           stur     w2, [x0, #0x13]
0x34d540  e00100f9           str      x0, [x15]
0x34d544  c483f997           bl       #0x1ae454
0x34d548  ef031daa           mov      x15, x29
0x34d54c  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x34d550  c0035fd6           ret      
0x34d554  b5030394           bl       #0x40e428
0x34d558  f0ffff17           b        #0x34d518
# CFG: 0x34d4f8->0x34d518/ConditionalFalse 0x34d4f8->0x34d554/ConditionalTrue 0x34d554->0x34d518/Branch

# top_level.main at 0x410690 (48 bytes)
0x410690  fd79bfa9           stp      x29, x30, [x15, #-0x10]!
0x410694  fd030faa           mov      x29, x15
0x410698  502740f9           ldr      x16, [x26, #0x48]
0x41069c  ff0110eb           cmp      x15, x16
0x4106a0  c9000054           b.ls     #0x4106b8
0x4106a4  07000094           bl       #0x4106c0
0x4106a8  e00316aa           mov      x0, x22
0x4106ac  ef031daa           mov      x15, x29
0x4106b0  fd79c1a8           ldp      x29, x30, [x15], #0x10
0x4106b4  c0035fd6           ret      
0x4106b8  5cf7ff97           bl       #0x40e428
0x4106bc  faffff17           b        #0x4106a4
# CFG: 0x410690->0x4106a4/ConditionalFalse 0x410690->0x4106b8/ConditionalTrue 0x4106b8->0x4106a4/Branch
