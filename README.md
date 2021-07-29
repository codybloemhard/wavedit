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

--outputbits (default 0) bitdepth of the output, default will use whatever is the input bitdepth

<file> (string) input file

<outfile> (default outp.wav) output file
