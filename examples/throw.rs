//! One sealed throw at a testnet-10 DAA score.
//!
//! Pass the `virtualDaaScore` from `https://api-tn10.kaspa.org/info/blockdag`.
//! The score is only the clock. This example does not connect anywhere.

fn main() {
    let mut args = std::env::args().skip(1);
    let raw = args.next().unwrap_or_else(|| {
        eprintln!("usage: throw <virtualDaaScore>");
        std::process::exit(2);
    });
    if args.next().is_some() {
        eprintln!("usage: throw <virtualDaaScore>");
        std::process::exit(2);
    }
    let daa: u64 = raw.parse().unwrap_or_else(|_| {
        eprintln!("virtualDaaScore must be a u64");
        std::process::exit(2);
    });
    match quiet_lane::script(daa) {
        Ok(report) => {
            for line in report.lines {
                println!("{line}");
            }
        }
        Err(err) => {
            eprintln!("throw failed: {err:?}");
            std::process::exit(1);
        }
    }
}
