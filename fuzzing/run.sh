mkdir -p /tmp/out
cd fuzzing
bwrap --ro-bind /usr /usr \
 --symlink usr/lib64 /lib64 \
 --proc /proc --dev /dev \
 --unshare-pid \
 --bind $HOME/Development $HOME/Development \
 --bind /tmp/out /home/user/Development/hana/fuzzing/out \
 --ro-bind $HOME/.cargo $HOME/.cargo \
 --ro-bind $HOME/.rustup $HOME/.rustup \
 --ro-bind /home/user/.local/share/afl.rs /home/user/.local/share/afl.rs \
 cargo afl fuzz  -i ../in -o ../out target/debug/fuzzing