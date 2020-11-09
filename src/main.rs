use hound;

fn main() {
    println!("Henlo!");
    let mut reader = hound::WavReader::open("/home/cody/temp/180101_0006.wav").expect("bruh");
    let mut copy = Vec::new();
    for s in reader.samples::<i16>(){
        if s.is_err() { continue; }
        let s = s.unwrap();
        copy.push(s);
    }
    let mut hist = vec![0usize; 2048];
    let mut total = 0;
    for s in &copy{
        let i = (*s).max(std::i16::MIN + 1).abs() >> 4;
        hist[i as usize] += 1usize;
        total += 1;
    }
    println!("Total samples: {}", total);
    fn depeaked_size(hist: &Vec<usize>, max: usize) -> usize{
        let mut i = hist.len() - 2;
        while i > 0{
            let c = hist[i as usize];
            if c > max { break; }
            i -= 1;
        }
        i
    }
    let cs = depeaked_size(&hist, 100);
    println!("Size(100): {}", cs);
    let thresh = (cs << 4) as i16;
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create("outp.wav", spec).unwrap();
    for s in copy.into_iter().map(|s| s.min(thresh).max(-thresh)) {
        writer.write_sample(s as i16).unwrap();
    }
}
