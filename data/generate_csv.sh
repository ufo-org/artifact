# Create file with header
echo "integer, logical, numeric, string" > $(dirname $0)/example.csv

# Populate csv
alphabet=( A B C D E F G H I J K L M N O P Q R S T U V W X Y Z )
for i in {0..2000}
do
    case $(( i % 3 )) in 0) b=TRUE;; 1) b=FALSE;; *) b=NULL;; esac
    r=$(echo "scale=2; $i / 5" | bc)
    s=${alphabet[$((i % 26))]}
    echo "${i} -> $i,$b,$r,$s"
    echo "$i,$b,$r,$s" >> $(dirname $0)/example.csv
done