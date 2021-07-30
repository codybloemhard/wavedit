use std::time::Instant;

fn main() {
    let args = lapp::parse_args("
        Wavedit edits .wav files.
        -v, --verbose print more info
        -s, --stats calculate some extra statistics
        --histogram print the sample histogram
        --clippeaks clip peaks with histogram clipping
        --normalize normalize the audio if global peak is lower than normalize ceiling
        --max (default 20) maximum amount of samples allowed per cell
        --fac (default 0.0) if more than 0, the factor of samples that may be discarded
        --db (default 0.0) peak dB ceiling when normalizing(must be negative)
        --outputbits (default 0) bitdepth of the output, default will use whatever is the input bitdepth
        --cuts (integer...) timestamps(in N0) in ms alternating begin and end time to cut away material. Needs to be partially ordered
        --fades (integer...) timestamps(in N0) in ms(after the cuts) alternating begin and end time to fade in and out material.
            Fading in and out alternates per pair of points and starts with fade in. Needs to be partially ordered.
        <file> (string) input file
        <outfile> (default outp.wav) output file
    ");
    println!("Henlo!");
    let verbose = args.get_bool("verbose");
    let stats = args.get_bool("stats");
    let clippeaks = args.get_bool("clippeaks");
    let histo = args.get_bool("histogram");
    let norm = args.get_bool("normalize");
    let max = args.get_integer("max");
    let max = if max < 0 { panic!("{}", "Error: max must be in {{0..2^64 - 1}}"); }
    else { max as usize };
    let fac = args.get_float("fac");
    let db = args.get_float("db");
    let outputbits = args.get_integer("outputbits").max(0) as usize;
    let cuts = args.get_integers("cuts");
    let fades = args.get_integers("fades");
    let file = args.get_string("file");
    let outp = args.get_string("outfile");
    // setup and checks
    if !(histo || clippeaks || norm || verbose) {
        println!("Nothing to do!");
        return;
    }
    if !(outputbits == 0 || outputbits == 8 || outputbits == 16 || outputbits == 24 || outputbits == 32) {
        println!("Only 8, 16, 24 and 32 bit samples are supported!");
        return;
    }
    let mut stamper = Stamper::new(verbose);
    let mut reader = hound::WavReader::open(file).expect("Could not open file!");
    let mut copy = Vec::new();
    let specs = reader.spec();
    if verbose{
        println!("Track Info: Channels: {}, Sample Rate: {}, Bits: {}, Type: {:?}",
            specs.channels, specs.sample_rate, specs.bits_per_sample, specs.sample_format);
    }
    if outputbits > specs.bits_per_sample.into(){
        println!("Cannot render in a higher bitdepth than the input! (Would be possible but not that useful?).");
        return;
    }
    if !(histo || clippeaks || norm) { return; }
    if specs.sample_format != hound::SampleFormat::Int || specs.bits_per_sample > 32{
        println!("Format not supported, only integer formats up to 32 bits supported!");
        return;
    }
    if !fades.is_empty(){
        let mut last = 0;
        for point in &fades{
            if point < &last {
                println!("Points in fades list must be partially ordered!");
                return;
            }
            last = *point;
        }
    }
    // read samples and cut out parts
    // simple and fast version for cutless jobs
    if cuts.is_empty(){
        for s in reader.samples(){
            if s.is_err() { continue; }
            let s = s.unwrap();
            copy.push(s);
        }
    } else {
        // check if input is ok
        let mut points = Vec::new();
        let mut last = 0;
        for point in cuts{
            if point < last {
                println!("Points in cut list must be partially ordered!");
                return;
            }
            last = point;
            points.push((point.max(0) as f64 / 1000.0 * specs.sample_rate as f64) as u32 * 2);
        }
        points.push(std::u32::MAX);
        // we need to ignore certain parts
        let mut cut = false;
        let mut next = 0;
        for (i, s) in reader.samples().enumerate(){
            if points[next] == i as u32 {
                next += 1;
                cut = !cut;
            }
            if cut { continue; }
            if s.is_err() { continue; }
            let s = s.unwrap();
            copy.push(s);
        }
    }
    // move them into 32 bits
    let shift = 32 - specs.bits_per_sample;
    if specs.bits_per_sample < 32 {
        copy = copy.into_iter().map(|x| x << shift).collect::<Vec<_>>();
    }
    stamper.stamp_step("Copying");
    // apply fades
    if !fades.is_empty(){
        copy = fade(copy, &fades, specs.sample_rate);
        stamper.stamp_step("Fades");
    }
    // clip peaks
    let (total,hist) = if histo || clippeaks { build_histogram(&copy, &mut stamper, verbose) }
    else { (0, Vec::new()) };
    let mut loudest = 0;
    if histo { print_histo(&hist, verbose); }
    if clippeaks { copy = clip_peaks(copy, &hist, total, max, fac, verbose, stats, &mut loudest, &mut stamper); }
    if !(clippeaks || norm){
        stamper.stamp_abs("Total");
        return;
    }
    // normalize
    if loudest == 0 && norm { loudest = find_loudest(&copy, verbose, &mut stamper); }
    if norm { copy = normalize(copy, loudest, db, verbose, &mut stamper); }
    // move back into preffered bitsize
    let bits = if outputbits == 0 { specs.bits_per_sample as usize } else { outputbits };
    let shift = 32 - bits;
    let mut specs = specs;
    specs.bits_per_sample = bits as u16;
    let mut writer = hound::WavWriter::create(outp, specs).unwrap();
    if bits <= 16{
        for s in copy.into_iter().map(|s| (s >> shift) as i16).collect::<Vec<i16>>(){
            writer.write_sample(s).expect("Error: could not write sample");
        }
    } else {
        for s in copy.into_iter().map(|s| s >> shift).collect::<Vec<_>>(){
            writer.write_sample(s).expect("Error: could not write sample");
        }
    }
    stamper.stamp_step("Write");
    stamper.stamp_abs("Total");
}

fn sample_to_db(s: i32) -> f32{
    -20.0 * (std::i32::MAX as f32 / s.max(std::i32::MIN + 1).abs() as f32).log10()
}

fn db_to_sample(db: f32) -> i32{
    (10.0f32.powf(db / 20.0) * std::i32::MAX as f32) as i32
}

fn normalize(mut samples: Vec<i32>, max: i32, db: f32, verbose: bool, stamper: &mut Stamper) -> Vec<i32>{
    let peakmax = if db > 0.0 { panic!("db must be 0 or negative!"); }
    else if db == 0.0 { std::i32::MAX - 1 }
    else { db_to_sample(db) };
    if max >= peakmax {
        if verbose { println!("Audio is already normalized!"); }
        return samples;
    }
    let mul = if max > 0 { peakmax as f64 / max as f64 } else { 1.0 };
    for s in samples.iter_mut(){
        *s = (*s as f64 * mul) as i32
    }
    stamper.stamp_step("Normalize audio");
    if verbose { println!("Normalize with multiplier: {}", mul); }
    samples
}

fn find_loudest(samples: &[i32], verbose: bool, stamper: &mut Stamper) -> i32{
    let mut max = 0;
    for s in samples{
        let ns = (*s).max(std::i32::MIN + 1).abs();
        if ns > max { max = ns; }
    }
    stamper.stamp_step("Find global maximum");
    if verbose { println!("Highest sample: {} at {} dB", max, sample_to_db(max)); }
    max
}


fn fade(samples: Vec<i32>, fades: &[i32], sr: u32) -> Vec<i32>{
    let mut points = Vec::new();
    for point in fades{
        points.push(((*point).max(0) as f64 / 1000.0 * sr as f64) as u32 * 2);
    }
    let mut res = Vec::new();
    let mut next = 0;
    let mut fade = false;
    let mut f_in = true;
    let mut start = 0;
    let mut duration = 0.0;
    for (i, s) in samples.into_iter().enumerate(){
        // need to change state
        if i == points[next] as usize{
            fade = !fade; // swap fade state
            if !fade{ // after every fade section we go from fade in to out and vice versa
                f_in = !f_in;
            }
            // take start and duration for lerping
            start = i;
            if points.len() > next + 1{
                next += 1;
                let end = points[next] as usize;
                duration = (end - start) as f64;
            }
        }
        if !fade{
            res.push(s);
        } else {
            // lerp the sample
            let x = (i - start) as f64 / duration;
            let f = if f_in { x } else { 1.0 - x };
            res.push((s as f64 * f) as i32);
        }
    }
    res
}

fn print_histo(hist: &[usize], verbose: bool){
    for (i, count) in hist.iter().enumerate(){
        if *count == 0 { continue; }
        if verbose { println!("Cell {}: {}", i, count); }
        else { print!("{}: {}, ", i, count); }
    }
}

fn build_histogram(samples: &[i32], stamper: &mut Stamper, verbose: bool) -> (usize,Vec<usize>){
    let mut hist = vec![0usize; 2048];
    let mut scount = 0;
    for s in samples{
        let i = (*s).max(std::i32::MIN + 1).abs() >> 20;
        hist[i as usize] += 1usize;
        scount += 1;
    }
    stamper.stamp_step("Histogram");
    if verbose { println!("Total samples: {}", scount); }
    (scount, hist)
}

fn clip_peaks(mut samples: Vec<i32>, hist: &[usize], total: usize, max: usize, fac: f32, verbose: bool, stats: bool, loudest: &mut i32, stamper: &mut Stamper) -> Vec<i32>{
    let max = if fac > 0.0 { (total as f64 * fac as f64) as usize } else { max };
    let cs = if fac > 0.0 { depeaked_size_acc(&hist, (total as f64 * fac as f64) as usize) }
    else { depeaked_size_until(hist, max) };
    let cs = if let Some(inner) = cs { inner } else {
        if verbose { println!("No clipping needed!"); }
        return samples;
    };
    let thresh = (cs << 20) as i32;
    *loudest = thresh;
    stamper.stamp_step("Depeak scan");
    if verbose {
        println!("upwards from cell {} out of {} will be clipped with max cell length > {} ({} dB headroom)", cs, hist.len() - 1, max, -sample_to_db(thresh));
    }
    if stats{
        let mut diff_count = 0;
        for s in samples.iter_mut(){
            let ns = (*s).min(thresh).max(-thresh);
            if ns != *s { diff_count += 1; }
            *s = ns
        }
        let fraction = if diff_count == 0 { 0 } else { total / diff_count };
        let percentage = if total == 0 { 0.0 } else { diff_count as f64 / total as f64 * 100.0 };
        println!("Samples clipped: {} out of {} which is 1/{} or {}%", diff_count, total, fraction, percentage);
    } else {
        for s in samples.iter_mut(){
            *s = (*s).min(thresh).max(-thresh);
        }
    }
    stamper.stamp_step("Peak clipping");
    samples
}

fn depeaked_size_until(hist: &[usize], max: usize) -> Option<usize>{
    if hist[2047] > max { return None; }
    let mut i = hist.len() - 1;
    while i > 0{
        let c = hist[i as usize];
        if c > max { break; }
        i -= 1;
    }
    Some(i)
}

fn depeaked_size_acc(hist: &[usize], max: usize) -> Option<usize>{
    if hist[2047] > max { return None; }
    let mut i = hist.len() - 1;
    let mut acc = 0;
    while i > 0{
        acc += hist[i as usize];
        if acc > max { break; }
        i -= 1;
    }
    Some(i)
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
