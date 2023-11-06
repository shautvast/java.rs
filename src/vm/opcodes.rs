//comment line should be above declaration

use once_cell::sync::Lazy;

pub const NOP: u8 = 0;
// (0x0) Do nothing
pub const ACONST_NULL: u8 = 1;
// (0x01) Push null
pub const ICONST_M1: u8 = 2;
// (0x2) Push int constant -1
pub const ICONST_0: u8 = 3;
// (0x3) Push int constant 0
pub const ICONST_1: u8 = 4;
// (0x4) Push int constant 1
pub const ICONST_2: u8 = 5;
// (0x5) Push int constant 2
pub const ICONST_3: u8 = 6;
// (0x6) Push int constant 3
pub const ICONST_4: u8 = 7;
// (0x7) Push int constant 4
pub const ICONST_5: u8 = 8;
// (0x8) Push int constant 5
pub const LCONST_0: u8 = 9;
// (0x9) Push long constant 0
pub const LCONST_1: u8 = 10;
// (0xa) Push long constant 1
pub const FCONST_0: u8 = 11;
// (0xb) Push float 0
pub const FCONST_1: u8 = 12;
// (0xc) Push float 1
pub const FCONST_2: u8 = 13;
// (0xd) Push float 2
pub const DCONST_0: u8 = 14;
// (0xe) push double 0
pub const DCONST_1: u8 = 15;
// (0xe) push double 1
pub const BIPUSH: u8 = 16;
// (0x10) Push byte
pub const SIPUSH: u8 = 17;
// (0x11) Push short
pub const LDC: u8 = 18;
// (0x12) Push item from run-time pub constant pool
pub const LDC_W: u8 = 19;
// (0x13) Push item from run-time constant pool (wide index)
pub const LDC2_W: u8 = 20;
// (0x14) Push long or double from run-time constant pool (wide index)
pub const ILOAD: u8 = 21;
// (0x15) Load int from local variable
pub const LLOAD: u8 = 22;
// (0x16) Load long from local variable
pub const FLOAD: u8 = 23;
// (0x16) Load float from local variable
pub const DLOAD: u8 = 24;
// (0x18) load double from local variable
pub const ALOAD: u8 = 25;
// 0x19 Load reference from local variable
pub const ILOAD_0: u8 = 26;
// (0x1a) Load int from local variable 0
pub const ILOAD_1: u8 = 27;
// (0x1b) Load int from local variable 1
pub const ILOAD_2: u8 = 28;
// (0x1c) Load int from local variable 2
pub const ILOAD_3: u8 = 29;
// (0x1d) Load int from local variable 3
pub const LLOAD_0: u8 = 30;
// (0x1e) Load long from local variable 0
pub const LLOAD_1: u8 = 31;
// (0x1f) Load long from local variable 1
pub const LLOAD_2: u8 = 32;
// (0x20) Load long from local variable 2
pub const LLOAD_3: u8 = 33;
// (0x21) Load long from local variable 3
pub const FLOAD_0: u8 = 34;
// (0x22) Load float from local variable 0
pub const FLOAD_1: u8 = 35;
// (0x23) Load float from local variable 1
pub const FLOAD_2: u8 = 36;
// (0x24) Load float from local variable 2
pub const FLOAD_3: u8 = 37;
// (0x25) Load float from local variable 3
pub const DLOAD_0: u8 = 38;
//  (0x26) Load double from local variable 0
pub const DLOAD_1: u8 = 39;
//  (0x27) Load double from local variable 1
pub const DLOAD_2: u8 = 40;
//  (0x28) Load double from local variable 2
pub const DLOAD_3: u8 = 41;
// (0x29) Load double from local variable 3
pub const ALOAD_0: u8 = 42;
// (0x2a) Load reference from local variable 0
pub const ALOAD_1: u8 = 43;
// (0x2b) Load reference from local variable 1
pub const ALOAD_2: u8 = 44;
// (0x2c) Load reference from local variable 2
pub const ALOAD_3: u8 = 45;
// (0x2d) Load reference from local variable 3
pub const IALOAD: u8 = 46;
// (0x2e) Load int from array
pub const LALOAD: u8 = 47;
// (0x2f) Load long from array
pub const FALOAD: u8 = 48;
// (0x30) Load float from array
pub const DALOAD: u8 = 49;
// (0x31) Load double from array
pub const AALOAD: u8 = 50;
// (0x3d) Load reference from array
pub const BALOAD: u8 = 51;
// (0x33) Load byte or boolean from array
pub const CALOAD: u8 = 52;
// (0x34) Load char from array
pub const SALOAD: u8 = 53;
// (0x34) Load short from array
pub const ISTORE: u8 = 54;
// (0x36) Store int into local variable
pub const LSTORE: u8 = 55;
// (0x37) Store long into local variable
pub const FSTORE: u8 = 56;
// (0x38) Store float into local variable
pub const DSTORE: u8 = 57;
// (0x39) store double in local variable
pub const ASTORE: u8 = 58;
// (0x3a)
pub const ISTORE_0: u8 = 59;
// (0x3b) Store int into local variable 0
pub const ISTORE_1: u8 = 60;
// (0x3c) Store int into local variable 1
pub const ISTORE_2: u8 = 61;
// (0x3d) Store int into local variable 2
pub const ISTORE_3: u8 = 62;
// (0x3e) Store int into local variable 3
pub const LSTORE_0: u8 = 63;
// (0x3f) Store long into local variable 0
pub const LSTORE_1: u8 = 64;
// (0x40) Store long into local variable 1
pub const LSTORE_2: u8 = 65;
// (0x41) Store long into local variable 2
pub const LSTORE_3: u8 = 66;
// (0x42) Store long into local variable 3
pub const FSTORE_0: u8 = 67;
// (0x43) Store float into local variable 0
pub const FSTORE_1: u8 = 68;
// (0x44) Store float into local variable 1
pub const FSTORE_2: u8 = 69;
// (0x45) Store float into local variable 2
pub const FSTORE_3: u8 = 70;
// (0x46) Store float into local variable 3
pub const DSTORE_0: u8 = 71;
// (0x47) store double in local variable 0
pub const DSTORE_1: u8 = 72;
// (0x48) store double in local variable 1
pub const DSTORE_2: u8 = 73;
// (0x49) store double in local variable 2
pub const DSTORE_3: u8 = 74;
// (0x4a) store double in local variable 3
pub const ASTORE_0: u8 = 75;
// (0x4b)
pub const ASTORE_1: u8 = 76;
// (0x4c)
pub const ASTORE_2: u8 = 77;
// (0x4d)
pub const ASTORE_3: u8 = 78;
// (0x4e)
pub const IASTORE: u8 = 79;
// (0x4f) Store into int array
pub const LASTORE: u8 = 80;
// (0x50) Store into long array
pub const FASTORE: u8 = 81;
// (0x51) Store into float array
pub const DASTORE: u8 = 82;
// (0x52) store into double array
pub const AASTORE: u8 = 83;
// (0x53) Store into object array
pub const BASTORE: u8 = 84;
// (0x54) Store into byte or boolean array
pub const CASTORE: u8 = 85;
// (0x55) Store into char array
pub const SASTORE: u8 = 86;
// (0x56) Store into short array
pub const POP: u8 = 87;
// (0x57) Pop the top operand stack value
pub const DUP: u8 = 89;
// (0x59) duplicate the top operand stack value
pub const _DUP_X1: u8 = 90;
// (0x5a) Duplicate the top operand stack value and insert two values down
pub const _DUP_X2: u8 = 91;
// (0x5b) Duplicate the top operand stack value and insert two or three values down
pub const _DUP2: u8 = 92;
// (0x5c) Duplicate the top one or two operand stack values
pub const _DUP2_X1: u8 = 93;
//(0x5d) Duplicate the top one or two operand stack values and insert two or three values down
pub const _DUP2_X2: u8 = 94; // (0x5e) Duplicate the top one or two operand stack values and insert two, three, or four values down

pub const IADD:u8 = 96;
pub const _FADD: u8 = 98;
// (0x62) Add float
pub const _DADD: u8 = 99; // (0x63) add double

pub const _DSUB: u8 = 103;
// (0x67) subtract double
pub const _FMUL: u8 = 106;
// (0x6a) Multiply float
pub const _DMUL: u8 = 107; // (0x6b) Multiply double
pub const IDIV:u8 = 108;
pub const _FDIV: u8 = 110;
// (0x6e) Divide float
pub const _DDIV: u8 = 111;
// (0x6f) divide double
pub const _FREM: u8 = 114;
// (0x72) Remainder float
pub const _DREM: u8 = 115;
// (0x73) remainder double
pub const _FNEG: u8 = 118;
// (0x76) Negate float
pub const _DNEG: u8 = 119; // (0x77) Negate double

pub const ISHR:u8 = 122;
pub const _F2I: u8 = 139;
// (0x8b) Convert float to int
pub const _F2L: u8 = 140;
// (0x8c) Convert float to long
pub const _F2D: u8 = 141;
// (0x8d) Convert float to double
pub const _D2I: u8 = 142;
// (0x8e) double to int
pub const _D2L: u8 = 143;
// (0x8f) double to long
pub const _D2F: u8 = 144; // (0x90) double to float

pub const _FCMPL: u8 = 149;
// (0x95) Compare float (less than)
pub const _FCMPG: u8 = 150;
// (0x96) Compare float (greater than)
pub const _DCMPL: u8 = 151;
// (0x97) compare double (less than)
pub const _DCMPG: u8 = 152; // (0x98) compare double (greater than)

pub const IFEQ: u8 = 153;
// (0x99)
pub const IFNE: u8 = 154;
// (0x9a)
pub const IFLT: u8 = 155;
// (0x9b)
pub const IFGE: u8 = 156;
// (0x9c)
pub const IFGT: u8 = 157;
// (0x9d)
pub const IFLE: u8 = 158; // (0x9e)

pub const IF_ICMPEQ: u8 = 159;
// (0x9f) Branch if int comparison succeeds
pub const IF_ICMPNE: u8 = 160;
// (0x9f) Branch if int comparison succeeds
pub const IF_ICMPLT: u8 = 161;
// (0x9f) Branch if int comparison succeeds
pub const IF_ICMPGE: u8 = 162;
// (0x9f) Branch if int comparison succeeds
pub const IF_ICMPGT: u8 = 163;
// (0x9f) Branch if int comparison succeeds
pub const IF_ICMPLE: u8 = 164;
// (0x9f) Branch if int comparison succeeds
pub const GOTO: u8 = 167; // (0xa7) Branch always

pub const IRETURN: u8 = 172;
// (0xac) ireturn
pub const FRETURN: u8 = 174;
// (0xae) Return float from method
pub const DRETURN: u8 = 175;
// (0xaf) Return double from method
pub const ARETURN: u8 = 176;
//(0xb0) return reference
pub const RETURN_VOID: u8 = 177;
// (0xb1) Return void from method (actually 'return' but that's a keyword)
pub const GETSTATIC: u8 = 178;
// (0xb2) Get static field from class
pub const PUTSTATIC: u8 = 179;
// (0xb3) Set static field in class
pub const GETFIELD: u8 = 180;
// (0xb4) Fetch field from object3
pub const PUTFIELD: u8 = 181;
// (0xb5) Set field in object
pub const INVOKEVIRTUAL: u8 = 182;
// (0xb6) Invoke instance method; dispatch based on class
pub const INVOKESPECIAL: u8 = 183;
// (0xb7) // nvoke instance method; direct invocation of instance initialization methods and methods of the current class and its supertypes
pub const INVOKESTATIC: u8 = 184;
// (0xb8) Invoke a class (static) method
pub const NEW: u8 = 187;
// (0xbb) Create new object
pub const NEWARRAY:u8 = 188;
pub const ANEWARRAY: u8 = 189;
// (0xbd)
pub const ARRAYLENGTH: u8 = 190;
// (0xbe)
pub const _ATHROW: u8 = 191;
// (0xbf)
pub const _CHECKCAST: u8 = 192;
// (0xc0)
pub const MONITORENTER: u8 = 194;
pub const MONITOREXIT: u8 = 195;
pub const IFNULL: u8 = 198;
pub const IFNONNULL: u8 = 199;

pub const OPCODES:Lazy<Vec<&str>> = Lazy::new(|| {
    let mut opcodes = vec!["";256];
    opcodes[NOP  as usize] = "NOP" ;
    opcodes[ACONST_NULL  as usize] = "ACONST_NULL" ;
    opcodes[ICONST_M1  as usize] = "ICONST_M1" ;
    opcodes[ICONST_0  as usize] = "ICONST_0" ;
    opcodes[ICONST_1  as usize] = "ICONST_1" ;
    opcodes[ICONST_2  as usize] = "ICONST_2" ;
    opcodes[ICONST_3  as usize] = "ICONST_3" ;
    opcodes[ICONST_4  as usize] = "ICONST_4" ;
    opcodes[ICONST_5  as usize] = "ICONST_5" ;
    opcodes[LCONST_0  as usize] = "LCONST_0" ;
    opcodes[LCONST_1  as usize] = "LCONST_1" ;
    opcodes[FCONST_0  as usize] = "FCONST_0" ;
    opcodes[FCONST_1  as usize] = "FCONST_1" ;
    opcodes[FCONST_2  as usize] = "FCONST_2" ;
    opcodes[DCONST_0  as usize] = "DCONST_0" ;
    opcodes[DCONST_1  as usize] = "DCONST_1" ;
    opcodes[BIPUSH  as usize] = "BIPUSH" ;
    opcodes[SIPUSH  as usize] = "SIPUSH" ;
    opcodes[LDC  as usize] = "LDC" ;
    opcodes[LDC_W  as usize] = "LDC_W" ;
    opcodes[LDC2_W  as usize] = "LDC2_W" ;
    opcodes[ILOAD  as usize] = "ILOAD" ;
    opcodes[LLOAD  as usize] = "LLOAD" ;
    opcodes[FLOAD  as usize] = "FLOAD" ;
    opcodes[DLOAD  as usize] = "DLOAD" ;
    opcodes[ALOAD  as usize] = "ALOAD" ;
    opcodes[ILOAD_0  as usize] = "ILOAD_0" ;
    opcodes[ILOAD_1  as usize] = "ILOAD_1" ;
    opcodes[ILOAD_2  as usize] = "ILOAD_2" ;
    opcodes[ILOAD_3  as usize] = "ILOAD_3" ;
    opcodes[LLOAD_0  as usize] = "LLOAD_0" ;
    opcodes[LLOAD_1  as usize] = "LLOAD_1" ;
    opcodes[LLOAD_2  as usize] = "LLOAD_2" ;
    opcodes[LLOAD_3  as usize] = "LLOAD_3" ;
    opcodes[FLOAD_0  as usize] = "FLOAD_0" ;
    opcodes[FLOAD_1  as usize] = "FLOAD_1" ;
    opcodes[FLOAD_2  as usize] = "FLOAD_2" ;
    opcodes[FLOAD_3  as usize] = "FLOAD_3" ;
    opcodes[DLOAD_0  as usize] = "DLOAD_0" ;
    opcodes[DLOAD_1  as usize] = "DLOAD_1" ;
    opcodes[DLOAD_2  as usize] = "DLOAD_2" ;
    opcodes[DLOAD_3  as usize] = "DLOAD_3" ;
    opcodes[ALOAD_0  as usize] = "ALOAD_0" ;
    opcodes[ALOAD_1  as usize] = "ALOAD_1" ;
    opcodes[ALOAD_2  as usize] = "ALOAD_2" ;
    opcodes[ALOAD_3  as usize] = "ALOAD_3" ;
    opcodes[IALOAD  as usize] = "IALOAD" ;
    opcodes[LALOAD  as usize] = "LALOAD" ;
    opcodes[FALOAD  as usize] = "FALOAD" ;
    opcodes[DALOAD  as usize] = "DALOAD" ;
    opcodes[AALOAD  as usize] = "AALOAD" ;
    opcodes[BALOAD  as usize] = "BALOAD" ;
    opcodes[CALOAD  as usize] = "CALOAD" ;
    opcodes[SALOAD  as usize] = "SALOAD" ;
    opcodes[ISTORE  as usize] = "ISTORE" ;
    opcodes[LSTORE  as usize] = "LSTORE" ;
    opcodes[FSTORE  as usize] = "FSTORE" ;
    opcodes[DSTORE  as usize] = "DSTORE" ;
    opcodes[ASTORE  as usize] = "ASTORE" ;
    opcodes[ISTORE_0  as usize] = "ISTORE_0" ;
    opcodes[ISTORE_1  as usize] = "ISTORE_1" ;
    opcodes[ISTORE_2  as usize] = "ISTORE_2" ;
    opcodes[ISTORE_3  as usize] = "ISTORE_3" ;
    opcodes[LSTORE_0  as usize] = "LSTORE_0" ;
    opcodes[LSTORE_1  as usize] = "LSTORE_1" ;
    opcodes[LSTORE_2  as usize] = "LSTORE_2" ;
    opcodes[LSTORE_3  as usize] = "LSTORE_3" ;
    opcodes[FSTORE_0  as usize] = "FSTORE_0" ;
    opcodes[FSTORE_1  as usize] = "FSTORE_1" ;
    opcodes[FSTORE_2  as usize] = "FSTORE_2" ;
    opcodes[FSTORE_3  as usize] = "FSTORE_3" ;
    opcodes[DSTORE_0  as usize] = "DSTORE_0" ;
    opcodes[DSTORE_1  as usize] = "DSTORE_1" ;
    opcodes[DSTORE_2  as usize] = "DSTORE_2" ;
    opcodes[DSTORE_3  as usize] = "DSTORE_3" ;
    opcodes[ASTORE_0  as usize] = "ASTORE_0" ;
    opcodes[ASTORE_1  as usize] = "ASTORE_1" ;
    opcodes[ASTORE_2  as usize] = "ASTORE_2" ;
    opcodes[ASTORE_3  as usize] = "ASTORE_3" ;
    opcodes[IASTORE  as usize] = "IASTORE" ;
    opcodes[LASTORE  as usize] = "LASTORE" ;
    opcodes[FASTORE  as usize] = "FASTORE" ;
    opcodes[DASTORE  as usize] = "DASTORE" ;
    opcodes[AASTORE  as usize] = "AASTORE" ;
    opcodes[BASTORE  as usize] = "BASTORE" ;
    opcodes[CASTORE  as usize] = "CASTORE" ;
    opcodes[SASTORE  as usize] = "SASTORE" ;
    opcodes[POP  as usize] = "POP" ;
    opcodes[DUP  as usize] = "DUP" ;
    opcodes[_DUP_X1  as usize] = "_DUP_X1" ;
    opcodes[_DUP_X2  as usize] = "_DUP_X2" ;
    opcodes[_DUP2  as usize] = "_DUP2" ;
    opcodes[_DUP2_X1  as usize] = "_DUP2_X1" ;
    opcodes[_DUP2_X2  as usize] = "_DUP2_X2" ;
    opcodes[IADD as usize] = "IADD";
    opcodes[_FADD  as usize] = "_FADD" ;
    opcodes[_DADD  as usize] = "_DADD" ;
    opcodes[_DSUB  as usize] = "_DSUB" ;
    opcodes[_FMUL  as usize] = "_FMUL" ;
    opcodes[_DMUL  as usize] = "_DMUL" ;
    opcodes[IDIV  as usize] = "IDIV" ;
    opcodes[_FDIV  as usize] = "_FDIV" ;
    opcodes[_DDIV  as usize] = "_DDIV" ;
    opcodes[_FREM  as usize] = "_FREM" ;
    opcodes[_DREM  as usize] = "_DREM" ;
    opcodes[_FNEG  as usize] = "_FNEG" ;
    opcodes[_DNEG  as usize] = "_DNEG" ;
    opcodes[ISHR  as usize] = "ISHR" ;
    opcodes[_F2I  as usize] = "_F2I" ;
    opcodes[_F2L  as usize] = "_F2L" ;
    opcodes[_F2D  as usize] = "_F2D" ;
    opcodes[_D2I  as usize] = "_D2I" ;
    opcodes[_D2L  as usize] = "_D2L" ;
    opcodes[_D2F  as usize] = "_D2F" ;
    opcodes[_FCMPL  as usize] = "_FCMPL" ;
    opcodes[_FCMPG  as usize] = "_FCMPG" ;
    opcodes[_DCMPL  as usize] = "_DCMPL" ;
    opcodes[_DCMPG  as usize] = "_DCMPG" ;
    opcodes[IFEQ  as usize] = "IFEQ" ;
    opcodes[IFNE  as usize] = "IFNE" ;
    opcodes[IFLT  as usize] = "IFLT" ;
    opcodes[IFGE  as usize] = "IFGE" ;
    opcodes[IFGT  as usize] = "IFGT" ;
    opcodes[IFLE  as usize] = "IFLE" ;
    opcodes[IF_ICMPEQ  as usize] = "IF_ICMPEQ" ;
    opcodes[IF_ICMPNE  as usize] = "IF_ICMPNE" ;
    opcodes[IF_ICMPLT  as usize] = "IF_ICMPLT" ;
    opcodes[IF_ICMPGE  as usize] = "IF_ICMPGE" ;
    opcodes[IF_ICMPGT  as usize] = "IF_ICMPGT" ;
    opcodes[IF_ICMPLE  as usize] = "IF_ICMPLE" ;
    opcodes[GOTO  as usize] = "GOTO" ;
    opcodes[IRETURN  as usize] = "IRETURN" ;
    opcodes[FRETURN  as usize] = "FRETURN" ;
    opcodes[DRETURN  as usize] = "DRETURN" ;
    opcodes[ARETURN  as usize] = "ARETURN" ;
    opcodes[RETURN_VOID  as usize] = "RETURN_VOID" ;
    opcodes[GETSTATIC  as usize] = "GETSTATIC" ;
    opcodes[PUTSTATIC  as usize] = "PUTSTATIC" ;
    opcodes[GETFIELD  as usize] = "GETFIELD" ;
    opcodes[PUTFIELD  as usize] = "PUTFIELD" ;
    opcodes[INVOKEVIRTUAL  as usize] = "INVOKEVIRTUAL" ;
    opcodes[INVOKESPECIAL  as usize] = "INVOKESPECIAL" ;
    opcodes[INVOKESTATIC  as usize] = "INVOKESTATIC" ;
    opcodes[NEW  as usize] = "NEW" ;
    opcodes[NEWARRAY  as usize] = "NEWARRAY" ;
    opcodes[ANEWARRAY  as usize] = "ANEWARRAY" ;
    opcodes[ARRAYLENGTH  as usize] = "ARRAYLENGTH" ;
    opcodes[_ATHROW  as usize] = "_ATHROW" ;
    opcodes[_CHECKCAST  as usize] = "_CHECKCAST" ;
    opcodes[MONITORENTER  as usize] = "MONITORENTER" ;
    opcodes[MONITOREXIT  as usize] = "MONITOREXIT" ;
    opcodes[IFNULL  as usize] = "IFNULL" ;
    opcodes[IFNONNULL  as usize] = "IFNONNULL" ;
    opcodes
});