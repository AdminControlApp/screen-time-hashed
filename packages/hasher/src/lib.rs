use std::{
  sync::{
    mpsc::{self, channel, Receiver, SendError, Sender, TryRecvError},
    Arc, Mutex, RwLock,
  },
  thread::{self, JoinHandle},
  time::Duration,
};
use threadpool::ThreadPool;

use sha256::digest;

#[macro_use]
extern crate napi_derive;

#[napi]
fn measure_hashes_per_second() -> i32 {
  let num_cpus = num_cpus::get();

  let total_hash_count = Arc::new(RwLock::new(0));

  let mut thread_tx: Vec<Sender<()>> = vec![];
  let mut threads: Vec<JoinHandle<()>> = vec![];

  for _ in 0..num_cpus {
    let (tx, rx) = mpsc::channel();
    let total_hash_count = Arc::clone(&total_hash_count);
    threads.push(thread::spawn(move || {
      let mut hash_count = 0;
      let mut input = String::from("1234");

      loop {
        input = digest(&input);
        hash_count += 1;

        match rx.try_recv() {
          Ok(_) | Err(TryRecvError::Disconnected) => {
            let mut total_hash_count = total_hash_count
              .write()
              .expect("Failed to get `total_hash_count` write guard");
            *total_hash_count += hash_count;
            break;
          }
          Err(TryRecvError::Empty) => {}
        }
      }
    }));

    thread_tx.push(tx);
  }

  thread::sleep(Duration::from_millis(1000));

  for tx in thread_tx {
    tx.send(()).unwrap();
  }

  for thread in threads {
    thread.join().unwrap();
  }

  let total_hash_count = total_hash_count.read().unwrap();
  *total_hash_count
}

#[napi]
fn repeating_hash(passcode: String, num_hashes: u32) -> String {
  let mut hashed_passcode = passcode;
  for _ in 0..num_hashes {
    hashed_passcode = digest(&hashed_passcode);
  }

  hashed_passcode
}

#[napi]
fn brute_force_hash(hash: String, num_hashes: u32) -> Option<String> {
  let num_cpus = num_cpus::get();
  let pool = ThreadPool::new(num_cpus);

  let (tx_answer, rx_answer) = channel::<String>();

  let abort_senders = Arc::new(Mutex::new(Vec::<Sender<()>>::new()));

  for i in 0..10000 {
    let (tx_abort, rx_abort) = channel::<()>();
    let abort_senders = Arc::clone(&abort_senders);
    abort_senders.lock().unwrap().push(tx_abort);
    let passcode = format!("{:0>4}", i);
    let num_hashes = u32::clone(&num_hashes);
    let expected_hash = String::clone(&hash);
    let tx_answer = Sender::clone(&tx_answer);

    pool.execute(move || {
      let mut hashed_passcode = String::clone(&passcode);

      for _ in 0..num_hashes {
        hashed_passcode = digest(&hashed_passcode);

        // Exit early if the passcode has already been found
        match rx_abort.try_recv() {
          Ok(_) | Err(TryRecvError::Disconnected) => {
            break;
          }
          Err(TryRecvError::Empty) => {}
        }
      }

      if hashed_passcode == expected_hash {
        tx_answer.send(passcode).expect("Failed to send answer");

        let abort_senders = abort_senders
          .lock()
          .expect("Failed to acquire abort_senders lock");
        for abort_sender in abort_senders.iter() {
          let _ = abort_sender.send(());
        }
      }
    })
  }

  pool.join();

  match rx_answer.try_recv() {
    Ok(passcode) => Some(passcode),
    Err(_) => None,
  }
}
