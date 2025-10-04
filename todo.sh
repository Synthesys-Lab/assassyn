for ((i=0; i<1; i++)); do
  claude -p "look at @TODO.md and act on the first unchecked item. After finishing, check it!" \
    --allowedTools "Bash,Read,Write,Edit,Create" \
    --permission-mode acceptEdits \
    --verbose
done
