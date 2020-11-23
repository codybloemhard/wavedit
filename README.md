# wavedit
Wavedit edits .wav files.

-v, --verbose print more info

-s, --stats calculate some extra statistics

--histogram print the sample histogram

--clippeaks clip peaks with histogram clipping

--drc dynamic range compression: reduces dynamics

--normalize normalize the audio if global peak is lower than normalize ceiling

--max (default 100) maximum amount of samples allowed per cell

--fac (default 0.0) if more than 0, the factor of samples that may be discarded

--db (default 0.0) peak dB ceiling when normalizing(must be negative)

--ratio (default 1.5) dynamic range compression ratio (higher is more compression, should be > 1.0)

--attack (default 15) dynamic range compression attack time in ms (should be >= 0)

--release (default 10) dynamic range compression release time in ms (should be >= 0)

--threshold (default 20.0) dynamic range compression threshold in dB (times -1, should be > 0.0, so 20 is -20 dB)

<file> (string) input file

<outfile> (default outp.wav) output file