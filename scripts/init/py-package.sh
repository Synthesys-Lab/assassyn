pip install --user -r \
    $ASSASSYN_HOME/python/requirements.txt --break-system-packages

if [ $? -ne 0 ]; then
  echo "Failed to install Python dependencies. Please check the requirements."
  exit 1
else
  echo "Python dependencies installed successfully."
fi