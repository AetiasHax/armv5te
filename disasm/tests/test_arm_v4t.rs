use unarm::v4t::arm::Ins;

macro_rules! assert_asm {
    ($code:literal, $disasm:literal) => {{
        let ins = Ins::new($code);
        let parsed = ins.parse();
        assert_eq!(parsed.display(Default::default()).to_string(), $disasm)
    }};
}

#[test]
fn test_adc() {
    assert_asm!(0xe0a12003, "adc r2, r1, r3");
    assert_asm!(0xe2a45e23, "adc r5, r4, #0x230");
    assert_asm!(0x10ab960a, "adcne r9, r11, r10, lsl #0xc");
    assert_asm!(0x40a5f238, "adcmi pc, r5, r8, lsr r2");
    assert_asm!(0x70a2046e, "adcvc r0, r2, lr, ror #0x8");
    assert_asm!(0xb0a87060, "adclt r7, r8, r0, rrx");
    assert_asm!(0xe0b52153, "adcs r2, r5, r3, asr r1");
}

#[test]
fn test_add() {
    assert_asm!(0xe0812003, "add r2, r1, r3");
    assert_asm!(0xe2845e23, "add r5, r4, #0x230");
    assert_asm!(0x108b960a, "addne r9, r11, r10, lsl #0xc");
    assert_asm!(0x4085f238, "addmi pc, r5, r8, lsr r2");
    assert_asm!(0x7082046e, "addvc r0, r2, lr, ror #0x8");
    assert_asm!(0xb0887060, "addlt r7, r8, r0, rrx");
    assert_asm!(0xe0952153, "adds r2, r5, r3, asr r1");
    assert_asm!(0xe28f41a5, "add r4, pc, #0x40000029");
}

#[test]
fn test_and() {
    assert_asm!(0xe0012003, "and r2, r1, r3");
    assert_asm!(0xe2045e23, "and r5, r4, #0x230");
    assert_asm!(0x100b960a, "andne r9, r11, r10, lsl #0xc");
    assert_asm!(0x4005f238, "andmi pc, r5, r8, lsr r2");
    assert_asm!(0x7002046e, "andvc r0, r2, lr, ror #0x8");
    assert_asm!(0xb0087060, "andlt r7, r8, r0, rrx");
    assert_asm!(0xe0152153, "ands r2, r5, r3, asr r1");
}

#[test]
fn test_asr() {
    assert_asm!(0xe1b02153, "asrs r2, r3, r1");
}

#[test]
fn test_b() {
    assert_asm!(0xea000000, "b #0x8");
    assert_asm!(0x0a012345, "beq #0x48d1c");
    assert_asm!(0x1affffff, "bne #0x4");
    assert_asm!(0x2afffffe, "bhs #0x0");
    assert_asm!(0x3afffffd, "blo #-0x4");
}

#[test]
fn test_bl() {
    assert_asm!(0xeb000000, "bl #0x8");
    assert_asm!(0x0b012345, "bleq #0x48d1c");
    assert_asm!(0x1bffffff, "blne #0x4");
    assert_asm!(0x2bfffffe, "blhs #0x0");
    assert_asm!(0x3bfffffd, "bllo #-0x4");
}

#[test]
fn test_bic() {
    assert_asm!(0xe1c12003, "bic r2, r1, r3");
    assert_asm!(0xe3c45e23, "bic r5, r4, #0x230");
    assert_asm!(0x11cb960a, "bicne r9, r11, r10, lsl #0xc");
    assert_asm!(0x41c5f238, "bicmi pc, r5, r8, lsr r2");
    assert_asm!(0x71c2046e, "bicvc r0, r2, lr, ror #0x8");
    assert_asm!(0xb1c87060, "biclt r7, r8, r0, rrx");
    assert_asm!(0xe1d52153, "bics r2, r5, r3, asr r1");
}

#[test]
fn test_bx() {
    assert_asm!(0xe12fff10, "bx r0");
    assert_asm!(0x512fff15, "bxpl r5");
}

#[test]
fn test_cdp() {
    assert_asm!(0xee1234a6, "cdp p4, #1, c3, c2, c6, #5");
    assert_asm!(0xae654341, "cdpge p3, #6, c4, c5, c1, #2");
}

#[test]
fn test_cmn() {
    assert_asm!(0xe1710003, "cmn r1, r3");
    assert_asm!(0xe3740e23, "cmn r4, #0x230");
    assert_asm!(0x117b060a, "cmnne r11, r10, lsl #0xc");
    assert_asm!(0x41750238, "cmnmi r5, r8, lsr r2");
    assert_asm!(0x7172046e, "cmnvc r2, lr, ror #0x8");
    assert_asm!(0xb1780060, "cmnlt r8, r0, rrx");
    assert_asm!(0xe1750153, "cmn r5, r3, asr r1");
}

#[test]
fn test_cmp() {
    assert_asm!(0xe1510003, "cmp r1, r3");
    assert_asm!(0xe3540e23, "cmp r4, #0x230");
    assert_asm!(0x115b060a, "cmpne r11, r10, lsl #0xc");
    assert_asm!(0x41550238, "cmpmi r5, r8, lsr r2");
    assert_asm!(0x7152046e, "cmpvc r2, lr, ror #0x8");
    assert_asm!(0xb1580060, "cmplt r8, r0, rrx");
    assert_asm!(0xe1550153, "cmp r5, r3, asr r1");
}

#[test]
fn test_eor() {
    assert_asm!(0xe0212003, "eor r2, r1, r3");
    assert_asm!(0xe2245e23, "eor r5, r4, #0x230");
    assert_asm!(0x102b960a, "eorne r9, r11, r10, lsl #0xc");
    assert_asm!(0x4025f238, "eormi pc, r5, r8, lsr r2");
    assert_asm!(0x7022046e, "eorvc r0, r2, lr, ror #0x8");
    assert_asm!(0xb0287060, "eorlt r7, r8, r0, rrx");
    assert_asm!(0xe0352153, "eors r2, r5, r3, asr r1");
}

#[test]
fn test_ldc() {
    assert_asm!(0xed132169, "ldc p1, c2, [r3, #-0x1a4]");
    assert_asm!(0x4d332169, "ldcmi p1, c2, [r3, #-0x1a4]!");
    assert_asm!(0x6c332169, "ldcvs p1, c2, [r3], #-0x1a4");
    assert_asm!(0x3db32169, "ldclo p1, c2, [r3, #0x1a4]!");
    assert_asm!(0x5c932169, "ldcpl p1, c2, [r3], {0x69}");
}

#[test]
fn test_ldm() {
    assert_asm!(0xe831aaaa, "ldmda r1!, {r1, r3, r5, r7, r9, r11, sp, pc}");
    assert_asm!(0xb8b25555, "ldmlt r2!, {r0, r2, r4, r6, r8, r10, r12, lr}");
    assert_asm!(0xd913cccc, "ldmdble r3, {r2, r3, r6, r7, r10, r11, lr, pc}");
    assert_asm!(0xc9943333, "ldmibgt r4, {r0, r1, r4, r5, r8, r9, r12, sp}");
    assert_asm!(0xe8550003, "ldmda r5, {r0, r1}^");
    assert_asm!(0xe8568003, "ldmda r6, {r0, r1, pc}^");
    assert_asm!(0xe8778003, "ldmda r7!, {r0, r1, pc}^");
}

#[test]
fn test_ldr() {
    assert_asm!(0xe5912fff, "ldr r2, [r1, #0xfff]");
    assert_asm!(0xe7112003, "ldr r2, [r1, -r3]");
    assert_asm!(0xe79120e3, "ldr r2, [r1, r3, ror #0x1]");
    assert_asm!(0xe5312fff, "ldr r2, [r1, #-0xfff]!");
    assert_asm!(0xe7b12003, "ldr r2, [r1, r3]!");
    assert_asm!(0xe7312063, "ldr r2, [r1, -r3, rrx]!");
    assert_asm!(0xe4912fff, "ldr r2, [r1], #0xfff");
    assert_asm!(0xe6112003, "ldr r2, [r1], -r3");
    assert_asm!(0xe6912023, "ldr r2, [r1], r3, lsr #0x20");
}

#[test]
fn test_ldrb() {
    assert_asm!(0xe5d12fff, "ldrb r2, [r1, #0xfff]");
    assert_asm!(0xe7512003, "ldrb r2, [r1, -r3]");
    assert_asm!(0xe7d120e3, "ldrb r2, [r1, r3, ror #0x1]");
    assert_asm!(0xe5712fff, "ldrb r2, [r1, #-0xfff]!");
    assert_asm!(0xe7f12003, "ldrb r2, [r1, r3]!");
    assert_asm!(0xe7712063, "ldrb r2, [r1, -r3, rrx]!");
    assert_asm!(0xe4d12fff, "ldrb r2, [r1], #0xfff");
    assert_asm!(0xe6512003, "ldrb r2, [r1], -r3");
    assert_asm!(0xe6d12023, "ldrb r2, [r1], r3, lsr #0x20");
}

#[test]
fn test_ldrbt() {
    assert_asm!(0xe4f12fff, "ldrbt r2, [r1], #0xfff");
    assert_asm!(0xe6712003, "ldrbt r2, [r1], -r3");
    assert_asm!(0xe6f12023, "ldrbt r2, [r1], r3, lsr #0x20");
}

#[test]
fn test_ldrh() {
    assert_asm!(0xe1d12fbf, "ldrh r2, [r1, #0xff]");
    assert_asm!(0xe11120b3, "ldrh r2, [r1, -r3]");
    assert_asm!(0xe1712fbf, "ldrh r2, [r1, #-0xff]!");
    assert_asm!(0xe1b120b3, "ldrh r2, [r1, r3]!");
    assert_asm!(0xe0d12fbf, "ldrh r2, [r1], #0xff");
    assert_asm!(0xe01120b3, "ldrh r2, [r1], -r3");
}

#[test]
fn test_ldrsb() {
    assert_asm!(0xe1d12fdf, "ldrsb r2, [r1, #0xff]");
    assert_asm!(0xe11120d3, "ldrsb r2, [r1, -r3]");
    assert_asm!(0xe1712fdf, "ldrsb r2, [r1, #-0xff]!");
    assert_asm!(0xe1b120d3, "ldrsb r2, [r1, r3]!");
    assert_asm!(0xe0d12fdf, "ldrsb r2, [r1], #0xff");
    assert_asm!(0xe01120d3, "ldrsb r2, [r1], -r3");
}

#[test]
fn test_ldrsh() {
    assert_asm!(0xe1d12fff, "ldrsh r2, [r1, #0xff]");
    assert_asm!(0xe11120f3, "ldrsh r2, [r1, -r3]");
    assert_asm!(0xe1712fff, "ldrsh r2, [r1, #-0xff]!");
    assert_asm!(0xe1b120f3, "ldrsh r2, [r1, r3]!");
    assert_asm!(0xe0d12fff, "ldrsh r2, [r1], #0xff");
    assert_asm!(0xe01120f3, "ldrsh r2, [r1], -r3");
}

#[test]
fn test_ldrt() {
    assert_asm!(0xe4b12fff, "ldrt r2, [r1], #0xfff");
    assert_asm!(0xe6312003, "ldrt r2, [r1], -r3");
    assert_asm!(0xe6b12023, "ldrt r2, [r1], r3, lsr #0x20");
}

#[test]
fn test_lsl() {
    assert_asm!(0x11a0960a, "lslne r9, r10, #0xc");
}

#[test]
fn test_lsr() {
    assert_asm!(0x41a0f238, "lsrmi pc, r8, r2");
}

#[test]
fn test_mcr() {
    assert_asm!(0xee2234b6, "mcr p4, #1, r3, c2, c6, #5");
    assert_asm!(0x3ec54351, "mcrlo p3, #6, r4, c5, c1, #2");
}

#[test]
fn test_mla() {
    assert_asm!(0xe0212394, "mla r2, r4, r3, r1");
    assert_asm!(0xa0312394, "mlasge r2, r4, r3, r1");
}

#[test]
fn test_mov() {
    assert_asm!(0xe1a02003, "mov r2, r3");
    assert_asm!(0xe3a05e23, "mov r5, #0x230");
}

#[test]
fn test_mrc() {
    assert_asm!(0xee3234b6, "mrc p4, #1, r3, c2, c6, #5");
    assert_asm!(0x3ed54351, "mrclo p3, #6, r4, c5, c1, #2");
}

#[test]
fn test_mrs() {
    assert_asm!(0xe10f7000, "mrs r7, cpsr");
    assert_asm!(0xe14f7000, "mrs r7, spsr");
}

#[test]
fn test_msr() {
    assert_asm!(0xe36cf042, "msr spsr_fs, #0x42");
    assert_asm!(0xe323f042, "msr cpsr_xc, #0x42");
    assert_asm!(0xe165f001, "msr spsr_sc, r1");
    assert_asm!(0xe12af001, "msr cpsr_fx, r1");
}

#[test]
fn test_mul() {
    assert_asm!(0xe0010293, "mul r1, r3, r2");
    assert_asm!(0x10110293, "mulsne r1, r3, r2");
}

#[test]
fn test_mvn() {
    assert_asm!(0xe1e02003, "mvn r2, r3");
    assert_asm!(0xe3e05e23, "mvn r5, #0x230");
    assert_asm!(0x11e0960a, "mvnne r9, r10, lsl #0xc");
    assert_asm!(0x41e0f238, "mvnmi pc, r8, lsr r2");
    assert_asm!(0x71e0046e, "mvnvc r0, lr, ror #0x8");
    assert_asm!(0xb1e07060, "mvnlt r7, r0, rrx");
    assert_asm!(0xe1f02153, "mvns r2, r3, asr r1");
}

#[test]
fn test_orr() {
    assert_asm!(0xe1812003, "orr r2, r1, r3");
    assert_asm!(0xe3845e23, "orr r5, r4, #0x230");
    assert_asm!(0x118b960a, "orrne r9, r11, r10, lsl #0xc");
    assert_asm!(0x4185f238, "orrmi pc, r5, r8, lsr r2");
    assert_asm!(0x7182046e, "orrvc r0, r2, lr, ror #0x8");
    assert_asm!(0xb1887060, "orrlt r7, r8, r0, rrx");
    assert_asm!(0xe1952153, "orrs r2, r5, r3, asr r1");
}

#[test]
fn test_pop() {
    assert_asm!(0xe8bd0505, "pop {r0, r2, r8, r10}");
    assert_asm!(0xa49d5004, "popge {r5}");
}

#[test]
fn test_push() {
    assert_asm!(0xe92d0505, "push {r0, r2, r8, r10}");
    assert_asm!(0xa52d5004, "pushge {r5}");
}

#[test]
fn test_ror() {
    assert_asm!(0x71a0046e, "rorvc r0, lr, #0x8");
}

#[test]
fn test_rrx() {
    assert_asm!(0xb1a07060, "rrxlt r7, r0");
}

#[test]
fn test_rsb() {
    assert_asm!(0xe0612003, "rsb r2, r1, r3");
    assert_asm!(0xe2645e23, "rsb r5, r4, #0x230");
    assert_asm!(0x106b960a, "rsbne r9, r11, r10, lsl #0xc");
    assert_asm!(0x4065f238, "rsbmi pc, r5, r8, lsr r2");
    assert_asm!(0x7062046e, "rsbvc r0, r2, lr, ror #0x8");
    assert_asm!(0xb0687060, "rsblt r7, r8, r0, rrx");
    assert_asm!(0xe0752153, "rsbs r2, r5, r3, asr r1");
}

#[test]
fn test_rsc() {
    assert_asm!(0xe0e12003, "rsc r2, r1, r3");
    assert_asm!(0xe2e45e23, "rsc r5, r4, #0x230");
    assert_asm!(0x10eb960a, "rscne r9, r11, r10, lsl #0xc");
    assert_asm!(0x40e5f238, "rscmi pc, r5, r8, lsr r2");
    assert_asm!(0x70e2046e, "rscvc r0, r2, lr, ror #0x8");
    assert_asm!(0xb0e87060, "rsclt r7, r8, r0, rrx");
    assert_asm!(0xe0f52153, "rscs r2, r5, r3, asr r1");
}

#[test]
fn test_smull() {
    assert_asm!(0xe0c12394, "smull r2, r1, r4, r3");
    assert_asm!(0xa0d12394, "smullsge r2, r1, r4, r3");
}

#[test]
fn test_stc() {
    assert_asm!(0xed032169, "stc p1, c2, [r3, #-0x1a4]");
    assert_asm!(0x4d232169, "stcmi p1, c2, [r3, #-0x1a4]!");
    assert_asm!(0x6c232169, "stcvs p1, c2, [r3], #-0x1a4");
    assert_asm!(0x3da32169, "stclo p1, c2, [r3, #0x1a4]!");
    assert_asm!(0x5c832169, "stcpl p1, c2, [r3], {0x69}");
}

#[test]
fn test_stm() {
    assert_asm!(0xe821aaaa, "stmda r1!, {r1, r3, r5, r7, r9, r11, sp, pc}");
    assert_asm!(0xb8a25555, "stmlt r2!, {r0, r2, r4, r6, r8, r10, r12, lr}");
    assert_asm!(0xd903cccc, "stmdble r3, {r2, r3, r6, r7, r10, r11, lr, pc}");
    assert_asm!(0xc9843333, "stmibgt r4, {r0, r1, r4, r5, r8, r9, r12, sp}");
    assert_asm!(0xe8450003, "stmda r5, {r0, r1}^");
    assert_asm!(0xe8468003, "stmda r6, {r0, r1, pc}^");
}

#[test]
fn test_str() {
    assert_asm!(0xe5812fff, "str r2, [r1, #0xfff]");
    assert_asm!(0xe7012003, "str r2, [r1, -r3]");
    assert_asm!(0xe78120e3, "str r2, [r1, r3, ror #0x1]");
    assert_asm!(0xe5212fff, "str r2, [r1, #-0xfff]!");
    assert_asm!(0xe7a12003, "str r2, [r1, r3]!");
    assert_asm!(0xe7212063, "str r2, [r1, -r3, rrx]!");
    assert_asm!(0xe4812fff, "str r2, [r1], #0xfff");
    assert_asm!(0xe6012003, "str r2, [r1], -r3");
    assert_asm!(0xe6812023, "str r2, [r1], r3, lsr #0x20");
}

#[test]
fn test_strb() {
    assert_asm!(0xe5c12fff, "strb r2, [r1, #0xfff]");
    assert_asm!(0xe7412003, "strb r2, [r1, -r3]");
    assert_asm!(0xe7c120e3, "strb r2, [r1, r3, ror #0x1]");
    assert_asm!(0xe5612fff, "strb r2, [r1, #-0xfff]!");
    assert_asm!(0xe7e12003, "strb r2, [r1, r3]!");
    assert_asm!(0xe7612063, "strb r2, [r1, -r3, rrx]!");
    assert_asm!(0xe4c12fff, "strb r2, [r1], #0xfff");
    assert_asm!(0xe6412003, "strb r2, [r1], -r3");
    assert_asm!(0xe6c12023, "strb r2, [r1], r3, lsr #0x20");
}

#[test]
fn test_strbt() {
    assert_asm!(0xe4e12fff, "strbt r2, [r1], #0xfff");
    assert_asm!(0xe6612003, "strbt r2, [r1], -r3");
    assert_asm!(0xe6e12023, "strbt r2, [r1], r3, lsr #0x20");
}

#[test]
fn test_strh() {
    assert_asm!(0xe1c12fbf, "strh r2, [r1, #0xff]");
    assert_asm!(0xe10120b3, "strh r2, [r1, -r3]");
    assert_asm!(0xe1612fbf, "strh r2, [r1, #-0xff]!");
    assert_asm!(0xe1a120b3, "strh r2, [r1, r3]!");
    assert_asm!(0xe0c12fbf, "strh r2, [r1], #0xff");
    assert_asm!(0xe00120b3, "strh r2, [r1], -r3");
}

#[test]
fn test_strt() {
    assert_asm!(0xe4a12fff, "strt r2, [r1], #0xfff");
    assert_asm!(0xe6212003, "strt r2, [r1], -r3");
    assert_asm!(0xe6a12023, "strt r2, [r1], r3, lsr #0x20");
}

#[test]
fn test_sub() {
    assert_asm!(0xe0412003, "sub r2, r1, r3");
    assert_asm!(0xe2445e23, "sub r5, r4, #0x230");
    assert_asm!(0x104b960a, "subne r9, r11, r10, lsl #0xc");
    assert_asm!(0x4045f238, "submi pc, r5, r8, lsr r2");
    assert_asm!(0x7042046e, "subvc r0, r2, lr, ror #0x8");
    assert_asm!(0xb0487060, "sublt r7, r8, r0, rrx");
    assert_asm!(0xe0552153, "subs r2, r5, r3, asr r1");
    assert_asm!(0xe24f41a5, "sub r4, pc, #0x40000029");
}

#[test]
fn test_svc() {
    assert_asm!(0xef123456, "svc #0x123456");
    assert_asm!(0x0fabcdef, "svceq #0xabcdef");
}

#[test]
fn test_swp() {
    assert_asm!(0xe1012093, "swp r2, r3, [r1]");
}

#[test]
fn test_swpb() {
    assert_asm!(0xe1412093, "swpb r2, r3, [r1]");
}

#[test]
fn test_teq() {
    assert_asm!(0xe1310003, "teq r1, r3");
    assert_asm!(0xe3340e23, "teq r4, #0x230");
    assert_asm!(0x113b060a, "teqne r11, r10, lsl #0xc");
    assert_asm!(0x41350238, "teqmi r5, r8, lsr r2");
    assert_asm!(0x7132046e, "teqvc r2, lr, ror #0x8");
    assert_asm!(0xb1380060, "teqlt r8, r0, rrx");
    assert_asm!(0xe1350153, "teq r5, r3, asr r1");
}

#[test]
fn test_tst() {
    assert_asm!(0xe1110003, "tst r1, r3");
    assert_asm!(0xe3140e23, "tst r4, #0x230");
    assert_asm!(0x111b060a, "tstne r11, r10, lsl #0xc");
    assert_asm!(0x41150238, "tstmi r5, r8, lsr r2");
    assert_asm!(0x7112046e, "tstvc r2, lr, ror #0x8");
    assert_asm!(0xb1180060, "tstlt r8, r0, rrx");
    assert_asm!(0xe1150153, "tst r5, r3, asr r1");
}

#[test]
fn test_umlal() {
    assert_asm!(0xe0a12394, "umlal r2, r1, r4, r3");
    assert_asm!(0xa0b12394, "umlalsge r2, r1, r4, r3");
}

#[test]
fn test_umull() {
    assert_asm!(0xe0812394, "umull r2, r1, r4, r3");
    assert_asm!(0xa0912394, "umullsge r2, r1, r4, r3");
}
