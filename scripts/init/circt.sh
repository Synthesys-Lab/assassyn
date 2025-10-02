# Install PyCDE

# TODO: Later add a flag to force CIRCT installation via source
pip install --user pycde --break-system-packages
if [ $? -eq 0 ]; then
  echo "CIRCT installed successfully via pip."
  # Verify that PyCDE can be imported
  python3 -c "import pycde; print('PyCDE import verification: SUCCESS')"
  if [ $? -eq 0 ]; then
    echo "PyCDE import test passed."
    return 0
  else
    echo "WARNING: PyCDE installed via pip but import test failed."
    return 1
  fi
fi

RESTORE=`pwd`

echo "Failed to install CIRCT via pip. Fall back to building from source using PyCDE setup."
cd $ASSASSYN_HOME/3rd-party/circt/frontends/PyCDE

# Install the built package to local directory

CIRCT_DIRECTORY="`pwd`/../../" CIRCT_EXTRA_CMAKE_ARGS="-DESI_RUNTIME=OFF -DZ3_DISABLE=ON -DOR_TOOLS_DISABLE=ON" python -m build
pip install ./dist/*.whl

if [ $? -ne 0 ]; then
  echo "Failed to install PyCDE. Please check the installation output."
  cd $RESTORE
  return 1
fi

  
# Verify that PyCDE can be imported
python3 -c "import pycde; print('PyCDE import verification: SUCCESS')"
if [ $? -eq 0 ]; then
  echo "PyCDE import test passed."
else
  echo "WARNING: PyCDE built and installed but import test failed."
  cd $RESTORE
  return 1
fi

cd $RESTORE

echo "PyCDE built and installed successfully to local directory."
return 0