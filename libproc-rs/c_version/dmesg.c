#include <sys/cdefs.h>
#include <sys/msgbuf.h>

#include <fcntl.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <unistd.h>
#include <vis.h>
#include <libproc.h>

void usage __P((void));

void print_message_buffer(struct msgbuf * pMessageBuf) {
    printf("Magic: %#010x\n", pMessageBuf->msg_magic);
}

int
main(argc, argv)
	int argc;
	char *argv[];
{
	register int ch, newl, skip;
	register char *p, *ep;
	struct msgbuf cur;
	char buf[5];

	if (argc > 1)
		usage();

	if (proc_kmsgbuf(&cur, sizeof(struct msgbuf)) == 0){
		perror("Unable to obtain kernel buffer");
		usage();
		exit(1);
	}

    print_message_buffer(&cur);

	if (cur.msg_magic != MSG_MAGIC) {
		perror("magic number incorrect");
		exit(1);
	}
	if (cur.msg_bufx >= MAX_MSG_BSIZE)
		cur.msg_bufx = 0;


	/*
	 * The message buffer is circular; start at the read pointer, and
	 * go to the write pointer - 1.
	 */
	p = cur.msg_bufc + cur.msg_bufx;
	ep = cur.msg_bufc + cur.msg_bufx - 1;
	for (newl = skip = 0; p != ep; ++p) {
		if (p == cur.msg_bufc + MAX_MSG_BSIZE)
			p = cur.msg_bufc;
		ch = *p;
		/* Skip "\n<.*>" syslog sequences. */
		if (skip) {
			if (ch == '>')
				newl = skip = 0;
			continue;
		}
		if (newl && ch == '<') {
			skip = 1;
			continue;
		}
		if (ch == '\0')
			continue;
		newl = ch == '\n';
		(void)vis(buf, ch, 0, 0);
		if (buf[1] == 0)
			(void)putchar(buf[0]);
		else
			(void)printf("%s", buf);
	}
	if (!newl)
		(void)putchar('\n');
	exit(0);
}

void
usage()
{
	(void)fprintf(stderr, "usage: sudo dmesg\n");
	exit(1);
}
