// Re-con (single-agent, read-only) at the post-do-init DM-construction fork.
// SH186 recon task-0 mapped the ONE reachable DM-touching path:
//   nativeAppBridgeStartLuaAppDM (0x1023efe2c) -> dispatcher 0x102baeeec
//   -> GlobalInit do-init (0x102206c40) -> match-dispatch 0x102206df4 `br x1`.
// Fresh disasm (this file) pins the once-guard acquire, the closure build, and
// the dispatch contract so a future drive (or a proof-of-dead-end) starts from a
// concrete, byte-anchored map instead of the SH186 "judged same-difficulty".
// Read-only. Usage: dm_construction_fork [libroblox.so]
use arm64jit::decode::{decode, Inst};
use libloader::elf::load_elf_image;

fn dump(el: &libloader::elf::LoadedElf, label: &str, start: u64, max: u64) {
    println!("\n== {label}: {start:#x} (+{max} bytes) ==");
    let host = match el.host_addr_of(start) {
        Some(h) => h,
        None => {
            println!("  (not mapped / not in image)");
            return;
        }
    };
    let bytes = unsafe { std::slice::from_raw_parts(host as *const u8, max as usize) };
    let words = unsafe { bytes.align_to::<u32>().1 };
    for i in 0..words.len().min((max / 4) as usize) {
        let w = words[i];
        let s = format!("{:?}", decode(w));
        let one = s.lines().next().unwrap_or("").trim().to_string();
        let annotation = if one.starts_with("B {") || one.starts_with("BCond") || one.starts_with("Tbz") || one.starts_with("Cbz") {
            let d2 = decode(w);
            let disp = match &d2 {
                Inst::B { imm, .. } => *imm as i64,
                Inst::BCond { imm, .. } => *imm as i64,
                Inst::Tbz { imm, .. } => *imm as i64,
                Inst::Cbz { imm, .. } => *imm as i64,
                _ => 0,
            };
            let dest = (start as i64) + (i as i64) * 4 + disp;
            format!("  ; {one} -> {dest:#x}")
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

    dump(&el, "StartLuaAppDM 0x1023efe2c", 0x1023efe2c, 0x50);
    dump(&el, "startapp dispatcher 0x102baeeec", 0x102baeeec, 0x70);
    dump(&el, "do-init 0x102206c40 (once-guard tbz + lambda)", 0x102206c40, 0xf8);
    dump(&el, "closure-build 0x102206db8 -> br x1", 0x102206db8, 0x78);
    dump(&el, "init-path 0x102206d10 (once-guard bit0 clear)", 0x102206d10, 0x40);

    println!("\n(contract anchors: once-guard [0x106a68410]; dispatch reads x0=[x19+4] -> x8=[x0] -> x1=[x8+0x30] -> br x1)");
}