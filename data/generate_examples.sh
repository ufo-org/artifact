#!/bin/bash

dir="$(dirname $0)"

[ ! -e "$dir/gen" ] && (gcc -o $dir/gen $dir/gen.c)

function generate {
    echo "generating ${1}_rand_int.bin" 
    $dir/gen "$dir/${1}_rand_int.bin" $2 random

    #echo "generating ${1}_ones_int.bin" 
    #$dir/gen "$$dir/{1}_ones_int.bin" $2 ones

    echo "generating ${1}_seq_int.bin" 
    $dir/gen "$dir/${1}_seq_int.bin" $2 sequence
}

#                   T  G  M  K  1   
generate 1K             1000    # 3.9 KB
generate 100K         100000    # 392 KB 
generate 1M          1000000    # 3.8 MB
generate 10M        10000000    # 38  MB
#                   T  G  M  K  1   

# Generate BZip2 examples
for f in $dir/*.bin
do 
    bzip2 -k "$f" 
done

# Bzip exits with an error if the file already exists
exit 0
