#!/bin/bash

LOG_FILE="output.log"
> $LOG_FILE

success_count=0
failure_count=0
success_scripts=()
failure_scripts=()

for script in $(find . -name "*.py"); do
    echo "Running script: $script" | tee -a $LOG_FILE

    echo "Command: CARGO_HTTP_PROXY=\"\" CARGO_HTTPS_PROXY=\"\" python $script" | tee -a $LOG_FILE
    CARGO_HTTP_PROXY="" CARGO_HTTPS_PROXY="" python "$script" >> $LOG_FILE 2>&1
    
    if [ $? -eq 0 ]; then
        success_count=$((success_count + 1))
        success_scripts+=("$script")
        echo "Execution successful for: $script" | tee -a $LOG_FILE
    else
        failure_count=$((failure_count + 1))
        failure_scripts+=("$script")
        echo "Failed to run script: $script" | tee -a $LOG_FILE
    fi

    echo "-----------------------------------------" | tee -a $LOG_FILE
done

echo "All scripts executed. Check $LOG_FILE for details." | tee -a $LOG_FILE
echo "" | tee -a $LOG_FILE

echo "Total successful executions: $success_count" | tee -a $LOG_FILE
echo "Successful scripts:" | tee -a $LOG_FILE
for script in "${success_scripts[@]}"; do
    echo " - $script" | tee -a $LOG_FILE
done

echo "" | tee -a $LOG_FILE
echo "Total failed executions: $failure_count" | tee -a $LOG_FILE
echo "Failed scripts:" | tee -a $LOG_FILE
for script in "${failure_scripts[@]}"; do
    echo " - $script" | tee -a $LOG_FILE
done
