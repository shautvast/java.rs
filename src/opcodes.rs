// pub const aconst_null: u8 = 1; // (0x01)
// pub const fconst_0: u8 = 11; // (0xb) Push float 0
// pub const fconst_1: u8 = 12; // (0xc) Push float 1
// pub const fconst_2: u8 = 13; // (0xd) Push float 2
// pub const dconst_0:u8 = 14; // (0xe) push double 0
// pub const dconst_1:u8 = 15; // (0xf) push double 1
// TODO turn all into references
pub const BIPUSH: &u8 = &16; // (0x10) Push byte
pub const LDC: &u8 = &18; // (0x12) Push item from run-time pub constant pool
pub const LDC_W: &u8 = &19; // (0x13) Push item from run-time constant pool (wide index)
pub const LDC2_W: &u8 = &20; // (0x14) Push long or double from run-time constant pool (wide index)
                             // pub const fload:u8 = 23; // (0x17) Load float from local variable
                             // pub const dload:u8 = 24; // (0x18) load double from local variable
                             // pub const aload:u8 = 25; //0x19
                             //
                             // pub const fload_0:u8 = 34; // (0x22) Load float 0 from local variable
                             // pub const fload_1:u8 = 35; // (0x23) Load float 1 from local variable
                             // pub const fload_2:u8 = 36; // (0x24) Load float 2 from local variable
                             // pub const fload_3:u8 = 37; // (0x25) Load float 3 from local variable
                             // pub const dload_0:u8 = 38; //  (0x26) Load double 0 from local variable
                             // pub const dload_1:u8 = 39; //  (0x27) Load double 1 from local variable
                             // pub const dload_2:u8 = 40; //  (0x28) Load double 2 from local variable
                             // pub const dload_3:u8 = 41; // (0x29) Load double 3 from local variable
pub const ALOAD_0: &u8 = &42; // (0x2a)
                              // pub const aload_1:u8 = 43;// (0x2a)
                              // pub const aload_2:u8 = 44;// (0x2b)
                              // pub const aload_3:u8 = 45;// (0x2c)

// pub const faload: u8 = 48; // (0x30) Load float from array
// pub const daload:u8 = 49; // (0x31) load double from array
// pub const aaload: u8 = 50; // (0x3d)
// pub const baload: u8 = 51; //(0x33)
// pub const caload: u8 = 52; // (0x34)
//
// pub const fstore: u8 = 56; // (0x38) Store float into local variable
// pub const dstore: u8 = 57; // (0x39) store double in local variable
// pub const astore:u8 = 58; // (0x3a)
//
// pub const dstore_0: u8 = 71; // (0x47) store double 0 in local variable
// pub const dstore_1: u8 = 72; // (0x48) store double 1 in local variable
// pub const dstore_2: u8 = 73; // (0x49) store double 2 in local variable
// pub const dstore_3: u8 = 74; // (0x4a) store double 3 in local variable
// pub const astore_0: u8 = 75; // (0x4b)
// pub const astore_1: u8 = 76; // (0x4c)
// pub const astore_2: u8 = 77; // (0x4d)
// pub const astore_3: u8 = 78; // (0x4e)
// pub const fastore: u8 = 81; // (0x51) Store into float array
// pub const dastore:u8 = 82; //(0x52) store into double array
// pub const aastore: u8 = 83; // (0x53)
//
// pub const bastore:u8 = 84; // (0x54)
//
// pub const castore:u8 = 85; // (0x55)
pub const DUP: &u8 = &89; // (0x59) duplicate the top operand stack value
                          // pub const dup_x1: u8 = 90; // (0x5a) Duplicate the top operand stack value and insert two values down
                          // pub const dup_x2: u8 = 91; // (0x5b) Duplicate the top operand stack value and insert two or three values down
                          // pub const dup2: u8 = 92; // (0x5c) Duplicate the top one or two operand stack values
                          // pub const dup2_x1: u8 = 93; //(0x5d) Duplicate the top one or two operand stack values and insert two or three values down
                          // pub const dup2_x2:u8 = 94; // (0x5e) Duplicate the top one or two operand stack values and insert two, three, or four values down
                          // pub const fadd: u8 = 98; // (0x62) Add float
                          // pub const dadd: u8 = 99; // (0x63) add double
                          //
                          // pub const dsub:u8 = 103; // (0x67) subtract double
                          // pub const fmul: u8 = 106; // (0x6a) Multiply float
                          // pub const dmul: u8 = 107; // (0x6b) Multiply double
                          //
                          // pub const fdiv: u8 = 110; // (0x6e) Divide float
                          // pub const ddiv:u8 = 111; // (0x6f) divide double
                          // pub const frem: u8 = 114; // (0x72) Remainder float
                          // pub const drem: u8 = 115; // (0x73) remainder double
                          // pub const fneg: u8 = 118; // (0x76) Negate float
                          // pub const dneg: u8 = 119; // (0x77) Negate double
                          // pub const f2i: u8 = 139; // (0x8b) Convert float to int
                          // pub const f2l: u8 = 140; // (0x8c) Convert float to long
                          // pub const f2d: u8 = 141; // (0x8d) Convert float to double
                          // pub const d2i:u8 = 142; // (0x8e) double to int
                          // pub const d2l:u8 = 143; // (0x8f) double to long
                          // pub const d2f: u8 = 144; // (0x90) double to float
                          // pub const fcmpl:u8 = 149; // (0x95) Compare float (less than)
                          // pub const fcmpg: u8 = 150; // (0x96) Compare float (greater than)
                          // pub const dcmpl:u8 = 151; // (0x97) compare double (less than)
                          // pub const dcmpg:u8 = 152; // (0x98) compare double (greater than)
                          //
pub const IRETURN: &u8 = &172; // (0xac) ireturn
pub const FRETURN: &u8 = &174; // (0xae) Return float from method
pub const DRETURN: &u8 = &175; // (0xaf) Return double from method
                               // pub const areturn: u8 = 176; //(0xb0) return reference
                               // pub const return_v: u8 = 177; // (0xb1) Return void from method (actually 'return' but that's a keyword)
                               // pub const getstatic: u8 = 178; // (0xb2) Get static field from class
pub const GETFIELD: &u8 = &180; // (0xb4) Fetch field from object3
pub const NEW: &u8 = &187; // (0xbb) Create new object
                           // pub const invokevirtual: u8 = 182; // (0xb6) Invoke instance method; dispatch based on class
                           //
pub const INVOKESPECIAL: &u8 = &183; // (0xb7) // nvoke instance method; direct invocation of instance initialization methods and methods of the current class and its supertypes
                                     // pub const anewarray: u8 = 189; // (0xbd)
                                     //
                                     // pub const arraylength: u8 = 190; // (0xbe)
                                     //
                                     // pub const athrow: u8 = 191; // (0xbf)
                                     //
                                     // pub const checkcast: u8 = 192; // (0xc0)
