# this will take a long time to run (like 30+ minutes)

# move to the home directory
cd ~

# install the depot_tools
git clone https://chromium.googlesource.com/chromium/tools/depot_tools.git
# add depot_tools to the PATH
LINE='export PATH=$PATH:$HOME/depot_tools'
FILE=~/.bashrc
grep -qF -- "$LINE" "$FILE" || echo "$LINE" >> "$FILE"
# reload the .bashrc file
source ~/.bashrc
# update gclient
gclient

# check out the v8 source code 
#mkdir ~/v8
#cd ~/v8
fetch v8
cd v8

# install the build dependencies
sudo ./build/install-build-deps.sh

# pull the v8 source code
git pull && gclient sync

# compile v8
tools/dev/gm.py x64.release

# run the tests
tools/run-tests.py --gn

# move to ./bin/v8
mv ~/v8/out/x64.release ./bin/v8