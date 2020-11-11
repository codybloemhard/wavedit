use std::time::Instant;

fn main() {
    let args = lapp::parse_args("
        Wavedit edits .wav files.
        -v, --verbose print more info
        -s, --stats calculate some extra statistics
        --max (default 100) maximum amount of samples allowed per cell
        --fac (default 0.0) if more than 0, the factor of samples that may be discarded
        <file> (string) input file
    ");
    println!("Henlo!");
    let verbose = args.get_bool("verbose");
    let stats = args.get_bool("stats");
    let max = args.get_integer("max");
    let max = if max < 0 { panic!("Error: max must be in {{0..2^64 - 1}}"); }
    else { max as usize };
    let fac = args.get_float("fac");
    let file = args.get_string("file");
    let mut stamper = Stamper::new(verbose);
    let mut reader = hound::WavReader::open(file).expect("Could not open file!");
    let mut copy = Vec::new();
    for s in reader.samples::<i16>(){
        if s.is_err() { continue; }
        let s = s.unwrap();
        copy.push(s);
    }
    stamper.stamp_step("Copying");
    let mut hist = vec![0usize; 2048];
    let mut total = 0;
    for s in &copy{
        let i = (*s).max(std::i16::MIN + 1).abs() >> 4;
        hist[i as usize] += 1usize;
        total += 1;
    }
    let max = if fac > 0.0 { (total as f64 * fac as f64) as usize } else { max };
    stamper.stamp_step("Histogram");
    if verbose { println!("Total samples: {}", total); }
    let cs = if fac > 0.0 { depeaked_size_acc(&hist, (total as f64 * fac as f64) as usize) }
    else { depeaked_size_until(&hist, max) };
    stamper.stamp_step("Depeak scan");
    if verbose {
        println!("upwards from cell {} out of {} will be clipped with max cell length > {}", cs, hist.len() - 1, max);
    }
    let thresh = (cs << 4) as i16;
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create("outp.wav", spec).unwrap();
    if stats{
        let mut diff_count = 0;
        for (os, ns) in copy.into_iter().map(|s| (s, s.min(thresh).max(-thresh))) {
            if ns != os { diff_count += 1; }
            writer.write_sample(ns as i16).expect("Error: could not write sample");
        }
        println!("Samples clipped: {} out of {} which is 1/{} or {}%", diff_count, total, total / diff_count, diff_count as f64 / total as f64 * 100.0);
    } else {
        for s in copy.into_iter().map(|s| s.min(thresh).max(-thresh) as i16) {
            writer.write_sample(s).expect("Error: could not write sample");
        }
    }
    stamper.stamp_step("Write");
    stamper.stamp_abs("Total");
}

fn depeaked_size_until(hist: &[usize], max: usize) -> usize{
    let mut i = hist.len() - 2;
    while i > 0{
        let c = hist[i as usize];
        if c > max { break; }
        i -= 1;
    }
    i
}

fn depeaked_size_acc(hist: &[usize], max: usize) -> usize{
    let mut i = hist.len() - 2;
    let mut acc = 0;
    while i > 0{
        acc += hist[i as usize];
        if acc > max { break; }
        i -= 1;
    }
    i
}

struct Stamper{
    start: Instant,
    till: u128,
    verbose: bool,
}

impl Stamper{
    pub fn new(verbose: bool) -> Self{
        Self{
            start: Instant::now(),
            till: 0,
            verbose,
        }
    }

    pub fn stamp_step(&mut self, action: &str){
        if !self.verbose { return; }
        let elapsed = self.start.elapsed().as_millis();
        println!("{} took {} ms", action, elapsed - self.till);
        self.till = elapsed;
    }

    pub fn stamp_abs(&self, action: &str){
        if !self.verbose { return; }
        let elapsed = self.start.elapsed().as_millis();
        println!("{} took {} ms", action, elapsed);
    }
}
