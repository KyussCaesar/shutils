use std::io;
use std::io::Read;
use std::process::exit;

use sha2::Sha256;
use sha2::digest::Digest;

fn main()
{
    let mut hasher = Sha256::default();
    for byte in io::stdin().lock().bytes()
    {
        match byte
        {
            Ok(b) => hasher.input(&[b]),
            Err(e) =>
            {
                eprintln!("[s256] failed to read byte! {}", e);
                exit(1);
            }
        }
    }

    print!("{}", hex::encode(hasher.result()));

    exit(0);
}
