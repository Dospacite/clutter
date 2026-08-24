// Temporary probe: decode ARM32 VFP words with the project's Capstone to pin
// exact mnemonics for the fallback decoder. Run with:
//   cargo run --manifest-path .tmp-probe/Cargo.toml
use capstone::prelude::*;

fn main() {
    let mut cs = Capstone::new()
        .arm()
        .mode(arch::arm::ArchMode::Arm)
        
        .build()
        .expect("capstone");
    cs.set_skipdata(false).unwrap();
    let words: [&[u8]; 10] = [
        &hex_bytes("ee224b00"),
        &hex_bytes("ee304b02"),
        &hex_bytes("ee38ab42"),
        &hex_bytes("ee306b42"),
        &hex_bytes("ee80eb0e"),
        &hex_bytes("eeb52b40"),
        &hex_bytes("eeb50b40"),
        &hex_bytes("eeb80bc0"),
        &hex_bytes("eeb8ebce"),
        &hex_bytes("eebdebce"),
    ];
    for (index, bytes) in words.iter().enumerate() {
        let code = (*bytes).to_vec();
        match cs.disasm_all(&code, 0x100) {
            Ok(instructions) => {
                for instruction in instructions.iter() {
                    println!(
                        "#{index} {} {}",
                        instruction.mnemonic().unwrap(),
                        instruction.op_str().unwrap_or("")
                    );
                }
            }
            Err(error) => println!("#{index} decode error: {error}"),
        }
    }
}

fn hex_bytes(text: &str) -> [u8; 4] {
    let mut out = [0u8; 4];
    for (index, pair) in text.as_bytes().chunks(2).enumerate() {
        let high = (pair[0] as char).to_digit(16).unwrap() as u8;
        let low = (pair[1] as char).to_digit(16).unwrap() as u8;
        out[index] = high * 16 + low;
    }
    out
}
