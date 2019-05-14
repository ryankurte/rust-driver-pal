
use std::{panic, println, vec};
use std::vec::Vec;
use std::sync::{Arc, Mutex};

use crate::{Transaction, Transactional};

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
    Write((Vec<u8>, Vec<u8>)),
    Read((Vec<u8>, Vec<u8>)),
    Exec(Vec<MockExec>),
}

/// MockExec type for composing mock exec transactions
#[derive(Clone, Debug, PartialEq)]
pub enum MockExec {
    Write(Vec<u8>),
    Read(Vec<u8>),
}

impl <'a> From<&Transaction<'a>> for MockExec {
    fn from(t: &Transaction<'a>) -> Self {
        match t {
            Transaction::Read(ref d) => {
                let mut v = Vec::with_capacity(d.len());
                v.copy_from_slice(d);
                MockExec::Read(v)
            },
            Transaction::Write(ref d) => {
                let mut v = Vec::with_capacity(d.len());
                v.copy_from_slice(d);
                MockExec::Write(v)
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
    fn read(&mut self, prefix: &[u8], data: &mut [u8]) -> Result<(), Self::Error> {
        let mut i = self.inner.lock().unwrap();
        let index = i.index;

        // Copy read data from expectation
        match &i.expected[index] {
            MockTransaction::Read(expected) => data.copy_from_slice(&expected.1),
            _ => (),
        };
                       
        // Save actual call
        let actual = &mut i.actual[index];
        *actual = MockTransaction::Read((prefix.into(), data.into()));
        
        // Update expectation index
        i.index += 1;

        Ok(())
    }

    /// Write data to a specified register address
    fn write(&mut self, prefix: &[u8], data: &[u8]) -> Result<(), Self::Error> {
        let mut i = self.inner.lock().unwrap();
        let index = i.index;
        
        // Save actual call
        let actual = &mut i.actual[index];
        *actual = MockTransaction::Write((prefix.into(), data.into()));

        // TODO: Any expectation checking here..?
        let _expected = &i.expected[index];

        // Update expectation index
        i.index += 1;

        Ok(())
    }

    /// Execute the provided transactions
    fn exec(&mut self, transactions: &mut [Transaction]) -> Result<(), Self::Error> {
        let mut i = self.inner.lock().unwrap();
        let index = i.index;

        // Save actual calls
        let actual = &mut i.actual[index];
        let t: Vec<MockExec> = transactions.iter().map(|ref v| MockExec::from(*v) ).collect();
        *actual = MockTransaction::Exec(t);

        // Load expected reads
        if let MockTransaction::Exec(e) = &i.expected[index] {
            for i in 0..transactions.len() {
                let t = &mut transactions[i];
                let x = e.get(i);

                match (t, x) {
                    (Transaction::Read(ref mut v), Some(MockExec::Read(d))) => v.copy_from_slice(d),
                    _ => ()
                }
            }
        }
        
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
    fn test_read() {
        let mut m = Mock::new();

        let prefix = vec![0xFF];
        let data = vec![0xAA, 0xBB];

        m.expect(vec![MockTransaction::Read((prefix.clone(), data.clone()))]);

        let mut d = [0u8; 2];
        m.read(&prefix, &mut d).expect("read failure");

        m.finalise();
        assert_eq!(&data, &d);
    }

    #[test]
    #[should_panic]
    fn test_read_expect_write() {
        let mut m = Mock::new();

        let prefix = vec![0xFF];
        let data = vec![0xAA, 0xBB];

        m.expect(vec![MockTransaction::Write((prefix.clone(), data.clone()))]);

        let mut d = [0u8; 2];
        m.read(&prefix, &mut d).expect("read failure");

        m.finalise();
        assert_eq!(&data, &d);
    }

    #[test]
    fn test_write() {
        let mut m = Mock::new();

        let prefix = vec![0xFF];
        let data = vec![0xAA, 0xBB];

        m.expect(vec![MockTransaction::Write((prefix.clone(), data.clone()))]);

        m.write(&prefix, &data).expect("write failure");

        m.finalise();
    }

    #[test]
    #[should_panic]
    fn test_write_expect_read() {
        let mut m = Mock::new();

        let prefix = vec![0xFF];
        let data = vec![0xAA, 0xBB];

        m.expect(vec![MockTransaction::Read((prefix.clone(), data.clone()))]);

        m.write(&prefix, &data).expect("write failure");

        m.finalise();
    }

}