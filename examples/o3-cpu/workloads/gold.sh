for i in 3 5; do

trace=$(cat $1 | grep "raw:" | awk "{ i=$i; if (\$i ~ /raw:/) print \$(i+1) }")

if [ "$trace" != "" ]; then
  cat $1 | grep "raw:" | awk "{ i=$i; if (\$i ~ /raw:/) print \$(i+1) }" > a.trace
fi

done

cat $2 | awk '{ print substr($5, 2, length($5) - 2) }' > a.gold

