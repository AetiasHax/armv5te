mod v5te;

use std::time::Instant;

use unarm::parse::ArmVersion;

fn main() {
    let (threads, iterations, arm, thumb, version) = {
        let mut threads = num_cpus::get();
        let mut iterations = 1;
        let mut arm = false;
        let mut thumb = false;
        let mut version = None;
        let mut args = std::env::args();
        args.next(); // skip program name
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-t" => threads = args.next().and_then(|a| a.parse().ok()).expect("Expected number after -t"),
                "-n" => iterations = args.next().and_then(|a| a.parse().ok()).expect("Expected number after -n"),
                "arm" => arm = true,
                "thumb" => thumb = true,
                "v5te" => version = Some(ArmVersion::V5Te),
                _ => panic!("Unknown argument '{}'", arg),
            }
        }
        (threads, iterations, arm, thumb, version)
    };
    if threads == 0 {
        panic!("Number of threads must be positive");
    }
    if iterations == 0 {
        panic!("Number of iterations must be positive");
    }
    if arm == thumb {
        panic!("Expected one of: arm, thumb");
    }
    let Some(version) = version else {
        panic!("Expected one of: v5te");
    };

    println!("Starting {} threads running {} iterations", threads, iterations);
    let start = Instant::now();
    match version {
        ArmVersion::V5Te => {
            if arm {
                v5te::arm::fuzz(threads, iterations);
            }
            if thumb {
                v5te::thumb::fuzz(threads, iterations);
            }
        }
    }
    println!("Finished in {:.2}s", start.elapsed().as_secs_f32());
}
