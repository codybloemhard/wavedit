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

--cuts (integer...) timestamps in ms alternating begin and end time to cut away material. needs to be partially ordered

--fades (integer...) timestamps(in N0) in ms(after the cuts) alternating begin and end time to fade in and out material.
    Fading in and out alternates per pair of points and starts with fade in. Needs to be partially ordered.

<file> (string) input file

<outfile> (default outp.wav) output file
