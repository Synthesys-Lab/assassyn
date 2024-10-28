
cat $1 | grep "raw:" | awk '{ i=3; if ($i ~ /raw:/) print $(i+1) }'
