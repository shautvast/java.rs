use std::collections::HashMap;

use log::debug;

use crate::classloader::io::{read_i32, read_lookupswitch, read_tableswitch, read_u16, read_u8, read_wide_opcode};
use crate::vm::opcodes::Opcode::{self, *};

pub(crate) fn parse_code(opcodes: &[u8]) -> Vec<Opcode> {
    let mut code: HashMap<u16, (u16, Opcode)> = HashMap::new();
    let mut c = 0;
    let mut opcode_index: u16 = 0;
    // debug!("len {:?}", opcodes.len());
    while c < opcodes.len() {
        let opcode = get_opcode(opcodes, &mut c);
        code.insert(c as u16, (opcode_index, opcode));
        opcode_index += 1;
    }
    let code2 = code.clone(); //clone to look up

    // for jumps, map index of opcode as u8 to index of opcode as enum
    code.into_iter().map(|(_, (_, opcode))|
        match opcode {
            IFNULL(goto) => {
                debug!("goto {:?}", goto);
                debug!("{:?}", code2);
                IFNULL(code2.get(&goto).unwrap().0)
            }
            IFNONNULL(goto) => {
                debug!("goto {:?}", &goto);
                debug!("{:?}", code2);
                IFNONNULL(code2.get(&goto).unwrap().0)
            }
            //TODO more jump instructions
            _ => opcode
        }
    ).collect()
}

fn get_opcode(opcodes: &[u8], c: &mut usize) -> Opcode {
    let opcode_u8 = read_u8(opcodes, c);

    let opcode = match opcode_u8 {
        0 => NOP,
        1 => ACONST_NULL,
        2 => ICONST(-1),
        3 => ICONST(0),
        4 => ICONST(1),
        5 => ICONST(2),
        6 => ICONST(3),
        7 => ICONST(4),
        8 => ICONST(5),
        9 => LCONST(0),
        10 => LCONST(1),
        11 => FCONST(0),
        12 => FCONST(1),
        13 => FCONST(2),
        14 => DCONST(0),
        15 => DCONST(1),
        16 => BIPUSH(read_u8(opcodes, c)),
        17 => SIPUSH(read_u16(opcodes, c)),
        18 => LDC(read_u8(opcodes, c) as u16),
        19 => LDC_W(read_u16(opcodes, c) as u16),
        20 => LDC2_W(read_u16(opcodes, c)),
        21 => ILOAD(read_u8(opcodes, c)),
        22 => LLOAD(read_u8(opcodes, c)),
        23 => FLOAD(read_u8(opcodes, c)),
        24 => DLOAD(read_u8(opcodes, c)),
        25 => ALOAD(read_u8(opcodes, c)),
        26 => ILOAD(0),
        27 => ILOAD(1),
        28 => ILOAD(2),
        29 => ILOAD(3),
        30 => LLOAD(0),
        31 => LLOAD(1),
        32 => LLOAD(2),
        33 => LLOAD(3),
        34 => FLOAD(0),
        35 => FLOAD(1),
        36 => FLOAD(2),
        37 => FLOAD(3),
        38 => DLOAD(0),
        39 => DLOAD(1),
        40 => DLOAD(2),
        41 => DLOAD(3),
        42 => ALOAD(0),
        43 => ALOAD(1),
        44 => ALOAD(2),
        45 => ALOAD(3),
        46 => IALOAD,
        47 => LALOAD,
        48 => FALOAD,
        49 => DALOAD,
        50 => AALOAD,
        51 => BALOAD,
        52 => CALOAD,
        53 => SALOAD,
        54 => ISTORE(read_u8(opcodes, c)),
        55 => LSTORE(read_u8(opcodes, c)),
        56 => FSTORE(read_u8(opcodes, c)),
        57 => DSTORE(read_u8(opcodes, c)),
        58 => ASTORE(read_u8(opcodes, c)),
        59 => ISTORE(0),
        60 => ISTORE(1),
        61 => ISTORE(2),
        62 => ISTORE(3),
        63 => LSTORE(0),
        64 => LSTORE(1),
        65 => LSTORE(2),
        66 => LSTORE(3),
        67 => FSTORE(0),
        68 => FSTORE(1),
        69 => FSTORE(2),
        70 => FSTORE(3),
        71 => DSTORE(0),
        72 => DSTORE(1),
        73 => DSTORE(2),
        74 => DSTORE(3),
        75 => ASTORE(0),
        76 => ASTORE(1),
        77 => ASTORE(2),
        78 => ASTORE(3),
        79 => IASTORE,
        80 => LASTORE,
        81 => FASTORE,
        82 => DASTORE,
        83 => AASTORE,
        84 => BASTORE,
        85 => CASTORE,
        86 => SASTORE,
        87 => POP,
        89 => DUP,
        90 => DUP_X1,
        91 => DUP_X2,
        92 => DUP2,
        93 => DUP2_X1,
        94 => DUP2_X2,
        96 => IADD,
        97 => LADD,
        98 => FADD,
        99 => DADD,
        100 => ISUB,
        101 => LSUB,
        102 => FSUB,
        103 => DSUB,
        104 => IMUL,
        105 => LMUL,
        106 => FMUL,
        107 => DMUL,
        108 => IDIV,
        109 => LDIV,
        110 => FDIV,
        111 => DDIV,
        112 => IREM,
        113 => LREM,
        114 => FREM,
        115 => DREM,
        116 => INEG,
        117 => LNEG,
        118 => FNEG,
        119 => DNEG,
        120 => ISHL,
        121 => LSHL,
        122 => ISHR,
        123 => LSHR,
        126 => IAND,
        127 => LAND,
        128 => IOR,
        129 => LOR,
        130 => IXOR,
        131 => LXOR,
        132 => IINC(read_u8(opcodes, c), read_u8(opcodes, c)),
        133 => I2L,
        134 => I2F,
        135 => I2D,
        136 => L2I,
        137 => L2F,
        138 => L2D,
        139 => F2I,
        140 => F2L,
        141 => F2D,
        142 => D2I,
        143 => D2L,
        144 => D2F,
        145 => I2B,
        146 => I2C,
        147 => I2S,
        148 => LCMP,
        149 => FCMPL,
        150 => FCMPG,
        151 => DCMPL,
        152 => DCMPG,
        153 => IFEQ(read_u16(opcodes, c)),
        154 => IFNE(read_u16(opcodes, c)),
        155 => IFLT(read_u16(opcodes, c)),
        156 => IFGE(read_u16(opcodes, c)),
        157 => IFGT(read_u16(opcodes, c)),
        158 => IFLE(read_u16(opcodes, c)),
        159 => IF_ICMPEQ(read_u16(opcodes, c)),
        160 => IF_ICMPNE(read_u16(opcodes, c)),
        161 => IF_ICMPLT(read_u16(opcodes, c)),
        162 => IF_ICMPGE(read_u16(opcodes, c)),
        163 => IF_ICMPGT(read_u16(opcodes, c)),
        164 => IF_ICMPLE(read_u16(opcodes, c)),
        165 => IF_ACMPEQ(read_u16(opcodes, c)),
        166 => IF_ACMPNE(read_u16(opcodes, c)),
        167 => GOTO(read_u16(opcodes, c)),
        168 => JSR(read_u16(opcodes, c)),
        169 => RET(read_u8(opcodes, c)),
        170 => TABLESWITCH(read_tableswitch(opcodes, c)),
        171 => LOOKUPSWITCH(read_lookupswitch(opcodes, c)),
        172 => IRETURN,
        174 => FRETURN,
        175 => DRETURN,
        176 => ARETURN,
        177 => RETURN_VOID,
        178 => GETSTATIC(read_u16(opcodes, c)),
        179 => PUTSTATIC(read_u16(opcodes, c)),
        180 => GETFIELD(read_u16(opcodes, c)),
        181 => PUTFIELD(read_u16(opcodes, c)),
        182 => INVOKEVIRTUAL(read_u16(opcodes, c)),
        183 => INVOKESPECIAL(read_u16(opcodes, c)),
        184 => INVOKESTATIC(read_u16(opcodes, c)),
        185 => {
            let index = read_u16(opcodes, c);
            let count = read_u8(opcodes, c);
            *c += 1;
            INVOKEINTERFACE(index, count)
        }
        186 => {
            let i = read_u16(opcodes, c);
            *c += 2;
            INVOKEDYNAMIC(i)
        }
        187 => NEW(read_u16(opcodes, c)),
        188 => NEWARRAY(read_u8(opcodes, c)),
        189 => ANEWARRAY(read_u16(opcodes, c)),
        190 => ARRAYLENGTH,
        191 => ATHROW,
        192 => CHECKCAST(read_u16(opcodes, c)),
        193 => INSTANCEOF(read_u16(opcodes, c)),
        194 => MONITORENTER,
        195 => MONITOREXIT,
        196 => WIDE(Box::new(read_wide_opcode(opcodes, c))),
        197 => MULTIANEWARRAY(read_u16(opcodes, c), read_u8(opcodes, c)),
        198 => {
            let j = read_u16(opcodes, c);
            debug!("ifnull {}",*c as u16 + j - 3);
            IFNULL(*c as u16 + j - 3)
        }
        199 => {
            let j = read_u16(opcodes, c);
            debug!("ifnonnull {} ", *c as u16 + j - 3);
            IFNONNULL(*c as u16 + j - 3)
        }
        200 => GOTOW(read_i32(opcodes, c)),
        201 => JSR_W(read_i32(opcodes, c)),


        _ => panic!("{}", opcode_u8),
    };
    opcode
}