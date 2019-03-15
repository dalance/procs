#!/bin/zsh
rsync -auzv -e "ssh -p 10022" --exclude target --exclude .git /home/hatta/work/repos/procs/ hatta-pc:/mnt/c/Users/hatta/procs/
