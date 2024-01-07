#![no_std]
#![no_main]

use cortex_m_rtic::app;
use embedded_hal::digital::v2::OutputPin;
use nb::block;
use stm32f4::stm32f401;
use dht_sensor::dht11::DHT11;
use nrf24::NRF24;

#[app(device = stm32f401, peripherals = true)]
const APP: () = {
    struct Shared {
        dht_sensor: DHT11<stm32f401::GPIOA<stm32f401::RCC>, stm32f401::RCC>,
        nrf24: NRF24<stm32f401::SPI1, stm32f401::GPIOA<stm32f401::RCC>>,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        let peripherals = cx.device;

        let dht_sensor = DHT11::new(peripherals.GPIOA, peripherals.RCC);

        peripherals.RCC.apb2enr.modify(|_, w| w.spi1en().set_bit());
        peripherals.GPIOA.moder.modify(|_, w| w.moder5().bits(0b10).moder6().bits(0b10).moder7().bits(0b10));
        peripherals.GPIOA.afrl.modify(|_, w| w.afrl5().bits(5).afrl6().bits(5).afrl7().bits(5));
        peripherals.SPI1.cr1.write(|w| w.mstr().set_bit().br().div32().spe().set_bit());

        let nrf24 = NRF24::new(peripherals.SPI1, peripherals.GPIOA, 10).unwrap();

        init::LateResources { dht_sensor, nrf24 }
    }

    #[idle(resources = [dht_sensor, nrf24])]
    fn idle(cx: idle::Context) -> ! {
        let dht_sensor = cx.resources.dht_sensor;
        let nrf24 = cx.resources.nrf24;


        nrf24.init().unwrap();
        nrf24.set_rx_addr(0, b"SENSOR").unwrap();
        nrf24.set_tx_addr(b"SENSOR").unwrap();
        nrf24.set_channel(76).unwrap();
        nrf24.set_rf(NRF24::DataRate::R250Kbps, NRF24::TXPower::PAHigh).unwrap();
        nrf24.rx().unwrap();


        loop {

            let (temperature, humidity) = dht_sensor.read().unwrap();


            let data: [u8; 4] = [temperature as u8, (temperature >> 8) as u8, humidity as u8, (humidity >> 8) as u8];
            nrf24.send(&data).unwrap();


            

            for _ in 0..1_000_000 {
                cortex_m::asm::nop();
            }
        }
    }
};

