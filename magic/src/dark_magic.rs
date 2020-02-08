use bytesize::ByteSize;
use std::process::{Command, Stdio};

/// tranform a number being into a RGB color format
pub fn number_to_rgb(num: impl Into<u64>) -> (u8, u8, u8) {
    let n = num.into() % 0xffffff;

    let r = n >> 16;
    let g = (n >> 8) & ((1 << 8) - 1);
    let b = n & ((1 << 8) - 1);

    (r as u8, g as u8, b as u8)
}

/// into low endian bytes, trim all trailing zero
pub fn number_to_le_bytes<N: Into<u64>>(n: N) -> Vec<u8> {
    let mut res = n.into().to_le_bytes().to_vec();

    while res.len() > 1 && *res.last().unwrap() == 0 {
        res.pop();
    }

    res
}

/// Get u64 value from low endian bytes
pub fn bytes_to_le_u64<B: AsRef<[u8]>>(bytes: B) -> u64 {
    let mut data: [u8; 8] = [0; 8];
    bytes
        .as_ref()
        .iter()
        .take(8)
        .enumerate()
        .for_each(|(i, v)| {
            data[i] = *v;
        });

    u64::from_le_bytes(data)
}

#[inline]
/// Check if the command exist or not
pub fn has_external_command<C: AsRef<str>>(c: C) -> bool {
    Command::new(c.as_ref())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|mut v| v.kill().ok())
        .is_ok()
}

/// Get the progress bar
pub fn progress_bar(per: u8, row: u8) -> String {
    let a = (per as f32 / (100.0 / row as f32)) as usize;
    format!("{}{}", "█".repeat(a), "░".repeat(row as usize - a),)
}

#[inline]
/// Report the bytes in a human readable form
pub fn report_bytes(b: u64) -> String {
    ByteSize(b).to_string_as(false)
}

#[inline]
pub fn report_kb(b: u64) -> String {
    ByteSize::kb(b).to_string_as(false)
}

// /// Needs FFMPEG in order to work
// pub fn cut_mp3<P: AsRef<Path>>(source: P, from: f32, duration: f32) -> MyResult<Vec<u8>> {
//     let from_str = from.to_string();
//     let duration_str = duration.to_string();
//     let args = vec![
//         "-ss",
//         &from_str,
//         "-t",
//         &duration_str,
//         "-i",
//         &source.as_ref().to_str().unwrap(),
//         "-f",
//         "mp3",
//         "pipe:",
//     ];

//     let cmd = Command::new("ffmpeg").args(args).output()?;

//     Ok(cmd.stdout)
// }
