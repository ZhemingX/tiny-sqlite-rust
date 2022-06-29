pub fn string2arr(s: String, dst: &mut [char]) -> Result<(), String> {
    let s_len = s.len();
    if s_len >= dst.len() {
        return Err("string length exceeds dst arr length limit".to_owned());
    }

    let v: Vec<char> = s.chars().collect();
    dst[..v.len()].copy_from_slice(&v);
    dst[v.len()] = '\n';

    Ok(())
}

pub fn arr2string(src: &[char]) -> String {
    let mut s = String::from("");
    for ch in src {
        if *ch == '\n' {
            // compatible with c string
            break;
        }
        s.push(*ch);
    }
    s
}
