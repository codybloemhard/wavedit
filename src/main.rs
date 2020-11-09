use hound;
use std::time::Instant;

fn main() {
    println!("Henlo!");
    let max = 100;
    let verbose = true;
    let mut stamper = Stamper::new();
    let mut reader = hound::WavReader::open("/home/cody/temp/180101_0006.wav").expect("bruh");
    let mut copy = Vec::new();
    for s in reader.samples::<i16>(){
        if s.is_err() { continue; }
        let s = s.unwrap();
        copy.push(s);
    }
    stamper.stamp("Copying");
    let mut hist = vec![0usize; 2048];
    let mut total = 0;
    for s in &copy{
        let i = (*s).max(std::i16::MIN + 1).abs() >> 4;
        hist[i as usize] += 1usize;
        total += 1;
    }
    stamper.stamp("Histogram");
    println!("Total samples: {}", total);
    let cs = depeaked_size(&hist, max);
    stamper.stamp("Depeak scan");
    println!("Upwards from cell {} out of {} will be clipped with max cell length > {}", cs, hist.len() - 1, max);
    let thresh = (cs << 4) as i16;
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create("outp.wav", spec).unwrap();
    let mut diff_count = 0;
    for (os, ns) in copy.into_iter().map(|s| (s, s.min(thresh).max(-thresh))) {
        if ns != os { diff_count += 1; }
        writer.write_sample(ns as i16).unwrap();
    }
    println!("Samples clipped: {} out of {} which is 1/{} or {}%", diff_count, total, total / diff_count, diff_count as f64 / total as f64 * 100.0);
    stamper.stamp("Write");
    println!("Total took {} ms", stamper.elapsed());
}

fn depeaked_size(hist: &Vec<usize>, max: usize) -> usize{
    let mut i = hist.len() - 2;
    while i > 0{
        let c = hist[i as usize];
        if c > max { break; }
        i -= 1;
    }
    i
}

struct Stamper{
    start: Instant,
    till: u128,
}

impl Stamper{
    pub fn new() -> Self{
        Self{
            start: Instant::now(),
            till: 0,
        }
    }

    pub fn stamp(&mut self, action: &str){
        let elapsed = self.start.elapsed().as_millis();
        println!("{} took {} ms", action, elapsed - self.till);
        self.till = elapsed;
    }

    pub fn elapsed(&self) -> u128{
        self.start.elapsed().as_millis()
    }
}
