// Re-decode the genuine RBX::DataModel vtable rows at the SH187-CORRECTED
// vptr base (0x1067162e8 / 0x1067163a0 / 0x1067163f8) and reconcile the
// SH186c "manufacture-lever row is a NULL-stub, cannot reach a consumer"
// verdict, which was decoded at the +8-slipped addresses (0x1067162f0 /
// 0x1067163a8 / 0x106716400) and predates SH187's base correction.
//
// SH187 (frontier-sh187-dm-vptr-base-corrected.md) FALSIFIED the SH179-186
// proof-of-dead-end and left an explicit open NEXT: "re-derive the post-DM
// content path against the corrected base - prior 'migration-gate' closures
// were built on the wrong +8 vptr." This is that re-derivation at the vtable
// row level: read the loader-relocated slots (load_elf_image applies the
// R_AARCH64_RELATIVE relocs to .data.rel.ro) and classify each row's slot-2
// dispatch target against the three SH186c targets.
//
// Usage: dm_vtable_corrected [libroblox.so]
use libloader::elf::load_elf_image;

// Guest vptr bases. Rows decode 16 slots (slot N at base + 8*N).
const ROWS: &[(&str, u64)] = &[
    ("primary   (corrected base, SH187)", 0x106_7162e8),
    ("secondary (corrected base, SH187)", 0x106_7163a0),
    ("tertiary  (corrected base, SH187)", 0x106_7163f8),
    // The +8-slipped addresses every SH179-186 closure decoded (for contrast).
    ("primary   (+8 slip, SH179-186)", 0x106_7162f0),
    ("secondary (+8 slip, SH179-186)", 0x106_7163a8),
    ("tertiary  (+8 slip, SH179-186)", 0x106_716400),
];

// SH186c slot-2 (vt+0x10) classifications (guest).
const NULL_STUB: u64 = 0x102_29c2a4; // mov x0,xzr; ret  (SH186c/SH195)
const REAL_CONSUMER: u64 = 0x102_40a8b8; // -> 0x10240a8 (real app-shell consumer)
const DESTRUCTOR: u64 = 0x105_7d07f8; // destructor path

fn classify(target: u64) -> &'static str {
    if target == NULL_STUB {
        "NULL-stub (mov x0,xzr; ret)"
    } else if target == REAL_CONSUMER {
        "REAL consumer row (dies on deep NULL members, no GuiObject)"
    } else if target == DESTRUCTOR {
        "DESTRUCTOR row"
    } else {
        "OTHER (unclassified)"
    }
}

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/home/hermes-worker/.cache/open-sober/android-env/lib/libroblox.so".to_string());
    let el = load_elf_image(std::path::Path::new(&path)).expect("load elf");

    println!("=== genuine RBX::DataModel vtable rows at corrected vs +8 base ===");
    println!("(slots read from loader-relocated .data.rel.ro; guest = file_vaddr + 0x100000000)\n");

    for (label, row) in ROWS {
        println!("-- {label} --");
        for i in 0..16usize {
            let slot_addr = row + (i as u64) * 8;
            let host = el.host_addr_of(slot_addr);
            let val = host.map(|h| unsafe { (h as *const u64).read_unaligned() }).unwrap_or(0);
            let extra = match i {
                2 => format!("  <- slot-2 {}", classify(val)),
                6 => format!("  <- vt+0x30"),
                0 => format!("  <- vt+0x0"),
                _ => String::new(),
            };
            println!("  slot[{i:2}] @{slot_addr:#x} = {val:#x}{extra}");
        }
        println!();
    }

    // Disassemble the notable dispatch targets so we can classify what the
    // SH187-corrected manufactured DM actually reaches:
    // * corrected primary slot-2 (the cb's `blr [DM_vt+0x10]` target)
    // * the "consumer" 0x10240a8b8 and "null-stub" 0x10229c2a4 for contrast.
    println!("=== dispatch-target disassembly ===");
    let targets: &[(&str, u64)] = &[
        ("corrected primary slot-2 (cb dispatch target)", 0x1057d19bc),
        ("corrected primary slot-6 vt+0x30", 0x1057d1b9c),
        ("corrected primary slot-7 (SH187 'app-shell ctor')", 0x1057d6ef4),
        ("corrected secondary slot-0 (SH186c 'real consumer')", 0x10240a8b8),
        ("'null-stub' (SH186c slot-2 +8 placeholder)", 0x10229c2a4),
        ("'destructor' (tertiary slot-1)", 0x1057d07f8),
        ("tertiary slot-0", 0x1057d07a4),
    ];
    for (label, t) in targets {
        let host = el.host_addr_of(*t).expect("target in image");
        println!("-- {label}: {t:#x} --");
        let mut off = 0u64;
        for _ in 0..16 {
            let w = unsafe { (host as *const u8).add(off as usize).cast::<u32>().read_unaligned() };
            let d = arm64jit::decode::decode(w);
            // Compact one-line: decode's Display is multi-line; print raw + a small tag.
            let tag = format!("{d:?}");
            // decode() inst Display tends to be long; show first field only.
            let tag = tag.trim_start_matches('"').lines().next().unwrap_or("").to_string();
            println!("  {:#x}: {w:08x}  {tag}", t + off);
            // rough heuristic to stop at a ret
            if w == 0xd65f03c0 {
                break;
            }
            off += 4;
        }
        println!();
    }
}