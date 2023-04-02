#[cfg(test)]
mod tests {

    use std::io::prelude::*;
    use std::net::TcpStream;

    static mut STREAM: Option<TcpStream> = None;

    #[test]
    fn aaaa_initial_conection() {
        unsafe {
            STREAM = Some(TcpStream::connect("192.168.1.16:8000").unwrap());  
            match &STREAM {
                Some(_) => { }
                None => { panic!() }
                
            }
        }
    }

    fn recibir() -> u16 {
        let mut buffer = [0; 2];
        let mut stream;
        unsafe{
            stream = match &STREAM {
                Some(s) => {s}
                None => { panic!() }
            };
        }

        let n_bytes = stream.read(&mut buffer).unwrap();

        assert_eq!(n_bytes, 2); 

        <u16>::from_be_bytes(buffer)
    }

    fn enviar(mensaje: u32){
        let buffer: [u8;4] = mensaje.to_be_bytes();
        let mut stream;
        unsafe{
            stream = match &STREAM {
                Some(s) => {s}
                None => { panic!() }
            };
        }

        stream.write_all(&buffer).unwrap();
        stream.flush().unwrap();
    }

    #[test]
    fn escribir_registros_del_raspberry(){
        enviar(0x23000BB8);
        let ret_val = recibir();
        assert_eq!(ret_val, 0x0BB8);
    }

    #[test]
    fn escribir_numero_serie(){
        enviar(0x3C130000);
        let org_val = recibir();

        enviar(0x25131234);
        let ret_val = recibir();
        assert_eq!(ret_val, 0);

        enviar(0x3C130000);
        let ret_val = recibir();
        assert_ne!(ret_val, org_val);
    }

    #[test]
    fn escribir_numero_serie_fantasma(){
        enviar(0x25644321);
        let ret_val = recibir();
        assert_eq!(ret_val, 0x4321);

        enviar(0x3C130000);
        let ret_val = recibir();
        assert_eq!(ret_val, 0x4321);
    }

    #[test]
    fn sspa_active(){
        enviar(0x2305FFFF);
        let ret_val = recibir();
        assert_eq!(ret_val, 0xFFFF);
        enviar(0x3C000000);
        let ret_val = recibir()&0x4000;
        assert_eq!(ret_val, 0x4000);
    }

    #[test]
    fn sspa_inactive(){
        enviar(0x23050000);
        let ret_val = recibir();
        assert_eq!(ret_val, 0x0000);
        enviar(0x3C000000);
        let ret_val = recibir()&0x4000;
        assert_eq!(ret_val, 0);
    }

    #[test]
    fn alarm_reset(){
        enviar(0x25010020);
        let ret_val = recibir();
        assert_eq!(ret_val, 0x0020);
        enviar(0x3C000000);
        let ret_val = recibir()&0x00EF;
        assert_eq!(ret_val, 0x0000);
    }

    #[test]
    fn overdrive_alarm_a(){
        alarm_reset();
        enviar(0x3C0F0000);
        let trh_val = recibir() as u32 + 0x50;
        enviar(0x2A010000+trh_val);
        let ret_val = recibir() as u32;
        assert_eq!(ret_val, trh_val);
        enviar(0x3C000000);
        let ret_val = recibir()&0x0008;
        assert_eq!(ret_val, 0x0008);
        let trh_val = trh_val - 0xA0;
        enviar(0x2A010000+trh_val);
        let ret_val = recibir() as u32;
        assert_eq!(ret_val, trh_val);
        alarm_reset();
    }

    #[test]
    fn overdrive_alarm_b(){
        alarm_reset();
        let trh_val = 700;
        enviar(0x2A010050+trh_val);
        let ret_val = recibir() as u32;
        assert_eq!(ret_val, trh_val+0x50);
        enviar(0x250F0000+trh_val);
        let ret_val = recibir() as u32;
        assert_eq!(ret_val, trh_val);
        enviar(0x3C000000);
        let ret_val = recibir()&0x0008;
        assert_eq!(ret_val, 0x0008);
        let trh_val = trh_val - 0xA0;
        enviar(0x2A010000+trh_val);
        let ret_val = recibir() as u32;
        assert_eq!(ret_val, trh_val);
        alarm_reset();
    }

    #[test]
    fn underdrive_alarm_a(){
        alarm_reset();
        enviar(0x3C100000);
        let trh_val = recibir() as u32 - 0x50;
        enviar(0x2A010000+trh_val);
        let ret_val = recibir() as u32;
        assert_eq!(ret_val, trh_val);
        enviar(0x3C000000);
        let ret_val = recibir()&0x0010;
        assert_eq!(ret_val, 0x0010);
        let trh_val = trh_val + 0xA0;
        enviar(0x2A010000+trh_val);
        let ret_val = recibir() as u32;
        assert_eq!(ret_val, trh_val);
        alarm_reset();
    }

    #[test]
    fn underdrive_alarm_b(){
        alarm_reset();
        let trh_val = 200;
        enviar(0x2A010050+trh_val);
        let ret_val = recibir() as u32;
        assert_eq!(ret_val, trh_val-0x50);
        enviar(0x25100000+trh_val);
        let ret_val = recibir() as u32;
        assert_eq!(ret_val, trh_val);
        enviar(0x3C000000);
        let ret_val = recibir()&0x0010;
        assert_eq!(ret_val, 0x0010);
        let trh_val = trh_val + 0xA0;
        enviar(0x2A010000+trh_val);
        let ret_val = recibir() as u32;
        assert_eq!(ret_val, trh_val);
        alarm_reset();
    }

    #[test]
    fn outputpower_alarm_a(){
        alarm_reset();
        enviar(0x3C110000);
        let trh_val = recibir() as u32 - 0x50;
        enviar(0x2A020000+trh_val);
        let ret_val = recibir() as u32;
        assert_eq!(ret_val, trh_val);
        enviar(0x3C000000);
        let ret_val = recibir()&0x0020;
        assert_eq!(ret_val, 0x0020);
        let trh_val = trh_val + 0xA0;
        enviar(0x2A020000+trh_val);
        let ret_val = recibir() as u32;
        assert_eq!(ret_val, trh_val);
        alarm_reset();
    }

    #[test]
    fn outputpower_alarm_b(){
        alarm_reset();
        let trh_val = 200;
        enviar(0x2A020050+trh_val);
        let ret_val = recibir() as u32;
        assert_eq!(ret_val, trh_val-0x50);
        enviar(0x25110000+trh_val);
        let ret_val = recibir() as u32;
        assert_eq!(ret_val, trh_val);
        enviar(0x3C000000);
        let ret_val = recibir()&0x0020;
        assert_eq!(ret_val, 0x0020);
        let trh_val = trh_val + 0xA0;
        enviar(0x2A020000+trh_val);
        let ret_val = recibir() as u32;
        assert_eq!(ret_val, trh_val);
        alarm_reset();
    }

    #[test]
    #[ignore]
    fn tnr_wave(){
        aaaa_initial_conection();
        enviar(0x230005DC);
        let ret_val = recibir();
        assert_eq!(ret_val, 0x05DC);
        enviar(0x23010064);
        let ret_val = recibir();
        assert_eq!(ret_val, 0x0064);
        enviar(0x23020004);
        let ret_val = recibir();
        assert_eq!(ret_val, 0x0004);
        enviar(0x23030004);
        let ret_val = recibir();
        assert_eq!(ret_val, 0x0004);
        enviar(0x23040000);
        let ret_val = recibir();
        assert_eq!(ret_val, 0x0000);
        enviar(0x2305FFFF);
        let ret_val = recibir();
        assert_eq!(ret_val, 0xFFFF);
        enviar(0xA3000000);
        let ret_val = recibir();
        assert_eq!(ret_val, 0x0000);

    }
}
