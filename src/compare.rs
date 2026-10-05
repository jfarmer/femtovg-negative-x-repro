//! Counts the pixels in which two PNGs differ.
//!
//! usage: compare <first.png> <second.png>
//!
//! Prints one line. The exit status is 0 when the pixels are the same, 1 when they differ or the
//! sizes differ, and 2 when a file cannot be read.

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let [_, first_path, second_path] = args.as_slice() else {
        eprintln!("usage: {} <first.png> <second.png>", args[0]);
        std::process::exit(2);
    };
    let open = |path: &String| match image::open(path) {
        Ok(picture) => picture.to_rgba8(),
        Err(error) => {
            println!("cannot read {path}: {error}");
            std::process::exit(2);
        }
    };
    let (first, second) = (open(first_path), open(second_path));
    if first.dimensions() != second.dimensions() {
        println!("sizes differ: {:?} and {:?}", first.dimensions(), second.dimensions());
        std::process::exit(1);
    }
    let differing = first.pixels().zip(second.pixels()).filter(|(a, b)| a != b).count();
    if differing == 0 {
        println!("identical");
        return;
    }
    let largest = first
        .iter()
        .zip(second.iter())
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap_or(0);
    println!(
        "{differing} of {} pixels differ, by at most {largest}/255 in one channel",
        first.width() * first.height()
    );
    std::process::exit(1);
}
