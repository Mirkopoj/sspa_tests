import socket
import time
# Create a TCP/IP socket
sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
# Connect the socket to the port where the server is listening
server_address = ('192.168.1.16', 8000)
print('connecting to {} port {}'.format(*server_address))
sock.connect(server_address)
def send_command(message):
        # Send data
        sock.sendall(message.to_bytes(4, byteorder= 'big'))
        # Look for the response
        data = sock.recv(2)
        return int.from_bytes(data, 'big')
def leer_registros():
    print('00=>', bin(send_command(0x3C000000))[2:].zfill(16))
    print('01=>', bin(send_command(0x3C010000))[2:].zfill(16))
    print('02=>', send_command(0x3C020000)&0x7FFF7FFF)
    print('03=>', send_command(0x3C030000)&0x7FFF7FFF)
    print('04=>', send_command(0x3C040000)&0x7FFF7FFF)
    print('05=>', send_command(0x3C050000)&0x7FFF7FFF)
    print('06=>', send_command(0x3C060000)&0x7FFF7FFF)
    print('07=>', send_command(0x3C070000)&0x7FFF7FFF)
    print('08=>', send_command(0x3C080000)&0x7FFF7FFF)
    print('09=>', send_command(0x3C090000)&0x7FFF7FFF)
    print('10=>', send_command(0x3C0A0000)&0x7FFF7FFF)
    print('11=>', send_command(0x3C0B0000)&0x7FFF7FFF)
    print('12=>', send_command(0x3C0C0000)&0x7FFF7FFF)
    print('13=>', send_command(0x3C0D0000)&0x7FFF7FFF)
    print('14=>', send_command(0x3C0E0000)&0x7FFF7FFF)
    print('15=>', send_command(0x3C0F0000)&0x7FFF7FFF)
    print('16=>', send_command(0x3C100000)&0x7FFF7FFF)
    print('17=>', send_command(0x3C110000)&0x7FFF7FFF)
    print('18=>', send_command(0x3C120000)&0x7FFF7FFF)
    print('19=>', send_command(0x3C130000)&0x7FFF7FFF)
def gen_tnr(per, ancho, off1, off2, count):
    send_command(0x23000000+per)
    send_command(0x23010000+ancho)
    send_command(0x23020000+off1)
    send_command(0x23030000+off2)
    send_command(0x23040000+count)
    send_command(0xA3000000)
def powen(onoff):
    send_command(0x23050000+onoff)
def alarm_reset():
    send_command(0x25010020)
def relay(onoff):
    send_command(0x2D000000+onoff)
def program(onoff):
    send_command(0x3D000000+onoff)
def dac(addr, valor):
    suma = (addr<<16) + valor
    print(hex(suma))
    send_command(0x2A000000+suma)
def dacs_cero():
    for i in range(0,8):
        dac(i,0)
def rampa_dac(addr):
    for i in range(0,1023):
        dac(addr,i)
        time.sleep(0.001)
def sspa_reset():
    send_command(0x25010040)

def which_alarm():
    stat = send_command(0x3C130000)
    valor = stat & 0x03FF
    alarma = (stat&0x7FFF)>>10
    print("alarma: ", alarma, ", valor: ", valor)

alarm_reset()
leer_registros()
which_alarm()

gen_tnr(3000, 250, 4, 4, 0)
gen_tnr(3000, 180, 4, 4, 0)
gen_tnr(1500, 250, 4, 4, 0)
gen_tnr(800, 250, 4, 4, 1)
gen_tnr(800, 800, 0, 0, 0)

send_command(0x25010001)
send_command(0x25640258)

powen(0)
powen(1)

relay(0)
relay(1)

program(0)
program(1)

sspa_reset()

dacs_cero()
dac(4, 1023)
dac(1, 511)
dac(2, 511)
dac(0, 0)

for _ in range(100):
    rampa_dac(2)

print('closing socket')
sock.close()
