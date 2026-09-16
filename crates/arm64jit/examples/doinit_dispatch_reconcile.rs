// Resolve the SH225-vs-SH156 discrepancy in the do-init closure-build dispatch.
// SH156 decoded: 0x2206df4 `ldr x0,[x19,#32]`; x8=[x0]=sp->0x635dd68; x1=[x8,#48]=table[+0x30]=0x23eff4c; br x1.
// SH225 decoded: 0x2206df4 `ldr x0,[x19,#4]`; x8=[x0]; x1=[x8,#0x30]; br x1 (NULL binder -> cbz 0x206ea4 soft return).
// These are different loads with different semantics. Fresh authoritative decode of the real .so.
// Read-only recon. Usage: doinit_dispatch_reconcile [libroblox.so]
use arm64jit::decode::{decode, Inst};
use libloader::elf::load_elf_image;

fn dump(el: &libloader::elf::LoadedElf, label: &str, start: u64, max: u64) {
    println!("\n== {label}: {start:#x} (+{max} bytes) ==");
    let host = match el.host_addr_of(start) {
        Some(h) => h,
        None => { println!("  (not mapped / not in image)"); return; }
    };
    let bytes = unsafe { std::slice::from_raw_parts(host as *const u8, max as usize) };
    let words = unsafe { bytes.align_to::<u32>().1 };
    for i in 0..words.len().min((max / 4) as usize) {
        let w = words[i];
        let s = format!("{:?}", decode(w));
        let one = s.lines().next().unwrap_or("").trim().to_string();
        let annotation = if one.starts_with("B {") || one.starts_with("BCond") || one.starts_with("Tbz")
            || one.starts_with("Cbz") || one.starts_with("Adrp") {
            let d2 = decode(w);
            let disp = match &d2 {
                Inst::B { imm, .. } => *imm as i64,
                Inst::BCond { imm, .. } => *imm as i64,
                Inst::Tbz { imm, .. } => *imm as i64,
                Inst::Cbz { imm, .. } => *imm as i64,
                Inst::Adrp { imm, .. } => (*imm as i64) << 12,
                _ => 0,
            };
            format!("  ; {one} -> {:#x}", (start as i64) + (i as i64) * 4 + disp)
        } else {
            format!("  ; {one}")
        };
        println!("  {:#x}: {w:08x}{}", start + (i as u64) * 4, annotation);
    }
}

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/home/hermes-worker/.cache/open-sober/robbox/libroblox.so".to_string());
    let el = load_elf_image(std::path::Path::new(&path)).expect("load elf");
    // closure-build
    dump(&el, "closure-build 0x102206db8 (dispatch region)", 0x102206db8, 0x90);
    // do-init 0x102206c40 full body -> where does the x1 passed to closure-build come from?
    dump(&el, "do-init body 0x102206c40 (x19/x1 provenance)", 0x102206c40, 0x180);
    // dispatcher 0x102baeeec -> do-init call (what x1/x2 does it pass?)
    dump(&el, "dispatcher 0x102baeeec", 0x102baeeec, 0xa0);
    // StartLuaAppDM 0x1023efe2c -> dispatcher call site
    dump(&el, "StartLuaAppDM 0x1023efe2c", 0x1023efe2c, 0x80);
    // do-init body completion continuations (SH225 only pinned fork 0x10221942c prologue)
    dump(&el, "do-init tail dispatch 0x102b9ec9c (0x206db4 target)", 0x102b9ec9c, 0x60);
    dump(&el, "do-init earlier sub-call 0x102bc2eec->? no; sub 0x10220671c", 0x10220671c, 0x40);
    dump(&el, "fork 0x10221942c (0x206ce4 bl target)", 0x10221942c, 0x40);
    dump(&el, "sub-init 0x108201bd4 (0x206cbc bl target)", 0x108201bd4, 0x40);
    // continueAfterFlagsLoaded_ 0x102bd1d68 (the SH165-fwd "Next" session-forward target)
    dump(&el, "continueAfterFlagsLoaded_ 0x102bd1d68", 0x102bd1d68, 0x110);
    // the continuation target it bls into at 0x2bd1df8
    dump(&el, "0x102bcdfc4 (0x2bd1df8 call target)", 0x102bcdfc4, 0x80);
    // engine-init dispatcher 0x102bd8ce8 (decides when vt+0xf8/+0x108/+0x1f0 run)
    dump(&el, "engine-init dispatcher 0x102bd8ce8", 0x102bd8ce8, 0x400);
    // the StartLuaAppDM union-build + dispatcher->do-init call anchors
    dump(&el, "StartLuaAppDM union-build 0x1023efe98", 0x1023efe98, 0x20);
    // dispatcher -> do-init x1/x2 setup + call
    dump(&el, "dispatcher do-init call 0x102baef54", 0x102baef54, 0x20);
    // soft-return epilogue referenced as 0x206ea4 / 0x206e28
    dump(&el, "soft-return epilogue 0x102206e28", 0x102206e28, 0x30);
    println!("\n(contracts to reconcile: [x19+4]=binder vs [x19+32]=union; x1=[x8+0x30] vs [x8+0x30]=table[+0x30]; br target 0x23eff4c vs vt+0x30)");
}