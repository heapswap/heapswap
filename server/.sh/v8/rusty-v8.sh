# add to .bashrc
RUSTY_V8_MIRROR="$HOME/.cache/rusty_v8"
LINE='export RUSTY_V8_MIRROR=$RUSTY_V8_MIRROR'
FILE=~/.bashrc
grep -qF -- "$LINE" "$FILE" || echo "$LINE" >> "$FILE"
source ~/.bashrc

# populate the cache
for REL in v0.83.1; do
  mkdir -p $RUSTY_V8_MIRROR/$REL
  for FILE in \
    librusty_v8_release_x86_64-unknown-linux-gnu.a \
  ; do
    if [ ! -f $RUSTY_V8_MIRROR/$REL/$FILE ]; then
      echo "File not found, downloading..."
      wget -O $RUSTY_V8_MIRROR/$REL/$FILE \
        https://github.com/denoland/rusty_v8/releases/download/$REL/$FILE
    else
      echo "File already exists, not downloading."
    fi
  done
done

echo "Listing contents of $RUSTY_V8_MIRROR:"
ls $RUSTY_V8_MIRROR