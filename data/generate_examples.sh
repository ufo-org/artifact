#!/bin/bash

[ ! -e "$(dirname $0)/gen" ] && (gcc -o $(dirname $0)/gen $(dirname $0)/gen.c)

function generate {
    echo "generating ${1}_rand_int.bin" 
    ./gen "${1}_rand_int.bin" $2 random

    #echo "generating ${1}_ones_int.bin" 
    #./gen "${1}_ones_int.bin" $2 ones

    echo "generating ${1}_seq_int.bin" 
    ./gen "${1}_seq_int.bin" $2 sequence
}
    
function generate_ones {
    echo "generating ${1}_ones_int.bin" 
    ./gen "${1}_ones_int.bin" $2 ones
}

#                   T  G  M  K  1   
generate 1K             1000    # 3.9 KB
generate 100K         100000    # 392 KB 
generate 1M          1000000    # 3.8 MB
generate 10M        10000000    # 38  MB
generate 250M      250000000    #  ~1 GB
#                   T  G  M  K  1   

# Generate BZip2 examples
for f in *.bin ; do bzip2 -k "$f"; done

