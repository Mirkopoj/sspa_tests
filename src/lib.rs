#[cfg(test)]
mod tests {

    const WAIT_MILLIS: u64 = 100;
    const WAIT_CLEAR: u64 = 1000;
    const WAIT_DAC: u64 = 1500;
    const WAIT_RELAY: u64 = 1000;
    const WAIT_RESET: u64 = 2000;
    const WAIT_TNR: u64 = 5000;
    const THRESHOLE_HIGH: u32 = 700;
    const THRESHOLE_LOW: u32 = 300;

    use std::io::prelude::*;
    use std::net::TcpStream;
    use std::thread::sleep;
    use std::time::Duration;

    static mut STREAM: Option<TcpStream> = None;

    fn initial_conection() {
        unsafe {
            if STREAM.is_some() { return; }
            STREAM = Some(TcpStream::connect("192.168.1.16:8000").unwrap());  
            match &STREAM {
                Some(_) => { }
                None => { panic!() }
                
            }
        }
        sleep(Duration::from_millis(WAIT_MILLIS));
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

    fn enviar(mensaje: u32) -> u16{
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
        recibir()
    }

    #[test]
    #[ignore]
    fn escribir_registros_del_raspberry(){
        initial_conection();
        let ret_val = enviar(0x23000BB8);
        assert_eq!(ret_val, 0x0BB8);
    }

    #[test]
    fn escribir_numero_serie_raw(){
        initial_conection();
        let org_val = enviar(0x3C130000);

        let ret_val = enviar(0x25131234);
        assert_eq!(ret_val, 0);

        let ret_val = enviar(0x3C130000);
        assert_eq!(ret_val, org_val);
    }

    #[test]
    fn escribir_numero_serie_fantasma(){
        initial_conection();
        let ret_val = enviar(0x25644321)&0x7FFF;
        assert_eq!(ret_val, 0x4321);

        let ret_val = enviar(0x3C130000)&0x7FFF;
        assert_eq!(ret_val, 0x4321);
    }

    #[test]
    fn sspa_active(){
        initial_conection();
        let ret_val = enviar(0x2305FFFF);
        assert_eq!(ret_val, 0xFFFF);
        let ret_val = enviar(0x3C000000)&0x4000;
        assert_eq!(ret_val, 0x4000);
    }

    #[test]
    fn sspa_inactive(){
        initial_conection();
        let ret_val = enviar(0x23050000);
        assert_eq!(ret_val, 0x0000);
        let ret_val = enviar(0x3C000000)&0x4000;
        assert_eq!(ret_val, 0);
    }

    #[test]
    #[ignore]
    fn alarm_reset_test(){
        initial_conection();
        dac_clear();
        enviar(0x2A010000+511);
        enviar(0x2A020000+1023);
        sleep(Duration::from_millis(WAIT_DAC));
        alarm_reset();
        let ret_val = enviar(0x3C000000)&0x007F;
        assert_eq!(ret_val, 0);
        dac_clear();
        sleep(Duration::from_millis(WAIT_DAC));
    }

    fn alarm_reset(){
        let ctrl = enviar(0x3C010000) as u32;
        let ctrl = (ctrl|0x0020)&0x7FFF;
        enviar(0x25010000+ctrl);
    }

    fn dac_clear(){
        for addr in 0..8 {
            enviar(0x2A000000+(addr<<16));
        }
    }

    struct AnalogSig {
        trh: u32,
        alarma: u16,
        value: u32,
        dac: u32,
        disable: u32
    }

    const REG_TAB: [AnalogSig;9] = [
        AnalogSig{trh: 0x11, alarma: 0x20, value: 2, dac: 2, disable: 0x00}, //Output Power
        AnalogSig{trh: 0x12, alarma: 0x40, value: 3, dac: 0, disable: 0x10}, //Reflected Power
        AnalogSig{trh: 0x10, alarma: 0x10, value: 4, dac: 1, disable: 0x00}, //Underdrive
        AnalogSig{trh: 0x0F, alarma: 0x08, value: 4, dac: 1, disable: 0x08}, //Overdrive
        AnalogSig{trh: 0x0A, alarma: 0x02, value: 5, dac: 7, disable: 0x02}, //Temperature
        AnalogSig{trh: 0x0C, alarma: 0x01, value: 6, dac: 3, disable: 0x01}, //Gan1 Current
        AnalogSig{trh: 0x0C, alarma: 0x01, value: 7, dac: 4, disable: 0x01}, //Gan2 Current
        AnalogSig{trh: 0x0C, alarma: 0x01, value: 8, dac: 5, disable: 0x01}, //Gan3 Current
        AnalogSig{trh: 0x0C, alarma: 0x01, value: 9, dac: 6, disable: 0x01}, //Gan4 Current
    ];

    fn adc_alarm_a(reg: usize, dac_val: u32, trg_if: &str){
        initial_conection();
        dac_clear();
        let trh_val = (enviar(0x3C000000+(REG_TAB[reg].trh<<16))&0x7FFF).clamp(0, 1023) as u32;
        let ret_val = enviar(0x2A000000+dac_val+(REG_TAB[reg].dac<<16)) as u32;
        assert_eq!(ret_val, dac_val);
        sleep(Duration::from_millis(WAIT_DAC));
        alarm_reset();
        sleep(Duration::from_millis(WAIT_RESET));
        let measured_voltage = enviar(0x3C000000+(REG_TAB[reg].value<<16))&0x7FFF;
        println!("Measured level = {}, Dac value = {}, Threshole level = {}, Triggered if {}",
                 measured_voltage,
                 dac_val,
                 trh_val,
                 trg_if);
        let ret_val = enviar(0x3C000000)&REG_TAB[reg].alarma;
        alarm_reset();
        dac_clear();
        sleep(Duration::from_millis(WAIT_DAC));
        assert_eq!(ret_val, REG_TAB[reg].alarma);
    }

    fn adc_alarm_b(reg: usize, dac_val: u32, trg_if: &str){
        let trh_val = match trg_if {
            "lower" => { THRESHOLE_LOW },
            "higher" => { THRESHOLE_HIGH },
            _ => { panic!("Invalid Threshole"); }
        };
        initial_conection();
        dac_clear();
        let ret_val = enviar(0x2A000000+dac_val+(REG_TAB[reg].dac<<16)) as u32;
        assert_eq!(ret_val, dac_val);
        let ret_val = (enviar(0x25000000+trh_val+(REG_TAB[reg].trh<<16))&0x7FFF) as u32;
        assert_eq!(ret_val, trh_val);
        sleep(Duration::from_millis(WAIT_DAC));
        alarm_reset();
        sleep(Duration::from_millis(WAIT_RESET));
        let measured_voltage = enviar(0x3C000000+(REG_TAB[reg].value<<16))&0x7FFF;
        println!("Measured level = {}, Dac value = {}, Threshole level = {}, Triggered if {}",
                 measured_voltage,
                 dac_val,
                 trh_val,
                 trg_if);
        let ret_val = enviar(0x3C000000)&REG_TAB[reg].alarma;
        alarm_reset();
        dac_clear();
        sleep(Duration::from_millis(WAIT_DAC));
        assert_eq!(ret_val, REG_TAB[reg].alarma);
    }

    fn adc_no_alarm(reg: usize, dac_val: u32, trg_if: &str){ 
        initial_conection();
        dac_clear();
        let trh_val = (enviar(0x3C000000+(REG_TAB[reg].trh<<16))&0x7FFF).clamp(0, 1023) as u32;
        let ret_val = enviar(0x2A000000+dac_val+(REG_TAB[reg].dac<<16)) as u32;
        assert_eq!(ret_val, dac_val);
        sleep(Duration::from_millis(WAIT_DAC));
        alarm_reset();
        sleep(Duration::from_millis(WAIT_RESET));
        let measured_voltage = enviar(0x3C000000+(REG_TAB[reg].value<<16))&0x7FFF;
        println!("Measured level = {}, Dac value = {}, Threshole level = {}, Triggered if {}",
                 measured_voltage,
                 dac_val,
                 trh_val,
                 trg_if);
        let ret_val = enviar(0x3C000000)&REG_TAB[reg].alarma;
        alarm_reset();
        dac_clear();
        sleep(Duration::from_millis(WAIT_DAC));
        assert_eq!(ret_val, 0);
    }
    
    #[test]
    fn adc_outputpower_alarm_a(){
        adc_alarm_a(0, 0, "lower");
    }

    #[test]
    fn adc_outputpower_alarm_b(){
        adc_alarm_b(0, 0, "lower");
    }

    #[test]
    fn adc_outputpower_no_alarm(){
        adc_no_alarm(0, 511, "lower");
    }

    #[test]
    fn adc_reflectedpower_alarm_a(){
        adc_alarm_a(1, 1023, "higher");
        sspa_reset();
    }

    #[test]
    fn adc_reflectedpower_alarm_b(){
        adc_alarm_b(1, 1023, "higher");
        sspa_reset();
    }

    #[test]
    fn adc_reflectedpower_no_alarm(){
        adc_no_alarm(1, 511, "higher");
    }

    #[test]
    fn adc_underdrive_alarm_a(){
        adc_alarm_a(2, 0, "lower");
    }

    #[test]
    fn adc_underdrive_alarm_b(){
        adc_alarm_b(2, 0, "lower");
    }

    #[test]
    fn adc_underdrive_no_alarm(){
        adc_no_alarm(2, 511, "lower");
    }

    #[test]
    fn adc_overdrive_alarm_a(){
        adc_alarm_a(3, 1023, "higher");
    }

    #[test]

    fn adc_overdrive_alarm_b(){
        adc_alarm_b(3, 1023, "higher");
    }

    #[test]
    fn adc_overdrive_no_alarm(){
        adc_no_alarm(3, 511, "higher");
    }

    #[test]
    fn adc_overtemperature_alarm_a(){
        adc_alarm_a(4, 1023, "higher");
    }

    #[test]
    fn adc_overtemperature_alarm_b(){
        adc_alarm_b(4, 1023, "higher");
    }

    #[test]
    fn adc_overtemperature_no_alarm(){
        adc_no_alarm(4, 0, "higher");
    }

    #[test]
    fn adc_gan1current_alarm_a(){
        adc_alarm_a(5, 1023, "higher");
        sspa_reset();
    }

    #[test]
    fn adc_gan1current_alarm_b(){
        adc_alarm_b(5, 1023, "higher");
        sspa_reset();
    }

    #[test]
    fn adc_gan1current_no_alarm(){
        adc_no_alarm(5, 511, "higher");
    }

    #[test]
    fn adc_gan2current_alarm_a(){
        adc_alarm_a(6, 1023, "higher");
        sspa_reset();
    }

    #[test]
    fn adc_gan2current_alarm_b(){
        adc_alarm_b(6, 1023, "higher");
        sspa_reset();
    }

    #[test]
    fn adc_gan2current_no_alarm(){
        adc_no_alarm(6, 511, "higher");
    }

    #[test]
    fn adc_gan3current_alarm_a(){
        adc_alarm_a(7, 1023, "higher");
        sspa_reset();
    }

    #[test]
    fn adc_gan3current_alarm_b(){
        adc_alarm_b(7, 1023, "higher");
        sspa_reset();
    }

    #[test]
    fn adc_gan3current_no_alarm(){
        adc_no_alarm(7, 511, "higher");
    }

    #[test]
    fn adc_gan4current_alarm_a(){
        adc_alarm_a(8, 1023, "higher");
        sspa_reset();
    }

    #[test]
    fn adc_gan4current_alarm_b(){
        adc_alarm_b(8, 1023, "higher");
        sspa_reset();
    }

    #[test]
    fn adc_gan4current_no_alarm(){
        adc_no_alarm(8, 511, "higher");
    }

    fn temp_hist_core(){
        let reg = 4;
        let trg_if = "higher";
        let dac_val= 1023;
        let trh_val = (enviar(0x3C000000+(REG_TAB[reg].trh<<16))&0x7FFF).clamp(0, 1023) as u32;
        let ret_dac_val = enviar(0x2A000000+dac_val+(REG_TAB[reg].dac<<16)) as u32;
        assert_eq!(ret_dac_val, dac_val);
        sleep(Duration::from_millis(WAIT_DAC));
        alarm_reset();
        sleep(Duration::from_millis(WAIT_MILLIS));
        let measured_voltage = enviar(0x3C000000+(REG_TAB[reg].value<<16))&0x7FFF;
        println!("Measured level = {}, Dac value = {}, Threshole level = {}, Triggered if {}",
                 measured_voltage,
                 dac_val,
                 trh_val,
                 trg_if);
        let ret_val_1 = enviar(0x3C000000)&REG_TAB[reg].alarma;
        let trg_if = "lower";
        let dac_val= 511;
        let ret_dac_val = enviar(0x2A000000+dac_val+(REG_TAB[reg].dac<<16)) as u32;
        assert_eq!(ret_dac_val, dac_val);
        sleep(Duration::from_millis(WAIT_DAC));
        alarm_reset();
        sleep(Duration::from_millis(WAIT_MILLIS));
        let measured_voltage = enviar(0x3C000000+(REG_TAB[reg].value<<16))&0x7FFF;
        let histeresis = (enviar(0x3C0B0000)&0x7FFF) as u32;
        println!("Measured level = {}, Dac value = {}, Threshole level = {}, Triggered if {}",
                 measured_voltage,
                 dac_val,
                 trh_val-histeresis,
                 trg_if);
        let ret_val_2 = enviar(0x3C000000)&REG_TAB[reg].alarma;
        let dac_val= 0;
        let ret_dac_val = enviar(0x2A000000+dac_val+(REG_TAB[reg].dac<<16)) as u32;
        assert_eq!(ret_dac_val, dac_val);
        sleep(Duration::from_millis(WAIT_DAC));
        alarm_reset();
        sleep(Duration::from_millis(WAIT_MILLIS));
        let measured_voltage = enviar(0x3C000000+(REG_TAB[reg].value<<16))&0x7FFF;
        let histeresis = (enviar(0x3C0B0000)&0x7FFF) as u32;
        println!("Measured level = {}, Dac value = {}, Threshole level = {}, Triggered if {}",
                 measured_voltage,
                 dac_val,
                 trh_val-histeresis,
                 trg_if);
        let ret_val_3 = enviar(0x3C000000)&REG_TAB[reg].alarma;
        alarm_reset();
        dac_clear();
        sleep(Duration::from_millis(WAIT_DAC));
        assert_eq!(ret_val_1, REG_TAB[reg].alarma);
        assert_eq!(ret_val_2, REG_TAB[reg].alarma);
        assert_eq!(ret_val_3, 0);
    }

    #[test]
    fn adc_temperature_histeresis(){
        initial_conection();
        dac_clear();
        temp_hist_core();
    }

    #[test]
    fn fresh_boot_adc_temperature_histeresis(){
        initial_conection();
        dac_clear();
        relay_off();
        relay_on();
        temp_hist_core();
    }

    fn temp_protection_core(dac_val: u32) -> (u16, u16){
        let reg = 4;
        let trg_if = "higher";
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        let trh_val = (enviar(0x3C000000+(REG_TAB[reg].trh<<16))&0x7FFF).clamp(0, 1023) as u32;
        let ret_val = enviar(0x2A000000+dac_val+(REG_TAB[reg].dac<<16)) as u32;
        assert_eq!(ret_val, dac_val);
        sleep(Duration::from_millis(WAIT_DAC));
        alarm_reset();
        sleep(Duration::from_millis(WAIT_DAC));
        let measured_voltage = enviar(0x3C000000+(REG_TAB[reg].value<<16))&0x7FFF;
        println!("Measured level = {}, Dac value = {}, Threshole level = {}, Triggered if {}",
                 measured_voltage,
                 dac_val,
                 trh_val,
                 trg_if);
        let alarm_val = enviar(0x3C000000)&REG_TAB[reg].alarma;
        powen_on();
        sleep(Duration::from_millis(WAIT_MILLIS));
        tnr_set(3000, 250, 4, 4, 0);
        let ret_val = enviar(0x4D000000+(WAIT_TNR as u32));
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        alarm_reset();
        sleep(Duration::from_millis(WAIT_DAC));
        (ret_val, alarm_val)
    }

    #[test]
    fn temp_protection(){
        initial_conection();
        dac_clear();
        sleep(Duration::from_millis(WAIT_DAC));
        let (ret_val, alarm_val) = temp_protection_core(1023);
        dac_clear();
        sleep(Duration::from_millis(WAIT_DAC));
        assert_eq!(ret_val, 0);
        assert_ne!(alarm_val, 0);
    }

    #[test]
    fn temp_no_protection(){
        initial_conection();
        dac_clear();
        sleep(Duration::from_millis(WAIT_DAC));
        let (ret_val, alarm_val) = temp_protection_core(0);
        dac_clear();
        sleep(Duration::from_millis(WAIT_DAC));
        assert_ne!(ret_val, 0);
        assert_eq!(alarm_val, 0);
    }

    #[test]
    fn fresh_boot_temp_protection(){
        initial_conection();
        dac_clear();
        sleep(Duration::from_millis(WAIT_DAC));
        relay_off();
        relay_on();
        let (ret_val, alarm_val) = temp_protection_core(1023);
        dac_clear();
        sleep(Duration::from_millis(WAIT_DAC));
        assert_eq!(ret_val, 0);
        assert_ne!(alarm_val, 0);
    }

    #[test]
    fn fresh_boot_temp_no_protection(){
        initial_conection();
        dac_clear();
        sleep(Duration::from_millis(WAIT_DAC));
        relay_off();
        relay_on();
        let (ret_val, alarm_val) = temp_protection_core(0);
        dac_clear();
        sleep(Duration::from_millis(WAIT_DAC));
        assert_ne!(ret_val, 0);
        assert_eq!(alarm_val, 0);
    }

    #[test]
    fn temp_protection_disabled(){
        initial_conection();
        dac_clear();
        sleep(Duration::from_millis(WAIT_DAC));
        protect_disable(4);
        let (ret_val, alarm_val) = temp_protection_core(1023);
        protect_enable(4);
        dac_clear();
        sleep(Duration::from_millis(WAIT_DAC));
        assert_ne!(alarm_val, 0);
        assert_ne!(ret_val, 0);
    }

    #[test]
    fn fresh_boot_temp_protection_disabled(){
        initial_conection();
        dac_clear();
        sleep(Duration::from_millis(WAIT_DAC));
        relay_off();
        relay_on();
        protect_disable(4);
        let (ret_val, alarm_val) = temp_protection_core(1023);
        protect_enable(4);
        dac_clear();
        sleep(Duration::from_millis(WAIT_DAC));
        assert_ne!(alarm_val, 0);
        assert_ne!(ret_val, 0);
    }

    #[test]
    fn temp_histeresis_protection(){
        initial_conection();
        dac_clear();
        sleep(Duration::from_millis(WAIT_DAC));
        let (ret_val, alarm_val) = temp_protection_core(1023);
        assert_eq!(ret_val, 0);
        assert_ne!(alarm_val, 0);
        let (ret_val, alarm_val) = temp_protection_core(511);
        assert_eq!(ret_val, 0);
        assert_ne!(alarm_val, 0);
        let (ret_val, alarm_val) = temp_protection_core(0);
        dac_clear();
        sleep(Duration::from_millis(WAIT_DAC));
        assert_ne!(ret_val, 0);
        assert_eq!(alarm_val, 0);
    }

    #[test]
    fn fresh_boot_temp_histeresis_protection(){
        initial_conection();
        dac_clear();
        sleep(Duration::from_millis(WAIT_DAC));
        relay_off();
        relay_on();
        let (ret_val, alarm_val) = temp_protection_core(1023);
        assert_eq!(ret_val, 0);
        assert_ne!(alarm_val, 0);
        let (ret_val, alarm_val) = temp_protection_core(511);
        assert_eq!(ret_val, 0);
        assert_ne!(alarm_val, 0);
        let (ret_val, alarm_val) = temp_protection_core(0);
        dac_clear();
        sleep(Duration::from_millis(WAIT_DAC));
        assert_ne!(ret_val, 0);
        assert_eq!(alarm_val, 0);
    }

    #[test]
    #[ignore]
    fn store_to_non_volatile(){
        initial_conection();
        let org_val = enviar(0x3C0A0000)&0x7FFF;
        let new_val = (!org_val)&0x7FFF;
        let conf1 = enviar(0x250A0000+(new_val as u32))&0x7FFF;
        let ctrl = enviar(0x3C010000) as u32;
        let ctrl = (ctrl|0x0200)&0x7FFF;
        enviar(0x25010000+ctrl);
        sleep(Duration::from_millis(WAIT_MILLIS));
        relay_off();
        relay_on();
        let post_reset = enviar(0x3C0A0000)&0x7FFF;
        let conf2 = enviar(0x250A0000+(org_val as u32))&0x7FFF;
        let ctrl = enviar(0x3C010000) as u32;
        let ctrl = (ctrl|0x0200)&0x7FFF;
        enviar(0x25010000+ctrl);
        sleep(Duration::from_millis(WAIT_MILLIS));
        relay_off();
        relay_on();
        let clean_up = enviar(0x3C0A0000)&0x7FFF;
        assert_eq!(conf1, new_val);
        assert_eq!(post_reset, new_val);
        assert_eq!(conf2, org_val);
        assert_eq!(clean_up, org_val);
    }

    #[test]
    #[ignore]
    fn load_from_non_volatile(){
        initial_conection();
        let org_val = enviar(0x3C0A0000)&0x7FFF;
        let new_val = (!org_val)&0x7FFF;
        let conf1 = enviar(0x250A0000+(new_val as u32))&0x7FFF;
        let ctrl = enviar(0x3C010000) as u32;
        let ctrl = (ctrl|0x0200)&0x7FFF;
        enviar(0x25010000+ctrl);
        sleep(Duration::from_millis(WAIT_CLEAR));
        relay_off();
        relay_on();
        let post_reset = enviar(0x3C0A0000)&0x7FFF;
        let conf2 = enviar(0x250A0000+(org_val as u32))&0x7FFF;
        let ctrl = enviar(0x3C010000) as u32;
        let ctrl = (ctrl|0x0100)&0x7FFF;
        enviar(0x25010000+ctrl);
        sleep(Duration::from_millis(WAIT_MILLIS));
        let post_load = enviar(0x3C0A0000)&0x7FFF;
        let conf3 = enviar(0x250A0000+(org_val as u32))&0x7FFF;
        let ctrl = enviar(0x3C010000) as u32;
        let ctrl = (ctrl|0x0200)&0x7FFF;
        enviar(0x25010000+ctrl);
        sleep(Duration::from_millis(WAIT_MILLIS));
        relay_off();
        relay_on();
        let clean_up = enviar(0x3C0A0000)&0x7FFF;
        assert_eq!(conf1, new_val);
        assert_eq!(post_reset, new_val);
        assert_eq!(conf2, org_val);
        assert_eq!(post_load, new_val);
        assert_eq!(conf3, org_val);
        assert_eq!(clean_up, org_val);
    }

    #[test]
    #[ignore]
    fn sspa_reset_test(){
        initial_conection();
        powen_on();
        let ctrl = enviar(0x3C010000) as u32;
        let ctrl = (ctrl|0x0040)&0x7FFF;
        enviar(0x25010000+ctrl);
        let status_pre = enviar(0x3C000000)&0x4000;
        sleep(Duration::from_millis(WAIT_RESET));
        let status_post = enviar(0x3C000000)&0x4000;
        powen_off();
        assert_eq!(status_pre, 0);
        assert_ne!(status_post, 0);
    }

    #[test]
    #[ignore]
    fn sspa_disable_test(){
        initial_conection();
        powen_on();
        let ctrl = enviar(0x3C010000) as u32;
        let ctrl = (ctrl|0x0080)&0x7FFF;
        enviar(0x25010000+ctrl);
        let status_pre = enviar(0x3C000000)&0x4000;
        sleep(Duration::from_millis(WAIT_RESET));
        let status_mid = enviar(0x3C000000)&0x4000;
        sspa_reset();
        let status_post = enviar(0x3C000000)&0x4000;
        powen_off();
        assert_eq!(status_pre, 0);
        assert_eq!(status_mid, 0);
        assert_ne!(status_post, 0);
    }

    fn sspa_reset(){
        let ctrl = enviar(0x3C010000) as u32;
        let ctrl = (ctrl|0x0040)&0x7FFF;
        enviar(0x25010000+ctrl);
        sleep(Duration::from_millis(WAIT_RESET));
    }

    fn disable_protection_core(reg: usize, dac_val: u32) -> (u16, u16){
        let trg_if = "higher";
        dac_clear();
        powen_on();
        sleep(Duration::from_millis(WAIT_CLEAR));
        let trh_val = (enviar(0x3C000000+(REG_TAB[reg].trh<<16))&0x7FFF).clamp(0, 1023) as u32;
        let ret_val = enviar(0x2A000000+dac_val+(REG_TAB[reg].dac<<16)) as u32;
        assert_eq!(ret_val, dac_val);
        sleep(Duration::from_millis(WAIT_DAC));
        alarm_reset();
        sleep(Duration::from_millis(WAIT_RESET));
        let measured_voltage = enviar(0x3C000000+(REG_TAB[reg].value<<16))&0x7FFF;
        println!("Measured level = {}, Dac value = {}, Threshole level = {}, Triggered if {}",
                 measured_voltage,
                 dac_val,
                 trh_val,
                 trg_if);
        let status_reg = enviar(0x3C000000);
        let alarm_val = status_reg&REG_TAB[reg].alarma;
        let sspa_active = status_reg&0x4000;
        dac_clear();
        powen_off();
        sleep(Duration::from_millis(WAIT_DAC));
        alarm_reset();
        sspa_reset();
        (sspa_active, alarm_val)
    }

    fn protect_disable(reg: usize){
        let ctrl = enviar(0x3C010000) as u32;
        let ctrl = (ctrl | REG_TAB[reg].disable)&0x7FFF;
        enviar(0x25010000+ctrl);
    }

    fn protect_enable(reg: usize){
        let ctrl = enviar(0x3C010000) as u32;
        let ctrl = (ctrl & !REG_TAB[reg].disable)&0x7FFF;
        enviar(0x25010000+ctrl);
    }

    #[test]
    fn refpow_protection(){
        initial_conection();
        let (sspa_active, alarm_val) = disable_protection_core(1, 1023);
        assert_ne!(alarm_val, 0);
        assert_eq!(sspa_active, 0);
    }

    #[test]
    fn refpow_no_protection(){
        initial_conection();
        let (sspa_active, alarm_val) = disable_protection_core(1, 0);
        assert_eq!(alarm_val, 0);
        assert_ne!(sspa_active, 0);
    }

    #[test]
    fn fresh_boot_refpow_protection(){
        initial_conection();
        relay_off();
        relay_on();
        let (sspa_active, alarm_val) = disable_protection_core(1, 1023);
        assert_ne!(alarm_val, 0);
        assert_eq!(sspa_active, 0);
    }

    #[test]
    fn fresh_boot_refpow_no_protection(){
        initial_conection();
        relay_off();
        relay_on();
        let (sspa_active, alarm_val) = disable_protection_core(1, 0);
        assert_eq!(alarm_val, 0);
        assert_ne!(sspa_active, 0);
    }

    #[test]
    fn refpow_protection_disabled(){
        initial_conection();
        protect_disable(1);
        let (sspa_active, alarm_val) = disable_protection_core(1, 1023);
        protect_enable(1);
        assert_ne!(alarm_val, 0);
        assert_ne!(sspa_active, 0);
    }

    #[test]
    fn fresh_boot_refpow_protection_disabled(){
        initial_conection();
        relay_off();
        relay_on();
        protect_disable(1);
        let (sspa_active, alarm_val) = disable_protection_core(1, 1023);
        protect_enable(1);
        assert_ne!(alarm_val, 0);
        assert_ne!(sspa_active, 0);
    }

    #[test]
    fn gan1curr_protection(){
        initial_conection();
        let (sspa_active, alarm_val) = disable_protection_core(5, 1023);
        assert_ne!(alarm_val, 0);
        assert_eq!(sspa_active, 0);
    }

    #[test]
    fn gan1curr_no_protection(){
        initial_conection();
        let (sspa_active, alarm_val) = disable_protection_core(5, 0);
        assert_eq!(alarm_val, 0);
        assert_ne!(sspa_active, 0);
    }

    #[test]
    fn fresh_boot_gan1curr_protection(){
        initial_conection();
        relay_off();
        relay_on();
        let (sspa_active, alarm_val) = disable_protection_core(5, 1023);
        assert_ne!(alarm_val, 0);
        assert_eq!(sspa_active, 0);
    }

    #[test]
    fn fresh_boot_gan1curr_no_protection(){
        initial_conection();
        relay_off();
        relay_on();
        let (sspa_active, alarm_val) = disable_protection_core(5, 0);
        assert_eq!(alarm_val, 0);
        assert_ne!(sspa_active, 0);
    }

    #[test]
    fn gan1curr_protection_disabled(){
        initial_conection();
        protect_disable(5);
        let (sspa_active, alarm_val) = disable_protection_core(5, 1023);
        protect_enable(5);
        assert_ne!(alarm_val, 0);
        assert_ne!(sspa_active, 0);
    }

    #[test]
    fn fresh_boot_gan1curr_protection_disabled(){
        initial_conection();
        relay_off();
        relay_on();
        protect_disable(5);
        let (sspa_active, alarm_val) = disable_protection_core(5, 1023);
        protect_enable(5);
        assert_ne!(alarm_val, 0);
        assert_ne!(sspa_active, 0);
    }

    #[test]
    fn gan2curr_protection(){
        initial_conection();
        let (sspa_active, alarm_val) = disable_protection_core(6, 1023);
        assert_ne!(alarm_val, 0);
        assert_eq!(sspa_active, 0);
    }

    #[test]
    fn gan2curr_no_protection(){
        initial_conection();
        let (sspa_active, alarm_val) = disable_protection_core(6, 0);
        assert_eq!(alarm_val, 0);
        assert_ne!(sspa_active, 0);
    }

    #[test]
    fn fresh_boot_gan2curr_protection(){
        initial_conection();
        relay_off();
        relay_on();
        let (sspa_active, alarm_val) = disable_protection_core(6, 1023);
        assert_ne!(alarm_val, 0);
        assert_eq!(sspa_active, 0);
    }

    #[test]
    fn fresh_boot_gan2curr_no_protection(){
        initial_conection();
        relay_off();
        relay_on();
        let (sspa_active, alarm_val) = disable_protection_core(6, 0);
        assert_eq!(alarm_val, 0);
        assert_ne!(sspa_active, 0);
    }

    #[test]
    fn gan2curr_protection_disabled(){
        initial_conection();
        protect_disable(6);
        let (sspa_active, alarm_val) = disable_protection_core(6, 1023);
        protect_enable(6);
        assert_ne!(alarm_val, 0);
        assert_ne!(sspa_active, 0);
    }

    #[test]
    fn fresh_boot_gan2curr_protection_disabled(){
        initial_conection();
        relay_off();
        relay_on();
        protect_disable(6);
        let (sspa_active, alarm_val) = disable_protection_core(6, 1023);
        protect_enable(6);
        assert_ne!(alarm_val, 0);
        assert_ne!(sspa_active, 0);
    }

    #[test]
    fn gan3curr_protection(){
        initial_conection();
        let (sspa_active, alarm_val) = disable_protection_core(7, 1023);
        assert_ne!(alarm_val, 0);
        assert_eq!(sspa_active, 0);
    }

    #[test]
    fn gan3curr_no_protection(){
        initial_conection();
        let (sspa_active, alarm_val) = disable_protection_core(7, 0);
        assert_eq!(alarm_val, 0);
        assert_ne!(sspa_active, 0);
    }

    #[test]
    fn fresh_boot_gan3curr_protection(){
        initial_conection();
        relay_off();
        relay_on();
        let (sspa_active, alarm_val) = disable_protection_core(7, 1023);
        assert_ne!(alarm_val, 0);
        assert_eq!(sspa_active, 0);
    }

    #[test]
    fn fresh_boot_gan3curr_no_protection(){
        initial_conection();
        relay_off();
        relay_on();
        let (sspa_active, alarm_val) = disable_protection_core(7, 0);
        assert_eq!(alarm_val, 0);
        assert_ne!(sspa_active, 0);
    }

    #[test]
    fn gan3curr_protection_disabled(){
        initial_conection();
        protect_disable(7);
        let (sspa_active, alarm_val) = disable_protection_core(7, 1023);
        protect_enable(7);
        assert_ne!(alarm_val, 0);
        assert_ne!(sspa_active, 0);
    }

    #[test]
    fn fresh_boot_gan3curr_protection_disabled(){
        initial_conection();
        relay_off();
        relay_on();
        protect_disable(7);
        let (sspa_active, alarm_val) = disable_protection_core(7, 1023);
        protect_enable(7);
        assert_ne!(alarm_val, 0);
        assert_ne!(sspa_active, 0);
    }

    #[test]
    fn gan4curr_protection(){
        initial_conection();
        let (sspa_active, alarm_val) = disable_protection_core(8, 1023);
        assert_ne!(alarm_val, 0);
        assert_eq!(sspa_active, 0);
    }

    #[test]
    fn gan4curr_no_protection(){
        initial_conection();
        let (sspa_active, alarm_val) = disable_protection_core(8, 0);
        assert_eq!(alarm_val, 0);
        assert_ne!(sspa_active, 0);
    }

    #[test]
    fn fresh_boot_gan4curr_protection(){
        initial_conection();
        relay_off();
        relay_on();
        let (sspa_active, alarm_val) = disable_protection_core(8, 1023);
        assert_ne!(alarm_val, 0);
        assert_eq!(sspa_active, 0);
    }

    #[test]
    fn fresh_boot_gan4curr_no_protection(){
        initial_conection();
        relay_off();
        relay_on();
        let (sspa_active, alarm_val) = disable_protection_core(8, 0);
        assert_eq!(alarm_val, 0);
        assert_ne!(sspa_active, 0);
    }

    #[test]
    fn gan4curr_protection_disabled(){
        initial_conection();
        protect_disable(8);
        let (sspa_active, alarm_val) = disable_protection_core(8, 1023);
        protect_enable(8);
        assert_ne!(alarm_val, 0);
        assert_ne!(sspa_active, 0);
    }

    #[test]
    fn fresh_boot_gan4curr_protection_disabled(){
        initial_conection();
        relay_off();
        relay_on();
        protect_disable(8);
        let (sspa_active, alarm_val) = disable_protection_core(8, 1023);
        protect_enable(8);
        assert_ne!(alarm_val, 0);
        assert_ne!(sspa_active, 0);
    }

    fn fresh_boot_adc_alarm_a(reg: usize, dac_val: u32, trg_if: &str){
        initial_conection();
        dac_clear();
        let trh_val = (enviar(0x3C000000+(REG_TAB[reg].trh<<16))&0x7FFF) as u32;
        let ret_val = (enviar(0x2A000000+dac_val+(REG_TAB[reg].dac<<16))&0x7FFF) as u32;
        assert_eq!(ret_val, dac_val);
        sleep(Duration::from_millis(WAIT_DAC));
        relay_off();
        relay_on();
        sleep(Duration::from_millis(WAIT_RESET));
        let measured_voltage = enviar(0x3C000000+(REG_TAB[reg].value<<16))&0x7FFF;
        println!("Measured level = {}, Dac value = {}, Threshole level = {}, Triggered if {}",
                 measured_voltage,
                 dac_val,
                 trh_val,
                 trg_if);
        let ret_val = enviar(0x3C000000)&REG_TAB[reg].alarma;
        alarm_reset();
        dac_clear();
        sleep(Duration::from_millis(WAIT_DAC));
        assert_eq!(ret_val, REG_TAB[reg].alarma);
    }

    fn fresh_boot_adc_alarm_b(reg: usize, dac_val: u32, trg_if: &str){
        let trh_val = match trg_if {
            "lower" => { THRESHOLE_LOW },
            "higher" => { THRESHOLE_HIGH },
            _ => { panic!("Invalid Threshole"); }
        };
        initial_conection();
        dac_clear();
        let ret_val = (enviar(0x2A000000+dac_val+(REG_TAB[reg].dac<<16))&0x7FFF) as u32;
        assert_eq!(ret_val, dac_val);
        relay_off();
        relay_on();
        let ret_val = (enviar(0x25000000+trh_val+(REG_TAB[reg].trh<<16))&0x7FFF) as u32;
        assert_eq!(ret_val, trh_val);
        sleep(Duration::from_millis(WAIT_DAC));
        alarm_reset();
        sleep(Duration::from_millis(WAIT_RESET));
        let measured_voltage = enviar(0x3C000000+(REG_TAB[reg].value<<16))&0x7FFF;
        println!("Measured level = {}, Dac value = {}, Threshole level = {}, Triggered if {}",
                 measured_voltage,
                 dac_val,
                 trh_val,
                 trg_if);
        let ret_val = enviar(0x3C000000)&REG_TAB[reg].alarma;
        alarm_reset();
        dac_clear();
        sleep(Duration::from_millis(WAIT_DAC));
        assert_eq!(ret_val, REG_TAB[reg].alarma);
    }

    fn fresh_boot_adc_no_alarm(reg: usize, dac_val: u32, trg_if: &str){
        initial_conection();
        dac_clear();
        let trh_val = (enviar(0x3C000000+(REG_TAB[reg].trh<<16))&0x7FFF).clamp(0, 1023) as u32;
        let ret_val = (enviar(0x2A000000+dac_val+(REG_TAB[reg].dac<<16))&0x7FFF) as u32;
        assert_eq!(ret_val, dac_val);
        sleep(Duration::from_millis(WAIT_DAC));
        relay_off();
        relay_on();
        alarm_reset();
        sleep(Duration::from_millis(WAIT_RESET));
        let measured_voltage = enviar(0x3C000000+(REG_TAB[reg].value<<16))&0x7FFF;
        println!("Measured level = {}, Dac value = {}, Threshole level = {}, Triggered if {}",
                 measured_voltage,
                 dac_val,
                 trh_val,
                 trg_if);
        let ret_val = enviar(0x3C000000)&REG_TAB[reg].alarma;
        alarm_reset();
        dac_clear();
        sleep(Duration::from_millis(WAIT_DAC));
        assert_eq!(ret_val, 0);
    }

    #[test]
    fn fresh_boot_adc_outputpower_alarm_a(){
        fresh_boot_adc_alarm_a(0, 0, "lower");
    }

    #[test]
    fn fresh_boot_adc_outputpower_alarm_b(){
        fresh_boot_adc_alarm_b(0, 0, "lower");
    }

    #[test]
    fn fresh_boot_adc_outputpower_no_alarm(){
        fresh_boot_adc_no_alarm(0, 511, "lower");
    }

    #[test]
    fn fresh_boot_adc_reflectedpower_alarm_a(){
        fresh_boot_adc_alarm_a(1, 1023, "higher");
    }

    #[test]
    fn fresh_boot_adc_reflectedpower_alarm_b(){
        fresh_boot_adc_alarm_b(1, 1023, "higher");
    }

    #[test]
    fn fresh_boot_adc_reflectedpower_no_alarm(){
        fresh_boot_adc_no_alarm(1, 511, "higher");
    }

    #[test]
    fn fresh_boot_adc_underdrive_alarm_a(){
        fresh_boot_adc_alarm_a(2, 0, "lower");
    }

    #[test]
    fn fresh_boot_adc_underdrive_alarm_b(){
        fresh_boot_adc_alarm_b(2, 0, "lower");
    }

    #[test]
    fn fresh_boot_adc_underdrive_no_alarm(){
        fresh_boot_adc_no_alarm(2, 511, "lower");
    }
    #[test]
    fn fresh_boot_adc_overdrive_alarm_a(){
        fresh_boot_adc_alarm_a(3, 1023, "higher");
    }

    #[test]
    fn fresh_boot_adc_overdrive_alarm_b(){
        fresh_boot_adc_alarm_b(3, 1023, "higher");
    }

    #[test]
    fn fresh_boot_adc_overdrive_no_alarm(){
        fresh_boot_adc_no_alarm(3, 511, "higher");
    }

    #[test]
    fn fresh_boot_adc_overtemperature_alarm_a(){
        fresh_boot_adc_alarm_a(4, 1023, "higher");
    }

    #[test]
    fn fresh_boot_adc_overtemperature_alarm_b(){
        fresh_boot_adc_alarm_b(4, 1023, "higher");
    }

    #[test]
    fn fresh_boot_adc_overtemperature_no_alarm(){
        fresh_boot_adc_no_alarm(4, 511, "higher");
    }

    #[test]
    fn fresh_boot_adc_gan1current_alarm_a(){
        fresh_boot_adc_alarm_a(5, 1023, "higher");
    }

    #[test]
    fn fresh_boot_adc_gan1current_alarm_b(){
        fresh_boot_adc_alarm_b(5, 1023, "higher");
    }

    #[test]
    fn fresh_boot_adc_gan1current_no_alarm(){
        fresh_boot_adc_no_alarm(5, 511, "higher");
    }

    #[test]
    fn fresh_boot_adc_gan2current_alarm_a(){
        fresh_boot_adc_alarm_a(6, 1023, "higher");
    }

    #[test]
    fn fresh_boot_adc_gan2current_alarm_b(){
        fresh_boot_adc_alarm_b(6, 1023, "higher");
    }

    #[test]
    fn fresh_boot_adc_gan2current_no_alarm(){
        fresh_boot_adc_no_alarm(6, 511, "higher");
    }

    #[test]
    fn fresh_boot_adc_gan3current_alarm_a(){
        fresh_boot_adc_alarm_a(7, 1023, "higher");
    }

    #[test]
    fn fresh_boot_adc_gan3current_alarm_b(){
        fresh_boot_adc_alarm_b(7, 1023, "higher");
    }

    #[test]
    fn fresh_boot_adc_gan3current_no_alarm(){
        fresh_boot_adc_no_alarm(7, 511, "higher");
    }

    #[test]
    fn fresh_boot_adc_gan4current_alarm_a(){
        fresh_boot_adc_alarm_a(8, 1023, "higher");
    }

    #[test]
    fn fresh_boot_adc_gan4current_alarm_b(){
        fresh_boot_adc_alarm_b(8, 1023, "higher");
    }

    #[test]
    fn fresh_boot_adc_gan4current_no_alarm(){
        fresh_boot_adc_no_alarm(8, 511, "higher");
    }

    #[test]
    #[ignore]
    fn tnr_wave(){
        initial_conection();
        powen_on();
        tnr_set(3000, 250, 4, 4, 0);
    }

    fn tnr_clear(){
        tnr_off();
        powen_off()
    }

    fn tnr_off(){
        let ret_val = enviar(0x23040001);
        assert_eq!(ret_val, 0x0001);
        let ret_val = enviar(0x2301000A);
        assert_eq!(ret_val, 0x000A);
        let ret_val = enviar(0xA3000000);
        assert_eq!(ret_val, 0x0000);
    }

    fn tnr_set(per: u16, ancho: u16, off1: u16, off2: u16, count: u16){
        let ret_val = enviar(0x23000000+per as u32);
        assert_eq!(ret_val, per);
        let ret_val = enviar(0x23010000+ancho as u32);
        assert_eq!(ret_val, ancho);
        let ret_val = enviar(0x23020000+off1 as u32);
        assert_eq!(ret_val, off1);
        let ret_val = enviar(0x23030000+off2 as u32);
        assert_eq!(ret_val, off2);
        let ret_val = enviar(0x23040000+count as u32);
        assert_eq!(ret_val, count);
        let ret_val = enviar(0xA3000000);
        assert_eq!(ret_val, 0x0000);
    }

    fn powen_on(){
        let ret_val = enviar(0x2305FFFF);
        assert_eq!(ret_val, 0xFFFF);
    }
    
    fn powen_off(){
        let ret_val = enviar(0x23050000);
        assert_eq!(ret_val, 0x0000000);
    }
    
    #[test]
    fn tnr_secuencia_correcta(){
        initial_conection();
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        alarm_reset();
        powen_on();
        sleep(Duration::from_millis(WAIT_MILLIS));
        tnr_set(3000, 250, 4, 4, 0);
        sleep(Duration::from_millis(WAIT_TNR));
        //Ver que no esté la alarma
        let ret_val = enviar(0x3C000000)&0x4;
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        assert_eq!(ret_val, 0);
    }
    
    #[test]
    fn tnr_secuencia_invertida(){
        initial_conection();
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        alarm_reset();
        tnr_set(3000, 250, 4, 4, 0);
        sleep(Duration::from_millis(WAIT_TNR));
        powen_on();
        sleep(Duration::from_millis(WAIT_TNR));
        //Ver que no esté la alarma
        let ret_val = enviar(0x3C000000)&0x4;
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        assert_eq!(ret_val, 0);
    }
    
    #[test]
    fn tnr_invalido_periodo_corto_secuencia_correcta(){
        initial_conection();
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        alarm_reset();
        powen_on();
        sleep(Duration::from_millis(WAIT_MILLIS));
        tnr_set(1500, 250, 4, 4, 0);
        sleep(Duration::from_millis(WAIT_TNR));
        //Ver que si esté la alarma
        let ret_val = enviar(0x3C000000)&0x4;
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        assert_ne!(ret_val, 0);
    }
    
    #[test]
    fn tnr_invalido_periodo_corto_secuencia_invertida(){
        initial_conection();
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        alarm_reset();
        tnr_set(1500, 250, 4, 4, 0);
        sleep(Duration::from_millis(WAIT_TNR));
        powen_on();
        sleep(Duration::from_millis(WAIT_TNR));
        //Ver que si esté la alarma
        let ret_val = enviar(0x3C000000)&0x4;
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        assert_ne!(ret_val, 0);
    }
   
    #[test]
    fn tnr_invalido_pulso_ancho_secuencia_correcta(){
        initial_conection();
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        alarm_reset();
        powen_on();
        sleep(Duration::from_millis(WAIT_MILLIS));
        tnr_set(4000, 400, 4, 4, 0);
        sleep(Duration::from_millis(WAIT_TNR));
        //Ver que si esté la alarma
        let ret_val = enviar(0x3C000000)&0x4;
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        assert_ne!(ret_val, 0);
    }
    
    #[test]
    fn tnr_invalido_pulso_ancho_secuencia_invertida(){
        initial_conection();
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        alarm_reset();
        tnr_set(4000, 400, 4, 4, 0);
        sleep(Duration::from_millis(WAIT_TNR));
        powen_on();
        sleep(Duration::from_millis(WAIT_TNR));
        //Ver que si esté la alarma
        let ret_val = enviar(0x3C000000)&0x4;
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        assert_ne!(ret_val, 0);
    }

    #[test]
    fn tnr_sin_powen(){
        initial_conection();
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        alarm_reset();
        tnr_set(3000, 250, 4, 4, 0);
        sleep(Duration::from_millis(WAIT_TNR));
        //Ver que no esté la alarma
        let ret_val = enviar(0x3C000000)&0x4;
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        assert_eq!(ret_val, 0);
    }
    
    #[test]
    fn tnr_invalido_sin_powen(){
        initial_conection();
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        alarm_reset();
        tnr_set(1000, 250, 4, 4, 0);
        sleep(Duration::from_millis(WAIT_TNR));
        //Ver que no esté la alarma
        let ret_val = enviar(0x3C000000)&0x4;
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        assert_eq!(ret_val, 0);
    }

    #[test]
    fn tnr_invalido_pulso_durante_habilitacion_secuencia_correcta(){
        initial_conection();
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        alarm_reset();
        powen_on();
        sleep(Duration::from_millis(WAIT_MILLIS));
        tnr_set(2375, 250, 4, 4, 0);
        sleep(Duration::from_millis(WAIT_TNR));
        //Ver que si esté la alarma
        let ret_val = enviar(0x3C000000)&0x4;
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        assert_ne!(ret_val, 0);
    }

    #[test]
    fn tnr_invalido_pulso_durante_habilitacion_secuencia_invertida(){
        initial_conection();
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        alarm_reset();
        tnr_set(2375, 250, 4, 4, 0);
        sleep(Duration::from_millis(WAIT_TNR));
        powen_on();
        sleep(Duration::from_millis(WAIT_TNR));
        //Ver que si esté la alarma
        let ret_val = enviar(0x3C000000)&0x4;
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        assert_ne!(ret_val, 0);
    }

    #[test]
    fn tnr_nunca_baja_secuencia_correcta(){
        initial_conection();
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        powen_on();
        sleep(Duration::from_millis(WAIT_MILLIS));
        alarm_reset();
        tnr_set(800, 800, 0, 0, 0);
        sleep(Duration::from_millis(WAIT_TNR));
        //Ver que si esté la alarma
        let ret_val = enviar(0x3C000000)&0x4;
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        assert_ne!(ret_val, 0);
    }

    #[test]
    fn tnr_nunca_baja_secuencia_invertida(){
        initial_conection();
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        alarm_reset();
        tnr_set(800, 800, 0, 0, 0);
        sleep(Duration::from_millis(WAIT_TNR));
        powen_on();
        sleep(Duration::from_millis(WAIT_TNR));
        //Ver que si esté la alarma
        let ret_val = enviar(0x3C000000)&0x4;
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        assert_ne!(ret_val, 0);
    }

    fn relay_on(){
        enviar(0x2D000001);
        sleep(Duration::from_millis(WAIT_RELAY));
    }

    fn relay_off(){
        enviar(0x2D000000);
        sleep(Duration::from_millis(WAIT_RELAY));
    }


    #[test]
    fn fresh_boot_tnr_secuencia_correcta(){
        initial_conection();
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        alarm_reset();
        relay_off();
        relay_on();
        powen_on();
        sleep(Duration::from_millis(WAIT_MILLIS));
        tnr_set(3000, 250, 4, 4, 0);
        sleep(Duration::from_millis(WAIT_TNR));
        //Ver que no esté la alarma
        let ret_val = enviar(0x3C000000)&0x4;
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        assert_eq!(ret_val, 0);
    }
    
    #[test]
    fn fresh_boot_tnr_secuencia_invertida(){
        initial_conection();
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        alarm_reset();
        relay_off();
        relay_on();
        tnr_set(3000, 250, 4, 4, 0);
        sleep(Duration::from_millis(WAIT_TNR));
        powen_on();
        sleep(Duration::from_millis(WAIT_TNR));
        //Ver que no esté la alarma
        let ret_val = enviar(0x3C000000)&0x4;
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        assert_eq!(ret_val, 0);
    }
    
    #[test]
    fn fresh_boot_tnr_invalido_periodo_corto_secuencia_correcta(){
        initial_conection();
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        alarm_reset();
        relay_off();
        relay_on();
        powen_on();
        sleep(Duration::from_millis(WAIT_MILLIS));
        tnr_set(1500, 250, 4, 4, 0);
        sleep(Duration::from_millis(WAIT_TNR));
        //Ver que si esté la alarma
        let ret_val = enviar(0x3C000000)&0x4;
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        assert_ne!(ret_val, 0);
    }
    
    #[test]
    fn fresh_boot_tnr_invalido_periodo_corto_secuencia_invertida(){
        initial_conection();
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        alarm_reset();
        relay_off();
        relay_on();
        tnr_set(1500, 250, 4, 4, 0);
        sleep(Duration::from_millis(WAIT_TNR));
        powen_on();
        sleep(Duration::from_millis(WAIT_TNR));
        //Ver que si esté la alarma
        let ret_val = enviar(0x3C000000)&0x4;
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        assert_ne!(ret_val, 0);
    }
   
    #[test]
    fn fresh_boot_tnr_invalido_pulso_ancho_secuencia_correcta(){
        initial_conection();
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        alarm_reset();
        relay_off();
        relay_on();
        powen_on();
        sleep(Duration::from_millis(WAIT_MILLIS));
        tnr_set(4000, 400, 4, 4, 0);
        sleep(Duration::from_millis(WAIT_TNR));
        //Ver que si esté la alarma
        let ret_val = enviar(0x3C000000)&0x4;
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        assert_ne!(ret_val, 0);
    }
    
    #[test]
    fn fresh_boot_tnr_invalido_pulso_ancho_secuencia_invertida(){
        initial_conection();
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        alarm_reset();
        relay_off();
        relay_on();
        tnr_set(4000, 400, 4, 4, 0);
        sleep(Duration::from_millis(WAIT_TNR));
        powen_on();
        sleep(Duration::from_millis(WAIT_TNR));
        //Ver que si esté la alarma
        let ret_val = enviar(0x3C000000)&0x4;
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        assert_ne!(ret_val, 0);
    }

    #[test]
    fn fresh_boot_tnr_sin_powen(){
        initial_conection();
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        alarm_reset();
        relay_off();
        relay_on();
        tnr_set(3000, 250, 4, 4, 0);
        sleep(Duration::from_millis(WAIT_TNR));
        //Ver que no esté la alarma
        let ret_val = enviar(0x3C000000)&0x4;
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        assert_eq!(ret_val, 0);
    }
    
    #[test]
    fn fresh_boot_tnr_invalido_sin_powen(){
        initial_conection();
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        alarm_reset();
        relay_off();
        relay_on();
        tnr_set(1000, 250, 4, 4, 0);
        sleep(Duration::from_millis(WAIT_TNR));
        //Ver que no esté la alarma
        let ret_val = enviar(0x3C000000)&0x4;
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        assert_eq!(ret_val, 0);
    }

    #[test]
    fn fresh_boot_tnr_invalido_pulso_durante_habilitacion_secuencia_correcta(){
        initial_conection();
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        alarm_reset();
        relay_off();
        relay_on();
        powen_on();
        sleep(Duration::from_millis(WAIT_MILLIS));
        tnr_set(2375, 250, 4, 4, 0);
        sleep(Duration::from_millis(WAIT_TNR));
        //Ver que si esté la alarma
        let ret_val = enviar(0x3C000000)&0x4;
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        assert_ne!(ret_val, 0);
    }

    #[test]
    fn fresh_boot_tnr_invalido_pulso_durante_habilitacion_secuencia_invertida(){
        initial_conection();
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        alarm_reset();
        relay_off();
        relay_on();
        tnr_set(2375, 250, 4, 4, 0);
        sleep(Duration::from_millis(WAIT_TNR));
        powen_on();
        sleep(Duration::from_millis(WAIT_TNR));
        //Ver que si esté la alarma
        let ret_val = enviar(0x3C000000)&0x4;
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        assert_ne!(ret_val, 0);
    }

    #[test]
    fn fresh_boot_tnr_nunca_baja_secuencia_correcta(){
        initial_conection();
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        powen_on();
        sleep(Duration::from_millis(WAIT_MILLIS));
        relay_off();
        relay_on();
        alarm_reset();
        tnr_set(800, 800, 0, 0, 0);
        sleep(Duration::from_millis(WAIT_TNR));
        //Ver que si esté la alarma
        let ret_val = enviar(0x3C000000)&0x4;
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        assert_ne!(ret_val, 0);
    }

    #[test]
    fn fresh_boot_tnr_nunca_baja_secuencia_invertida(){
        initial_conection();
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        alarm_reset();
        tnr_set(800, 800, 0, 0, 0);
        sleep(Duration::from_millis(WAIT_TNR));
        relay_off();
        relay_on();
        powen_on();
        sleep(Duration::from_millis(WAIT_TNR));
        //Ver que si esté la alarma
        let ret_val = enviar(0x3C000000)&0x4;
        tnr_clear();
        sleep(Duration::from_millis(WAIT_CLEAR));
        assert_ne!(ret_val, 0);
    }
}

