use std::sync::{Arc, Mutex};
use std::vec::Vec;
use std::{panic, vec};

use crate::{Busy, Error, PinState, Ready, Reset};

use embedded_hal::spi::blocking::{Operation as SpiOperation};


/// Base mock type
pub struct Mock {
    inner: Arc<Mutex<Inner>>,
    count: Id,
}

pub type Id = u32;

/// Mock Transactional SPI implementation
#[derive(Clone, Debug)]
pub struct Spi {
    id: Id,
    inner: Arc<Mutex<Inner>>,
}

/// Mock Pin implementation
#[derive(Clone, Debug)]
pub struct Pin {
    id: Id,
    inner: Arc<Mutex<Inner>>,
}

/// Mock Delay implementation
#[derive(Clone, Debug)]
pub struct Delay {
    id: Id,
    inner: Arc<Mutex<Inner>>,
}

/// Mock transaction type for setting and checking expectations
#[derive(Clone, Debug, PartialEq)]
pub enum MockTransaction {
    None,

    SpiWrite(Id, Vec<u8>, Vec<u8>),
    SpiRead(Id, Vec<u8>, Vec<u8>),

    SpiExec(Id, Vec<MockExec>),

    Busy(Id, PinState),
    Ready(Id, PinState),
    Reset(Id, PinState),

    Write(Id, Vec<u8>),
    Transfer(Id, Vec<u8>, Vec<u8>),

    IsHigh(Id, bool),
    IsLow(Id, bool),
    SetHigh(Id),
    SetLow(Id),

    DelayMs(u32),
    DelayUs(u32),
}

impl MockTransaction {
    pub fn spi_write<A, B>(spi: &Spi, prefix: A, outgoing: B) -> Self
    where
        A: AsRef<[u8]>,
        B: AsRef<[u8]>,
    {
        MockTransaction::SpiWrite(spi.id, prefix.as_ref().to_vec(), outgoing.as_ref().to_vec())
    }

    pub fn spi_read<A, B>(spi: &Spi, prefix: A, incoming: B) -> Self
    where
        A: AsRef<[u8]>,
        B: AsRef<[u8]>,
    {
        MockTransaction::SpiRead(spi.id, prefix.as_ref().to_vec(), incoming.as_ref().to_vec())
    }

    pub fn spi_exec<O>(spi: &Spi, o: O) -> Self
    where
        O: AsRef<[MockExec]>,
    {
        MockTransaction::SpiExec(spi.id, o.as_ref().to_vec())
    }

    pub fn busy(spi: &Spi, value: PinState) -> Self {
        MockTransaction::Busy(spi.id, value)
    }

    pub fn ready(spi: &Spi, value: PinState) -> Self {
        MockTransaction::Ready(spi.id, value)
    }

    pub fn reset(spi: &Spi, value: PinState) -> Self {
        MockTransaction::Reset(spi.id, value)
    }

    pub fn delay_ms(v: u32) -> Self {
        MockTransaction::DelayMs(v)
    }

    pub fn write<B>(spi: &Spi, outgoing: B) -> Self
    where
        B: AsRef<[u8]>,
    {
        MockTransaction::Write(spi.id, outgoing.as_ref().to_vec())
    }

    pub fn transfer<B>(spi: &Spi, outgoing: B, incoming: B) -> Self
    where
        B: AsRef<[u8]>,
    {
        MockTransaction::Transfer(
            spi.id,
            outgoing.as_ref().to_vec(),
            incoming.as_ref().to_vec(),
        )
    }

    pub fn is_high(pin: &Pin, value: bool) -> Self {
        MockTransaction::IsHigh(pin.id, value)
    }

    pub fn is_low(pin: &Pin, value: bool) -> Self {
        MockTransaction::IsLow(pin.id, value)
    }

    pub fn set_high(pin: &Pin) -> Self {
        MockTransaction::SetHigh(pin.id)
    }

    pub fn set_low(pin: &Pin) -> Self {
        MockTransaction::SetLow(pin.id)
    }
}

/// MockExec type for composing mock exec transactions
#[derive(Clone, Debug, PartialEq)]
pub enum MockExec {
    SpiWrite(Vec<u8>),
    SpiTransfer(Vec<u8>, Vec<u8>),
}

impl<'a> From<&SpiOperation<'a, u8>> for MockExec {
    fn from(t: &SpiOperation<'a, u8>) -> Self {
        match t {
            SpiOperation::Write(ref d) => MockExec::SpiWrite(d.to_vec()),
            SpiOperation::Transfer(ref d) => {
                MockExec::SpiTransfer(d.to_vec(), vec![0u8; d.len()])
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Inner {
    index: usize,
    expected: Vec<MockTransaction>,
    actual: Vec<MockTransaction>,
}

impl Inner {
    fn finalise(&mut self) {
        assert_eq!(self.expected, self.actual);
    }
}

impl Mock {
    /// Create a new mock instance
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                index: 0,
                expected: Vec::new(),
                actual: Vec::new(),
            })),
            count: 0,
        }
    }

    /// Set expectations on the instance
    pub fn expect<T>(&mut self, transactions: T)
    where
        T: AsRef<[MockTransaction]>,
    {
        let expected: Vec<_> = transactions.as_ref().to_vec();
        let actual = vec![];

        let i = Inner {
            index: 0,
            expected,
            actual,
        };

        *self.inner.lock().unwrap() = i;
    }

    pub fn spi(&mut self) -> Spi {
        let id = self.count;
        self.count += 1;
        Spi {
            inner: self.inner.clone(),
            id,
        }
    }

    pub fn pin(&mut self) -> Pin {
        let id = self.count;
        self.count += 1;
        Pin {
            inner: self.inner.clone(),
            id,
        }
    }

    pub fn delay(&mut self) -> Delay {
        let id = self.count;
        self.count += 1;
        Delay {
            inner: self.inner.clone(),
            id,
        }
    }

    /// Finalise expectations
    /// This will cause previous expectations to be evaluated
    pub fn finalise(&self) {
        let mut i = self.inner.lock().unwrap();
        i.finalise();
    }
}

impl Busy for Spi {
    type Error = Error<(), (), ()>;

    /// Check peripheral busy status
    fn get_busy(&mut self) -> Result<PinState, Self::Error> {
        let mut i = self.inner.lock().unwrap();
        let index = i.index;

        let state = match &i.expected.get(index) {
            Some(MockTransaction::Busy(_id, state)) => state.clone(),
            _ => PinState::Low,
        };

        i.actual.push(MockTransaction::Busy(self.id, state.clone()));

        i.index += 1;

        Ok(state)
    }
}

impl Ready for Spi {
    type Error = Error<(), (), ()>;

    /// Check peripheral ready status
    fn get_ready(&mut self) -> Result<PinState, Self::Error> {
        let mut i = self.inner.lock().unwrap();
        let index = i.index;

        let state = match &i.expected.get(index) {
            Some(MockTransaction::Ready(_id, state)) => state.clone(),
            _ => PinState::Low,
        };

        i.actual
            .push(MockTransaction::Ready(self.id, state.clone()));

        i.index += 1;

        Ok(state)
    }
}

impl Reset for Spi {
    type Error = Error<(), (), ()>;

    /// Check peripheral ready status
    fn set_reset(&mut self, state: PinState) -> Result<(), Self::Error> {
        let mut i = self.inner.lock().unwrap();

        i.actual.push(MockTransaction::Reset(self.id, state));

        i.index += 1;

        Ok(())
    }
}

impl embedded_hal::delay::blocking::DelayMs<u32> for Spi {
    type Error = ();

    fn delay_ms(&mut self, t: u32) -> Result<(), Self::Error> {
        let mut i = self.inner.lock().unwrap();

        // Save actual call
        i.actual.push(MockTransaction::DelayMs(t));

        // Update expectation index
        i.index += 1;

        Ok(())
    }
}

impl embedded_hal::delay::blocking::DelayUs<u32> for Spi {
    type Error = ();

    fn delay_us(&mut self, t: u32) -> Result<(), Self::Error> {
        let mut i = self.inner.lock().unwrap();

        // Save actual call
        i.actual.push(MockTransaction::DelayUs(t));

        // Update expectation index
        i.index += 1;

        Ok(())
    }
}

impl embedded_hal::spi::blocking::Transfer<u8> for Spi {
    type Error = Error<(), (), ()>;

    fn transfer<'w>(&mut self, data: &'w mut [u8]) -> Result<(), Self::Error> {
        let mut i = self.inner.lock().unwrap();
        let index = i.index;

        let incoming: Vec<_> = data.into();

        // Copy read data from expectation
        match &i.expected.get(index) {
            Some(MockTransaction::Transfer(_id, _outgoing, incoming)) => {
                if incoming.len() == data.len() {
                    data.copy_from_slice(&incoming);
                }
            }
            _ => (),
        };

        // Save actual call
        i.actual
            .push(MockTransaction::Transfer(self.id, incoming, data.into()));

        // Update expectation index
        i.index += 1;

        Ok(())
    }
}

impl embedded_hal::spi::blocking::Write<u8> for Spi {
    type Error = Error<(), (), ()>;

    fn write<'w>(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        let mut i = self.inner.lock().unwrap();

        // Save actual call
        i.actual.push(MockTransaction::Write(self.id, data.into()));

        // Update expectation index
        i.index += 1;

        Ok(())
    }
}

impl embedded_hal::spi::blocking::Transactional<u8> for Spi {
    type Error = Error<(), (), ()>;

    fn exec<'a>(
        &mut self,
        operations: &mut [SpiOperation<'a, u8>],
    ) -> Result<(), Self::Error> {
        let mut i = self.inner.lock().unwrap();
        let index = i.index;

        // Save actual calls
        let t: Vec<MockExec> = operations
            .as_mut()
            .iter()
            .map(|ref v| MockExec::from(*v))
            .collect();
        i.actual.push(MockTransaction::SpiExec(self.id, t));

        let transactions = operations.as_mut();

        // Load expected reads
        if let MockTransaction::SpiExec(_id, e) = &i.expected[index] {
            for i in 0..transactions.len() {
                let t = &mut transactions[i];
                let x = e.get(i);

                match (t, x) {
                    (
                        SpiOperation::Transfer(ref mut t_in),
                        Some(MockExec::SpiTransfer(_x_out, x_in)),
                    ) => t_in.copy_from_slice(&x_in),
                    (SpiOperation::Write(ref _t_out), Some(MockExec::SpiWrite(ref _x_out))) => {
                        //assert_eq!(t_out, x_out);
                    }
                    _ => (),
                }
            }
        }

        // Update expectation index
        i.index += 1;

        Ok(())
    }
}

impl embedded_hal::digital::blocking::InputPin for Pin {
    type Error = ();

    fn is_high(&self) -> Result<bool, Self::Error> {
        let mut i = self.inner.lock().unwrap();
        let index = i.index;

        // Fetch expectation if found
        let v = match &i.expected.get(index) {
            Some(MockTransaction::IsHigh(_id, v)) => *v,
            _ => false,
        };

        // Save actual call
        i.actual.push(MockTransaction::IsHigh(self.id, v));

        // Update expectation index
        i.index += 1;

        Ok(v)
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        let mut i = self.inner.lock().unwrap();
        let index = i.index;

        // Fetch expectation if found
        let v = match &i.expected.get(index) {
            Some(MockTransaction::IsLow(_id, v)) => *v,
            _ => false,
        };

        // Save actual call
        i.actual.push(MockTransaction::IsLow(self.id, v));

        // Update expectation index
        i.index += 1;

        Ok(v)
    }
}

impl embedded_hal::digital::blocking::OutputPin for Pin {
    type Error = ();

    fn set_high(&mut self) -> Result<(), Self::Error> {
        let mut i = self.inner.lock().unwrap();

        // Save actual call
        i.actual.push(MockTransaction::SetHigh(self.id));

        // Update expectation index
        i.index += 1;

        Ok(())
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        let mut i = self.inner.lock().unwrap();

        // Save actual call
        i.actual.push(MockTransaction::SetLow(self.id));

        // Update expectation index
        i.index += 1;

        Ok(())
    }
}

impl embedded_hal::delay::blocking::DelayMs<u32> for Delay {
    type Error = ();

    fn delay_ms(&mut self, t: u32) -> Result<(), Self::Error> {
        let mut i = self.inner.lock().unwrap();

        // Save actual call
        i.actual.push(MockTransaction::DelayMs(t));

        // Update expectation index
        i.index += 1;

        Ok(())
    }
}

impl embedded_hal::delay::blocking::DelayUs<u32> for Delay {
    type Error = ();

    fn delay_us(&mut self, t: u32) -> Result<(), Self::Error> {
        let mut i = self.inner.lock().unwrap();

        // Save actual call
        i.actual.push(MockTransaction::DelayUs(t));

        // Update expectation index
        i.index += 1;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::*;
    use std::{panic, vec};

    use embedded_hal::spi::blocking::*;
    use embedded_hal::digital::blocking::*;
    use embedded_hal::delay::blocking::*;

    use super::*;
    use crate::{PrefixRead, PrefixWrite};

    // TODO: needs fixing
    #[test]
    #[ignore]
    fn test_transactional_read() {
        let mut m = Mock::new();
        let mut s = m.spi();

        let prefix = vec![0xFF];
        let data = vec![0xAA, 0xBB];

        m.expect(vec![MockTransaction::spi_exec(
            &s,
            &[
                MockExec::SpiWrite(prefix.clone()),
                MockExec::SpiTransfer(vec![0u8; 2], data.clone()),
            ],
        )]);

        let mut d = [0u8; 2];
        s.prefix_read(&prefix, &mut d).expect("read failure");

        m.finalise();
        assert_eq!(&data, &d);
    }

    #[test]
    #[should_panic]
    fn test_transactional_read_expect_write() {
        let mut m = Mock::new();
        let mut s = m.spi();

        let prefix = vec![0xFF];
        let data = vec![0xAA, 0xBB];

        m.expect(vec![MockTransaction::spi_write(
            &s,
            prefix.clone(),
            data.clone(),
        )]);

        let mut d = [0u8; 2];
        s.prefix_read(&prefix, &mut d).expect("read failure");

        m.finalise();
        assert_eq!(&data, &d);
    }

    // TODO: needs fixing
    #[test]
    #[ignore]
    fn test_transactional_write() {
        let mut m = Mock::new();
        let mut s = m.spi();

        let prefix = vec![0xFF];
        let data = vec![0xAA, 0xBB];

        m.expect(vec![MockTransaction::spi_write(
            &s,
            prefix.clone(),
            data.clone(),
        )]);

        s.prefix_write(&prefix, &data).expect("write failure");

        m.finalise();
    }

    #[test]
    #[should_panic]
    fn test_transactional_write_expect_read() {
        let mut m = Mock::new();
        let mut s = m.spi();

        let prefix = vec![0xFF];
        let data = vec![0xAA, 0xBB];

        m.expect(vec![MockTransaction::spi_read(
            &s,
            prefix.clone(),
            data.clone(),
        )]);

        s.prefix_write(&prefix, &data).expect("write failure");

        m.finalise();
    }

    #[test]
    fn test_standard_write() {
        let mut m = Mock::new();
        let mut s = m.spi();

        let data = vec![0xAA, 0xBB];

        m.expect(vec![MockTransaction::write(&s, data.clone())]);

        s.write(&data).expect("write failure");

        m.finalise();
    }

    #[test]
    fn test_standard_transfer() {
        let mut m = Mock::new();
        let mut s = m.spi();

        let outgoing = vec![0xAA, 0xBB];
        let incoming = vec![0xCC, 0xDD];

        m.expect(vec![MockTransaction::transfer(
            &s,
            outgoing.clone(),
            incoming.clone(),
        )]);

        let mut d = outgoing.clone();
        s.transfer(&mut d).expect("read failure");

        m.finalise();
        assert_eq!(&incoming, &d);
    }

    #[test]
    fn test_pins() {
        let mut m = Mock::new();
        let mut p = m.pin();

        m.expect(vec![
            MockTransaction::is_high(&p, true),
            MockTransaction::is_low(&p, false),
            MockTransaction::set_high(&p),
            MockTransaction::set_low(&p),
        ]);

        assert_eq!(true, p.is_high().unwrap());
        assert_eq!(false, p.is_low().unwrap());

        p.set_high().unwrap();
        p.set_low().unwrap();

        m.finalise();
    }

    #[test]
    #[should_panic]
    fn test_incorrect_pin() {
        let mut m = Mock::new();
        let p1 = m.pin();
        let p2 = m.pin();

        m.expect(vec![MockTransaction::is_high(&p1, true)]);

        p2.is_high().unwrap();

        m.finalise();
    }
}
