
.global fork
fork:
li a7, 0
ecall
ret

.global exit
exit:
li a7, 1
ecall
ret

.global wait
wait:
li a7, 2
ecall
ret

.global pipe
pipe:
li a7, 3
ecall
ret

.global read
read:
li a7, 4
ecall
ret

.global write
write:
li a7, 5
ecall
ret

.global close
close:
li a7, 6
ecall
ret

.global kill
kill:
li a7, 7
ecall
ret

.global exec
exec:
li a7, 8
ecall
ret

.global open
open:
li a7, 9
ecall
ret

.global mknod
mknod:
li a7, 10
ecall
ret

.global unlink
unlink:
li a7, 11
ecall
ret

.global fstat
fstat:
li a7, 12
ecall
ret

.global link
link:
li a7, 13
ecall
ret

.global mkdir
mkdir:
li a7, 14
ecall
ret

.global chdir
chdir:
li a7, 15
ecall
ret

.global dup
dup:
li a7, 16
ecall
ret

.global getpid
getpid:
li a7, 17
ecall
ret

.global sbrk
sbrk:
li a7, 18
ecall
ret

.global sleep
sleep:
li a7, 19
ecall
ret

.global uptime
uptime:
li a7, 20
ecall
ret
