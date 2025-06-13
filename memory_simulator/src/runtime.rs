use num_bigint::{BigInt, BigUint, ToBigInt, ToBigUint};
use num_traits::Num;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::fs::read_to_string;

pub trait Cycled {
    fn cycle(&self) -> usize;
    fn pusher(&self) -> &'static str;
}

pub struct ArrayWrite<T: Sized + Default + Clone> {
    cycle: usize,
    addr: usize,
    data: T,
    pusher: &'static str,
}

impl<T: Sized + Default + Clone> ArrayWrite<T> {
    pub fn new(cycle: usize, addr: usize, data: T, pusher: &'static str) -> Self {
        ArrayWrite {
            cycle,
            addr,
            data,
            pusher,
        }
    }
}

pub struct Array<T: Sized + Default + Clone> {
    pub payload: Vec<T>,
    pub write: XEQ<ArrayWrite<T>>,
}

impl<T: Sized + Default + Clone> Array<T> {
    pub fn new(n: usize) -> Self {
        Array {
            payload: vec![T::default(); n],
            write: XEQ::new(),
        }
    }
    pub fn new_with_init(payload: Vec<T>) -> Self {
        Array {
            payload,
            write: XEQ::new(),
        }
    }
    pub fn tick(&mut self, cycle: usize) {
        if let Some(event) = self.write.pop(cycle) {
            self.payload[event.addr] = event.data;
        }
    }
}

pub struct FIFOPush<T: Sized> {
    cycle: usize,
    data: T,
    pusher: &'static str,
}

impl<T: Sized> FIFOPush<T> {
    pub fn new(cycle: usize, data: T, pusher: &'static str) -> Self {
        FIFOPush {
            cycle,
            data,
            pusher,
        }
    }
}

pub struct FIFOPop {
    cycle: usize,
    pusher: &'static str,
}

impl FIFOPop {
    pub fn new(cycle: usize, pusher: &'static str) -> Self {
        FIFOPop { cycle, pusher }
    }
}

pub struct FIFO<T: Sized> {
    pub payload: VecDeque<T>,
    pub push: XEQ<FIFOPush<T>>,
    pub pop: XEQ<FIFOPop>,
}

impl<T: Sized> FIFO<T> {
    pub fn new() -> Self {
        FIFO {
            payload: VecDeque::new(),
            push: XEQ::new(),
            pop: XEQ::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.payload.is_empty()
    }

    pub fn front(&self) -> Option<&T> {
        self.payload.front()
    }

    pub fn tick(&mut self, cycle: usize) {
        // it assume the FIFO is not empty
        if let Some(_) = self.pop.pop(cycle) {
            println!("Pop pop from rdata.");
            if !self.is_empty(){
                // looks like it never runs
                self.payload.pop_front().unwrap();
            }
        }
        if let Some(event) = self.push.pop(cycle) {
            self.payload.push_back(event.data);
        }
    }
}

impl<T: Sized + Default + Clone> Cycled for ArrayWrite<T> {
    fn cycle(&self) -> usize {
        self.cycle
    }
    fn pusher(&self) -> &'static str {
        self.pusher
    }
}

impl<T: Sized> Cycled for FIFOPush<T> {
    fn cycle(&self) -> usize {
        self.cycle
    }
    fn pusher(&self) -> &'static str {
        self.pusher
    }
}

impl Cycled for FIFOPop {
    fn cycle(&self) -> usize {
        self.cycle
    }
    fn pusher(&self) -> &'static str {
        self.pusher
    }
}

pub struct XEQ<T: Sized + Cycled> {
    pub q: BTreeMap<usize, T>,
}

impl<T: Sized + Cycled> XEQ<T> {
    pub fn new() -> Self {
        XEQ { q: BTreeMap::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.q.is_empty()
    }

    pub fn push(&mut self, event: T) {
        if let Some(a) = self.q.get(&event.cycle()) {
            panic!(
                "{}: Already occupied by {}, cannot accept {}!",
                cyclize(a.cycle()),
                a.pusher(),
                event.pusher()
            );
        } else {
            self.q.insert(event.cycle(), event);
        }
    }

    pub fn pop(&mut self, current: usize) -> Option<T> {
        if self
            .q
            .first_key_value()
            .map_or(false, |(cycle, _)| *cycle <= current)
        {
            self.q.pop_first().map(|(_, event)| event)
        } else {
            None
        }
    }
}

pub fn cyclize(stamp: usize) -> String {
    format!("Cycle @{}.{:02}", stamp / 100, stamp % 100)
}

pub fn load_hex_file<T: Num>(array: &mut Vec<T>, init_file: &str) {
    let mut idx = 0;
    for line in read_to_string(init_file)
        .expect("can not open hex file")
        .lines()
    {
        let line = if let Some(to_strip) = line.find("//") {
            line[..to_strip].trim()
        } else {
            line.trim()
        };
        if line.len() == 0 {
            continue;
        }
        let line = line.replace("_", "");
        if line.starts_with("@") {
            let addr = usize::from_str_radix(&line[1..], 16).unwrap();
            idx = addr;
            continue;
        }
        array[idx] = T::from_str_radix(line.as_str(), 16).ok().unwrap();
        idx += 1;
    }
}

pub trait ValueCastTo<T> {
    fn cast(&self) -> T;
}
impl ValueCastTo<bool> for bool {
    fn cast(&self) -> bool {
        self.clone()
    }
}
impl ValueCastTo<BigInt> for BigInt {
    fn cast(&self) -> BigInt {
        self.clone()
    }
}
impl ValueCastTo<BigUint> for BigInt {
    fn cast(&self) -> BigUint {
        self.to_biguint().unwrap()
    }
}
impl ValueCastTo<BigInt> for bool {
    fn cast(&self) -> BigInt {
        if *self {
            1.to_bigint().unwrap()
        } else {
            0.to_bigint().unwrap()
        }
    }
}
impl ValueCastTo<bool> for BigInt {
    fn cast(&self) -> bool {
        !self.eq(&0.to_bigint().unwrap())
    }
}
impl ValueCastTo<BigUint> for BigUint {
    fn cast(&self) -> BigUint {
        self.clone()
    }
}
impl ValueCastTo<BigInt> for BigUint {
    fn cast(&self) -> BigInt {
        self.to_bigint().unwrap()
    }
}
impl ValueCastTo<BigUint> for bool {
    fn cast(&self) -> BigUint {
        if *self {
            1.to_biguint().unwrap()
        } else {
            0.to_biguint().unwrap()
        }
    }
}
impl ValueCastTo<bool> for BigUint {
    fn cast(&self) -> bool {
        !self.eq(&0.to_biguint().unwrap())
    }
}
impl ValueCastTo<bool> for u8 {
    fn cast(&self) -> bool {
        *self != 0
    }
}
impl ValueCastTo<u8> for bool {
    fn cast(&self) -> u8 {
        if *self {
            1
        } else {
            0
        }
    }
}
impl ValueCastTo<BigInt> for u8 {
    fn cast(&self) -> BigInt {
        self.to_bigint().unwrap()
    }
}
impl ValueCastTo<BigUint> for u8 {
    fn cast(&self) -> BigUint {
        self.to_biguint().unwrap()
    }
}
impl ValueCastTo<u8> for BigInt {
    fn cast(&self) -> u8 {
        let (sign, data) = self.to_u64_digits();
        if data.is_empty() {
            return 0;
        }
        match sign {
            num_bigint::Sign::Plus => data[0] as u8,
            num_bigint::Sign::Minus => ((!data[0] + 1) & (u8::MAX as u64)) as u8,
            num_bigint::Sign::NoSign => data[0] as u8,
        }
    }
}
impl ValueCastTo<u8> for BigUint {
    fn cast(&self) -> u8 {
        let data = self.to_u64_digits();
        if data.is_empty() {
            return 0;
        } else {
            return data[0] as u8;
        }
    }
}
impl ValueCastTo<u8> for u8 {
    fn cast(&self) -> u8 {
        self.clone()
    }
}
impl ValueCastTo<u16> for u8 {
    fn cast(&self) -> u16 {
        *self as u16
    }
}
impl ValueCastTo<u32> for u8 {
    fn cast(&self) -> u32 {
        *self as u32
    }
}
impl ValueCastTo<u64> for u8 {
    fn cast(&self) -> u64 {
        *self as u64
    }
}
impl ValueCastTo<i8> for u8 {
    fn cast(&self) -> i8 {
        *self as i8
    }
}
impl ValueCastTo<i16> for u8 {
    fn cast(&self) -> i16 {
        *self as i16
    }
}
impl ValueCastTo<i32> for u8 {
    fn cast(&self) -> i32 {
        *self as i32
    }
}
impl ValueCastTo<i64> for u8 {
    fn cast(&self) -> i64 {
        *self as i64
    }
}
impl ValueCastTo<bool> for u16 {
    fn cast(&self) -> bool {
        *self != 0
    }
}
impl ValueCastTo<u16> for bool {
    fn cast(&self) -> u16 {
        if *self {
            1
        } else {
            0
        }
    }
}
impl ValueCastTo<BigInt> for u16 {
    fn cast(&self) -> BigInt {
        self.to_bigint().unwrap()
    }
}
impl ValueCastTo<BigUint> for u16 {
    fn cast(&self) -> BigUint {
        self.to_biguint().unwrap()
    }
}
impl ValueCastTo<u16> for BigInt {
    fn cast(&self) -> u16 {
        let (sign, data) = self.to_u64_digits();
        if data.is_empty() {
            return 0;
        }
        match sign {
            num_bigint::Sign::Plus => data[0] as u16,
            num_bigint::Sign::Minus => ((!data[0] + 1) & (u16::MAX as u64)) as u16,
            num_bigint::Sign::NoSign => data[0] as u16,
        }
    }
}
impl ValueCastTo<u16> for BigUint {
    fn cast(&self) -> u16 {
        let data = self.to_u64_digits();
        if data.is_empty() {
            return 0;
        } else {
            return data[0] as u16;
        }
    }
}
impl ValueCastTo<u8> for u16 {
    fn cast(&self) -> u8 {
        *self as u8
    }
}
impl ValueCastTo<u16> for u16 {
    fn cast(&self) -> u16 {
        self.clone()
    }
}
impl ValueCastTo<u32> for u16 {
    fn cast(&self) -> u32 {
        *self as u32
    }
}
impl ValueCastTo<u64> for u16 {
    fn cast(&self) -> u64 {
        *self as u64
    }
}
impl ValueCastTo<i8> for u16 {
    fn cast(&self) -> i8 {
        *self as i8
    }
}
impl ValueCastTo<i16> for u16 {
    fn cast(&self) -> i16 {
        *self as i16
    }
}
impl ValueCastTo<i32> for u16 {
    fn cast(&self) -> i32 {
        *self as i32
    }
}
impl ValueCastTo<i64> for u16 {
    fn cast(&self) -> i64 {
        *self as i64
    }
}
impl ValueCastTo<bool> for u32 {
    fn cast(&self) -> bool {
        *self != 0
    }
}
impl ValueCastTo<u32> for bool {
    fn cast(&self) -> u32 {
        if *self {
            1
        } else {
            0
        }
    }
}
impl ValueCastTo<BigInt> for u32 {
    fn cast(&self) -> BigInt {
        self.to_bigint().unwrap()
    }
}
impl ValueCastTo<BigUint> for u32 {
    fn cast(&self) -> BigUint {
        self.to_biguint().unwrap()
    }
}
impl ValueCastTo<u32> for BigInt {
    fn cast(&self) -> u32 {
        let (sign, data) = self.to_u64_digits();
        if data.is_empty() {
            return 0;
        }
        match sign {
            num_bigint::Sign::Plus => data[0] as u32,
            num_bigint::Sign::Minus => ((!data[0] + 1) & (u32::MAX as u64)) as u32,
            num_bigint::Sign::NoSign => data[0] as u32,
        }
    }
}
impl ValueCastTo<u32> for BigUint {
    fn cast(&self) -> u32 {
        let data = self.to_u64_digits();
        if data.is_empty() {
            return 0;
        } else {
            return data[0] as u32;
        }
    }
}
impl ValueCastTo<u8> for u32 {
    fn cast(&self) -> u8 {
        *self as u8
    }
}
impl ValueCastTo<u16> for u32 {
    fn cast(&self) -> u16 {
        *self as u16
    }
}
impl ValueCastTo<u32> for u32 {
    fn cast(&self) -> u32 {
        self.clone()
    }
}
impl ValueCastTo<u64> for u32 {
    fn cast(&self) -> u64 {
        *self as u64
    }
}
impl ValueCastTo<i8> for u32 {
    fn cast(&self) -> i8 {
        *self as i8
    }
}
impl ValueCastTo<i16> for u32 {
    fn cast(&self) -> i16 {
        *self as i16
    }
}
impl ValueCastTo<i32> for u32 {
    fn cast(&self) -> i32 {
        *self as i32
    }
}
impl ValueCastTo<i64> for u32 {
    fn cast(&self) -> i64 {
        *self as i64
    }
}
impl ValueCastTo<bool> for u64 {
    fn cast(&self) -> bool {
        *self != 0
    }
}
impl ValueCastTo<u64> for bool {
    fn cast(&self) -> u64 {
        if *self {
            1
        } else {
            0
        }
    }
}
impl ValueCastTo<BigInt> for u64 {
    fn cast(&self) -> BigInt {
        self.to_bigint().unwrap()
    }
}
impl ValueCastTo<BigUint> for u64 {
    fn cast(&self) -> BigUint {
        self.to_biguint().unwrap()
    }
}
impl ValueCastTo<u64> for BigInt {
    fn cast(&self) -> u64 {
        let (sign, data) = self.to_u64_digits();
        if data.is_empty() {
            return 0;
        }
        match sign {
            num_bigint::Sign::Plus => data[0] as u64,
            num_bigint::Sign::Minus => ((!data[0] + 1) & (u64::MAX as u64)) as u64,
            num_bigint::Sign::NoSign => data[0] as u64,
        }
    }
}
impl ValueCastTo<u64> for BigUint {
    fn cast(&self) -> u64 {
        let data = self.to_u64_digits();
        if data.is_empty() {
            return 0;
        } else {
            return data[0] as u64;
        }
    }
}
impl ValueCastTo<u8> for u64 {
    fn cast(&self) -> u8 {
        *self as u8
    }
}
impl ValueCastTo<u16> for u64 {
    fn cast(&self) -> u16 {
        *self as u16
    }
}
impl ValueCastTo<u32> for u64 {
    fn cast(&self) -> u32 {
        *self as u32
    }
}
impl ValueCastTo<u64> for u64 {
    fn cast(&self) -> u64 {
        self.clone()
    }
}
impl ValueCastTo<i8> for u64 {
    fn cast(&self) -> i8 {
        *self as i8
    }
}
impl ValueCastTo<i16> for u64 {
    fn cast(&self) -> i16 {
        *self as i16
    }
}
impl ValueCastTo<i32> for u64 {
    fn cast(&self) -> i32 {
        *self as i32
    }
}
impl ValueCastTo<i64> for u64 {
    fn cast(&self) -> i64 {
        *self as i64
    }
}
impl ValueCastTo<bool> for i8 {
    fn cast(&self) -> bool {
        *self != 0
    }
}
impl ValueCastTo<i8> for bool {
    fn cast(&self) -> i8 {
        if *self {
            1
        } else {
            0
        }
    }
}
impl ValueCastTo<BigInt> for i8 {
    fn cast(&self) -> BigInt {
        self.to_bigint().unwrap()
    }
}
impl ValueCastTo<BigUint> for i8 {
    fn cast(&self) -> BigUint {
        self.to_biguint().unwrap()
    }
}
impl ValueCastTo<i8> for BigInt {
    fn cast(&self) -> i8 {
        let (sign, data) = self.to_u64_digits();
        if data.is_empty() {
            return 0;
        }
        match sign {
            num_bigint::Sign::Plus => data[0] as i8,
            num_bigint::Sign::Minus => ((!data[0] + 1) & (i8::MAX as u64)) as i8,
            num_bigint::Sign::NoSign => data[0] as i8,
        }
    }
}
impl ValueCastTo<i8> for BigUint {
    fn cast(&self) -> i8 {
        let data = self.to_u64_digits();
        if data.is_empty() {
            return 0;
        } else {
            return data[0] as i8;
        }
    }
}
impl ValueCastTo<u8> for i8 {
    fn cast(&self) -> u8 {
        *self as u8
    }
}
impl ValueCastTo<u16> for i8 {
    fn cast(&self) -> u16 {
        *self as u16
    }
}
impl ValueCastTo<u32> for i8 {
    fn cast(&self) -> u32 {
        *self as u32
    }
}
impl ValueCastTo<u64> for i8 {
    fn cast(&self) -> u64 {
        *self as u64
    }
}
impl ValueCastTo<i8> for i8 {
    fn cast(&self) -> i8 {
        self.clone()
    }
}
impl ValueCastTo<i16> for i8 {
    fn cast(&self) -> i16 {
        *self as i16
    }
}
impl ValueCastTo<i32> for i8 {
    fn cast(&self) -> i32 {
        *self as i32
    }
}
impl ValueCastTo<i64> for i8 {
    fn cast(&self) -> i64 {
        *self as i64
    }
}
impl ValueCastTo<bool> for i16 {
    fn cast(&self) -> bool {
        *self != 0
    }
}
impl ValueCastTo<i16> for bool {
    fn cast(&self) -> i16 {
        if *self {
            1
        } else {
            0
        }
    }
}
impl ValueCastTo<BigInt> for i16 {
    fn cast(&self) -> BigInt {
        self.to_bigint().unwrap()
    }
}
impl ValueCastTo<BigUint> for i16 {
    fn cast(&self) -> BigUint {
        self.to_biguint().unwrap()
    }
}
impl ValueCastTo<i16> for BigInt {
    fn cast(&self) -> i16 {
        let (sign, data) = self.to_u64_digits();
        if data.is_empty() {
            return 0;
        }
        match sign {
            num_bigint::Sign::Plus => data[0] as i16,
            num_bigint::Sign::Minus => ((!data[0] + 1) & (i16::MAX as u64)) as i16,
            num_bigint::Sign::NoSign => data[0] as i16,
        }
    }
}
impl ValueCastTo<i16> for BigUint {
    fn cast(&self) -> i16 {
        let data = self.to_u64_digits();
        if data.is_empty() {
            return 0;
        } else {
            return data[0] as i16;
        }
    }
}
impl ValueCastTo<u8> for i16 {
    fn cast(&self) -> u8 {
        *self as u8
    }
}
impl ValueCastTo<u16> for i16 {
    fn cast(&self) -> u16 {
        *self as u16
    }
}
impl ValueCastTo<u32> for i16 {
    fn cast(&self) -> u32 {
        *self as u32
    }
}
impl ValueCastTo<u64> for i16 {
    fn cast(&self) -> u64 {
        *self as u64
    }
}
impl ValueCastTo<i8> for i16 {
    fn cast(&self) -> i8 {
        *self as i8
    }
}
impl ValueCastTo<i16> for i16 {
    fn cast(&self) -> i16 {
        self.clone()
    }
}
impl ValueCastTo<i32> for i16 {
    fn cast(&self) -> i32 {
        *self as i32
    }
}
impl ValueCastTo<i64> for i16 {
    fn cast(&self) -> i64 {
        *self as i64
    }
}
impl ValueCastTo<bool> for i32 {
    fn cast(&self) -> bool {
        *self != 0
    }
}
impl ValueCastTo<i32> for bool {
    fn cast(&self) -> i32 {
        if *self {
            1
        } else {
            0
        }
    }
}
impl ValueCastTo<BigInt> for i32 {
    fn cast(&self) -> BigInt {
        self.to_bigint().unwrap()
    }
}
impl ValueCastTo<BigUint> for i32 {
    fn cast(&self) -> BigUint {
        self.to_biguint().unwrap()
    }
}
impl ValueCastTo<i32> for BigInt {
    fn cast(&self) -> i32 {
        let (sign, data) = self.to_u64_digits();
        if data.is_empty() {
            return 0;
        }
        match sign {
            num_bigint::Sign::Plus => data[0] as i32,
            num_bigint::Sign::Minus => ((!data[0] + 1) & (i32::MAX as u64)) as i32,
            num_bigint::Sign::NoSign => data[0] as i32,
        }
    }
}
impl ValueCastTo<i32> for BigUint {
    fn cast(&self) -> i32 {
        let data = self.to_u64_digits();
        if data.is_empty() {
            return 0;
        } else {
            return data[0] as i32;
        }
    }
}
impl ValueCastTo<u8> for i32 {
    fn cast(&self) -> u8 {
        *self as u8
    }
}
impl ValueCastTo<u16> for i32 {
    fn cast(&self) -> u16 {
        *self as u16
    }
}
impl ValueCastTo<u32> for i32 {
    fn cast(&self) -> u32 {
        *self as u32
    }
}
impl ValueCastTo<u64> for i32 {
    fn cast(&self) -> u64 {
        *self as u64
    }
}
impl ValueCastTo<i8> for i32 {
    fn cast(&self) -> i8 {
        *self as i8
    }
}
impl ValueCastTo<i16> for i32 {
    fn cast(&self) -> i16 {
        *self as i16
    }
}
impl ValueCastTo<i32> for i32 {
    fn cast(&self) -> i32 {
        self.clone()
    }
}
impl ValueCastTo<i64> for i32 {
    fn cast(&self) -> i64 {
        *self as i64
    }
}
impl ValueCastTo<bool> for i64 {
    fn cast(&self) -> bool {
        *self != 0
    }
}
impl ValueCastTo<i64> for bool {
    fn cast(&self) -> i64 {
        if *self {
            1
        } else {
            0
        }
    }
}
impl ValueCastTo<BigInt> for i64 {
    fn cast(&self) -> BigInt {
        self.to_bigint().unwrap()
    }
}
impl ValueCastTo<BigUint> for i64 {
    fn cast(&self) -> BigUint {
        self.to_biguint().unwrap()
    }
}
impl ValueCastTo<i64> for BigInt {
    fn cast(&self) -> i64 {
        let (sign, data) = self.to_u64_digits();
        if data.is_empty() {
            return 0;
        }
        match sign {
            num_bigint::Sign::Plus => data[0] as i64,
            num_bigint::Sign::Minus => ((!data[0] + 1) & (i64::MAX as u64)) as i64,
            num_bigint::Sign::NoSign => data[0] as i64,
        }
    }
}
impl ValueCastTo<i64> for BigUint {
    fn cast(&self) -> i64 {
        let data = self.to_u64_digits();
        if data.is_empty() {
            return 0;
        } else {
            return data[0] as i64;
        }
    }
}
impl ValueCastTo<u8> for i64 {
    fn cast(&self) -> u8 {
        *self as u8
    }
}
impl ValueCastTo<u16> for i64 {
    fn cast(&self) -> u16 {
        *self as u16
    }
}
impl ValueCastTo<u32> for i64 {
    fn cast(&self) -> u32 {
        *self as u32
    }
}
impl ValueCastTo<u64> for i64 {
    fn cast(&self) -> u64 {
        *self as u64
    }
}
impl ValueCastTo<i8> for i64 {
    fn cast(&self) -> i8 {
        *self as i8
    }
}
impl ValueCastTo<i16> for i64 {
    fn cast(&self) -> i16 {
        *self as i16
    }
}
impl ValueCastTo<i32> for i64 {
    fn cast(&self) -> i32 {
        *self as i32
    }
}
impl ValueCastTo<i64> for i64 {
    fn cast(&self) -> i64 {
        self.clone()
    }
}
