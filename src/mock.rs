
use std::{panic, println, vec};
use std::vec::Vec;
use std::sync::{Arc, Mutex};

use crate::{Transaction, Transactional};

use embedded_hal::blocking::spi;

/// Mock Transactional SPI implementation
pub struct Mock {
    inner: Arc<Mutex<Inner>>
}

/// Error type combining SPI and Pin errors for utility
#[derive(Debug, Clone, PartialEq)]
pub enum Error<SpiError, PinError> {
    Spi(SpiError),
    Pin(PinError),
    Aborted,
}

/// Mock transaction type for setting and checking expectations
#[derive(Clone, Debug, PartialEq)]
pub enum MockTransaction {
    None,
    SpiWrite((Vec<u8>, Vec<u8>)),
    SpiRead((Vec<u8>, Vec<u8>)),
    SpiExec(Vec<MockExec>),

    Write(Vec<u8>),
    Transfer((Vec<u8>, Vec<u8>)),
}

/// MockExec type for composing mock exec transactions
#[derive(Clone, Debug, PartialEq)]
pub enum MockExec {
    SpiWrite(Vec<u8>),
    SpiRead(Vec<u8>),
}

impl <'a> From<&Transaction<'a>> for MockExec {
    fn from(t: &Transaction<'a>) -> Self {
        match t {
            Transaction::Read(ref d) => {
                let mut v = Vec::with_capacity(d.len());
                v.copy_from_slice(d);
                MockExec::SpiRead(v)
            },
            Transaction::Write(ref d) => {
                let mut v = Vec::with_capacity(d.len());
                v.copy_from_slice(d);
                MockExec::SpiWrite(v)
            },
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
        Self{ inner: Arc::new(Mutex::new(Inner{ index: 0, expected: Vec::new(), actual: Vec::new() })) } 
    }

    /// Set expectations on the instance
    pub fn expect<T>(&mut self, transactions: T) 
    where 
        T: IntoIterator<Item=MockTransaction> 
    {
        let expected: Vec<_> = transactions.into_iter().map(|v| v.clone()).collect();
        let actual = vec![MockTransaction::None; expected.len()];

        let i = Inner{
            index: 0,
            expected,
            actual,
        };

        println!("i: {:?}", i);
        
        *self.inner.lock().unwrap() = i;
    }

    /// Finalise expectations
    /// This will cause previous expectations to be evaluated
    pub fn finalise(&self) {
        let mut i = self.inner.lock().unwrap();
        i.finalise();
    }
}

impl Transactional for Mock {
    type Error = Error<(), ()>;

    /// Read data from a specified address
    /// This consumes the provided input data array and returns a reference to this on success
    fn spi_read(&mut self, prefix: &[u8], data: &mut [u8]) -> Result<(), Self::Error> {
        let mut i = self.inner.lock().unwrap();
        let index = i.index;

        // Copy read data from expectation
        match &i.expected[index] {
            MockTransaction::SpiRead(expected) => data.copy_from_slice(&expected.1),
            _ => (),
        };
                       
        // Save actual call
        let actual = &mut i.actual[index];
        *actual = MockTransaction::SpiRead((prefix.into(), data.into()));
        
        // Update expectation index
        i.index += 1;

        Ok(())
    }

    /// Write data to a specified register address
    fn spi_write(&mut self, prefix: &[u8], data: &[u8]) -> Result<(), Self::Error> {
        let mut i = self.inner.lock().unwrap();
        let index = i.index;
        
        // Save actual call
        let actual = &mut i.actual[index];
        *actual = MockTransaction::SpiWrite((prefix.into(), data.into()));

        // TODO: Any expectation checking here..?
        let _expected = &i.expected[index];

        // Update expectation index
        i.index += 1;

        Ok(())
    }

    /// Execute the provided transactions
    fn spi_exec(&mut self, transactions: &mut [Transaction]) -> Result<(), Self::Error> {
        let mut i = self.inner.lock().unwrap();
        let index = i.index;

        // Save actual calls
        let actual = &mut i.actual[index];
        let t: Vec<MockExec> = transactions.iter().map(|ref v| MockExec::from(*v) ).collect();
        *actual = MockTransaction::SpiExec(t);

        // Load expected reads
        if let MockTransaction::SpiExec(e) = &i.expected[index] {
            for i in 0..transactions.len() {
                let t = &mut transactions[i];
                let x = e.get(i);

                match (t, x) {
                    (Transaction::Read(ref mut v), Some(MockExec::SpiRead(d))) => v.copy_from_slice(d),
                    _ => ()
                }
            }
        }
        
        // Update expectation index
        i.index += 1;

        Ok(())
    }
}


impl spi::Transfer<u8> for Mock 
{
    type Error = Error<(), ()>;

    fn transfer<'w>(&mut self, data: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        let mut i = self.inner.lock().unwrap();
        let index = i.index;

        let incoming: Vec<_> = data.into();

        // Copy read data from expectation
        match &i.expected[index] {
            MockTransaction::Transfer((_outgoing, incoming)) => {
                if incoming.len() == data.len() {
                    data.copy_from_slice(&incoming);
                }
            },
            _ => (),
        };
                       
        // Save actual call
        let actual = &mut i.actual[index];
        *actual = MockTransaction::Transfer((incoming, data.into()));
        
        // Update expectation index
        i.index += 1;

        Ok(data)
    }
}

impl spi::Write<u8> for Mock  
{
    type Error = Error<(), ()>;
    
    fn write<'w>(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        let mut i = self.inner.lock().unwrap();
        let index = i.index;
        
        // Save actual call
        let actual = &mut i.actual[index];
        *actual = MockTransaction::Write(data.into());

        // TODO: Any expectation checking here..?
        let _expected = &i.expected[index];

        // Update expectation index
        i.index += 1;

        Ok(())
    }
}


#[cfg(test)]
mod test {
    use std::*;
    use std::{vec, panic};

    use super::*;

    #[test]
    fn test_transactional_read() {
        let mut m = Mock::new();

        let prefix = vec![0xFF];
        let data = vec![0xAA, 0xBB];

        m.expect(vec![MockTransaction::SpiRead((prefix.clone(), data.clone()))]);

        let mut d = [0u8; 2];
        m.spi_read(&prefix, &mut d).expect("read failure");

        m.finalise();
        assert_eq!(&data, &d);
    }

    #[test]
    #[should_panic]
    fn test_transactional_read_expect_write() {
        let mut m = Mock::new();

        let prefix = vec![0xFF];
        let data = vec![0xAA, 0xBB];

        m.expect(vec![MockTransaction::SpiWrite((prefix.clone(), data.clone()))]);

        let mut d = [0u8; 2];
        m.spi_read(&prefix, &mut d).expect("read failure");

        m.finalise();
        assert_eq!(&data, &d);
    }

    #[test]
    fn test_transactional_write() {
        let mut m = Mock::new();

        let prefix = vec![0xFF];
        let data = vec![0xAA, 0xBB];

        m.expect(vec![MockTransaction::SpiWrite((prefix.clone(), data.clone()))]);

        m.spi_write(&prefix, &data).expect("write failure");

        m.finalise();
    }

    #[test]
    #[should_panic]
    fn test_transactional_write_expect_read() {
        let mut m = Mock::new();

        let prefix = vec![0xFF];
        let data = vec![0xAA, 0xBB];

        m.expect(vec![MockTransaction::SpiRead((prefix.clone(), data.clone()))]);

        m.spi_write(&prefix, &data).expect("write failure");

        m.finalise();
    }

    #[test]
    fn test_standard_write() {
        use embedded_hal::blocking::spi::Write;

        let mut m = Mock::new();

        let data = vec![0xAA, 0xBB];

        m.expect(vec![MockTransaction::Write(data.clone())]);

        m.write(&data).expect("write failure");

        m.finalise();
    }

    #[test]
    fn test_standard_transfer() {
        use embedded_hal::blocking::spi::Transfer;

        let mut m = Mock::new();

        let prefix = vec![0xFF];
        let outgoing = vec![0xAA, 0xBB];
        let incoming = vec![0xCC, 0xDD];

        m.expect(vec![MockTransaction::Transfer((outgoing.clone(), incoming.clone()))]);

        let mut d = outgoing.clone();
        m.transfer(&mut d).expect("read failure");

        m.finalise();
        assert_eq!(&incoming, &d);
    }

}