#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <libproc.h>

int main (int argc, char* argv[])
{
    pid_t pid; int ret;
    char pathbuf[PROC_PIDPATHINFO_MAXSIZE];

    if ( argc > 1 ) {
        pid = (pid_t) atoi(argv[1]);
        ret = proc_pidpath (pid, pathbuf, sizeof(pathbuf));
        if ( ret <= 0 ) {
            fprintf(stderr, "PID %d: proc_pidpath ();\n", pid);
            fprintf(stderr, "    %s\n", strerror(errno));
        } else {
            printf("proc %d: %s\n", pid, pathbuf);
        }
    }

    return 0;
}