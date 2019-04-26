use std::thread;
use std::sync::mpsc;
use std::collections::BinaryHeap;
use std::cmp::Ordering;

use num_cpus;

/// Map `f` over `xs`.
///
/// This function initialises a thread pool and distributes values
/// in `xs` to each thread in a round-robin fashion.
///
/// Creates one thread per cpu, as obtained by `num_cpus::get`.
pub fn par_map<X, XI, Y, F>(xs: XI, f: F) -> impl Iterator<Item=Y>
where
    X: Send + 'static,
    XI: Iterator<Item=X>,
    Y: Send + 'static,
    F: Send + 'static + Clone + Fn(X) -> Y,
{
    // first, setup a threadpool
    // create one thread per CPU
    let n_workers = num_cpus::get();

    let mut workers = Vec::with_capacity(n_workers);
    let mut senders = Vec::with_capacity(n_workers);

    // child threads can send values on csend
    // parent thread can recieve them on precv
    let (csend, precv) = mpsc::channel();

    for _ in 0..n_workers
    {
        // parent thread can send values on psend
        // child thread will recieve values on crecv
        let (psend, crecv) = mpsc::channel();
        let csend = csend.clone();
        let f = f.clone();

        workers.push(
            thread::spawn(
                move ||
                loop
                {
                    use std::sync::mpsc::TryRecvError::*;
                    match crecv.try_recv()
                    {
                        Ok(Wrap(counter, item)) => csend.send(Wrap(counter, f(item))).unwrap(),
                        Err(Empty) => (),
                        Err(Disconnected) => return,
                    };
                }
            )
        );

        senders.push(psend);
    }

    // round-robin the items to each thread
    let mut counter = 0;
    for x in xs.into_iter()
    {
        senders[counter % senders.len()].send(Wrap(counter, x)).unwrap();
        counter += 1;
    }

    // collect results into vec and return it
    OutputStream::new(
        precv.into_iter().take(counter),
        counter
    )
}

struct Wrap<A>(usize, A);

impl<A> Ord for Wrap<A>
{
    fn cmp(&self, other: &Self) -> Ordering
    {
        let s = self.0;
        let o = other.0;

        // define as reverse ordering so that BinaryHeap becomes a min-heap
        s.cmp(&o).reverse()
    }
}

impl<A> Eq for Wrap<A> {}

impl<A> PartialEq for Wrap<A>
{
    fn eq(&self, other: &Self) -> bool
    {
        self.0 == other.0
    }
}

impl<A> PartialOrd for Wrap<A>
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering>
    {
        Some(<Self as Ord>::cmp(self, other))
    }
}

struct OutputStream<Y, I: Iterator<Item=Wrap<Y>>>
{
    iter: I,
    buffer: BinaryHeap<Wrap<Y>>,
    counter: usize,
}

impl<Y, I: Iterator<Item=Wrap<Y>> > OutputStream<Y, I>
{
    fn new(iter: I, counter: usize) -> Self
    {
        OutputStream
        {
            iter: iter,
            buffer: BinaryHeap::with_capacity(counter),
            counter: 0,
        }
    }
}

impl<Y, I: Iterator<Item=Wrap<Y>> > Iterator for OutputStream<Y, I>
{
    type Item = Y;
    fn next(&mut self) -> Option<Self::Item>
    {
        // if value at top of heap is equal to counter, pop it
        // else pop values from iterator into buffer until counter is found

        if let Some(Wrap(counter, _)) = self.buffer.peek()
        {
            if counter == &self.counter
            {
                self.counter += 1;
                let Wrap(_, item) = self.buffer.pop().unwrap(); // safe, because peek succeeded
                return Some(item);
            }
        }

        while let Some(Wrap(counter, item)) = self.iter.next()
        {
            if counter == self.counter
            {
                self.counter += 1;
                return Some(item);
            }

            else
            {
                self.buffer.push(Wrap(counter, item));
            }
        }

        return None;
    }
}

pub trait ParMap
{
    pub fn par_map<X, Y, F>(self, f: F) -> Box<dyn Iterator<Item=Y>>
    where
        X: Send + 'static,
        Y: Send + 'static,
        F: Send + 'static + Clone + Fn(X) -> Y,
        Self: Sized + Iterator<Item=X> + 'static,
    {
        Box::new(super::par_map(self, f))
    }
}

impl<X, T: Iterator<Item=X>> ParMap for T {}

